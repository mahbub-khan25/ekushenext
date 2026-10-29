import UIKit

/// The keyboard's own view, so iOS plays its key click (when the person has keyboard clicks on).
final class ClickInputView: UIInputView, UIInputViewAudioFeedback {
    var enableInputClicksWhenVisible: Bool { true }
}

/// Ekushe Next as an iOS keyboard: the KeyboardView, typing through Typist into the app's text box.
final class KeyboardViewController: UIInputViewController, TextTarget {
    private let kb = KeyboardView(frame: .zero)
    private var typist: Typist!
    private var heightC: NSLayoutConstraint?
    private var settings: SettingsPanel?

    override func loadView() {
        inputView = ClickInputView(frame: .zero, inputViewStyle: .keyboard)
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        inputView?.allowsSelfSizing = true
        Engine.start(bundle: Bundle(for: KeyboardViewController.self))
        English.shared.start(bundle: Bundle(for: KeyboardViewController.self))
        requestSupplementaryLexicon { lex in
            English.shared.addSupplementary(lex.entries.map { $0.documentText })
        }
        typist = Typist(target: self, view: kb)
        typist.onGlobeAction = { [weak self] in self?.advanceToNextInputMode() }
        typist.onHideAction = { [weak self] in self?.dismissKeyboard() }
        typist.onSettingsAction = { [weak self] in self?.showSettings() }
        typist.onHelpAction = { [weak self] in self?.showMarks() }
        typist.onClick = { if Prefs.shared.sound { UIDevice.current.playInputClick() } }
        kb.listener = typist
        kb.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(kb)
        NSLayoutConstraint.activate([
            kb.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            kb.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            kb.topAnchor.constraint(equalTo: view.topAnchor),
            kb.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])
        let h = view.heightAnchor.constraint(equalToConstant: kb.preferredHeight)
        h.priority = UILayoutPriority(999)
        h.isActive = true
        heightC = h
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        configureForField()
    }

    /// iPhone or iPad, upright or on its side: the keys and the keyboard's height follow
    private func updateShape(for size: CGSize? = nil) {
        kb.pad = traitCollection.userInterfaceIdiom == .pad
        let screen = size ?? view.window?.windowScene?.screen.bounds.size ?? UIScreen.main.bounds.size
        kb.landscape = screen.width > screen.height
        heightC?.constant = kb.preferredHeight
    }

    override func viewWillTransition(to size: CGSize, with coordinator: UIViewControllerTransitionCoordinator) {
        super.viewWillTransition(to: size, with: coordinator)
        // the keyboard is as wide as the screen, so its new width tells which way the device is held
        let screen = UIScreen.main.bounds.size
        let landscape = size.width > min(screen.width, screen.height) + 1
        coordinator.animate(alongsideTransition: { [weak self] _ in
            guard let self else { return }
            self.kb.landscape = landscape
            self.heightC?.constant = self.kb.preferredHeight
            self.view.layoutIfNeeded()
        })
    }

    override func viewWillDisappear(_ animated: Bool) {
        typist.finishWord()
        Engine.save(); English.shared.save()
        super.viewWillDisappear(animated)
    }

    private var dark: Bool {
        textDocumentProxy.keyboardAppearance == .dark || traitCollection.userInterfaceStyle == .dark
    }

    /// Set the keyboard up for the text box it is typing into
    private func configureForField() {
        let p = textDocumentProxy
        let type = p.keyboardType ?? .default
        let numbers: [UIKeyboardType] = [.numberPad, .decimalPad, .asciiCapableNumberPad, .phonePad, .namePhonePad]
        let plain: [UIKeyboardType] = [.emailAddress, .URL, .twitter]
        typist.plainField = numbers.contains(type) || plain.contains(type) || p.isSecureTextEntry == true
        typist.privateField = p.isSecureTextEntry == true || p.autocorrectionType == .no
        typist.autoCapitalize = (p.autocapitalizationType ?? .sentences) != .none
        kb.showsGlobe = needsInputModeSwitchKey
        kb.mode = numbers.contains(type) ? .numbers : .letters
        kb.enterLabel = {
            switch p.returnKeyType ?? .default {
            case .go, .join, .route: return "→"
            case .search, .google, .yahoo: return "🔍"
            case .send: return "➤"
            case .done: return "✓"
            case .next, .continue: return "⇥"
            default: return "↵"
            }
        }()
        kb.refreshTheme(dark: dark)
        updateShape()
        typist.begin()
    }

    override func textDidChange(_ textInput: UITextInput?) {
        super.textDidChange(textInput)
        kb.refreshTheme(dark: dark)
    }

    override func selectionDidChange(_ textInput: UITextInput?) {
        super.selectionDidChange(textInput)
        typist.externalChange()
    }

