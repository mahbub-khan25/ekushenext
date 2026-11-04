package com.ekushenext.keyboard

import kotlin.math.hypot
import kotlin.math.ln
import kotlin.math.max
import kotlin.math.min

/**
 * Typo correction that knows the keyboard: a letter typed instead of the one next to it ("thw" for
 * "the") costs less than one from across the keys, and where the finger actually landed counts
 * (a tap on the edge of w, towards e, makes e likely). Pure Kotlin, so it is tested on the computer.
 *
 * Positions are in key units on the QWERTY letters: q's centre is (0.5, 0.5), a's (1.0, 1.5), z's
 * (2.0, 2.5); one unit is one key's width (or one row's height).
 */
object Spatial {
    private val centers = HashMap<Char, FloatArray>().apply {
        "qwertyuiop".forEachIndexed { i, c -> put(c, floatArrayOf(i + 0.5f, 0.5f)) }
        "asdfghjkl".forEachIndexed { i, c -> put(c, floatArrayOf(i + 1.0f, 1.5f)) }
        "zxcvbnm".forEachIndexed { i, c -> put(c, floatArrayOf(i + 2.0f, 2.5f)) }
    }

    fun center(c: Char): FloatArray? = centers[c.lowercaseChar()]

    private var rows = Array(3) { FloatArray(40) }

    /** Which letters a word has, one bit each (a quick first check before the full comparison) */
    fun letters(w: String): Int { var m = 0; for (c in w) if (c in 'a'..'z') m = m or (1 shl (c - 'a')); return m }

    /** How far a tap (or the typed key's centre) is from a key, in key widths */
    fun distance(tap: FloatArray?, typed: Char, key: Char): Float {
        val k = center(key) ?: return 9f
        val t = tap ?: center(typed) ?: return 9f
        // rows are a little taller than keys are wide, and misses between rows are rarer
        return hypot(t[0] - k[0], (t[1] - k[1]) * 1.25f)
    }

    /** One letter typed where another was meant: a neighbour is a cheap mistake, a far key isn't */
    fun substitution(tap: FloatArray?, typed: Char, meant: Char): Float {
        if (typed == meant) return 0f
        val d = distance(tap, typed, meant)
        return min(1f, max(0.2f, (d - 0.3f) * 0.8f))
    }

    /**
     * What it costs to turn what was typed into `cand` (both lowercase letters only): letters
     * swapped, missing, extra or mistyped, weighted as above. Stops early above `limit`
     * (returns a number above it).
     */
    fun cost(typed: String, taps: List<FloatArray?>, cand: String, limit: Float): Float {
        val n = typed.length
        val m = cand.length
        if (n == 0) return m * 0.9f
        val big = limit + 1f
        // three rows are enough for a swap of two letters (kept between calls: this runs for
        // thousands of words on every key)
        if (rows[0].size < m + 1) { rows = Array(3) { FloatArray(m + 8) } }
        var prev2 = rows[0]
        var prev = rows[1]
        var cur = rows[2]
        java.util.Arrays.fill(prev2, 0f)
        prev[0] = 0f
        for (j in 1..m) prev[j] = prev[j - 1] + insertCost(cand, j - 1)
        var lastMin = 0f
        for (i in 1..n) {
            val t = typed[i - 1]
            val tap = taps.getOrNull(i - 1)
            cur[0] = prev[0] + deleteCost(typed, i - 1)
            var rowMin = cur[0]
            for (j in 1..m) {
                val c = cand[j - 1]
                var best = prev[j - 1] + substitution(tap, t, c)
                best = min(best, prev[j] + deleteCost(typed, i - 1))
                best = min(best, cur[j - 1] + insertCost(cand, j - 1))
                if (i > 1 && j > 1 && t == cand[j - 2] && typed[i - 2] == c) best = min(best, prev2[j - 2] + 0.55f)
                cur[j] = best
                if (best < rowMin) rowMin = best
            }
            // a swap reaches back two rows, so stop only when both are past the limit
            if (rowMin > limit && lastMin > limit) return big
            lastMin = rowMin
            val r = prev2; prev2 = prev; prev = cur; cur = r
        }
        return prev[m]
    }

    /** A letter typed that wasn't meant: cheaper if it doubles the one before (a key hit twice) */
    private fun deleteCost(typed: String, i: Int): Float {
        val c = typed[i]
        if (i > 0 && typed[i - 1] == c) return 0.45f
        val before = typed.getOrNull(i - 1)
        if (before != null && distance(null, before, c) < 1.2f) return 0.65f
        return 0.8f
    }

    /** A letter left out: cheaper if it is a double letter ("leter" for "letter") */
    private fun insertCost(cand: String, j: Int): Float {
        if (j > 0 && cand[j - 1] == cand[j]) return 0.45f
        return 0.9f
    }

    /** How far a word may be from what was typed and still be offered as its correction */
    fun limit(length: Int): Float = when {
        length <= 2 -> 0.45f
        length <= 3 -> 0.7f
        length <= 5 -> 1.25f
        length <= 8 -> 1.75f
        else -> 2.25f
    }

    /** A word's likelihood: common words and the person's own words first, close words before far ones */
    fun score(rank: Int, learned: Int, cost: Float): Double =
        ln(1.0 / (rank + 50)) + ln(1.0 + learned) * 0.6 - 3.2 * cost

    /** Could `cand` start the way `typed` does (the same first key, a neighbour, or two letters swapped)? */
    fun startsLike(typed: String, firstTap: FloatArray?, cand: String): Boolean {
        val a = typed[0]
        val b = cand[0]
        if (a == b) return true
        if (distance(firstTap, a, b) < 1.3f) return true
        if (typed.length > 1 && cand.length > 1 && typed[1] == b && cand[1] == a) return true
        // the first letter left out, or one too many at the start
        if (cand.length > 1 && cand[1] == a) return true
        if (typed.length > 1 && typed[1] == b) return true
        return false
    }
}
