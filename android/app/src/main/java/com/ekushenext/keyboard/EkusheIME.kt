package com.ekushenext.keyboard

import android.content.Intent
import android.inputmethodservice.InputMethodService
import android.media.AudioManager
import android.os.Build
import android.text.InputType
import android.view.HapticFeedbackConstants
import android.view.KeyEvent
import android.view.View
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputMethodManager

/**
 * Ekushe Next as an Android keyboard. In Bangla, every key goes to the engine, which keeps the word
 * being typed (shown underlined in the app) and its suggestions; Space places the first suggestion.
 * In English, keys type themselves. Password and number fields get plain characters.
 */
class EkusheIME : InputMethodService(), KeyboardListener {
    private lateinit var prefs: Prefs
    private var view: KeyboardView? = null
    private var last = Output()
    /** Bangla typing is on (the user's choice; fields that need plain text override it) */
    private var bangla = true
    /** The field needs plain characters: a password, a number, an address */
    private var plainField = false
    private var privateField = false
    private var lastSpaceAt = 0L
    /** We are changing the app's text ourselves (ignore the selection updates that causes) */
    private var editing = false

    override fun onCreate() {
        super.onCreate()
        prefs = Prefs(this)
        bangla = prefs.bangla
        Engine.start(this)
    }

    override fun onCreateInputView(): View {
        val v = KeyboardView(this, this)
        view = v
        return v
    }

    override fun onEvaluateFullscreenMode() = false

    override fun onStartInputView(info: EditorInfo, restarting: Boolean) {
        super.onStartInputView(info, restarting)
        val v = view ?: return
        v.refreshTheme()
        val cls = info.inputType and InputType.TYPE_MASK_CLASS
        val variation = info.inputType and InputType.TYPE_MASK_VARIATION
        val password = cls == InputType.TYPE_CLASS_TEXT && variation in listOf(
            InputType.TYPE_TEXT_VARIATION_PASSWORD, InputType.TYPE_TEXT_VARIATION_VISIBLE_PASSWORD, InputType.TYPE_TEXT_VARIATION_WEB_PASSWORD,
        ) || cls == InputType.TYPE_CLASS_NUMBER && variation == InputType.TYPE_NUMBER_VARIATION_PASSWORD
        val address = cls == InputType.TYPE_CLASS_TEXT && variation in listOf(
            InputType.TYPE_TEXT_VARIATION_EMAIL_ADDRESS, InputType.TYPE_TEXT_VARIATION_WEB_EMAIL_ADDRESS, InputType.TYPE_TEXT_VARIATION_URI,
        )
        val noLearning = (info.imeOptions and EditorInfo.IME_FLAG_NO_PERSONALIZED_LEARNING) != 0
        privateField = password || noLearning
        plainField = password || address || cls == InputType.TYPE_CLASS_NUMBER || cls == InputType.TYPE_CLASS_PHONE || cls == InputType.TYPE_CLASS_DATETIME
        Engine.setPrivate(privateField)
        Engine.reset()
        last = Output()
        v.mode = when (cls) {
            InputType.TYPE_CLASS_NUMBER, InputType.TYPE_CLASS_DATETIME -> Mode.NUMBERS
            InputType.TYPE_CLASS_PHONE -> Mode.PHONE
            else -> Mode.LETTERS
        }
        v.bangla = typingBangla()
        v.enterLabel = enterGlyph(info)
        v.shift = Shift.OFF
        updateAutoShift()
        v.stripMessage = if (password) "পাসওয়ার্ড: সাধারণ অক্ষর" else null
        v.showSuggestions(emptyList(), emptyList(), emptyList())
        v.stripMessage = if (password) "পাসওয়ার্ড: সাধারণ অক্ষর · plain text" else null
    }

    override fun onFinishInputView(finishingInput: Boolean) {
        finishWord()
        Engine.save()
        super.onFinishInputView(finishingInput)
    }

    override fun onFinishInput() {
        finishWord()
        super.onFinishInput()
    }

    private fun typingBangla() = bangla && !plainField

    private fun enterGlyph(info: EditorInfo): String {
        if ((info.imeOptions and EditorInfo.IME_FLAG_NO_ENTER_ACTION) != 0) return "↵"
        return when (info.imeOptions and EditorInfo.IME_MASK_ACTION) {
            EditorInfo.IME_ACTION_SEARCH -> "🔍"
            EditorInfo.IME_ACTION_SEND -> "➤"
            EditorInfo.IME_ACTION_GO -> "→"
            EditorInfo.IME_ACTION_NEXT -> "⇥"
            EditorInfo.IME_ACTION_DONE -> "✓"
            else -> "↵"
        }
    }

    // ───────────────────────── the word being typed ─────────────────────────

