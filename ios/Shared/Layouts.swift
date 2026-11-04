import Foundation

/// What a key does.
enum KeyType { case char, shift, delete, enter, space, symbols, moreSymbols, letters, lang, emoji, globe }

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

    init(_ type: KeyType, _ label: String, _ text: String? = nil, width: CGFloat = 1, hint: String? = nil, hintShift: String? = nil, alts: [Alt] = [], shiftAlts: [Alt]? = nil, letter: Bool = false) {
        self.type = type; self.label = label; self.text = text ?? label; self.width = width
        self.hint = hint; self.hintShift = hintShift; self.alts = alts; self.shiftAlts = shiftAlts; self.letter = letter
    }

    /// An id for tests and logs: the character, or the kind of key
    var id: String { type == .char ? text : "\(type)".uppercased() }
}

/// The same layouts as the Android keyboard.
enum Layouts {
    private static func ch(_ c: Character, _ hint: String?, _ hintShift: String? = nil, alts: [Alt] = []) -> Key {
        Key(.char, String(c), String(c), hint: hint, hintShift: hintShift ?? hint, alts: alts, letter: true)
    }

    private static let bnDigits = Array("০১২৩৪৫৬৭৮৯")
    /// Before a mark that is typed exactly as it is, even while typing Bangla ("$" there is ৳)
    static let literal: Character = "\u{2}"
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
    static func letters(numberRow: Bool, globe: Bool) -> [[Key]] {
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
        rows.append(bottomRow(.symbols, "?১২৩", globe: globe))
        return rows
    }

    // ── English: digits on the top row's long-press (shown small), accents on the vowels
    private static let accents: [Character: String] = [
        "e": "éèêëēę", "y": "ýÿ", "u": "úùûüū", "i": "íìîïī", "o": "óòôöõøœō",
        "a": "áàâäãåæā", "s": "ßśš", "d": "ð", "l": "ł", "z": "žźż", "c": "çćč", "n": "ñń", "g": "ğ",
    ]

    /// An email or web address field
    enum Address { case none, email, web }

    /// English QWERTY: no Bangla, long-press for a digit (top row) and accented letters
    static func englishLetters(numberRow: Bool, globe: Bool, address: Address = .none) -> [[Key]] {
        func en(_ c: Character, _ digit: Character? = nil) -> Key {
            let acc = Array(accents[c] ?? "")
            let d = (digit != nil && !numberRow) ? [Alt(label: String(digit!), text: String(digit!))] : []
            return Key(.char, String(c), String(c), hint: numberRow ? nil : digit.map { String($0) },
                       alts: d + acc.map { Alt(label: String($0), text: String($0)) },
                       shiftAlts: d + acc.map { Alt(label: $0.uppercased(), text: $0.uppercased()) }, letter: true)
        }
        var rows: [[Key]] = []
        if numberRow { rows.append(self.numberRow) }
        rows.append(zip("qwertyuiop", "1234567890").map { en($0, $1) })
        rows.append("asdfghjkl".map { en($0) })
        rows.append([Key(.shift, "⇧", width: 1.5)] + "zxcvbnm".map { en($0) } + [Key(.delete, "⌫", width: 1.5)])
        rows.append(address == .none ? bottomRow(.symbols, "?১২৩", globe: globe, english: true) : addressRow(address, globe: globe))
        return rows
    }

    private static let domains = [".com", ".net", ".org", ".bd", ".com.bd", ".edu", ".io", ".ca", ".co.uk", ".info"].map { Alt(label: $0, text: $0) }

    /// In an address field: @ (email) or / (web) and .com where the comma was, a smaller Space
    private static func addressRow(_ a: Address, globe: Bool) -> [Key] {
        var row: [Key] = [Key(.symbols, "?১২৩", width: 1.5)]
        var space: CGFloat = 3.5
        if globe { row.append(Key(.globe, "🌐")); space -= 1 }
        row.append(a == .email
            ? Key(.char, "@", "@", alts: ["_", "-", "+"].map { Alt(label: $0, text: $0) })
            : Key(.char, "/", "/", alts: [":", "-", "_", "?", "=", "&", "#"].map { Alt(label: $0, text: $0) }))
        row.append(Key(.space, " ", " ", width: space))
        row.append(Key(.char, ".com", ".com", width: 1.5, alts: domains))
        row.append(Key(.char, ".", ".", alts: ["-", "_", "@", "/"].map { Alt(label: $0, text: $0) }))
        row.append(Key(.enter, "↵", "\n", width: 1.5))
        return row
    }

