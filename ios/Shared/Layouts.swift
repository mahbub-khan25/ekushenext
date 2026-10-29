import Foundation

/// What a key does.
enum KeyType { case char, shift, delete, enter, space, symbols, moreSymbols, letters, lang, emoji, globe, hide, spacer, splitGap }

/// Which keyboard to draw: the phone's, the iPad's, or the iPad's split in two halves.
enum FormFactor { case phone, pad, padSplit }

/// A long-press choice: what it shows, what it types, and a small note under it.
struct Alt: Equatable {
    let label: String
    let text: String
    var note: String? = nil
}

struct Key: Equatable {
    let type: KeyType
    let label: String
    var text: String
    var width: CGFloat = 1
    /// The Bangla letter this key gives in phonetic typing (shown small), and with Shift
    var hint: String? = nil
    var hintShift: String? = nil
    var alts: [Alt] = []
    var shiftAlts: [Alt]? = nil
    /// Letters follow Shift; symbols do not
    var letter = false

    init(_ type: KeyType, _ label: String, _ text: String? = nil, width: CGFloat = 1, hint: String? = nil, hintShift: String? = nil, alts: [Alt] = [], letter: Bool = false) {
        self.type = type; self.label = label; self.text = text ?? label; self.width = width
        self.hint = hint; self.hintShift = hintShift; self.alts = alts; self.letter = letter
    }

    /// An id for tests and logs: the character, or the kind of key
    var id: String { type == .char ? text : "\(type)".uppercased() }
    /// Takes space in its row but isn't a key (the iPad's indented second row, the split's middle)
    var isGap: Bool { type == .spacer || type == .splitGap }
}

/// The same layouts as the Android keyboard.
enum Layouts {
    private static func ch(_ c: Character, _ hint: String?, _ hintShift: String? = nil, alts: [Alt] = []) -> Key {
        Key(.char, String(c), String(c), hint: hint, hintShift: hintShift ?? hint, alts: alts, letter: true)
    }

    private static let bnDigits = Array("০১২৩৪৫৬৭৮৯")
    static func digitLabel(_ d: Character, bangla: Bool) -> String {
        guard bangla, let v = d.wholeNumberValue, d.isASCII else { return String(d) }
        return String(bnDigits[v])
    }

    // Long-press choices: capital letters change the sound in phonetic typing, so they are one
    // press away, with the marks for exact spellings
    private static let altT = [Alt(label: "T", text: "T", note: "ট"), Alt(label: "t``", text: "t``", note: "ৎ"), Alt(label: "th", text: "th", note: "থ")]
    private static let altD = [Alt(label: "D", text: "D", note: "ড"), Alt(label: "dh", text: "dh", note: "ধ")]
    private static let altN = [Alt(label: "N", text: "N", note: "ণ"), Alt(label: "ng", text: "ng", note: "ং"), Alt(label: "NG", text: "NG", note: "ঞ")]
    private static let altS = [Alt(label: "sh", text: "sh", note: "শ"), Alt(label: "Sh", text: "Sh", note: "ষ")]
    private static let altR = [Alt(label: "R", text: "R", note: "ড়"), Alt(label: "rr", text: "rr", note: "রেফ"), Alt(label: "rri", text: "rri", note: "ঋ")]
    private static let altG = [Alt(label: "Ng", text: "Ng", note: "ঙ"), Alt(label: "gh", text: "gh", note: "ঘ")]
    private static let altO = [Alt(label: "O", text: "O", note: "ও"), Alt(label: "oi", text: "oi", note: "ঐ"), Alt(label: "ou", text: "ou", note: "ঔ")]
    private static let altI = [Alt(label: "I", text: "I", note: "ঈ")]
    private static let altU = [Alt(label: "U", text: "U", note: "ঊ")]
    private static let altZ = [Alt(label: "Z", text: "Z", note: "্য")]
    private static let altH = [Alt(label: ":", text: ":", note: "ঃ")]
    private static let altK = [Alt(label: "kh", text: "kh", note: "খ"), Alt(label: "kkh", text: "kkh", note: "ক্ষ")]
    private static let altC = [Alt(label: "ch", text: "ch", note: "ছ")]
    private static let altP = [Alt(label: "ph", text: "ph", note: "ফ")]
    private static let altB = [Alt(label: "bh", text: "bh", note: "ভ")]
    private static let altJ = [Alt(label: "jh", text: "jh", note: "ঝ")]
    private static let altE = [Alt(label: "^", text: "^", note: "ঁ")]
    private static let altA = [Alt(label: "^", text: "^", note: "ঁ")]

