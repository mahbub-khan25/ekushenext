import UIKit

/// Where typing goes: the app's text box (through iOS's text document proxy), or a pretend box in
/// the tests.
protocol TextTarget: AnyObject {
    var contextBefore: String { get }
    func insert(_ s: String)
    func deleteBackward()
    /// Show `s` as the word being typed (underlined), replacing the previous one
    func setMarked(_ s: String)
    /// Keep the word being typed as it is shown
    func unmark()
    func moveCursor(_ by: Int)
}

/// The typing logic, the same as the Android keyboard's: in Bangla every key goes to the engine,
/// which keeps the word being typed and its suggestions, and Space places the first suggestion.
/// In English the word is shown the same way, with completions and corrections; Space places it,
/// correcting only a clear typo (never an address, a name or Banglish).
final class Typist: KeyboardListener {
    weak var target: TextTarget?
    weak var view: KeyboardView?
    private let prefs = Prefs.shared
    /// Extra actions the keyboard's controller handles (switching keyboards, settings, hiding)
    var onGlobeAction: (() -> Void)?
    var onSettingsAction: (() -> Void)?
    var onHideAction: (() -> Void)?
    var onHelpAction: (() -> Void)?
    var onClick: (() -> Void)?
    /// The suggestion a physical keyboard's Tab moved to (0: the engine's own choice)
    private var highlight = 0

    private var last = Output()
    var bangla = true
    /// The field wants plain characters: an address, a number
    var plainField = false
    var privateField = false
    var autoCapitalize = true
    private var marked = false
    private var lastSpaceAt = Date.distantPast

    // English
    private var en = ""
    private var enShown: [String] = []
    private var enUndo: (typed: String, placed: String)?
    private var enKeep: String?

    init(target: TextTarget, view: KeyboardView?) {
        self.target = target; self.view = view
        bangla = prefs.bangla
    }

    func typingBangla() -> Bool { bangla && !plainField }

    /// A new text box: start clean
    func begin() {
        Engine.setPrivate(privateField)
        Engine.reset()
        last = Output(); en = ""; enUndo = nil; enKeep = nil; marked = false
        view?.bangla = typingBangla()
        view?.shift = .off
        updateAutoShift()
        view?.showSuggestions([], [], [])
    }

    /// The person moved the cursor or the app changed the text: the word being typed stays as shown
    func externalChange() {
        guard marked || !en.isEmpty || last.composing else { return }
        target?.unmark(); marked = false
        en = ""; enUndo = nil
        if last.composing { Engine.reset(); last = Output() }
        view?.showSuggestions([], [], [])
    }

    // ───────────────────────── the text box ─────────────────────────

    private func setMarked(_ s: String) {
        target?.setMarked(s); marked = !s.isEmpty
        if s.isEmpty { target?.unmark() }
    }

    /// Remove `n` characters (Unicode scalars, as the engine counts them) before the cursor
    private func deleteScalars(_ n: Int) {
        guard let t = target else { return }
        var left = n, steps = 0
        while left > 0 && steps < n + 8 {
            steps += 1
            let before = Array(t.contextBefore.unicodeScalars)
            t.deleteBackward()
            let after = t.contextBefore.unicodeScalars.count
            let removed = max(1, before.count - after)
            left -= removed
            if left < 0, before.count >= removed {
                // iOS removed a whole letter cluster: put back what was not ours to remove
                let chunk = before[(before.count - removed)...]
                var keep = String.UnicodeScalarView()
                keep.append(contentsOf: chunk.prefix(-left))
                t.insert(String(keep))
                left = 0
            }
        }
    }

    private func apply(_ out: Output) {
        guard let t = target else { return }
        if out.deleteBefore > 0 {
            if marked { setMarked("") }
            deleteScalars(out.deleteBefore)
        }
        if let c = out.committed {
            if marked { t.setMarked(c); t.unmark(); marked = false } else { t.insert(c) }
        }
        if out.composing { setMarked(out.preedit.isEmpty ? out.roman : out.preedit) }
        else if out.committed == nil && marked { setMarked("") }
        last = out
        highlight = 0
        view?.showSuggestions(out.composing ? out.candidates : [], out.composing ? out.tips : [], !out.composing && !privateField ? out.next : [])
    }

