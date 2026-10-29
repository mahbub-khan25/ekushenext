import GameController
import SwiftUI
import UIKit

/// The writing pad: type Bangla phonetically with any keyboard, an iPad's physical keyboard
/// included (iPadOS doesn't let a third-party keyboard read a physical one, so this is where it
/// works). The suggestions sit in Ekushe's own strip above the keyboard; Tab moves through them,
/// Space places one, Backspace right after a word reopens it, and Shift tapped twice switches
/// Bangla ⇄ English. The text is kept between visits.
final class PadTextView: UITextView {
    var router: PadRouter?
    /// Ekushe is changing the text itself (the change is not a key to route)
    var applying = false

    // Tab, Shift and Esc on a physical keyboard come as key presses
    private var consumed = Set<UIKeyboardHIDUsage>()

    override func pressesBegan(_ presses: Set<UIPress>, with event: UIPressesEvent?) {
        var rest = Set<UIPress>()
        for p in presses {
            guard let key = p.key, let r = router else { rest.insert(p); continue }
            switch key.keyCode {
            case .keyboardLeftShift, .keyboardRightShift:
                if key.modifierFlags.subtracting(.shift).isEmpty { r.shiftTapped() }
                rest.insert(p)
            case .keyboardTab where key.modifierFlags.subtracting(.shift).isEmpty:
                r.otherKey()
                if r.tab(back: key.modifierFlags.contains(.shift)) { consumed.insert(key.keyCode) } else { rest.insert(p) }
            case .keyboardEscape:
                r.otherKey()
                if r.escape() { consumed.insert(key.keyCode) } else { rest.insert(p) }
            default:
                r.otherKey()
                rest.insert(p)
            }
        }
        if !rest.isEmpty { super.pressesBegan(rest, with: event) }
    }

    override func pressesEnded(_ presses: Set<UIPress>, with event: UIPressesEvent?) {
        let rest = presses.filter { p in
            guard let code = p.key?.keyCode, consumed.contains(code) else { return true }
            consumed.remove(code); return false
        }
        if !rest.isEmpty { super.pressesEnded(rest, with: event) }
    }

    /// Letters typed here go through Ekushe unless the keyboard already writes Bangla (Ekushe's own
    /// on-screen keyboard, or Apple's Bangla one). With a physical keyboard they always do.
    var translate: Bool {
        if GCKeyboard.coalesced != nil { return true }
        return !(textInputMode?.primaryLanguage ?? "").hasPrefix("bn")
    }
}

/// What the typing logic writes into, on the pad's text view. The word being typed is kept by
/// Ekushe itself (underlined in the text), not as iOS's "marked text": the system keyboard clears
/// marked text it didn't make at its next key, which would end the word after every letter.
final class PadTarget: TextTarget {
    weak var tv: PadTextView?
    /// Where the word being typed is, in the text
    private(set) var composing: NSRange?
    var onEdit: (() -> Void)?
    init(_ tv: PadTextView) { self.tv = tv }

    private func edit(_ f: (PadTextView) -> Void) {
        guard let tv else { return }
        let was = tv.applying
        tv.applying = true; f(tv); tv.applying = was
        onEdit?()
    }

    private var plainAttributes: [NSAttributedString.Key: Any] {
        var a: [NSAttributedString.Key: Any] = [.foregroundColor: UIColor.label]
        if let f = tv?.font { a[.font] = f }
        return a
    }

    /// Put `s` where `r` is, the cursor after it, and return where it went
    private func put(_ s: String, at r: NSRange, underline: Bool) -> NSRange {
        guard let tv else { return r }
        // never past the end of the text (the app may have changed it)
        let len = tv.textStorage.length, loc = min(r.location, len)
        let range = NSRange(location: loc, length: min(r.length, len - loc))
        var attrs = plainAttributes
        if underline { attrs[.underlineStyle] = NSUnderlineStyle.single.rawValue }
        tv.textStorage.replaceCharacters(in: range, with: NSAttributedString(string: s, attributes: attrs))
        let placed = NSRange(location: range.location, length: (s as NSString).length)
        tv.selectedRange = NSRange(location: placed.location + placed.length, length: 0)
        tv.typingAttributes = plainAttributes
        return placed
    }

