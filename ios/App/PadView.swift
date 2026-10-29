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

/// What the typing logic writes into, on the pad's text view
final class PadTarget: TextTarget {
    weak var tv: PadTextView?
    init(_ tv: PadTextView) { self.tv = tv }

    private func edit(_ f: (PadTextView) -> Void) {
        guard let tv else { return }
        let was = tv.applying
        tv.applying = true; f(tv); tv.applying = was
    }

    var contextBefore: String {
        guard let tv, let sel = tv.selectedTextRange, let r = tv.textRange(from: tv.beginningOfDocument, to: sel.start) else { return "" }
        return tv.text(in: r) ?? ""
    }
    func insert(_ s: String) { edit { $0.insertText(s) } }
    func deleteBackward() { edit { $0.deleteBackward() } }
    func setMarked(_ s: String) { edit { $0.setMarkedText(s, selectedRange: NSRange(location: (s as NSString).length, length: 0)) } }
    func unmark() { edit { $0.unmarkText() } }
    func moveCursor(_ by: Int) {
        edit { tv in
            guard let sel = tv.selectedTextRange, let p = tv.position(from: sel.start, offset: by) else { return }
            tv.selectedTextRange = tv.textRange(from: p, to: p)
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
        // the cursor moved (a tap, an arrow key): the word being typed stays as it is shown
        if !tv.applying && tv.markedTextRange == nil { typist.externalChange() }
    }

    func textViewDidChange(_ textView: UITextView) {
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
