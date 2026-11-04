package com.ekushenext.keyboard

import android.content.Context
import android.provider.UserDictionary
import java.io.File
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.concurrent.thread

/**
 * English typing help, kept small: complete a word from the 50,000 most common English words,
 * catch a typo one letter away, learn the words you use, and offer a few likely next words.
 * (The phone's own spell checker is asked as well, by the input method; see EnglishSpell.)
 */
object English {
    /** Words, most common first */
    private var words: Array<String> = emptyArray()
    /** A word without its apostrophe ("dont") → its rank; the word itself is words[rank] */
    private val rankOf = HashMap<String, Int>(80_000)
    /** The plain forms, sorted, for finding every word that starts with what was typed */
    private var sorted: Array<String> = emptyArray()
    /** The words exactly as listed ("its" and "it's" are both words) */
    private val exact = HashSet<String>(80_000)
    private val next = HashMap<String, List<String>>()
    /** Emoji offered beside a word (love → ❤️ 😍) */
    private val emojiOf = HashMap<String, List<String>>()
    /** How often you chose each word, and which word you wrote after which */
    private val learned = HashMap<String, Int>()
    private val pairs = HashMap<String, Int>()
    /** Words the person asked never to be suggested (long-press on a suggestion) */
    private val blocked = HashSet<String>()
    private val ready = AtomicBoolean(false)
    /** For corrections: the 40,000 most common words (plain form), by length */
    private var byLength: Array<Array<String>> = emptyArray()
    private var lettersByLength: Array<IntArray> = emptyArray()
    private var dir: File? = null
    private var dirty = false

    // Never offered unless typed out in full
    private val quiet = setOf("fuck", "fucking", "fucked", "fucker", "shit", "shitty", "bitch", "bitches", "cunt", "dick", "dicks",
        "pussy", "cock", "cocks", "asshole", "bastard", "slut", "whore", "fag", "faggot", "nigger", "nigga", "retard", "retarded",
        "porn", "sex", "sexy", "nude", "nudes", "boobs", "tits", "penis", "vagina", "rape", "motherfucker", "damn", "crap", "piss")

    private fun plain(w: String) = w.lowercase().replace("'", "")

    fun start(context: Context) {
        if (dir != null) return
        dir = context.filesDir
        val app = context.applicationContext
        thread(name = "ekushe-english") {
            runCatching {
                loadWords(app.assets.open("en/words.txt").bufferedReader().readLines())
                runCatching { loadEmoji(app.assets.open("en/emoji.tsv").bufferedReader().readLines()) }
                app.assets.open("en/next.tsv").bufferedReader().forEachLine { line ->
                    if (line.startsWith("#") || line.isBlank()) return@forEachLine
                    val parts = line.split('\t')
                    if (parts.size > 1) next[parts[0]] = (next[parts[0]].orEmpty() + parts.drop(1)).distinct()
                }
            }
            runCatching {
                File(dir, "en_learned.tsv").takeIf { it.exists() }?.forEachLine { line ->
                    val p = line.split('\t')
                    if (p.size == 3 && p[0] == "w") learned[p[1]] = p[2].toIntOrNull() ?: 0
                    if (p.size == 3 && p[0] == "p") pairs[p[1]] = p[2].toIntOrNull() ?: 0
                    if (p.size >= 2 && p[0] == "b") blocked += p[1]
                }
            }
            // Words the person saved in the phone's personal dictionary (names, places)
            runCatching {
                app.contentResolver.query(UserDictionary.Words.CONTENT_URI, arrayOf(UserDictionary.Words.WORD), null, null, null)?.use { c ->
                    while (c.moveToNext()) {
                        val w = c.getString(0)?.trim() ?: continue
                        if (w.isNotEmpty() && w.all { it.isLetter() || it == '\'' }) learned.putIfAbsent(w, 3)
                    }
                }
            }
            ready.set(true)
        }
    }

    /** The word list, most common first (also how the tests load it) */
    internal fun loadWords(lines: List<String>) {
        val list = lines.filter { it.isNotBlank() }.toTypedArray()
        list.forEachIndexed { i, w -> rankOf.putIfAbsent(plain(w), i); exact += w }
        words = list
        sorted = rankOf.keys.toTypedArray().also { it.sort() }
        val buckets = Array(32) { ArrayList<String>() }
        for ((k, r) in rankOf) if (r < 40_000 && k.length in 1..31 && k.all { it in 'a'..'z' }) buckets[k.length] += k
        byLength = Array(32) { buckets[it].toTypedArray() }
        lettersByLength = Array(32) { l -> IntArray(byLength[l].size) { Spatial.letters(byLength[l][it]) } }
    }