    override func traitCollectionDidChange(_ previousTraitCollection: UITraitCollection?) {
        super.traitCollectionDidChange(previousTraitCollection)
        kb.refreshTheme(dark: dark)
    }

    // ───────────────────────── TextTarget ─────────────────────────

    var contextBefore: String { textDocumentProxy.documentContextBeforeInput ?? "" }
    func insert(_ s: String) { textDocumentProxy.insertText(s) }
    func deleteBackward() { textDocumentProxy.deleteBackward() }
    func setMarked(_ s: String) { textDocumentProxy.setMarkedText(s, selectedRange: NSRange(location: (s as NSString).length, length: 0)) }
    func unmark() { textDocumentProxy.unmarkText() }
    func moveCursor(_ by: Int) { textDocumentProxy.adjustTextPosition(byCharacterOffset: by) }

    // ───────────────────────── settings ─────────────────────────

    private func showPanel(_ p: UIView) {
        p.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(p)
        NSLayoutConstraint.activate([
            p.leadingAnchor.constraint(equalTo: view.leadingAnchor), p.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            p.topAnchor.constraint(equalTo: view.topAnchor), p.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])
        UIAccessibility.post(notification: .screenChanged, argument: p)
    }

    /// The marks (+ ,, ` ^ rr …), inside the keyboard: a keyboard can't open its app
    private var marks: MarksPanel?
    private func showMarks() {
        let m = MarksPanel(frame: kb.bounds, palette: kb.palette)
        m.onClose = { [weak self] in self?.marks?.removeFromSuperview(); self?.marks = nil; UIAccessibility.post(notification: .screenChanged, argument: nil) }
        showPanel(m)
        marks = m
    }

    private func showSettings() {
        let s = SettingsPanel(frame: kb.bounds, palette: kb.palette, pad: kb.pad)
        s.onClose = { [weak self] in
            guard let self else { return }
            self.settings?.removeFromSuperview(); self.settings = nil
            self.configureForField()
        }
        showPanel(s)
        settings = s
    }
}

/// Settings, inside the keyboard: themes, the look of the keys, typing.
final class SettingsPanel: UIView {
    var onClose: (() -> Void)?
    private let palette: Palette
    private let pad: Bool
    private let stack = UIStackView()

    init(frame: CGRect, palette: Palette, pad: Bool = false) {
        self.palette = palette
        self.pad = pad
        super.init(frame: frame)
        backgroundColor = palette.background
        let scroll = UIScrollView()
        scroll.translatesAutoresizingMaskIntoConstraints = false
        addSubview(scroll)
        stack.axis = .vertical; stack.spacing = 10
        stack.translatesAutoresizingMaskIntoConstraints = false
        scroll.addSubview(stack)
        NSLayoutConstraint.activate([
            scroll.leadingAnchor.constraint(equalTo: leadingAnchor), scroll.trailingAnchor.constraint(equalTo: trailingAnchor),
            scroll.topAnchor.constraint(equalTo: topAnchor), scroll.bottomAnchor.constraint(equalTo: bottomAnchor),
            stack.leadingAnchor.constraint(equalTo: scroll.contentLayoutGuide.leadingAnchor, constant: 14),
            stack.trailingAnchor.constraint(equalTo: scroll.contentLayoutGuide.trailingAnchor, constant: -14),
            stack.topAnchor.constraint(equalTo: scroll.contentLayoutGuide.topAnchor, constant: 10),
            stack.bottomAnchor.constraint(equalTo: scroll.contentLayoutGuide.bottomAnchor, constant: -14),
            stack.widthAnchor.constraint(equalTo: scroll.frameLayoutGuide.widthAnchor, constant: -28),
        ])
        build()
    }
    required init?(coder: NSCoder) { fatalError() }

    private func label(_ s: String, _ size: CGFloat, _ weight: UIFont.Weight = .regular, _ color: UIColor? = nil) -> UILabel {
        let l = UILabel(); l.text = s; l.numberOfLines = 0
        l.font = .systemFont(ofSize: size, weight: weight); l.textColor = color ?? palette.text
        return l
    }

    private func button(_ title: String, on: Bool, _ action: @escaping () -> Void) -> UIButton {
        let b = UIButton(type: .system)
        b.setTitle(title, for: .normal)
        b.titleLabel?.font = .systemFont(ofSize: 15, weight: on ? .semibold : .regular)
        b.setTitleColor(on ? palette.onAccent : palette.text, for: .normal)
        b.backgroundColor = on ? palette.accent : palette.key
        b.layer.cornerRadius = 10
        b.contentEdgeInsets = UIEdgeInsets(top: 8, left: 14, bottom: 8, right: 14)
        b.addAction(UIAction { _ in action() }, for: .touchUpInside)
        return b
    }