    private fun apply(out: Output) {
        val ic = currentInputConnection ?: return
        editing = true
        ic.beginBatchEdit()
        if (out.deleteBefore > 0) {
            ic.finishComposingText()
            ic.deleteSurroundingText(out.deleteBefore, 0)
        }
        val committed = out.committed
        if (committed != null) {
            ic.commitText(committed, 1)
        }
        if (out.composing) ic.setComposingText(out.preedit.ifEmpty { out.roman }, 1)
        else if (committed == null && last.composing) ic.commitText("", 1)
        ic.endBatchEdit()
        editing = false
        last = out
        view?.showSuggestions(
            if (out.composing) out.candidates else emptyList(),
            if (out.composing) out.tips else emptyList(),
            if (!out.composing && !privateField) out.next else emptyList(),
        )
    }

    /** Close the word being typed, keeping what is shown */
    private fun finishWord() {
        if (last.composing) {
            currentInputConnection?.finishComposingText()
            Engine.reset()
            last = Output()
            view?.showSuggestions(emptyList(), emptyList(), emptyList())
        }
    }

    override fun onUpdateSelection(oldSelStart: Int, oldSelEnd: Int, newSelStart: Int, newSelEnd: Int, candidatesStart: Int, candidatesEnd: Int) {
        super.onUpdateSelection(oldSelStart, oldSelEnd, newSelStart, newSelEnd, candidatesStart, candidatesEnd)
        if (editing) return
        // The cursor moved away from the word being typed (a tap in the text): the word stays as shown
        if (last.composing && (candidatesStart == -1 || newSelEnd != candidatesEnd || newSelStart != newSelEnd)) {
            currentInputConnection?.finishComposingText()
            Engine.reset()
            last = Output()
            view?.showSuggestions(emptyList(), emptyList(), emptyList())
        }
        if (!last.composing) updateAutoShift()
    }

    /** English only: a capital letter at the start of a sentence (in Bangla, capitals change the sound) */
    private fun updateAutoShift() {
        val v = view ?: return
        if (v.mode != Mode.LETTERS || v.shift == Shift.LOCKED) return
        if (typingBangla()) { if (v.shift == Shift.ON) v.shift = Shift.OFF; return }
        val info = currentInputEditorInfo ?: return
        val caps = currentInputConnection?.getCursorCapsMode(info.inputType) ?: 0
        v.shift = if (caps != 0 && info.inputType and InputType.TYPE_MASK_CLASS == InputType.TYPE_CLASS_TEXT) Shift.ON else Shift.OFF
    }

    // ───────────────────────── keys ─────────────────────────

    override fun onText(text: String) {
        val ic = currentInputConnection ?: return
        if (!typingBangla()) {
            ic.commitText(text, 1)
            updateAutoShift()
            return
        }
        for (ch in text) {
            if (ch.code > 0x7f) {
                // Not a Roman letter or mark (৳, €, ©): close the word and type it as it is
                if (last.composing) apply(Engine.commit())
                ic.commitText(ch.toString(), 1)
                last = Output()
                view?.showSuggestions(emptyList(), emptyList(), emptyList())
            } else {
                apply(Engine.key(ch))
            }
        }
    }

    override fun onSpace() {
        val ic = currentInputConnection ?: return
        val now = System.currentTimeMillis()
        if (typingBangla()) {
            if (last.composing) {
                apply(Engine.select(last.selected, ' '))
                lastSpaceAt = now
                return
            }
            // Two quick spaces after a word: a dari (।)
            if (prefs.doubleSpaceDari && now - lastSpaceAt < 450) {
                val before = ic.getTextBeforeCursor(2, 0)?.toString() ?: ""
                if (before.length == 2 && before[1] == ' ' && before[0] != ' ' && before[0] != '।') {
                    ic.deleteSurroundingText(1, 0)
                    ic.commitText("। ", 1)
                    lastSpaceAt = 0
                    return
                }
            }
            apply(Engine.key(' '))
            lastSpaceAt = now
            return
        }
        if (now - lastSpaceAt < 450) {
            val before = ic.getTextBeforeCursor(2, 0)?.toString() ?: ""
            if (before.length == 2 && before[1] == ' ' && before[0].isLetterOrDigit()) {
                ic.deleteSurroundingText(1, 0)
                ic.commitText(". ", 1)
                lastSpaceAt = 0
                updateAutoShift()
                return
            }
        }
        ic.commitText(" ", 1)
        lastSpaceAt = now
        updateAutoShift()
    }