    internal fun loadEmoji(lines: List<String>) {
        for (line in lines) {
            if (line.startsWith("#") || line.isBlank()) continue
            val p = line.split('\t')
            if (p.size > 1) emojiOf[p[0].lowercase()] = p.drop(1).filter { it.isNotBlank() }
        }
    }

    /** Emoji for a word (love → ❤️, 😍), or none */
    fun emoji(word: String): List<String> = emojiOf[word.lowercase()] ?: emojiOf[plain(word)].orEmpty()

    /** Tests: the word list alone, ready to use */
    internal fun loadForTest(lines: List<String>) { loadWords(lines); ready.set(true) }

    fun isReady() = ready.get()

    /** A known word (from the list or what you have typed before) */
    fun known(w: String): Boolean {
        if (!ready.get()) return true
        return rankOf.containsKey(plain(w)) || learned.containsKey(w) || learned.containsKey(w.lowercase())
    }

    private fun score(w: String): Double {
        val r = rankOf[plain(w)]
        val base = if (r != null) 1.0 / (r + 50) else 0.0
        val l = learned[w] ?: learned[w.lowercase()] ?: 0
        return base + l * 0.004
    }

    /** The spelling to show: the list's (don't, i'm → I'm), in the case the person typed */
    fun spell(typed: String, w: String): String {
        var out = w
        if (out == "i" || out.startsWith("i'")) out = "I" + out.substring(1)
        return when {
            typed.length > 1 && typed.all { it.isUpperCase() } -> out.uppercase()
            typed.firstOrNull()?.isUpperCase() == true -> out.replaceFirstChar { it.uppercase() }
            else -> out
        }
    }

    // Typed without the apostrophe, these are almost always meant with it (unlike its, well, were, ill)
    private val contraction = setOf("im", "ive", "dont", "cant", "wont", "didnt", "doesnt", "isnt", "wasnt", "arent", "werent",
        "couldnt", "wouldnt", "shouldnt", "havent", "hasnt", "hadnt", "thats", "whats", "theres", "youre", "theyre", "youve",
        "weve", "theyve", "youll", "theyll", "shes", "hes", "whos", "heres", "wheres", "hows", "ain't", "aint", "lets")

    /** "i", or a contraction typed without its apostrophe (dont, im, cant) */
    fun isContraction(typed: String): Boolean = typed.lowercase().let { it == "i" || it in contraction }

    /** The list's spelling of a typed word, if it is known in another form ("dont" → "don't", "i" → "I") */
    fun canonical(typed: String): String? {
        if (!ready.get()) return null
        val low = typed.lowercase()
        if (low == "i") return "I"
        if (low in exact && low !in contraction) return null
        if (low == "lets") return null
        val r = rankOf[plain(typed)] ?: return null
        val w = spell(typed, words[r])
        return if (w != typed) w else null
    }

    /** Words that start with what was typed, best first */
    fun complete(typed: String, max: Int = 3): List<String> {
        if (!ready.get() || typed.isEmpty()) return emptyList()
        val p = plain(typed)
        val found = ArrayList<Pair<String, Double>>()
        // the person's own words first
        for ((w, n) in learned) if (n > 0 && w.lowercase().startsWith(p) && w.length > typed.length) found += w to score(w)
        var i = java.util.Arrays.binarySearch(sorted, p).let { if (it < 0) -it - 1 else it }
        var scanned = 0
        while (i < sorted.size && sorted[i].startsWith(p) && scanned < 30_000) {
            val key = sorted[i]
            if (key.length > p.length) {
                val w = words[rankOf[key]!!]
                if (w !in quiet) found += w to score(w)
            }
            i++; scanned++
        }
        return found.filter { it.first.lowercase() !in blocked }.sortedByDescending { it.second }.map { spell(typed, it.first) }.distinct().take(max)
    }

    /** A guess at what was meant, with how far it is from what was typed */
    class Guess(val word: String, val cost: Float, val score: Double)