    private func row(_ views: [UIView]) -> UIView {
        let s = UIScrollView(); s.showsHorizontalScrollIndicator = false
        let h = UIStackView(arrangedSubviews: views); h.spacing = 8; h.translatesAutoresizingMaskIntoConstraints = false
        s.addSubview(h)
        NSLayoutConstraint.activate([
            h.leadingAnchor.constraint(equalTo: s.contentLayoutGuide.leadingAnchor), h.trailingAnchor.constraint(equalTo: s.contentLayoutGuide.trailingAnchor),
            h.topAnchor.constraint(equalTo: s.contentLayoutGuide.topAnchor), h.bottomAnchor.constraint(equalTo: s.contentLayoutGuide.bottomAnchor),
            h.heightAnchor.constraint(equalTo: s.frameLayoutGuide.heightAnchor), s.heightAnchor.constraint(equalToConstant: 40),
        ])
        return s
    }

    private func toggle(_ title: String, _ value: Bool, _ set: @escaping (Bool) -> Void) -> UIView {
        let h = UIStackView(); h.alignment = .center
        h.addArrangedSubview(label(title, 15))
        let sw = UISwitch(); sw.isOn = value; sw.onTintColor = palette.accent
        sw.addAction(UIAction { _ in set(sw.isOn) }, for: .valueChanged)
        h.addArrangedSubview(sw)
        return h
    }

    private func build() {
        stack.arrangedSubviews.forEach { $0.removeFromSuperview() }
        let prefs = Prefs.shared
        let head = UIStackView(); head.alignment = .center
        head.addArrangedSubview(label("একুশে নেক্সট · সেটিংস", 17, .bold))
        head.addArrangedSubview(button("সম্পন্ন", on: true) { [weak self] in self?.onClose?() })
        stack.addArrangedSubview(head)

        stack.addArrangedSubview(label("থিম · Theme", 13, .semibold, palette.subtext))
        stack.addArrangedSubview(row(themeChoices.map { t in button(t.bn, on: prefs.theme == t.id) { [weak self] in prefs.theme = t.id; self?.build() } }))

        stack.addArrangedSubview(label("কী চাপার ধরন · Key press", 13, .semibold, palette.subtext))
        let effects = [("classic", "সাধারণ"), ("3d", "থ্রিডি"), ("glow", "আলো"), ("bounce", "লাফ")]
        stack.addArrangedSubview(row(effects.map { e in button(e.1, on: prefs.pressEffect == e.0) { [weak self] in prefs.pressEffect = e.0; self?.build() } }))

        stack.addArrangedSubview(label("উচ্চতা · Height", 13, .semibold, palette.subtext))
        stack.addArrangedSubview(row([("ছোট", 0), ("সাধারণ", 1), ("বড়", 2)].map { e in button(e.0, on: prefs.height == e.1) { [weak self] in prefs.height = e.1; self?.build() } }))

        let cfg = Engine.config()
        stack.addArrangedSubview(label("টাইপিং মোড · Typing mode", 13, .semibold, palette.subtext))
        let modes = [("Smart", "স্মার্ট"), ("PhoneticFirst", "ফোনেটিক আগে"), ("PhoneticOnly", "শুধু ফোনেটিক")]
        let mode = cfg["typing_mode"] as? String ?? "Smart"
        stack.addArrangedSubview(row(modes.map { m in button(m.1, on: mode == m.0) { [weak self] in Engine.setConfig(["typing_mode": m.0]); self?.build() } }))

        stack.addArrangedSubview(toggle("বাংলা সংখ্যা (০-৯)", cfg["bangla_digits"] as? Bool ?? true) { Engine.setConfig(["bangla_digits": $0]) })
        stack.addArrangedSubview(toggle("কীতে বাংলা অক্ষর", prefs.keyHints) { prefs.keyHints = $0 })
        stack.addArrangedSubview(toggle("কীর আলাদা ঘর", prefs.keyBorders) { prefs.keyBorders = $0 })
        stack.addArrangedSubview(toggle("ইমোজি বোতাম", prefs.emojiKey) { prefs.emojiKey = $0 })
        stack.addArrangedSubview(toggle("সংখ্যার সারি", prefs.numberRow) { prefs.numberRow = $0 })
        if pad { stack.addArrangedSubview(toggle("আড়াআড়ি ধরলে ভাঙা কিবোর্ড (দুই বুড়ো আঙুলে লেখা)", prefs.splitPad) { prefs.splitPad = $0 }) }
        stack.addArrangedSubview(toggle("দুবার Space-এ দাঁড়ি (।)", prefs.doubleSpaceDari) { prefs.doubleSpaceDari = $0 })
        stack.addArrangedSubview(toggle("ইংরেজি অটোকারেক্ট", prefs.enAutocorrect) { prefs.enAutocorrect = $0 })
        stack.addArrangedSubview(toggle("ইংরেজিতে পরের শব্দ", prefs.nextWordEnglish) { prefs.nextWordEnglish = $0 })
        stack.addArrangedSubview(toggle("কী চাপার শব্দ", prefs.sound) { prefs.sound = $0 })
        stack.addArrangedSubview(label("🔒 এই কিবোর্ড “Full Access” চায় না: আপনার লেখা ফোন থেকে বের হওয়ার কোনো পথ নেই।", 12.5, .regular, palette.subtext))
        stack.addArrangedSubview(button("শেখা শব্দগুলো মুছে দিন · Forget learned words", on: false) { Engine.forgetLearned(); English.shared.forget() })
    }
}