    private static let numberRow = "1234567890".map { Key(.char, String($0)) }

    /// Phonetic QWERTY, with the Bangla letter of each key shown small
    static func letters(numberRow: Bool, emojiKey: Bool, globe: Bool) -> [[Key]] {
        var rows: [[Key]] = []
        if numberRow { rows.append(self.numberRow) }
        rows.append([
            ch("q", "ক"), ch("w", "ও"), ch("e", "এ", alts: altE), ch("r", "র", "ড়", alts: altR), ch("t", "ত", "ট", alts: altT),
            ch("y", "য়"), ch("u", "উ", "ঊ", alts: altU), ch("i", "ই", "ঈ", alts: altI), ch("o", "অ", "ও", alts: altO), ch("p", "প", alts: altP),
        ])
        rows.append([
            ch("a", "আ", alts: altA), ch("s", "স", alts: altS), ch("d", "দ", "ড", alts: altD), ch("f", "ফ"), ch("g", "গ", alts: altG),
            ch("h", "হ", alts: altH), ch("j", "জ", alts: altJ), ch("k", "ক", alts: altK), ch("l", "ল"),
        ])
        rows.append([Key(.shift, "⇧", width: 1.5)] + [
            ch("z", "য", "্য", alts: altZ), ch("x", "ক্স"), ch("c", "চ", alts: altC), ch("v", "ভ"), ch("b", "ব", alts: altB),
            ch("n", "ন", "ণ", alts: altN), ch("m", "ম"),
        ] + [Key(.delete, "⌫", width: 1.5)])
        rows.append(bottomRow(.symbols, "?১২৩", emojiKey: emojiKey, globe: globe))
        return rows
    }

    private static let commaAlts = [Alt(label: ",,", text: ",,", note: "হসন্ত"), Alt(label: "+", text: "+", note: "যুক্ত"), Alt(label: "`", text: "`", note: "আলাদা"), Alt(label: ";", text: ";"), Alt(label: "'", text: "'"), Alt(label: "\"", text: "\""), Alt(label: "-", text: "-")]
    private static let periodAlts = [Alt(label: "..", text: "..", note: "বিন্দু"), Alt(label: "?", text: "?"), Alt(label: "!", text: "!"), Alt(label: ":", text: ":", note: "ঃ"), Alt(label: "^", text: "^", note: "ঁ"), Alt(label: "/", text: "/", note: "English"), Alt(label: "$", text: "$", note: "৳")]

    /// The bottom row: the layout switch, Bangla/English, the globe (when iOS asks for it), comma,
    /// the emoji key, Space, full stop and Return. Always ten units wide.
    static func bottomRow(_ switchType: KeyType, _ switchLabel: String, emojiKey: Bool, globe: Bool) -> [Key] {
        var row: [Key] = [Key(switchType, switchLabel, width: 1.3), Key(.lang, "বাং")]
        var space: CGFloat = 4.1
        if globe { row.append(Key(.globe, "🌐")); space -= 1 }
        row.append(Key(.char, ",", ",", alts: commaAlts))
        if emojiKey { row.append(Key(.emoji, "☺")); space -= 1 }
        row.append(Key(.space, " ", " ", width: max(2.4, space + 0.3)))
        row.append(Key(.char, ".", ".", alts: periodAlts))
        row.append(Key(.enter, "↵", "\n", width: 1.3))
        return row
    }

    /// Digits and the common symbols
    static func symbols(emojiKey: Bool, globe: Bool) -> [[Key]] {
        [
            "1234567890".map { Key(.char, String($0)) },
            ["@", "#", "৳", "_", "&", "-", "+", "(", ")", "/"].map {
                Key(.char, $0, $0, alts: $0 == "৳" ? [Alt(label: "$", text: "$"), Alt(label: "€", text: "€"), Alt(label: "£", text: "£"), Alt(label: "¥", text: "¥"), Alt(label: "₹", text: "₹")] : [])
            },
            [Key(.moreSymbols, "=\\<", width: 1.5)] + ["*", "\"", "'", ":", ";", "!", "?"].map { Key(.char, $0) } + [Key(.delete, "⌫", width: 1.5)],
            bottomRow(.letters, "ABC", emojiKey: emojiKey, globe: globe),
        ]
    }

