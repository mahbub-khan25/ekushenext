import Foundation

/// The writing pad (in the app): what a keypress becomes. A physical keyboard, or Apple's own
/// on-screen keyboard, sends English letters; they go through the same typing logic as Ekushe's
/// keyboard (Typist), so `amar` becomes আমার with the same suggestions. Ekushe's own keyboard
/// already sends Bangla, so with it the pad simply takes what arrives.
final class PadRouter {
    let typist: Typist
    init(typist: Typist) { self.typist = typist }

    /// The text box is about to replace `rangeLength` characters with `text`. Returns true when the
    /// text box should make the change itself (Ekushe has nothing to do with it).
    func shouldChange(_ text: String, rangeLength: Int, translate: Bool) -> Bool {
        guard translate else { return true }
        if text.isEmpty {
            // Backspace: the word being typed, reopening the last word, or a plain delete
            if rangeLength <= 1 { typist.onDelete(); return false }
            typist.finishWord(); return true
        }
        if rangeLength > 0 && !typist.composing { return true }   // typing over a selection
        switch text {
        case " ": typist.onSpace(); return false
        case "\n": typist.onEnter(); return false
        case "\t": if typist.moveHighlight() { return false }; typist.finishWord(); return true
        default: break
        }
        if text.unicodeScalars.count == 1, let c = text.unicodeScalars.first, c.value >= 0x21, c.value < 0x7F {
            typist.onText(text); return false
        }
        // pasted text, Bangla from another keyboard, an emoji: the word is finished and it goes in as it is
        typist.finishWord()
        return true
    }

    /// Tab (Shift+Tab goes back) on a physical keyboard; false when there are no suggestions
    func tab(back: Bool) -> Bool { typist.moveHighlight(back: back) }

    /// Esc: place the word as shown
    func escape() -> Bool {
        guard typist.composing else { return false }
        typist.placeHighlighted(); return true
    }

    /// Shift tapped twice in a row: Bangla ⇄ English, as on a computer
    private var lastShift = Date.distantPast
    func shiftTapped(at now: Date = Date()) {
        if now.timeIntervalSince(lastShift) < 0.4 { lastShift = .distantPast; typist.onLanguageToggle() } else { lastShift = now }
    }
    func otherKey() { lastShift = .distantPast }
}