    /// Close the word being typed, keeping what is shown
    func finishWord() {
        if !en.isEmpty { target?.unmark(); marked = false; en = ""; view?.showSuggestions([], [], []) }
        if last.composing { target?.unmark(); marked = false; Engine.reset(); last = Output(); view?.showSuggestions([], [], []) }
    }

    /// English only: a capital letter at the start of a sentence (in Bangla, capitals change the sound)
    func updateAutoShift() {
        guard let v = view, v.mode == .letters, v.shift != .locked else { return }
        if typingBangla() || !autoCapitalize { if v.shift == .on { v.shift = .off }; return }
        let ctx = target?.contextBefore ?? ""
        let head = ctx.trimmingCharacters(in: .whitespaces)
        let start = head.isEmpty || ctx.hasSuffix("\n") || ((head.last.map { ".!?".contains($0) } ?? false) && ctx.hasSuffix(" "))
        v.shift = start ? .on : .off
    }

    // ───────────────────────── keys ─────────────────────────

    func onText(_ text: String) {
        guard let t = target else { return }
        if !typingBangla() {
            if plainField { t.insert(text); updateAutoShift(); return }
            for ch in text {
                if (ch.isLetter && ch.isASCII) || (ch == "'" && !en.isEmpty) {
                    en.append(ch); enUndo = nil
                    showEnglish()
                } else {
                    // a mark or digit ends the word, as typed (makhan.aumi@gmail stays exactly that)
                    if !en.isEmpty { placeEnglish(correct: false) }
                    t.insert(String(ch))
                    enUndo = nil
                    view?.showSuggestions([], [], [])
                }
            }
            if en.isEmpty { updateAutoShift() }
            return
        }
        for ch in text {
            if !ch.isASCII {
                // not a Roman letter or mark (৳, €, ©): close the word and type it as it is
                if last.composing { apply(Engine.commit()) }
                t.insert(String(ch))
                last = Output()
                view?.showSuggestions([], [], [])
            } else {
                apply(Engine.key(ch))
            }
        }
    }

    func onSpace() {
        guard let t = target else { return }
        let now = Date()
        if typingBangla() {
            if last.composing { apply(Engine.select(highlight > 0 ? highlight : last.selected, trailing: " ")); lastSpaceAt = now; return }
            // two quick spaces after a word: a dari (।)
            if prefs.doubleSpaceDari && now.timeIntervalSince(lastSpaceAt) < 0.45 {
                let before = t.contextBefore
                if before.hasSuffix(" "), let c = before.dropLast().last, c != " ", c != "।" {
                    t.deleteBackward(); t.insert("। "); lastSpaceAt = .distantPast; return
                }
            }
            apply(Engine.key(" ")); lastSpaceAt = now
            return
        }
        if !en.isEmpty {
            if enHighlight > 0, enHighlight < enShown.count {
                // Tab chose another suggestion: place that one
                onCandidate(enHighlight); return
            }
            placeEnglish(correct: true)
            t.insert(" ")
            lastSpaceAt = now
            showNextEnglish(); updateAutoShift()
            return
        }
        enUndo = nil
        if now.timeIntervalSince(lastSpaceAt) < 0.45 {
            let before = t.contextBefore
            if before.hasSuffix(" "), let c = before.dropLast().last, c.isLetter || c.isNumber {
                t.deleteBackward(); t.insert(". "); lastSpaceAt = .distantPast; updateAutoShift(); return
            }
        }
        t.insert(" "); lastSpaceAt = now
        updateAutoShift()
    }

