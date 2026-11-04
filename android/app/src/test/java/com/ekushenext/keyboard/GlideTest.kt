package com.ekushenext.keyboard

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.BeforeClass
import org.junit.Test
import java.io.File
import kotlin.random.Random

/** Glide typing against the real word list, with lines drawn the way a finger draws them */
class GlideTest {
    companion object {
        @BeforeClass @JvmStatic fun load() {
            var dir = File("").absoluteFile
            while (!File(dir, "data/english/words.txt").exists()) dir = dir.parentFile
            English.loadForTest(File(dir, "data/english/words.txt").readLines())
        }
    }

    /** A finger's line for a word: through its letters, a little off each centre, with wobble */
    private fun drawn(word: String, seed: Int, off: Float = 0.3f): List<FloatArray> {
        val rnd = Random(seed)
        val keys = Glide.wordLine(word)!!.map { floatArrayOf(it[0] + (rnd.nextFloat() - 0.5f) * 2 * off, it[1] + (rnd.nextFloat() - 0.5f) * 2 * off) }
        val out = ArrayList<FloatArray>()
        for (i in 0 until keys.size - 1) {
            val a = keys[i]; val b = keys[i + 1]
            for (s in 0 until 8) {
                val t = s / 8f
                out += floatArrayOf(a[0] + (b[0] - a[0]) * t + (rnd.nextFloat() - 0.5f) * 0.1f, a[1] + (b[1] - a[1]) * t + (rnd.nextFloat() - 0.5f) * 0.1f)
            }
        }
        out += keys.last()
        return out
    }

    private val words = listOf("the", "hello", "thanks", "going", "good", "people", "keyboard", "what", "where", "because",
        "tomorrow", "morning", "you", "are", "love", "please", "today", "work", "home", "time", "dinner", "money", "phone")

    @Test fun commonWordsAreFound() {
        var top1 = 0; var top3 = 0
        val missed = ArrayList<String>()
        for (w in words) for (seed in 1..5) {
            val got = English.glide(drawn(w, seed * 31 + w.length), 3)
            if (got.firstOrNull()?.lowercase() == w) top1++ else missed += "$w→${got.firstOrNull()}"
            if (got.any { it.lowercase() == w }) top3++
        }
        val n = words.size * 5
        println("glide: top1 $top1/$n, top3 $top3/$n; missed: ${missed.take(20)}")
        assertTrue("top-1 $top1/$n: $missed", top1 >= n * 0.85)
        assertTrue("top-3 $top3/$n", top3 >= n * 0.95)
    }

    /** A hurried finger: further off the centres, and corners cut (each point averaged with its neighbours) */
    private fun hurried(word: String, seed: Int): List<FloatArray> {
        val p = drawn(word, seed, 0.45f)
        return p.indices.map { i ->
            val a = p[maxOf(0, i - 3)]; val b = p[i]; val c = p[minOf(p.size - 1, i + 3)]
            floatArrayOf((a[0] + 2 * b[0] + c[0]) / 4, (a[1] + 2 * b[1] + c[1]) / 4)
        }
    }

    @Test fun hurriedLinesStillFindTheWord() {
        var top1 = 0; var top3 = 0
        val missed = ArrayList<String>()
        for (w in words) for (seed in 1..5) {
            val got = English.glide(hurried(w, seed * 17 + w.length), 3)
            if (got.firstOrNull()?.lowercase() == w) top1++ else missed += "$w→${got.take(3)}"
            if (got.any { it.lowercase() == w }) top3++
        }
        val n = words.size * 5
        println("hurried glide: top1 $top1/$n, top3 $top3/$n; missed: ${missed.take(25)}")
        assertTrue("top-3 $top3/$n: $missed", top3 >= n * 0.85)
    }

    @Test fun aShortSlideIsNotAGlide() {
        assertTrue(English.glide(listOf(floatArrayOf(4.5f, 0.5f), floatArrayOf(4.8f, 0.6f))).isEmpty())
    }

    @Test fun exactLinesWin() {
        assertEquals("the", English.glide(Glide.wordLine("the")!!).first())
        assertEquals("hello", English.glide(Glide.wordLine("hello")!!).first())
    }
}
