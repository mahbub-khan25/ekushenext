package com.ekushenext.keyboard

import android.content.Context
import org.json.JSONObject
import java.util.concurrent.Executors

/** One suggestion: the word, and what kind it is (Dictionary, Phonetic, English, Emoji, Snippet …). */
data class Candidate(val text: String, val kind: String)

/** A tip on how to type a word on offer directly (হেঁসে ← he^se). */
data class Hint(val bangla: String, val keys: String)

/** What a key did: the word being typed, the suggestions, and any text to place in the app. */
data class Output(
    val preedit: String = "",
    val roman: String = "",
    val candidates: List<Candidate> = emptyList(),
    val selected: Int = 0,
    val hints: List<Hint> = emptyList(),
    val next: List<String> = emptyList(),
    val committed: String? = null,
    val deleteBefore: Int = 0,
) {
    val composing get() = roman.isNotEmpty()

    /** Hints that are tips for one of the suggestions (shown highlighted, like the desktop popup). */
    val tips get() = hints.takeWhile { h -> candidates.any { it.text == h.bangla } }

    companion object {
        fun parse(json: String?): Output {
            if (json.isNullOrEmpty()) return Output()
            val o = JSONObject(json)
            val cands = o.optJSONArray("candidates")
            val hints = o.optJSONArray("sign_hints")
            val next = o.optJSONArray("next_word_predictions")
            return Output(
                preedit = o.optString("preedit"),
                roman = o.optString("roman_buffer"),
                candidates = (0 until (cands?.length() ?: 0)).map {
                    val c = cands!!.getJSONObject(it)
                    Candidate(c.optString("text"), c.optString("kind"))
                },
                selected = o.optInt("selected_index"),
                hints = (0 until (hints?.length() ?: 0)).map {
                    val h = hints!!.getJSONObject(it)
                    Hint(h.optString("result_bangla"), h.optString("key_hint"))
                },
                next = (0 until (next?.length() ?: 0)).map { next!!.getString(it) },
                committed = if (o.isNull("committed")) null else o.optString("committed"),
                deleteBefore = o.optInt("delete_before"),
            )
        }
    }
}

/** The Rust engine (crates/ekushe-android). The dictionary loads in the background once per process. */
object Engine {
    init {
        System.loadLibrary("ekushe_android")
    }

    @Volatile
    var words = 0
        private set
    private val worker = Executors.newSingleThreadExecutor()
    @Volatile
    private var started = false

    fun start(context: Context, onReady: (() -> Unit)? = null) {
        if (started) { onReady?.invoke(); return }
        started = true
        val app = context.applicationContext
        worker.execute {
            val bytes = runCatching { app.assets.open("ekushe.dict.z").use { it.readBytes() } }.getOrDefault(ByteArray(0))
            words = nativeInit(bytes, app.filesDir.absolutePath)
            onReady?.invoke()
        }
    }

    fun key(c: Char) = Output.parse(nativeKey(c.code))
    fun backspace() = Output.parse(nativeBackspace())
    fun select(i: Int, trailing: Char? = null) = Output.parse(nativeSelect(i, trailing?.code ?: 0))
    fun commit(trailing: Char? = null) = Output.parse(nativeCommit(trailing?.code ?: 0))
    fun reset() = nativeReset()
    fun setPrivate(on: Boolean) = nativeSetPrivate(on)
    fun hasUndo() = nativeHasUndo()
    /** A Bangla word in English letters ("ami", "tomake")? English typing leaves those as they are. */
    fun isBanglish(word: String): Boolean = words > 0 && runCatching { nativeIsBanglish(word) }.getOrDefault(false)
    fun config(): JSONObject = runCatching { JSONObject(nativeGetConfig()) }.getOrDefault(JSONObject())
    fun setConfig(change: JSONObject) = nativeSetConfig(change.toString())
    fun save() = worker.execute { nativeSave() }
    fun forgetLearned() = nativeForgetLearned()
    val release: String get() = nativeRelease()
    fun reportUrl(os: String): String = nativeReportUrl(os)

    private external fun nativeInit(dict: ByteArray, dir: String): Int
    private external fun nativeKey(cp: Int): String?
    private external fun nativeBackspace(): String?
    private external fun nativeSelect(index: Int, trailing: Int): String?
    private external fun nativeCommit(trailing: Int): String?
    private external fun nativeReset()
    private external fun nativeSetPrivate(on: Boolean)
    private external fun nativeHasUndo(): Boolean
    private external fun nativeIsBanglish(word: String): Boolean
    private external fun nativeGetConfig(): String
    private external fun nativeSetConfig(json: String): Boolean
    private external fun nativeSave()
    private external fun nativeForgetLearned()
    private external fun nativeRelease(): String
    private external fun nativeReportUrl(os: String): String
}
