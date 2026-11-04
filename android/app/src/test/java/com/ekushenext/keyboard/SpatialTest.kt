package com.ekushenext.keyboard

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.BeforeClass
import org.junit.Test
import java.io.File

/** Typo correction against the real word list (data/english/words.txt) */
class SpatialTest {
    companion object {
        @BeforeClass @JvmStatic fun load() {
            var dir = File("").absoluteFile
            while (!File(dir, "data/english/words.txt").exists()) dir = dir.parentFile
            English.loadForTest(File(dir, "data/english/words.txt").readLines())
        }
    }

    /** Where a finger lands when it aims at `meant` but hits `typed`: on typed's side nearest meant */
    private fun tapsFor(typed: String, meant: String? = null): List<FloatArray?> = typed.mapIndexed { i, c ->
        val t = Spatial.center(c) ?: return@mapIndexed null
        val m = meant?.getOrNull(i)?.let { Spatial.center(it) }
        if (m == null || meant[i] == c) t else floatArrayOf(t[0] + (m[0] - t[0]) * 0.35f, t[1] + (m[1] - t[1]) * 0.35f)
    }

    private fun fix(typed: String, meant: String? = null) = English.guesses(typed, tapsFor(typed, meant)).firstOrNull()?.word

    @Test fun neighbourKeys() {
        assertEquals("the", fix("thw"))
        assertEquals("the", fix("rhe"))
        assertEquals("and", fix("abd"))
        assertEquals("you", fix("yiu"))
        assertEquals("with", fix("woth"))
        assertEquals("about", fix("abiut"))
        assertEquals("people", fix("peoplw"))
    }

    @Test fun swappedMissingAndExtraLetters() {
        assertEquals("the", fix("teh"))
        assertEquals("because", fix("becuase"))
        assertEquals("letter", fix("leter"))
        assertEquals("thanks", fix("thnaks"))
        assertEquals("would", fix("woudl"))
        assertEquals("really", fix("realy"))
        assertEquals("tomorrow", fix("tommorow"))
    }

    @Test fun twoMistakesInLongWords() {
        assertEquals("definitely", fix("definately"))
        assertEquals("beautiful", fix("beautifl"))
        assertEquals("different", fix("diffrent"))
        assertEquals("information", fix("infromation"))
    }

    @Test fun whereTheFingerLandedDecides() {
        // h and g are neighbours: a tap on the left edge of h, towards g, means "good"
        val towardsG = English.guesses("hiod", tapsFor("hiod", "good")).firstOrNull()?.word
        assertEquals("good", towardsG)
    }

    @Test fun wordsRunTogether() {
        assertEquals("a lot", fix("alot"))
        assertEquals("of the", fix("ofthe"))
        assertEquals("thank you", fix("thankyou"))
    }

    @Test fun namesAreNotGuessedForSmallLetters() {
        assertEquals("and", fix("adn"))
    }

    @Test fun knownWordsAndNonsenseAreLeftAlone() {
        assertTrue(English.guesses("form").isEmpty())
        assertTrue(English.guesses("zqxwv").isEmpty())
    }
}
