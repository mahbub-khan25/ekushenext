import Cocoa
import InputMethodKit

/// One controller per text field the user types into. Keys go to the Rust engine; the word being
/// typed is shown as marked (underlined) text, suggestions in the system candidate window.
@objc(EkusheInputController)
final class EkusheInputController: IMKInputController {
    private var output = EngineOutput.empty
    private var selected = 0
    /// The last word committed with a space (what Backspace-undo expects before the caret)
    private var lastCommitted = ""
    /// Shift went down with no other key since (for Double-Tap Shift)
    private var shiftClean = false

    private let notFound = NSRange(location: NSNotFound, length: NSNotFound)

    // MARK: Lifecycle

    override func activateServer(_ sender: Any!) {
        super.activateServer(sender)
        Engine.reloadConfig()
        if let client = sender as? IMKTextInput, let id = client.bundleIdentifier() {
            Engine.appFocus(id)
        }
    }

    override func deactivateServer(_ sender: Any!) {
        if let client = sender as? IMKTextInput { finish(client) }
        hideCandidates()
        Engine.saveLearned()
        super.deactivateServer(sender)
    }

    /// The system asks us to finish (click elsewhere, focus change): keep the text as shown.
    override func commitComposition(_ sender: Any!) {
        if let client = sender as? IMKTextInput { finish(client) }
    }

    override func recognizedEvents(_ sender: Any!) -> Int {
        Int(NSEvent.EventTypeMask([.keyDown, .flagsChanged]).rawValue)
    }

    // MARK: Keys

    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        guard let event = event, let client = sender as? IMKTextInput else { return false }

        if event.type == .flagsChanged {
            handleShift(event, client)
            return false
        }
        guard event.type == .keyDown else { return false }
        shiftClean = false

        let mods = event.modifierFlags.intersection(.deviceIndependentFlagsMask)
        let composing = Engine.isComposing

        // Control+Delete flips the last word between Bangla and Roman
        if mods.contains(.control) && event.keyCode == 51 && !composing {
            return flipLast(client)
        }
        // Shortcuts and Option characters belong to the application
        if mods.contains(.command) || mods.contains(.control) || mods.contains(.option) {
            if composing { finish(client) }
            return false
        }
        if !Engine.isBangla { return false }

        switch event.keyCode {
        case 49: // Space
            if composing { commit(selected, trailing: " ", client); return true }
            return false
        case 36, 76: // Return, Enter: write the word, then the application gets its Return
            if composing { commit(selected, trailing: nil, client) }
            return false
        case 51: // Delete (Backspace)
            if composing { apply(Engine.backspace(), client); return true }
            if Engine.canUndo { return undo(client) }
            return false
        case 53: // Escape: keep what is shown
            if composing { finish(client); return true }
            return false
        case 48: // Tab: next suggestion
            if composing && output.candidates.count > 1 { cycle(client); return true }
            if composing { finish(client) }
            return false
        case 115, 116, 117, 119, 121, 123, 124, 125, 126: // Home, PgUp, Fwd-Del, End, PgDn, arrows
            if composing { finish(client) }
            return false
        default:
            break
        }