    override fun onDelete() {
        val ic = currentInputConnection ?: return
        if (typingBangla() && last.composing) { apply(Engine.backspace()); return }
        // Right after a word: bring its Roman letters back (once)
        // Right after a word: take it back and reopen its Roman letters with their suggestions, so
        // another word can be picked
        if (typingBangla() && Engine.hasUndo()) {
            val out = Engine.backspace()
            if (out.deleteBefore > 0) {
                editing = true
                ic.deleteSurroundingText(out.deleteBefore, 0)
                editing = false
                last = Output()
                out.committed?.forEach { apply(Engine.key(it)) }
                return
            }
        }
        val sel = ic.getSelectedText(0)
        if (!sel.isNullOrEmpty()) ic.commitText("", 1)
        else {
            ic.sendKeyEvent(KeyEvent(KeyEvent.ACTION_DOWN, KeyEvent.KEYCODE_DEL))
            ic.sendKeyEvent(KeyEvent(KeyEvent.ACTION_UP, KeyEvent.KEYCODE_DEL))
        }
        if (!typingBangla()) updateAutoShift()
    }

    override fun onEnter() {
        val ic = currentInputConnection ?: return
        if (last.composing) apply(Engine.commit())
        val info = currentInputEditorInfo
        val action = (info?.imeOptions ?: 0) and EditorInfo.IME_MASK_ACTION
        val noAction = info == null || (info.imeOptions and EditorInfo.IME_FLAG_NO_ENTER_ACTION) != 0
        if (!noAction && action != EditorInfo.IME_ACTION_NONE && action != EditorInfo.IME_ACTION_UNSPECIFIED) {
            ic.performEditorAction(action)
        } else {
            ic.sendKeyEvent(KeyEvent(KeyEvent.ACTION_DOWN, KeyEvent.KEYCODE_ENTER))
            ic.sendKeyEvent(KeyEvent(KeyEvent.ACTION_UP, KeyEvent.KEYCODE_ENTER))
        }
        updateAutoShift()
    }

    override fun onCandidate(index: Int) {
        if (!last.composing) return
        apply(Engine.select(index, ' '))
        lastSpaceAt = System.currentTimeMillis()
    }

    override fun onPrediction(word: String) {
        val ic = currentInputConnection ?: return
        ic.commitText("$word ", 1)
        // what comes after this one
        Engine.reset()
        last = Output()
        view?.showSuggestions(emptyList(), emptyList(), emptyList())
    }

    override fun onLanguageToggle() {
        finishWord()
        if (plainField) {
            view?.stripMessage = "এই ঘরে শুধু ইংরেজি · English only here"
            view?.invalidate()
            return
        }
        // English in Gboard: hand over, and come back in Bangla when Gboard's 🌐 key returns here
        if (bangla && prefs.gboardEnglish) {
            val id = Gboard.enabledId(this)
            if (id != null && handOver(id)) return
            view?.stripMessage = if (Gboard.installed(this)) "Gboard চালু নেই · turn on Gboard in Settings" else "Gboard নেই · Gboard isn't on this phone"
        }
        bangla = !bangla
        prefs.bangla = bangla
        view?.bangla = bangla
        view?.shift = Shift.OFF
        updateAutoShift()
    }

    /** Switch to another keyboard; false if Android refused */
    private fun handOver(id: String): Boolean = runCatching {
        prefs.bangla = true
        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.P) switchInputMethod(id)
        else {
            val token = window?.window?.attributes?.token ?: return false
            @Suppress("DEPRECATION")
            (getSystemService(INPUT_METHOD_SERVICE) as InputMethodManager).setInputMethod(token, id)
        }
        true
    }.getOrDefault(false)

    override fun onSwitchInputMethod() {
        val imm = getSystemService(INPUT_METHOD_SERVICE) as InputMethodManager
        imm.showInputMethodPicker()
    }

    override fun onCursorMove(steps: Int) {
        finishWord()
        val code = if (steps < 0) KeyEvent.KEYCODE_DPAD_LEFT else KeyEvent.KEYCODE_DPAD_RIGHT
        repeat(kotlin.math.abs(steps)) {
            currentInputConnection?.sendKeyEvent(KeyEvent(KeyEvent.ACTION_DOWN, code))
            currentInputConnection?.sendKeyEvent(KeyEvent(KeyEvent.ACTION_UP, code))
        }
    }

    override fun onSettings() {
        finishWord()
        startActivity(Intent(this, MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK).putExtra("open", "settings"))
    }

    override fun onHide() {
        finishWord()
        requestHideSelf(0)
    }

    override fun onEmojiCommitted(e: String) {
        finishWord()
        currentInputConnection?.commitText(e, 1)
    }

    override fun onKeyFeedback(view: View, kind: Int) {
        if (prefs.vibrate) {
            val k = if (kind == HapticFeedbackConstants.LONG_PRESS) HapticFeedbackConstants.LONG_PRESS else HapticFeedbackConstants.KEYBOARD_TAP
            @Suppress("DEPRECATION")
            val flags = if (Build.VERSION.SDK_INT < 33) HapticFeedbackConstants.FLAG_IGNORE_GLOBAL_SETTING else 0
            view.performHapticFeedback(k, flags)
        }
        if (prefs.sound) (getSystemService(AUDIO_SERVICE) as AudioManager).playSoundEffect(AudioManager.FX_KEYPRESS_STANDARD, 0.5f)
    }
}
