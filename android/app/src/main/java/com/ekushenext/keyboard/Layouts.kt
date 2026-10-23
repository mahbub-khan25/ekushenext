package com.ekushenext.keyboard

/** What a key does. */
enum class KeyType { CHAR, SHIFT, DELETE, ENTER, SPACE, SYMBOLS, MORE_SYMBOLS, LETTERS, LANG, EMOJI }

/** A long-press choice: what it shows, what it types, and a small note under it. */
data class Alt(val label: String, val text: String, val note: String? = null)

data class Key(
    val type: KeyType,
    val label: String,
    val text: String = label,
    val width: Float = 1f,
    /** The Bangla letter this key gives in phonetic typing (shown small), and with Shift */
    val hint: String? = null,
    val hintShift: String? = null,
    val alts: List<Alt> = emptyList(),
    val shiftAlts: List<Alt>? = null,
    /** Letters follow Shift; symbols do not */
    val letter: Boolean = false,
)

object Layouts {
    private fun ch(c: Char, hint: String?, hintShift: String? = null, alts: List<Alt> = emptyList()) =
        Key(KeyType.CHAR, c.toString(), c.toString(), hint = hint, hintShift = hintShift ?: hint, alts = alts, letter = true)

    private val bnDigits = "০১২৩৪৫৬৭৮৯"
    fun digitLabel(d: Char, bangla: Boolean) = if (bangla && d in '0'..'9') bnDigits[d - '0'].toString() else d.toString()

    // Long-press choices: capital letters change the sound in phonetic typing, so they are one
    // press away, with the marks for exact spellings
    private val altT = listOf(Alt("T", "T", "ট"), Alt("t``", "t``", "ৎ"), Alt("th", "th", "থ"))
    private val altD = listOf(Alt("D", "D", "ড"), Alt("dh", "dh", "ধ"))
    private val altN = listOf(Alt("N", "N", "ণ"), Alt("ng", "ng", "ং"), Alt("NG", "NG", "ঞ"))
    private val altS = listOf(Alt("sh", "sh", "শ"), Alt("Sh", "Sh", "ষ"))
    private val altR = listOf(Alt("R", "R", "ড়"), Alt("rr", "rr", "রেফ"), Alt("rri", "rri", "ঋ"))
    private val altG = listOf(Alt("Ng", "Ng", "ঙ"), Alt("gh", "gh", "ঘ"))
    private val altO = listOf(Alt("O", "O", "ও"), Alt("oi", "oi", "ঐ"), Alt("ou", "ou", "ঔ"))
    private val altI = listOf(Alt("I", "I", "ঈ"))
    private val altU = listOf(Alt("U", "U", "ঊ"))
    private val altZ = listOf(Alt("Z", "Z", "্য"))
    private val altH = listOf(Alt(":", ":", "ঃ"))
    private val altK = listOf(Alt("kh", "kh", "খ"), Alt("kkh", "kkh", "ক্ষ"))
    private val altC = listOf(Alt("ch", "ch", "ছ"))
    private val altP = listOf(Alt("ph", "ph", "ফ"))
    private val altB = listOf(Alt("bh", "bh", "ভ"))
    private val altJ = listOf(Alt("jh", "jh", "ঝ"))
    private val altE = listOf(Alt("^", "^", "ঁ"))
    private val altA = listOf(Alt("^", "^", "ঁ"))

    private val numberRow = "1234567890".map { d ->
        Key(KeyType.CHAR, d.toString(), d.toString(), hint = null)
    }

    /** Phonetic QWERTY, with the Bangla letter of each key shown small */
    fun letters(numberRow: Boolean): List<List<Key>> = listOfNotNull(
        if (numberRow) this.numberRow else null,
        listOf(
            ch('q', "ক"), ch('w', "ও"), ch('e', "এ", alts = altE), ch('r', "র", "ড়", altR), ch('t', "ত", "ট", altT),
            ch('y', "য়"), ch('u', "উ", "ঊ", altU), ch('i', "ই", "ঈ", altI), ch('o', "অ", "ও", altO), ch('p', "প", alts = altP),
        ),
        listOf(
            ch('a', "আ", alts = altA), ch('s', "স", alts = altS), ch('d', "দ", "ড", altD), ch('f', "ফ"), ch('g', "গ", alts = altG),
            ch('h', "হ", alts = altH), ch('j', "জ", alts = altJ), ch('k', "ক", alts = altK), ch('l', "ল"),
        ),
        listOf(
            Key(KeyType.SHIFT, "⇧", width = 1.5f),
            ch('z', "য", "্য", altZ), ch('x', "ক্স"), ch('c', "চ", alts = altC), ch('v', "ভ"), ch('b', "ব", alts = altB),
            ch('n', "ন", "ণ", altN), ch('m', "ম"),
            Key(KeyType.DELETE, "⌫", width = 1.5f),
        ),
        bottomRow(KeyType.SYMBOLS, "?১২৩"),
    )

