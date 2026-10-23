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
    /** How often you chose each word, and which word you wrote after which */
    private val learned = HashMap<String, Int>()
    private val pairs = HashMap<String, Int>()
    private val ready = AtomicBoolean(false)
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
                val list = app.assets.open("en/words.txt").bufferedReader().readLines().filter { it.isNotBlank() }.toTypedArray()
                list.forEachIndexed { i, w -> rankOf.putIfAbsent(plain(w), i); exact += w }
                words = list
                sorted = rankOf.keys.toTypedArray().also { it.sort() }
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
        return found.sortedByDescending { it.second }.map { spell(typed, it.first) }.distinct().take(max)
    }

    /** Known words one letter away from a typo (a letter wrong, missing, extra or swapped), best first */
    fun corrections(typed: String, max: Int = 3): List<String> {
        if (!ready.get() || typed.length < 2 || known(typed)) return emptyList()
        val p = plain(typed)
        val letters = "abcdefghijklmnopqrstuvwxyz"
        val cand = HashSet<String>()
        for (i in 0..p.length) {
            if (i < p.length) cand += p.removeRange(i, i + 1)
            if (i < p.length - 1) cand += p.substring(0, i) + p[i + 1] + p[i] + p.substring(i + 2)
            for (c in letters) {
                if (i < p.length) cand += p.substring(0, i) + c + p.substring(i + 1)
                cand += p.substring(0, i) + c + p.substring(i)
            }
        }
        return cand.mapNotNull { k -> rankOf[k]?.let { r -> words[r] } }
            .filter { it !in quiet && (rankOf[plain(it)] ?: Int.MAX_VALUE) < 30_000 }
            .sortedByDescending { score(it) }
            .map { spell(typed, it) }.distinct().take(max)
    }

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
        return out.map { if (it == "i" || it.startsWith("i'")) "I" + it.substring(1) else it }.distinct().take(max)
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
            }
            val tmp = File(d, "en_learned.tsv.new")
            tmp.writeText(text)
            tmp.renameTo(File(d, "en_learned.tsv"))
        }
    }

    fun forget() {
        learned.clear(); pairs.clear(); dirty = false
        dir?.let { File(it, "en_learned.tsv").delete() }
    }

    fun wordCount() = words.size
}