    /**
     * Known words close to a typo, best first. `taps` are where each letter was tapped (key units,
     * see Spatial; null for a key from a physical keyboard). `extra` are other guesses to weigh the
     * same way (the phone's spell checker's).
     */
    fun guesses(typed: String, taps: List<FloatArray?> = emptyList(), extra: List<String> = emptyList(), max: Int = 3): List<Guess> {
        if (!ready.get() || typed.length < 2 || known(typed)) return emptyList()
        // the same word is asked about more than once per key (the strip, then Space's choice)
        val memoKey = Triple(typed, taps.map { it?.toList() }, extra)
        if (memoKey == memoAsk) return memoAnswer.take(max)
        val all = guessesNow(typed, taps, extra)
        memoAsk = memoKey; memoAnswer = all
        return all.take(max)
    }
    private var memoAsk: Any? = null
    private var memoAnswer: List<Guess> = emptyList()

    private fun guessesNow(typed: String, taps: List<FloatArray?>, extra: List<String>): List<Guess> {
        // letters only, each with its tap
        val t = StringBuilder()
        val tp = ArrayList<FloatArray?>()
        typed.forEachIndexed { i, c -> if (c.isLetter()) { t.append(c.lowercaseChar()); tp += taps.getOrNull(i) } }
        val p = t.toString()
        if (p.isEmpty() || p.any { it !in 'a'..'z' }) return emptyList()
        val limit = Spatial.limit(p.length)
        val reach = if (p.length >= 6) 2 else 1
        val found = HashMap<String, Guess>()
        fun weigh(key: String) {
            if (key in found) return
            val c = Spatial.cost(p, tp, key, limit)
            if (c > limit) return
            val r = rankOf[key] ?: 60_000
            val shown = words.getOrNull(rankOf[key] ?: -1) ?: key
            if (shown in quiet || shown.lowercase() in blocked) return
            // a name (Dan, Eth) for a word typed in small letters is a less likely guess
            val name = shown.first().isUpperCase() && typed.first().isLowerCase()
            found[key] = Guess(shown, c, Spatial.score(r, learned[shown] ?: learned[key] ?: 0, c) - if (name) 2.0 else 0.0)
        }
        // a word sharing too few of the typed letters can't be close: skip it before the full comparison
        val mine = Spatial.letters(p)
        val need = Integer.bitCount(mine) - (if (p.length >= 6) 3 else 2)
        for (len in (p.length - reach)..(p.length + reach)) {
            if (len < 1 || len >= byLength.size) continue
            val list = byLength[len]; val masks = lettersByLength[len]
            for (i in list.indices) {
                if (Integer.bitCount(masks[i] and mine) < need) continue
                val w = list[i]
                if (Spatial.startsLike(p, tp.firstOrNull(), w)) weigh(w)
            }
        }
        // the person's own words (names, places) count too
        for (w in learned.keys) {
            val k = plain(w)
            if (k.length >= 2 && kotlin.math.abs(k.length - p.length) <= reach && k.all { it in 'a'..'z' } && Spatial.startsLike(p, tp.firstOrNull(), k)) {
                if (k !in found) {
                    val c = Spatial.cost(p, tp, k, limit)
                    if (c <= limit) found[k] = Guess(w, c, Spatial.score(rankOf[k] ?: 60_000, learned[w] ?: 0, c))
                }
            }
        }
        for (e in extra) { val k = plain(e); if (k.isNotEmpty() && k.all { it in 'a'..'z' }) weigh(k) }
        // two words run together (alot, ofthe, inthe): both common
        if (p.length in 3..14) for (i in 1 until p.length) {
            val a = p.substring(0, i); val b = p.substring(i)
            val ra = rankOf[a] ?: continue; val rb = rankOf[b] ?: continue
            if ((a.length == 1 && a != "a" && a != "i") || ra > 3000 || rb > 3000) continue
            val both = "${words[ra]} ${words[rb]}"
            found[both] = Guess(both, 0.5f, Spatial.score(maxOf(ra, rb) * 2, 0, 0.5f))
        }
        return found.values.sortedByDescending { it.score }.take(8)
    }