    private static let enCommaAlts = [";", "'", "\"", "-", ":", "(", ")"].map { Alt(label: $0, text: $0) }
    private static let enPeriodAlts = ["?", "!", ",", "'", "\"", "…", "@", "/"].map { Alt(label: $0, text: $0) }

    private static let commaAlts = [Alt(label: ",,", text: ",,", note: "হসন্ত"), Alt(label: "+", text: "+", note: "যুক্ত"), Alt(label: "`", text: "`", note: "আলাদা"), Alt(label: ";", text: ";"), Alt(label: "'", text: "'"), Alt(label: "\"", text: "\""), Alt(label: "-", text: "-")]
    private static let periodAlts = [Alt(label: "..", text: "..", note: "বিন্দু"), Alt(label: "?", text: "?"), Alt(label: "!", text: "!"), Alt(label: ":", text: ":", note: "ঃ"), Alt(label: "^", text: "^", note: "ঁ"), Alt(label: "/", text: "/", note: "English"), Alt(label: "$", text: "$", note: "৳")]

    /// The bottom row: the layout switch, the globe (when iOS asks for it), comma, Space, full stop
    /// and Return. Bangla/English, emoji and ; are on the bar above the keys, so Space gets the room
    /// (a narrow Space caused many mistyped words). Always ten units wide.
    static func bottomRow(_ switchType: KeyType, _ switchLabel: String, globe: Bool, english: Bool = false) -> [Key] {
        var row: [Key] = [Key(switchType, switchLabel, width: 1.5)]
        var space: CGFloat = 5
        if globe { row.append(Key(.globe, "🌐")); space -= 1 }
        row.append(Key(.char, ",", ",", alts: english ? enCommaAlts : commaAlts))
        row.append(Key(.space, " ", " ", width: space))
        row.append(Key(.char, ".", ".", alts: english ? enPeriodAlts : periodAlts))
        row.append(Key(.enter, "↵", "\n", width: 1.5))
        return row
    }

    /// Digits and the common symbols
    /// Bangla ones (৳ first) while typing Bangla, English ones ($ first) in English
    static func symbols(globe: Bool, bangla: Bool) -> [[Key]] {
        let money = bangla
            ? Key(.char, "৳", "৳", alts: [Alt(label: "$", text: String(literal) + "$"), Alt(label: "€", text: "€"), Alt(label: "£", text: "£"), Alt(label: "¥", text: "¥"), Alt(label: "₹", text: "₹")])
            : Key(.char, "$", "$", alts: [Alt(label: "৳", text: "৳"), Alt(label: "€", text: "€"), Alt(label: "£", text: "£"), Alt(label: "¥", text: "¥"), Alt(label: "₹", text: "₹"), Alt(label: "¢", text: "¢")])
        return [
            "1234567890".map { Key(.char, String($0)) },
            ["@", "#", "MONEY", "_", "&", "-", "+", "(", ")", "/"].map { $0 == "MONEY" ? money : Key(.char, $0) },
            [Key(.moreSymbols, "=\\<", width: 1.5)] + ["*", "\"", "'", ":", ";", "!", "?"].map { Key(.char, $0) } + [Key(.delete, "⌫", width: 1.5)],
            bottomRow(.letters, "ABC", globe: globe),
        ]
    }

    static func moreSymbols(globe: Bool) -> [[Key]] {
        [
            ["~", "`", "|", "•", "√", "π", "÷", "×", "¶", "∆"].map { Key(.char, $0) },
            ["£", "€", "¥", "^", "°", "=", "{", "}", "\\", "%"].map { Key(.char, $0) },
            [Key(.symbols, "?১২৩", width: 1.5)] + ["©", "®", "™", "✓", "[", "]", "<"].map { Key(.char, $0) } + [Key(.delete, "⌫", width: 1.5)],
            bottomRow(.letters, "ABC", globe: globe),
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
}
