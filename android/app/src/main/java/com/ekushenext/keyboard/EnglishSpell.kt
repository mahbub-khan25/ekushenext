package com.ekushenext.keyboard

import android.content.Context
import android.view.textservice.SentenceSuggestionsInfo
import android.view.textservice.SpellCheckerSession
import android.view.textservice.SuggestionsInfo
import android.view.textservice.TextInfo
import android.view.textservice.TextServicesManager
import java.util.Locale

/**
 * The phone's own spell checker (usually Gboard's or Samsung's, working in the background even when
 * that keyboard isn't in use). Asked about one English word at a time; answers arrive later, on the
 * main thread. If the person has turned the spell checker off, there is simply no session.
 */
class EnglishSpell(context: Context, private val onAnswer: (word: String) -> Unit) : SpellCheckerSession.SpellCheckerSessionListener {
    class Answer(val suggestions: List<String>, val typo: Boolean)

    private val session: SpellCheckerSession? = runCatching {
        val tsm = context.getSystemService(Context.TEXT_SERVICES_MANAGER_SERVICE) as TextServicesManager
        tsm.newSpellCheckerSession(null, Locale.ENGLISH, this, false)
    }.getOrNull()

    private val answers = object : LinkedHashMap<String, Answer>(64, 0.75f, true) {
        override fun removeEldestEntry(eldest: MutableMap.MutableEntry<String, Answer>?) = size > 200
    }
    /** Requests waiting for an answer, by the sequence number sent with each one: answers come back
     *  with that number, so a late or dropped answer can never be matched to the wrong word. */
    private val pending = HashMap<Int, String>()
    private var nextSequence = 1

    fun available() = session != null

    fun answer(word: String): Answer? = answers[word]

    fun ask(word: String) {
        val s = session ?: return
        if (answers.containsKey(word) || word in pending.values) return
        if (pending.size > 32) pending.clear()
        val seq = nextSequence++
        pending[seq] = word
        runCatching { s.getSentenceSuggestions(arrayOf(TextInfo(word, 0, seq)), 4) }.onFailure { pending.remove(seq) }
    }

    override fun onGetSentenceSuggestions(results: Array<out SentenceSuggestionsInfo>?) {
        val r = results?.firstOrNull() ?: return
        var typo = false
        var word: String? = null
        val list = ArrayList<String>()
        for (i in 0 until r.suggestionsCount) {
            val info = r.getSuggestionsInfoAt(i) ?: continue
            if (word == null) word = pending.remove(info.sequence)
            val attrs = info.suggestionsAttributes
            if (attrs and SuggestionsInfo.RESULT_ATTR_IN_THE_DICTIONARY == 0 &&
                attrs and SuggestionsInfo.RESULT_ATTR_LOOKS_LIKE_TYPO != 0) typo = true
            for (j in 0 until info.suggestionsCount) info.getSuggestionAt(j)?.let { list += it }
        }
        val w = word ?: return
        answers[w] = Answer(list.distinct(), typo)
        onAnswer(w)
    }

    override fun onGetSuggestions(results: Array<out SuggestionsInfo>?) {}

    fun close() { runCatching { session?.close() } }
}