/// How to type conjuncts, hasanta and the other marks, inside the keyboard (the strip's ? button).
final class MarksPanel: UIView {
    var onClose: (() -> Void)?
    private let palette: Palette

    init(frame: CGRect, palette: Palette) {
        self.palette = palette
        super.init(frame: frame)
        backgroundColor = palette.background
        let scroll = UIScrollView(); scroll.translatesAutoresizingMaskIntoConstraints = false
        addSubview(scroll)
        let stack = UIStackView(); stack.axis = .vertical; stack.spacing = 6
        stack.translatesAutoresizingMaskIntoConstraints = false
        scroll.addSubview(stack)
        NSLayoutConstraint.activate([
            scroll.leadingAnchor.constraint(equalTo: leadingAnchor), scroll.trailingAnchor.constraint(equalTo: trailingAnchor),
            scroll.topAnchor.constraint(equalTo: topAnchor), scroll.bottomAnchor.constraint(equalTo: bottomAnchor),
            stack.leadingAnchor.constraint(equalTo: scroll.contentLayoutGuide.leadingAnchor, constant: 14),
            stack.trailingAnchor.constraint(equalTo: scroll.contentLayoutGuide.trailingAnchor, constant: -14),
            stack.topAnchor.constraint(equalTo: scroll.contentLayoutGuide.topAnchor, constant: 10),
            stack.bottomAnchor.constraint(equalTo: scroll.contentLayoutGuide.bottomAnchor, constant: -14),
            stack.widthAnchor.constraint(equalTo: scroll.frameLayoutGuide.widthAnchor, constant: -28),
        ])
        let head = UIStackView(); head.alignment = .center
        head.addArrangedSubview(label("চিহ্নের নিয়ম · Marks", 17, .bold, palette.text))
        let done = UIButton(type: .system)
        done.setTitle("সম্পন্ন", for: .normal)
        done.titleLabel?.font = .systemFont(ofSize: 15, weight: .semibold)
        done.setTitleColor(palette.onAccent, for: .normal); done.backgroundColor = palette.accent
        done.layer.cornerRadius = 10; done.contentEdgeInsets = UIEdgeInsets(top: 8, left: 14, bottom: 8, right: 14)
        done.addAction(UIAction { [weak self] _ in self?.onClose?() }, for: .touchUpInside)
        head.addArrangedSubview(done)
        stack.addArrangedSubview(head)
        stack.addArrangedSubview(label("যুক্তবর্ণ, হসন্ত, না আলাদা? দুটি ব্যঞ্জনের মাঝে:", 14, .semibold, palette.text))
        Marks.join.forEach { stack.addArrangedSubview(row($0)) }
        stack.addArrangedSubview(label("অন্য চিহ্ন", 14, .semibold, palette.text))
        Marks.others.forEach { stack.addArrangedSubview(row($0)) }
        stack.addArrangedSubview(label(Marks.whereBn + "\n" + Marks.whereEn, 12.5, .regular, palette.subtext))
    }
    required init?(coder: NSCoder) { fatalError() }

    private func label(_ s: String, _ size: CGFloat, _ weight: UIFont.Weight, _ color: UIColor) -> UILabel {
        let l = UILabel(); l.text = s; l.numberOfLines = 0; l.font = .systemFont(ofSize: size, weight: weight); l.textColor = color
        return l
    }

    private func row(_ r: Marks.Row) -> UIView {
        let keys = label(r.keys, 15, .medium, palette.hint); keys.font = .monospacedSystemFont(ofSize: 15, weight: .medium)
        keys.widthAnchor.constraint(equalToConstant: 96).isActive = true
        let bn = label(r.bangla, 19, .semibold, palette.text)
        bn.widthAnchor.constraint(equalToConstant: 82).isActive = true
        let what = label(r.what, 13, .regular, palette.subtext)
        let h = UIStackView(arrangedSubviews: [keys, bn, what]); h.alignment = .center; h.spacing = 8
        h.isAccessibilityElement = true
        h.accessibilityLabel = "\(r.keys.map { String($0) }.joined(separator: " ")) gives \(r.bangla). \(r.whatEn)"
        return h
    }
}