    func onDelete() {
        guard let t = target else { return }
        if typingBangla() && last.composing { apply(Engine.backspace()); return }
        // right after a word: take it back and reopen its Roman letters with their suggestions
        if typingBangla() && Engine.hasUndo() {
            let out = Engine.backspace()
            if out.deleteBefore > 0 {
                deleteScalars(out.deleteBefore)
                last = Output()
                for c in out.committed ?? "" { apply(Engine.key(c)) }
                return
            }
        }
        if !typingBangla() && !en.isEmpty {
            en.removeLast()
            if en.isEmpty { setMarked(""); view?.showSuggestions([], [], []); updateAutoShift() } else { showEnglish() }
            return
        }
        // right after Space corrected a word: put back what was typed, and leave it as it is
        if let u = enUndo {
            enUndo = nil
            if !typingBangla() && t.contextBefore.hasSuffix(u.placed + " ") {
                for _ in 0..<(u.placed.count + 1) { t.deleteBackward() }
                en = u.typed; enKeep = u.typed
                showEnglish()
                return
            }
        }
        t.deleteBackward()
        if !typingBangla() { updateAutoShift() }
    }

    func onEnter() {
        guard let t = target else { return }
        if last.composing { apply(Engine.commit()) }
        if !en.isEmpty { placeEnglish(correct: false); view?.showSuggestions([], [], []) }
        enUndo = nil
        t.insert("\n")
        updateAutoShift()
    }

    func onCandidate(_ index: Int) {
        guard let t = target else { return }
        if !en.isEmpty {
            guard index < enShown.count else { return }
            let word = enShown[index]
            learnEnglish(word)
            t.setMarked(word); t.unmark(); marked = false
            t.insert(" ")
            en = ""; enUndo = nil; enKeep = nil; enHighlight = 0
            lastSpaceAt = Date()
            showNextEnglish(); updateAutoShift()
            return
        }
        guard last.composing else { return }
        apply(Engine.select(index, trailing: " "))
        lastSpaceAt = Date()
    }

    func onPrediction(_ word: String) {
        guard let t = target else { return }
        if !typingBangla() {
            let w = view?.shift == .on ? word.prefix(1).uppercased() + word.dropFirst() : word
            learnEnglish(w)
            t.insert(w + " ")
            showNextEnglish(); updateAutoShift()
            return
        }
        t.insert(word + " ")
        Engine.reset(); last = Output()
        view?.showSuggestions([], [], [])
    }

    func onLanguageToggle() {
        finishWord()
        if plainField { view?.stripMessage = "এই ঘরে শুধু ইংরেজি · English only here"; view?.setNeedsDisplay(); return }
        bangla.toggle()
        prefs.bangla = bangla
        view?.bangla = bangla
        view?.shift = .off
        updateAutoShift()
    }

    /// A word is being typed (Bangla or English)
    var composing: Bool { last.composing || !en.isEmpty }

    /// Tab / Shift+Tab on a physical keyboard: move through the suggestions; false if there are none
    @discardableResult
    func moveHighlight(back: Bool = false) -> Bool {
        let n = min(last.composing ? last.candidates.count : enShown.count, 9)
        guard n > 1 else { return false }
        if last.composing {
            let cur = highlight > 0 ? highlight : last.selected
            highlight = (cur + (back ? n - 1 : 1)) % n
            view?.selected = highlight
        } else {
            // English: Tab places the next suggestion's word on Space, through the same strip
            enHighlight = (enHighlight + (back ? n - 1 : 1)) % n
            view?.selected = enHighlight
        }
        return true
    }
    private var enHighlight = 0

    /// Enter or Esc on a physical keyboard: place the word as highlighted, with nothing after it
    func placeHighlighted() {
        if last.composing { apply(Engine.select(highlight > 0 ? highlight : last.selected)); return }
        if !en.isEmpty {
            if enHighlight > 0, enHighlight < enShown.count { let w = enShown[enHighlight]; target?.setMarked(w); target?.unmark(); marked = false; en = ""; enHighlight = 0; view?.showSuggestions([], [], []) }
            else { placeEnglish(correct: false); view?.showSuggestions([], [], []) }
        }
    }

