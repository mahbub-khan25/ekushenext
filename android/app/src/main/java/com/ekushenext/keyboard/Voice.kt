package com.ekushenext.keyboard

import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.speech.RecognitionListener
import android.speech.RecognizerIntent
import android.speech.SpeechRecognizer
import java.util.Locale

/**
 * Voice typing with the phone's own speech service (Google's on most phones), in Bangla or English.
 * It asks for offline recognition first, so a phone that has the language downloaded keeps the voice
 * on the phone; otherwise the service may use the internet. Ekushe itself never sends anything:
 * it has no internet permission.
 *
 * It keeps listening, sentence after sentence, until stopped or until nothing is heard for a while.
 * Callbacks arrive on the main thread.
 */
class Voice(private val context: Context, private val events: Events) {
    interface Events {
        /** Words heard so far in this sentence (not final yet) */
        fun onHeard(partial: String)
        /** A sentence is final: type it */
        fun onSentence(text: String)
        /** How loud the voice is now, 0..1 */
        fun onLevel(level: Float)
        /** Listening ended; `problem` says why if it wasn't the person stopping it */
        fun onStopped(problem: String?)
    }

    private var recognizer: SpeechRecognizer? = null
    private var bangla = true
    private var offline = true
    private var running = false
    /** Silences in a row with nothing heard: two of them end the listening */
    private var quiet = 0

    val active get() = running

    companion object {
        fun available(context: Context) = SpeechRecognizer.isRecognitionAvailable(context)
    }

    fun start(bangla: Boolean) {
        stop(notify = false)
        this.bangla = bangla
        offline = true
        quiet = 0
        running = true
        listen()
    }

    fun stop(notify: Boolean = true) {
        val was = running
        running = false
        recognizer?.let { r -> runCatching { r.stopListening(); r.cancel(); r.destroy() } }
        recognizer = null
        if (was && notify) events.onStopped(null)
    }

    private fun language(): String = if (bangla) "bn-BD" else {
        // the phone's own English (en-CA, en-GB, en-IN …) if it is set to English, otherwise US English
        val l = Locale.getDefault()
        if (l.language == "en" && l.country.isNotEmpty()) "en-${l.country}" else "en-US"
    }

    private fun listen() {
        if (!running) return
        val r = runCatching { SpeechRecognizer.createSpeechRecognizer(context) }.getOrNull()
        if (r == null) { running = false; events.onStopped("unavailable"); return }
        recognizer = r
        r.setRecognitionListener(listener)
        val intent = Intent(RecognizerIntent.ACTION_RECOGNIZE_SPEECH).apply {
            putExtra(RecognizerIntent.EXTRA_LANGUAGE_MODEL, RecognizerIntent.LANGUAGE_MODEL_FREE_FORM)
            putExtra(RecognizerIntent.EXTRA_LANGUAGE, language())
            putExtra(RecognizerIntent.EXTRA_PARTIAL_RESULTS, true)
            putExtra(RecognizerIntent.EXTRA_CALLING_PACKAGE, context.packageName)
            putExtra(RecognizerIntent.EXTRA_MAX_RESULTS, 1)
            if (offline) putExtra(RecognizerIntent.EXTRA_PREFER_OFFLINE, true)
            // let a pause in the middle of a sentence pass
            putExtra(RecognizerIntent.EXTRA_SPEECH_INPUT_COMPLETE_SILENCE_LENGTH_MILLIS, 2000L)
            putExtra(RecognizerIntent.EXTRA_SPEECH_INPUT_POSSIBLY_COMPLETE_SILENCE_LENGTH_MILLIS, 1500L)
            if (Build.VERSION.SDK_INT >= 33) putExtra(RecognizerIntent.EXTRA_ENABLE_FORMATTING, RecognizerIntent.FORMATTING_OPTIMIZE_QUALITY)
        }
        runCatching { r.startListening(intent) }.onFailure { running = false; events.onStopped("unavailable") }
    }

    private fun again() {
        recognizer?.let { runCatching { it.destroy() } }
        recognizer = null
        if (running) listen()
    }

    private val listener = object : RecognitionListener {
        override fun onReadyForSpeech(params: Bundle?) {}
        override fun onBeginningOfSpeech() {}
        override fun onRmsChanged(rmsdB: Float) {
            // about -2 (silence) to 10 (loud)
            events.onLevel(((rmsdB + 2f) / 12f).coerceIn(0f, 1f))
        }
        override fun onBufferReceived(buffer: ByteArray?) {}
        override fun onEndOfSpeech() {}
        override fun onPartialResults(partialResults: Bundle?) {
            val t = partialResults?.getStringArrayList(SpeechRecognizer.RESULTS_RECOGNITION)?.firstOrNull().orEmpty()
            if (t.isNotBlank()) events.onHeard(t)
        }
        override fun onResults(results: Bundle?) {
            val t = results?.getStringArrayList(SpeechRecognizer.RESULTS_RECOGNITION)?.firstOrNull().orEmpty().trim()
            if (t.isNotEmpty()) { quiet = 0; events.onSentence(t) } else quiet++
            if (quiet >= 2) { stop(); return }
            again()
        }
        override fun onError(error: Int) {
            if (!running) return
            when (error) {
                // nothing heard: listen once more, then stop
                SpeechRecognizer.ERROR_NO_MATCH, SpeechRecognizer.ERROR_SPEECH_TIMEOUT -> {
                    quiet++
                    if (quiet >= 2) stop() else again()
                }
                // the language isn't on the phone for offline use: ask the service without that wish
                SpeechRecognizer.ERROR_LANGUAGE_NOT_SUPPORTED, SpeechRecognizer.ERROR_LANGUAGE_UNAVAILABLE,
                SpeechRecognizer.ERROR_SERVER, SpeechRecognizer.ERROR_SERVER_DISCONNECTED -> {
                    if (offline) { offline = false; again() } else { running = false; events.onStopped(if (error == SpeechRecognizer.ERROR_LANGUAGE_NOT_SUPPORTED) "language" else "network") }
                }
                SpeechRecognizer.ERROR_NETWORK, SpeechRecognizer.ERROR_NETWORK_TIMEOUT -> {
                    if (offline) { offline = false; again() } else { running = false; events.onStopped("network") }
                }
                SpeechRecognizer.ERROR_INSUFFICIENT_PERMISSIONS -> { running = false; events.onStopped("permission") }
                SpeechRecognizer.ERROR_RECOGNIZER_BUSY, SpeechRecognizer.ERROR_TOO_MANY_REQUESTS -> { running = false; events.onStopped("busy") }
                else -> { running = false; events.onStopped("unavailable") }
            }
        }
        override fun onEvent(eventType: Int, params: Bundle?) {}
    }
}
