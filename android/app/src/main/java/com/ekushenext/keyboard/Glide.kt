package com.ekushenext.keyboard

import kotlin.math.hypot
import kotlin.math.ln
import kotlin.math.max
import kotlin.math.min

/**
 * Glide typing: a finger drawn across the letters writes a word. The drawn line is compared with the
 * line each candidate word would make through the centres of its letters (both cut into the same
 * number of evenly spaced points), and the closest common words win. Pure Kotlin, tested on the
 * computer. Points are in key units, as in Spatial.
 */
object Glide {
    private const val N = 32

    /** `n` points evenly spaced along a line */
    fun resample(pts: List<FloatArray>, n: Int = N): Array<FloatArray> {
        if (pts.isEmpty()) return emptyArray()
        if (pts.size == 1) return Array(n) { pts[0] }
        val total = length(pts)
        if (total < 1e-4f) return Array(n) { pts[0] }
        val step = total / (n - 1)
        val out = ArrayList<FloatArray>(n)
        out += pts[0]
        var acc = 0f
        var prev = pts[0]
        var i = 1
        var cur = pts[1]
        while (out.size < n - 1 && i < pts.size) {
            val d = hypot(cur[0] - prev[0], cur[1] - prev[1])
            if (acc + d >= step && d > 0f) {
                val t = (step - acc) / d
                val q = floatArrayOf(prev[0] + t * (cur[0] - prev[0]), prev[1] + t * (cur[1] - prev[1]))
                out += q
                prev = q
                acc = 0f
            } else {
                acc += d
                prev = cur
                i++
                if (i < pts.size) cur = pts[i]
            }
        }
        while (out.size < n) out += pts.last()
        return out.toTypedArray()
    }

    fun length(pts: List<FloatArray>): Float {
        var s = 0f
        for (i in 1 until pts.size) s += hypot(pts[i][0] - pts[i - 1][0], pts[i][1] - pts[i - 1][1])
        return s
    }

    /** The line a word makes through its letters' centres (null if a letter isn't on the keys) */
    fun wordLine(word: String): List<FloatArray>? {
        val out = ArrayList<FloatArray>(word.length)
        for (c in word) {
            val p = Spatial.center(c) ?: return null
            if (out.isEmpty() || out.last()[0] != p[0] || out.last()[1] != p[1]) out += p
        }
        return out
    }

    /** Keys near a point: the letters whose centres are within `r` key widths */
    fun keysNear(p: FloatArray, r: Float): Set<Char> =
        "qwertyuiopasdfghjklzxcvbnm".filter { c -> Spatial.center(c)!!.let { hypot(it[0] - p[0], (it[1] - p[1]) * 1.1f) <= r } }.toSet()

    /** How far apart two lines of the same number of points are, on average */
    fun distance(a: Array<FloatArray>, b: Array<FloatArray>): Float {
        var s = 0f
        for (i in a.indices) s += hypot(a[i][0] - b[i][0], a[i][1] - b[i][1])
        return s / a.size
    }

    /** Every letter of the word passes close to the drawn line */
    fun covers(word: String, dense: Array<FloatArray>, r: Float): Boolean {
        for (c in word) {
            val k = Spatial.center(c) ?: return false
            var best = Float.MAX_VALUE
            for (p in dense) { val d = hypot(p[0] - k[0], p[1] - k[1]); if (d < best) best = d; if (best <= r) break }
            if (best > r) return false
        }
        return true
    }

    /** A word's score for a drawn line: close in shape, and common */
    fun score(path: Array<FloatArray>, line: Array<FloatArray>, rank: Int, learned: Int): Double {
        val shape = distance(path, line)
        val ends = hypot(path[0][0] - line[0][0], path[0][1] - line[0][1]) + hypot(path.last()[0] - line.last()[0], path.last()[1] - line.last()[1])
        return -3.0 * shape - 1.2 * ends + 0.35 * ln(1.0 / (rank + 50)) + 0.25 * ln(1.0 + learned)
    }

    /** Is a drawn line long enough to be a glide (not a sloppy tap)? */
    fun isGlide(pts: List<FloatArray>): Boolean = length(pts) >= 0.8f

    /** The ratio of the word's line to the drawn one must be plausible */
    fun lengthFits(pathLen: Float, wordLen: Float): Boolean {
        if (wordLen < 0.3f) return pathLen < 1.5f
        val r = pathLen / wordLen
        return r in 0.55f..2.2f
    }

    @Suppress("unused") private fun clamp(v: Float, lo: Float, hi: Float) = min(max(v, lo), hi)
}