    static func moreSymbols(emojiKey: Bool, globe: Bool) -> [[Key]] {
        [
            ["~", "`", "|", "•", "√", "π", "÷", "×", "¶", "∆"].map { Key(.char, $0) },
            ["£", "€", "¥", "^", "°", "=", "{", "}", "\\", "%"].map { Key(.char, $0) },
            [Key(.symbols, "?১২৩", width: 1.5)] + ["©", "®", "™", "✓", "[", "]", "<"].map { Key(.char, $0) } + [Key(.delete, "⌫", width: 1.5)],
            bottomRow(.letters, "ABC", emojiKey: emojiKey, globe: globe),
        ]
    }

    /// Number boxes: a full-width pad, four columns
    static func numbers() -> [[Key]] {
        let d = { (t: String) in Key(.char, t) }
        return [
            [d("1"), d("2"), d("3"), d("-")],
            [d("4"), d("5"), d("6"), d(",")],
            [d("7"), d("8"), d("9"), Key(.delete, "⌫")],
            [Key(.letters, "ABC"), d("0"), d("."), Key(.enter, "↵", "\n")],
        ]
    }

    // ───────────────────────── iPad ─────────────────────────
    // Apple's iPad proportions: every row 11.5 units, Delete at the top right, a wide Return on the
    // second row (indented half a key), Shift on both sides with , and . beside the right one, and a
    // key to hide the keyboard at the bottom right.

    private static func gap(_ w: CGFloat) -> Key { Key(.spacer, "", width: w) }
    private static func split() -> Key { Key(.splitGap, "", width: 0) }

    private static var letterRow1: [Key] {
        [ch("q", "ক"), ch("w", "ও"), ch("e", "এ", alts: altE), ch("r", "র", "ড়", alts: altR), ch("t", "ত", "ট", alts: altT),
         ch("y", "য়"), ch("u", "উ", "ঊ", alts: altU), ch("i", "ই", "ঈ", alts: altI), ch("o", "অ", "ও", alts: altO), ch("p", "প", alts: altP)]
    }
    private static var letterRow2: [Key] {
        [ch("a", "আ", alts: altA), ch("s", "স", alts: altS), ch("d", "দ", "ড", alts: altD), ch("f", "ফ"), ch("g", "গ", alts: altG),
         ch("h", "হ", alts: altH), ch("j", "জ", alts: altJ), ch("k", "ক", alts: altK), ch("l", "ল")]
    }
    private static var letterRow3: [Key] {
        [ch("z", "য", "্য", alts: altZ), ch("x", "ক্স"), ch("c", "চ", alts: altC), ch("v", "ভ"), ch("b", "ব", alts: altB),
         ch("n", "ন", "ণ", alts: altN), ch("m", "ম")]
    }
    private static var comma: Key { Key(.char, ",", ",", alts: commaAlts) }
    private static var period: Key { Key(.char, ".", ".", alts: periodAlts) }

    /// The iPad's bottom row: layout switch, Bangla/English, the globe (when iOS asks), emoji,
    /// Space, the layout switch again and the hide key
    private static func padBottom(_ switchType: KeyType, _ switchLabel: String, emojiKey: Bool, globe: Bool) -> [Key] {
        var row: [Key] = [Key(switchType, switchLabel, width: 1.5), Key(.lang, "বাং")]
        var space: CGFloat = 11.5 - 1.5 - 1 - 1.5 - 1.25
        if globe { row.append(Key(.globe, "🌐")); space -= 1 }
        if emojiKey { row.append(Key(.emoji, "☺")); space -= 1 }
        row.append(Key(.space, " ", " ", width: space))
        row.append(Key(switchType, switchLabel, width: 1.5))
        row.append(Key(.hide, "⌄", width: 1.25))
        return row
    }

    static func padLetters(numberRow: Bool, emojiKey: Bool, globe: Bool) -> [[Key]] {
        var rows: [[Key]] = []
        if numberRow { rows.append("1234567890".map { Key(.char, String($0)) } + [Key(.char, "-", "-"), gap(0.5)]) }
        rows.append(letterRow1 + [Key(.delete, "⌫", width: 1.5)])
        rows.append([gap(0.5)] + letterRow2 + [Key(.enter, "↵", "\n", width: 2)])
        rows.append([Key(.shift, "⇧", width: 1)] + letterRow3 + [comma, period, Key(.shift, "⇧", width: 1.5)])
        rows.append(padBottom(.symbols, "?১২৩", emojiKey: emojiKey, globe: globe))
        return rows
    }