        guard let chars = event.characters, chars.unicodeScalars.count == 1,
              let scalar = chars.unicodeScalars.first,
              scalar.value >= 0x20, scalar.value != 0x7F,
              !(0xF700...0xF8FF).contains(scalar.value) // function keys
        else {
            if composing { finish(client) }
            return false
        }
        apply(Engine.key(scalar.value), client)
        return true
    }

    private func handleShift(_ event: NSEvent, _ client: IMKTextInput) {
        guard event.keyCode == 56 || event.keyCode == 60 else { shiftClean = false; return }
        let mods = event.modifierFlags.intersection(.deviceIndependentFlagsMask)
        if mods.contains(.shift) {
            shiftClean = mods == .shift
        } else if shiftClean {
            shiftClean = false
            if let bangla = Engine.shiftTap() {
                if !bangla { finish(client) }
                NSSound.beep() // a small cue; the menu-bar icon stays "Ekushe"
            }
        }
    }

    // MARK: Text

    private func apply(_ out: EngineOutput, _ client: IMKTextInput) {
        if let committed = out.committed {
            client.insertText(committed, replacementRange: notFound)
            lastCommitted = committed.hasSuffix(" ") ? committed : ""
            output = .empty
            selected = 0
            hideCandidates()
            return
        }
        output = out
        selected = min(out.selected_index, max(0, out.candidates.count - 1))
        showMarked(out.preedit, client)
        if out.candidates.count > 1 { showCandidates() } else { hideCandidates() }
    }

    private func showMarked(_ text: String, _ client: IMKTextInput) {
        let attrs = mark(forStyle: kTSMHiliteSelectedConvertedText, at: NSRange(location: 0, length: (text as NSString).length))
        let s = NSAttributedString(string: text, attributes: attrs as? [NSAttributedString.Key: Any] ?? [:])
        client.setMarkedText(s, selectionRange: NSRange(location: (text as NSString).length, length: 0), replacementRange: notFound)
    }

    private func commit(_ index: Int, trailing: Character?, _ client: IMKTextInput) {
        let out = Engine.select(index, trailing: trailing)
        let text = out.committed ?? output.preedit + (trailing.map(String.init) ?? "")
        client.insertText(text, replacementRange: notFound)
        lastCommitted = trailing == " " ? text : ""
        output = .empty
        selected = 0
        hideCandidates()
    }

    /// Finish the word as it is shown (highlighted suggestion), without adding anything.
    private func finish(_ client: IMKTextInput) {
        guard Engine.isComposing else { return }
        commit(selected, trailing: nil, client)
        Engine.reset()
    }

    private func cycle(_ client: IMKTextInput) {
        let n = output.candidates.count
        selected = (selected + 1) % n
        showMarked(output.candidates[selected].text, client)
        if let panel = AppShared.candidates {
            panel.selectCandidate(withIdentifier: panel.candidateIdentifier(atLineNumber: selected))
        }
    }

    /// Backspace right after a committed word: give back the Roman letters, but only if the text
    /// before the caret is still that word (otherwise it is an ordinary Backspace).
    private func undo(_ client: IMKTextInput) -> Bool {
        let expect = lastCommitted
        let n = (expect as NSString).length
        let sel = client.selectedRange()
        guard n > 0, sel.location != NSNotFound, sel.length == 0, sel.location >= n else {
            Engine.reset()
            return false
        }
        let range = NSRange(location: sel.location - n, length: n)
        guard client.attributedSubstring(from: range)?.string == expect else {
            Engine.reset()
            return false
        }
        let out = Engine.backspace()
        guard let roman = out.committed else { return false }
        client.insertText(roman, replacementRange: range)
        lastCommitted = ""
        return true
    }

    private func flipLast(_ client: IMKTextInput) -> Bool {
        let expect = lastCommitted
        guard let (n, text) = Engine.flipLast() else { return false }
        let sel = client.selectedRange()
        guard sel.location != NSNotFound, sel.length == 0, sel.location >= n else { return false }
        let range = NSRange(location: sel.location - n, length: n)
        guard client.attributedSubstring(from: range)?.string == expect else { return false }
        client.insertText(text, replacementRange: range)
        lastCommitted = text
        return true
    }

    // MARK: Candidate window

    override func candidates(_ sender: Any!) -> [Any]! {
        output.candidates.map { $0.text }
    }

    override func candidateSelected(_ candidateString: NSAttributedString!) {
        guard let client = self.client(), let s = candidateString?.string,
              let i = output.candidates.firstIndex(where: { $0.text == s }) else { return }
        commit(i, trailing: " ", client)
    }

    override func candidateSelectionChanged(_ candidateString: NSAttributedString!) {
        guard let client = self.client(), let s = candidateString?.string,
              let i = output.candidates.firstIndex(where: { $0.text == s }) else { return }
        selected = i
        showMarked(s, client)
    }

    private func showCandidates() {
        guard let panel = AppShared.candidates else { return }
        panel.update()
        panel.show(kIMKLocateCandidatesBelowHint)
        if selected > 0 {
            panel.selectCandidate(withIdentifier: panel.candidateIdentifier(atLineNumber: selected))
        }
    }

    private func hideCandidates() {
        AppShared.candidates?.hide()
    }

    // MARK: Menu (the Ekushe item in the menu-bar input menu)

    override func menu() -> NSMenu! {
        let menu = NSMenu(title: "Ekushe")
        let studio = NSMenuItem(title: "Open Ekushe Studio…", action: #selector(openStudio(_:)), keyEquivalent: "")
        studio.target = self
        menu.addItem(studio)
        let settings = NSMenuItem(title: "Settings…", action: #selector(openSettings(_:)), keyEquivalent: "")
        settings.target = self
        menu.addItem(settings)
        menu.addItem(.separator())
        let lang = NSMenuItem(title: Engine.isBangla ? "Type English for now (double-tap Shift)" : "Back to Bangla (double-tap Shift)",
                              action: #selector(toggleLanguage(_:)), keyEquivalent: "")
        lang.target = self
        menu.addItem(lang)
        return menu
    }

    @objc func openStudio(_ sender: Any?) {
        if let url = Engine.studioURL() { NSWorkspace.shared.open(url) }
    }

    @objc func openSettings(_ sender: Any?) {
        if let url = Engine.studioURL(tab: "settings") { NSWorkspace.shared.open(url) }
    }

    @objc func toggleLanguage(_ sender: Any?) {
        Engine.toggleLanguage()
    }
}