    /**
     * Words for a line drawn across the keys (glide typing), best first: the shape of the line
     * against each word's line through its letters, and how common the word is. `path` is in key
     * units (see Spatial).
     */
    fun glide(path: List<FloatArray>, max: Int = 5, after: List<String> = emptyList()): List<String> {
        if (!ready.get() || path.size < 2 || !Glide.isGlide(path)) return emptyList()
        val rs = Glide.resample(path)
        val dense = Glide.resample(path, 64)
        val starts = Glide.keysNear(path.first(), 1.15f)
        val ends = Glide.keysNear(path.last(), 1.15f)
        val pathLen = Glide.length(path)
        val found = HashMap<String, Double>()
        fun weigh(key: String, shown: String, rank: Int, learnedN: Int) {
            if (key.length < 2 || key[0] !in starts || key.last() !in ends || key in found) return
            val line = Glide.wordLine(key) ?: return
            if (line.size < 2 || !Glide.lengthFits(pathLen, Glide.length(line))) return
            if (!Glide.covers(key, dense, 1.25f)) return
            if (shown in quiet || shown.lowercase() in blocked) return
            // names (Hugo, Sufi) only when you have written them yourself
            val name = shown.first().isUpperCase() && learnedN == 0
            // a likely next word after the one before it counts a little more
            val follows = if (shown.lowercase() in after) 0.8 else 0.0
            found[shown] = Glide.score(rs, Glide.resample(line), rank, learnedN) - (if (name) 1.0 else 0.0) + follows
        }
        for (len in 2 until byLength.size) for (w in byLength[len]) {
            val r = rankOf[w] ?: continue
            if (r > 30_000) continue
            val shown = words[r]
            weigh(w, shown, r, learned[shown] ?: 0)
        }
        for ((w, n) in learned) {
            val k = plain(w)
            if (k.all { it in 'a'..'z' }) weigh(k, w, rankOf[k] ?: 40_000, n)
        }
        return found.entries.sortedByDescending { it.value }.take(max).map { it.key }
    }

    /** Known words close to a typo, best first, in the case the person typed */
    fun corrections(typed: String, max: Int = 3, taps: List<FloatArray?> = emptyList()): List<String> =
        guesses(typed, taps, emptyList(), max).map { spell(typed, it.word) }.distinct()

    /** The most likely of some guesses: the most common word (words not in the list come last, in order) */
    fun best(guesses: List<String>): String? = guesses.withIndex().maxByOrNull { (i, w) -> score(w) - i * 1e-9 }?.value

    /** Likely words after the last one or two: what you wrote before, then the basics */
    fun nextWords(prev2: String?, prev: String?, max: Int = 3): List<String> {
        if (prev.isNullOrEmpty()) return emptyList()
        val a = prev.lowercase()
        val out = ArrayList<String>()
        pairs.entries.filter { it.key.startsWith("$a ") }.sortedByDescending { it.value }.forEach { out += it.key.substringAfter(' ') }
        prev2?.let { next["${it.lowercase()} $a"]?.let { l -> out += l } }
        next[a]?.let { out += it }
        return out.filter { it.lowercase() !in blocked }.map { if (it == "i" || it.startsWith("i'")) "I" + it.substring(1) else it }.distinct().take(max)
    }

    /** Remember a word that was written (not in private fields) */
    fun learn(word: String, prev: String?) {
        if (!ready.get() || word.length < 2 || !word.any { it.isLetter() }) return
        learned[word] = (learned[word] ?: 0) + 1
        if (!prev.isNullOrEmpty()) {
            val k = "${prev.lowercase()} ${word.lowercase()}"
            pairs[k] = (pairs[k] ?: 0) + 1
        }
        dirty = true
    }

    fun save() {
        if (!dirty) return
        dirty = false
        val d = dir ?: return
        runCatching {
            val text = buildString {
                // keep the file small: the words used most
                learned.entries.sortedByDescending { it.value }.take(5000).forEach { append("w\t${it.key}\t${it.value}\n") }
                pairs.entries.sortedByDescending { it.value }.take(5000).forEach { append("p\t${it.key}\t${it.value}\n") }
                blocked.forEach { append("b\t$it\n") }
            }
            val tmp = File(d, "en_learned.tsv.new")
            tmp.writeText(text)
            tmp.renameTo(File(d, "en_learned.tsv"))
        }
    }

    /** Never suggest this word again (it can still be typed) */
    fun block(word: String) {
        val k = word.lowercase()
        blocked += k
        learned.remove(word); learned.remove(k)
        pairs.keys.removeAll { it.endsWith(" $k") }
        dirty = true
    }

    /** One of your words: never corrected, and suggested like a common word */
    fun addWord(word: String) {
        blocked.remove(word.lowercase())
        learned[word] = (learned[word] ?: 0) + 20
        dirty = true
    }

    fun isBlocked(word: String) = word.lowercase() in blocked

    fun forget() {
        learned.clear(); pairs.clear(); blocked.clear(); dirty = false
        dir?.let { File(it, "en_learned.tsv").delete() }
    }

    fun wordCount() = words.size
}