    static func padSymbols(emojiKey: Bool, globe: Bool) -> [[Key]] {
        [
            "1234567890".map { Key(.char, String($0)) } + [Key(.delete, "⌫", width: 1.5)],
            [gap(0.5)] + ["@", "#", "৳", "&", "*", "(", ")", "'", "\""].map {
                Key(.char, $0, $0, alts: $0 == "৳" ? [Alt(label: "$", text: "$"), Alt(label: "€", text: "€"), Alt(label: "£", text: "£"), Alt(label: "¥", text: "¥"), Alt(label: "₹", text: "₹")] : [])
            } + [Key(.enter, "↵", "\n", width: 2)],
            [Key(.moreSymbols, "#+=", width: 1)] + ["%", "-", "+", "=", "/", ";", ":", ",", "."].map { Key(.char, $0) } + [Key(.moreSymbols, "#+=", width: 1.5)],
            padBottom(.letters, "ABC", emojiKey: emojiKey, globe: globe),
        ]
    }

    static func padMoreSymbols(emojiKey: Bool, globe: Bool) -> [[Key]] {
        [
            ["~", "`", "|", "•", "√", "π", "÷", "×", "¶", "∆"].map { Key(.char, $0) } + [Key(.delete, "⌫", width: 1.5)],
            [gap(0.5)] + ["£", "€", "¥", "_", "^", "[", "]", "{", "}"].map { Key(.char, $0) } + [Key(.enter, "↵", "\n", width: 2)],
            [Key(.symbols, "?১২৩", width: 1)] + ["\\", "<", ">", "©", "®", "™", "✓", "!", "?"].map { Key(.char, $0) } + [Key(.symbols, "?১২৩", width: 1.5)],
            padBottom(.letters, "ABC", emojiKey: emojiKey, globe: globe),
        ]
    }

    /// The iPad split in two, for thumbs (landscape): the left hand's keys at the left edge, the right
    /// hand's at the right, the suggestion strip still across the top. The middle is empty.
    static func padSplitLetters(emojiKey: Bool, globe: Bool) -> [[Key]] {
        let r1 = letterRow1, r2 = letterRow2, r3 = letterRow3
        var bottomLeft: [Key] = [Key(.symbols, "?১২৩", width: 1.5), Key(.lang, "বাং")]
        if globe { bottomLeft.append(Key(.globe, "🌐")) }
        bottomLeft.append(Key(.space, " ", " ", width: globe ? 2.5 : 3.5))
        var bottomRight: [Key] = [Key(.space, " ", " ", width: emojiKey ? 2.75 : 3.75)]
        if emojiKey { bottomRight.append(Key(.emoji, "☺")) }
        bottomRight += [Key(.symbols, "?১২৩", width: 1.5), Key(.hide, "⌄", width: 1.25)]
        return [
            Array(r1[0..<5]) + [split()] + Array(r1[5...]) + [Key(.delete, "⌫", width: 1.5)],
            [gap(0.5)] + Array(r2[0..<5]) + [split()] + Array(r2[5...]) + [Key(.enter, "↵", "\n", width: 2)],
            [Key(.shift, "⇧", width: 1)] + Array(r3[0..<5]) + [split()] + Array(r3[5...]) + [comma, period, Key(.shift, "⇧", width: 1.5)],
            bottomLeft + [split()] + bottomRight,
        ]
    }

    static func padSplitSymbols(more: Bool, emojiKey: Bool, globe: Bool) -> [[Key]] {
        let full = more ? padMoreSymbols(emojiKey: emojiKey, globe: globe) : padSymbols(emojiKey: emojiKey, globe: globe)
        let bottom = padSplitLetters(emojiKey: emojiKey, globe: globe)[3].map { k -> Key in
            k.type == .symbols ? Key(.letters, "ABC", width: k.width) : k
        }
        // each row in two at its middle key (the iPad rows are already 11.5 units)
        func halve(_ row: [Key]) -> [Key] {
            var units: CGFloat = 0
            for (i, k) in row.enumerated() {
                units += k.width
                if units >= 5.5 { return Array(row[...i]) + [split()] + Array(row[(i + 1)...]) }
            }
            return row
        }
        return Array(full.prefix(3)).map(halve) + [bottom]
    }
}