    private val commaAlts = listOf(Alt(",,", ",,", "হসন্ত"), Alt("+", "+", "যুক্ত"), Alt("`", "`", "আলাদা"), Alt(";", ";"), Alt("'", "'"), Alt("\"", "\""), Alt("-", "-"))
    private val periodAlts = listOf(Alt("..", "..", "বিন্দু"), Alt("?", "?"), Alt("!", "!"), Alt(":", ":", "ঃ"), Alt("^", "^", "ঁ"), Alt("/", "/", "English"), Alt("$", "$", "৳"))

    /** An emoji key on the bottom row, like Gboard and Samsung (a setting) */
    var emojiKey = true

    fun bottomRow(switchType: KeyType, switchLabel: String) = if (emojiKey) listOf(
        Key(switchType, switchLabel, width = 1.3f),
        Key(KeyType.LANG, "বাং", width = 1f),
        Key(KeyType.CHAR, ",", ",", alts = commaAlts),
        Key(KeyType.EMOJI, "☺", width = 1f),
        Key(KeyType.SPACE, " ", " ", width = 3.4f),
        Key(KeyType.CHAR, ".", ".", alts = periodAlts),
        Key(KeyType.ENTER, "↵", "\n", width = 1.3f),
    ) else listOf(
        Key(switchType, switchLabel, width = 1.5f),
        Key(KeyType.LANG, "বাং", width = 1f),
        Key(KeyType.CHAR, ",", ",", alts = commaAlts),
        Key(KeyType.SPACE, " ", " ", width = 4.5f),
        Key(KeyType.CHAR, ".", ".", alts = periodAlts),
        Key(KeyType.ENTER, "↵", "\n", width = 1.5f),
    )

    /** Digits and the common symbols */
    fun symbols(): List<List<Key>> = listOf(
        "1234567890".map { Key(KeyType.CHAR, it.toString(), it.toString()) },
        listOf("@", "#", "৳", "_", "&", "-", "+", "(", ")", "/").map {
            Key(KeyType.CHAR, it, it, alts = if (it == "৳") listOf(Alt("$", "$"), Alt("€", "€"), Alt("£", "£"), Alt("¥", "¥"), Alt("₹", "₹")) else emptyList())
        },
        listOf(Key(KeyType.MORE_SYMBOLS, "=\\<", width = 1.5f)) +
            listOf("*", "\"", "'", ":", ";", "!", "?").map { Key(KeyType.CHAR, it, it) } +
            listOf(Key(KeyType.DELETE, "⌫", width = 1.5f)),
        bottomRow(KeyType.LETTERS, "ABC"),
    )

    fun moreSymbols(): List<List<Key>> = listOf(
        listOf("~", "`", "|", "•", "√", "π", "÷", "×", "¶", "∆").map { Key(KeyType.CHAR, it, it) },
        listOf("£", "€", "¥", "^", "°", "=", "{", "}", "\\", "%").map { Key(KeyType.CHAR, it, it) },
        listOf(Key(KeyType.SYMBOLS, "?১২৩", width = 1.5f)) +
            listOf("©", "®", "™", "✓", "[", "]", "<", ">").map { Key(KeyType.CHAR, it, it) }.take(7) +
            listOf(Key(KeyType.DELETE, "⌫", width = 1.5f)),
        bottomRow(KeyType.LETTERS, "ABC"),
    )

    /** Number and phone fields */
    fun numbers(phone: Boolean): List<List<Key>> = listOf(
        listOf("1", "2", "3", if (phone) "+" else "-").map { Key(KeyType.CHAR, it, it) },
        listOf("4", "5", "6", if (phone) "*" else ",").map { Key(KeyType.CHAR, it, it) },
        listOf("7", "8", "9", if (phone) "#" else ".").map { Key(KeyType.CHAR, it, it) },
        listOf(Key(KeyType.LETTERS, "ABC"), Key(KeyType.CHAR, "0", "0"), Key(KeyType.SPACE, " ", " "), Key(KeyType.DELETE, "⌫"), Key(KeyType.ENTER, "↵", "\n")),
    )
}