    var contextBefore: String {
        guard let tv else { return "" }
        let end = min(tv.selectedRange.location, tv.textStorage.length)
        return (tv.textStorage.string as NSString).substring(to: end)
    }
    func insert(_ s: String) {
        edit { tv in _ = put(s, at: composing ?? tv.selectedRange, underline: false); composing = nil }
    }
    func deleteBackward() {
        edit { tv in
            if let c = composing { _ = put("", at: c, underline: false); composing = nil; return }
            tv.deleteBackward()
        }
    }
    func setMarked(_ s: String) {
        edit { tv in
            let r = composing ?? tv.selectedRange
            if s.isEmpty { _ = put("", at: r, underline: false); composing = nil }
            else { composing = put(s, at: r, underline: true) }
        }
    }
    func unmark() {
        edit { tv in
            if let c = composing, NSMaxRange(c) <= tv.textStorage.length {
                tv.textStorage.removeAttribute(.underlineStyle, range: c)
            }
            composing = nil
        }
    }
    func moveCursor(_ by: Int) {
        edit { tv in
            let n = max(0, min(tv.textStorage.length, tv.selectedRange.location + by))
            tv.selectedRange = NSRange(location: n, length: 0)
        }
    }
}

final class PadViewController: UIViewController, UITextViewDelegate {
    let tv = PadTextView()
    let bar = KeyboardView(frame: CGRect(x: 0, y: 0, width: 320, height: 52))
    private var typist: Typist!
    private var router: PadRouter!
    private var target: PadTarget!
    private let placeholder = UILabel()
    var onHelp: (() -> Void)?
    var onTextChange: ((String) -> Void)?

    static let saveKey = "pad_text"

    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = .systemBackground
        tv.translatesAutoresizingMaskIntoConstraints = false
        tv.backgroundColor = .clear
        tv.font = UIFontMetrics(forTextStyle: .body).scaledFont(for: .systemFont(ofSize: traitCollection.userInterfaceIdiom == .pad ? 22 : 19))
        tv.adjustsFontForContentSizeCategory = true
        tv.textContainerInset = UIEdgeInsets(top: 18, left: 14, bottom: 18, right: 14)
        tv.alwaysBounceVertical = true
        tv.keyboardDismissMode = .interactive
        // Ekushe gives the suggestions; Apple's would only fight it
        tv.autocorrectionType = .no
        tv.spellCheckingType = .no
        tv.smartQuotesType = .no
        tv.smartDashesType = .no
        tv.smartInsertDeleteType = .no
        tv.autocapitalizationType = .none
        if #available(iOS 17.0, *) { tv.inlinePredictionType = .no }
        tv.inputAssistantItem.leadingBarButtonGroups = []
        tv.inputAssistantItem.trailingBarButtonGroups = []
        tv.delegate = self
        tv.text = UserDefaults.standard.string(forKey: Self.saveKey) ?? ""
        tv.accessibilityLabel = "লেখার প্যাড. Writing pad"
        view.addSubview(tv)

        placeholder.text = "এখানে লিখুন… ইংরেজি অক্ষরে বাংলা, যেমন amar sonar bangla\nType Bangla in English letters, with any keyboard"
        placeholder.numberOfLines = 0
        placeholder.textColor = .placeholderText
        placeholder.font = UIFontMetrics(forTextStyle: .body).scaledFont(for: .systemFont(ofSize: 17))
        placeholder.adjustsFontForContentSizeCategory = true
        placeholder.translatesAutoresizingMaskIntoConstraints = false
        placeholder.isAccessibilityElement = false
        view.addSubview(placeholder)