    func onHelp() { finishWord(); onHelpAction?() }
    func onGlobe() { finishWord(); onGlobeAction?() }
    func onCursorMove(_ steps: Int) { finishWord(); target?.moveCursor(steps) }
    func onSettings() { finishWord(); onSettingsAction?() }
    func onHide() { finishWord(); onHideAction?() }
    func onEmoji(_ e: String) { finishWord(); target?.insert(e) }
    func onKeyFeedback() { onClick?() }

    // ───────────────────────── English help ─────────────────────────

    /// The text before the word being typed (the app may or may not count the word itself)
    private func headBefore(_ word: String) -> String {
        let ctx = target?.contextBefore ?? ""
        return marked && ctx.hasSuffix(word) ? String(ctx.dropLast(word.count)) : ctx
    }

    /// The word being typed follows a mark with no space (a.b, x@y, #z): an address or a code
    private func gluedToText(_ word: String) -> Bool {
        guard let c = headBefore(word).last else { return false }
        return !c.isWhitespace && c != "(" && c != "\"" && c != "“"
    }

    /// The word being typed starts a sentence (or the text)
    private func sentenceStart(_ word: String) -> Bool {
        let head = headBefore(word).trimmingCharacters(in: .whitespaces)
        guard let c = head.last else { return true }
        return ".!?\n।".contains(c)
    }

    /// What Space should place instead of what was typed, if it's a clear typo (or "dont", "i")
    func englishFix(_ typed: String) -> String? {
        let E = English.shared
        guard prefs.enAutocorrect, typed != enKeep else { return nil }
        if gluedToText(typed) { return nil }
        if E.isContraction(typed), let c = E.canonical(typed) { return c }
        // Banglish written in English mode (ami, tomake, bhalobashi) stays as it is
        if Engine.isBanglish(typed) { return nil }
        if let c = E.canonical(typed) { return c }
        if typed.count < 3 || E.known(typed) { return nil }
        if typed.dropFirst().contains(where: { $0.isUppercase }) { return nil }
        // a capital letter in the middle of a sentence is most likely a name
        if typed.first?.isUppercase == true && !sentenceStart(typed) { return nil }
        return E.best(E.corrections(typed, max: 3))
    }

    private func showEnglish() {
        let E = English.shared
        setMarked(en)
        let fix = englishFix(en)
        var list: [String] = fix.map { [$0, en] } ?? [en]
        list += E.complete(en, max: 3)
        if !E.known(en) { list += E.corrections(en, max: 2) }
        var seen = Set<String>()
        enShown = Array(list.filter { !$0.isEmpty && seen.insert($0).inserted }.prefix(6))
        enHighlight = 0
        view?.showSuggestions(enShown.map { Candidate(text: $0, kind: "") }, [], [])
    }

    /// Place the English word being typed (corrected if asked and it's a clear typo)
    private func placeEnglish(correct: Bool) {
        let typed = en
        let word = (correct ? englishFix(typed) : nil) ?? typed
        learnEnglish(word)
        target?.setMarked(word); target?.unmark(); marked = false
        enUndo = word != typed ? (typed, word) : nil
        if typed != enKeep { enKeep = nil }
        en = ""
    }

    private func learnEnglish(_ word: String) {
        if privateField { return }
        // the word before this one (the word being typed itself is left out)
        English.shared.learn(word, prev: previousWords(excluding: en.isEmpty ? nil : en).1)
    }

    /// The two words before the cursor (not counting the word being typed)
    private func previousWords(excluding word: String? = nil) -> (String?, String?) {
        var before = target?.contextBefore ?? ""
        if let w = word, marked, before.hasSuffix(w) { before = String(before.dropLast(w.count)) }
        let tail = before.trimmingCharacters(in: .whitespaces)
        if let c = tail.last, ".!?".contains(c) { return (nil, nil) }
        let ws = before.split(whereSeparator: { !($0.isLetter || $0 == "'") }).map(String.init)
        return (ws.count >= 2 ? ws[ws.count - 2] : nil, ws.last)
    }

    private func showNextEnglish() {
        guard prefs.nextWordEnglish else { view?.showSuggestions([], [], []); return }
        let (a, b) = previousWords()
        view?.showSuggestions([], [], English.shared.nextWords(a, b))
    }
}