        NSLayoutConstraint.activate([
            tv.leadingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor),
            tv.trailingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor),
            tv.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor),
            tv.bottomAnchor.constraint(equalTo: view.keyboardLayoutGuide.topAnchor),
            placeholder.leadingAnchor.constraint(equalTo: tv.leadingAnchor, constant: 19),
            placeholder.trailingAnchor.constraint(equalTo: tv.trailingAnchor, constant: -19),
            placeholder.topAnchor.constraint(equalTo: tv.topAnchor, constant: 18),
        ])

        // Ekushe's own suggestion strip, above whichever keyboard is in use (above a physical one, at the bottom)
        bar.stripOnly = true
        bar.toolIds = []
        bar.pad = traitCollection.userInterfaceIdiom == .pad
        bar.frame.size.height = bar.preferredHeight
        bar.autoresizingMask = [.flexibleWidth]
        tv.inputAccessoryView = bar

        target = PadTarget(tv)
        target.onEdit = { [weak self] in self?.textEdited() }
        typist = Typist(target: target, view: bar)
        typist.onHelpAction = { [weak self] in self?.onHelp?() }
        bar.listener = typist
        router = PadRouter(typist: typist)
        tv.router = router
        refreshBar()
        typist.begin()
        updatePlaceholder()
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)
        tv.becomeFirstResponder()
    }

    override func viewWillDisappear(_ animated: Bool) {
        typist.finishWord()
        save()
        Engine.save(); English.shared.save()
        super.viewWillDisappear(animated)
    }

    override func traitCollectionDidChange(_ previous: UITraitCollection?) {
        super.traitCollectionDidChange(previous)
        refreshBar()
    }

    private func refreshBar() {
        bar.refreshTheme(dark: traitCollection.userInterfaceStyle == .dark)
        bar.frame.size.height = bar.preferredHeight
    }

    private func updatePlaceholder() { placeholder.isHidden = !tv.text.isEmpty }

    func save() { UserDefaults.standard.set(tv.text, forKey: Self.saveKey) }

    // ───────────────────────── the text view ─────────────────────────

    func textView(_ textView: UITextView, shouldChangeTextIn range: NSRange, replacementText text: String) -> Bool {
        if tv.applying { return true }
        return router.shouldChange(text, rangeLength: range.length, translate: tv.translate)
    }

    func textViewDidChangeSelection(_ textView: UITextView) {
        // the cursor moved away from the word being typed (a tap, an arrow key): it stays as it is shown
        guard !tv.applying, let c = target.composing else { return }
        if tv.selectedRange != NSRange(location: NSMaxRange(c), length: 0) { typist.externalChange() }
    }

    func textViewDidChange(_ textView: UITextView) { textEdited() }

    private func textEdited() {
        updatePlaceholder()
        save()
        onTextChange?(tv.text)
    }

    // ───────────────────────── toolbar actions ─────────────────────────

    func copyAll() {
        typist.finishWord()
        UIPasteboard.general.string = tv.text
    }

    func clear() {
        typist.finishWord()
        tv.applying = true; tv.text = ""; tv.applying = false
        typist.begin()
        updatePlaceholder(); save(); onTextChange?("")
    }

    var text: String { tv.text }
}

/// The pad in SwiftUI, with its toolbar
struct PadScreen: View {
    var onHelp: () -> Void
    @State private var text = UserDefaults.standard.string(forKey: PadViewController.saveKey) ?? ""
    @State private var copied = false
    @State private var confirmClear = false
    @StateObject private var holder = PadHolder()

    init(onHelp: @escaping () -> Void) { self.onHelp = onHelp }

    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                if GCKeyboard.coalesced != nil || UIDevice.current.userInterfaceIdiom == .pad {
                    Text("আলাদা কিবোর্ডে: Shift দুবার বাংলা ⇄ English · Tab পরের শব্দ · Space বসায় · ⌫ শব্দ আবার খোলে")
                        .font(.footnote).foregroundColor(.secondary)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(.horizontal, 18).padding(.vertical, 8)
                        .background(Color(.secondarySystemBackground))
                        .accessibilityLabel("With a physical keyboard: tap Shift twice for Bangla or English, Tab for the next suggestion, Space to place it, Backspace to reopen a word")
                }
                PadRepresentable(holder: holder, onHelp: onHelp, text: $text)
            }
            .navigationTitle("লেখার প্যাড")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItemGroup(placement: .primaryAction) {
                    Button {
                        holder.vc?.copyAll(); copied = true
                        DispatchQueue.main.asyncAfter(deadline: .now() + 1.6) { copied = false }
                    } label: { Label(copied ? "কপি হয়েছে" : "কপি", systemImage: copied ? "checkmark" : "doc.on.doc") }
                        .disabled(text.isEmpty)
                    ShareLink(item: text) { Label("শেয়ার", systemImage: "square.and.arrow.up") }
                        .disabled(text.isEmpty)
                    Button(role: .destructive) { confirmClear = true } label: { Label("মুছুন", systemImage: "trash") }
                        .disabled(text.isEmpty)
                }
            }
            .confirmationDialog("পুরো লেখা মুছে ফেলবেন?", isPresented: $confirmClear, titleVisibility: .visible) {
                Button("মুছে ফেলুন · Clear", role: .destructive) { holder.vc?.clear() }
                Button("থাক · Cancel", role: .cancel) {}
            }
        }
    }
}

final class PadHolder: ObservableObject { weak var vc: PadViewController? }

struct PadRepresentable: UIViewControllerRepresentable {
    let holder: PadHolder
    var onHelp: () -> Void
    @Binding var text: String

    func makeUIViewController(context: Context) -> PadViewController {
        let vc = PadViewController()
        vc.onHelp = onHelp
        vc.onTextChange = { t in DispatchQueue.main.async { text = t } }
        holder.vc = vc
        return vc
    }
    func updateUIViewController(_ vc: PadViewController, context: Context) {}
}
