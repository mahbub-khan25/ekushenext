package com.ekushenext.keyboard

import android.content.Intent
import android.content.res.Configuration
import android.inputmethodservice.InputMethodService
import android.media.AudioManager
import android.os.Build
import android.provider.Settings
import android.text.InputType
import android.view.HapticFeedbackConstants
import android.view.KeyCharacterMap
import android.view.KeyEvent
import android.view.View
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputMethodManager

/**
 * Ekushe Next as an Android keyboard. In Bangla, every key goes to the engine, which keeps the word
 * being typed (shown underlined in the app) and its suggestions; Space places the first suggestion.
 * In English, the word being typed is shown underlined too, with completions and corrections from
 * English (a word list) and the phone's spell checker; Space places it, correcting a clear typo.
 * Password and number fields get plain characters.
 *
 * A physical keyboard (a tablet's keyboard cover, a Bluetooth keyboard) types through the engine too:
 * its keys are read as the US layout whatever layout Android picked for it, double-tap Shift (or
 * Ctrl+Space) switches Bangla ⇄ English as on a computer, Tab moves through the suggestions, and only
 * the suggestion strip is shown, not the keys.
 */
class EkusheIME : InputMethodService(), KeyboardListener {
    private lateinit var prefs: Prefs
    private var view: KeyboardView? = null
    private var last = Output()
    /** The Bangla word (with its space) last committed, so ⌫-undo only runs when it is still right
     *  before the cursor: after a tap elsewhere in the text, ⌫ must delete normally. */
    private var lastCommitted: String? = null
    /** Bangla typing is on (the user's choice; fields that need plain text override it) */
    private var bangla = true
    /** The field needs plain characters: a password, a number, an address */
    private var plainField = false
    private var privateField = false
    private var lastSpaceAt = 0L
    /** We are changing the app's text ourselves (ignore the selection updates that causes) */
    private var editing = false

    // ── English
    /** The English word being typed (shown underlined in the app) */
    private val en = StringBuilder()
    /** What the strip shows for it; the first is what Space places */
    private var enShown: List<String> = emptyList()
    /** After Space corrected a word: (what was typed, what was placed), so ⌫ can take it back */
    private var enUndo: Pair<String, String>? = null
    /** A word the person took back from a correction: leave it as typed */
    private var enKeep: String? = null
    private var spell: EnglishSpell? = null

    // ── a physical keyboard
    /** A key came from a physical keyboard in this field */
    private var hardTyping = false
    /** The keys were asked for again (⌨) while a physical keyboard is attached */
    private var keysWanted = false
    /** The suggestion Tab moved to (0: the engine's own choice) */
    private var hardSel = 0
    /** Keys handled here on the way down, so their key-up doesn't reach the app either */
    private val consumed = HashSet<Int>()
    /** Enter pressed while a word was open: the word is placed, then the Enter goes on to the app */
    private var enterForwarded = false
    private var lastShiftTap = 0L
    /** The US layout, to read a key whatever layout Android set for the physical keyboard */
    private val usKeys: KeyCharacterMap by lazy { KeyCharacterMap.load(KeyCharacterMap.VIRTUAL_KEYBOARD) }

    override fun onCreate() {
        super.onCreate()
        prefs = Prefs(this)
        bangla = prefs.bangla
        Engine.start(this)
        English.start(this)
        spell = EnglishSpell(this) { word -> if (en.toString() == word) showEnglish(ask = false) }
    }

    override fun onDestroy() {
        spell?.close()
        English.save()
        super.onDestroy()
    }

    override fun onCreateInputView(): View {
        val v = KeyboardView(this, this)
        view = v
        return v
    }

    override fun onEvaluateFullscreenMode() = false

    /** A physical keyboard is attached and open (not folded away) */
    private fun hardKeyboard(): Boolean {
        val c = resources.configuration
        return c.keyboard != Configuration.KEYBOARD_NOKEYS && c.hardKeyboardHidden != Configuration.HARDKEYBOARDHIDDEN_YES
    }

    /** Android's "Show on-screen keyboard" while a physical keyboard is attached */
    private fun systemShowsKeys() =
        runCatching { Settings.Secure.getInt(contentResolver, "show_ime_with_hard_keyboard", 0) == 1 }.getOrDefault(false)

    private fun updateStripOnly() {
        view?.stripOnly = !keysWanted && (hardTyping || (hardKeyboard() && !systemShowsKeys()))
    }

    // With a physical keyboard Android would hide the keyboard altogether; Ekushe shows its
    // suggestion strip, without which Bangla can't be typed
    override fun onEvaluateInputViewShown(): Boolean = hardKeyboard() || super.onEvaluateInputViewShown()

    // Android doesn't open a keyboard by itself (a tap in a text box) while a physical keyboard is
    // attached, unless "Show on-screen keyboard" is on. Without the strip there are no suggestions
    // to choose from, so Ekushe opens it (as the strip only)
    override fun onShowInputRequested(flags: Int, configChange: Boolean): Boolean =
        if (hardKeyboard()) true else super.onShowInputRequested(flags, configChange)

    override fun onStartInput(info: EditorInfo, restarting: Boolean) {
        super.onStartInput(info, restarting)
        // A new field: what it accepts, and typing starts fresh. This runs even when the keyboard
        // window stays hidden (a physical keyboard), so the field's rules hold for its keys too.
        classify(info)
        if (restarting && (last.composing || en.isNotEmpty())) currentInputConnection?.finishComposingText()
        Engine.reset()
        last = Output()
        lastCommitted = null
        en.clear(); enUndo = null; enKeep = null
        hardTyping = false; keysWanted = false; hardSel = 0
    }

    /** What the field accepts: passwords and private fields learn nothing; numbers, addresses and passwords get plain characters */
    private fun classify(info: EditorInfo): Boolean {
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
        return password
    }

    override fun onStartInputView(info: EditorInfo, restarting: Boolean) {
        super.onStartInputView(info, restarting)
        val v = view ?: return
        v.refreshTheme()
        val cls = info.inputType and InputType.TYPE_MASK_CLASS
        val password = classify(info)
        // The window opening in the middle of a word typed on a physical keyboard (it was hidden):
        // keep the word, and show its suggestions
        val midWord = !restarting && hardTyping && (last.composing || en.isNotEmpty())
        if (!midWord) {
            // An app's restartInput arrives without onFinishInput: a half-typed word still has its
            // composing span in the editor, which the next setComposingText would replace. Close it.
            if (restarting && (last.composing || en.isNotEmpty())) currentInputConnection?.finishComposingText()
            Engine.reset()
            last = Output()
            lastCommitted = null
            en.clear(); enUndo = null; enKeep = null
            if (!hardTyping) { keysWanted = false; hardSel = 0 }
        }
        v.mode = when (cls) {
            InputType.TYPE_CLASS_NUMBER, InputType.TYPE_CLASS_DATETIME -> Mode.NUMBERS
            InputType.TYPE_CLASS_PHONE -> Mode.PHONE
            else -> Mode.LETTERS
        }
        v.bangla = typingBangla()
        v.enterLabel = enterGlyph(info)
        v.shift = Shift.OFF
        updateAutoShift()
        if (midWord && last.composing) v.showSuggestions(last.candidates, last.tips, emptyList())
        else v.showSuggestions(emptyList(), emptyList(), emptyList())
        v.stripMessage = if (password) "পাসওয়ার্ড: সাধারণ অক্ষর · plain text" else null
        updateStripOnly()
    }

    override fun onFinishInputView(finishingInput: Boolean) {
        finishWord()
        Engine.save()
        English.save()
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
        hardSel = 0
        if (committed != null && !out.composing) lastCommitted = committed
        view?.showSuggestions(
            if (out.composing) out.candidates else emptyList(),
            if (out.composing) out.tips else emptyList(),
            if (!out.composing && !privateField) out.next else emptyList(),
        )
    }

    /** Close the word being typed, keeping what is shown */
    private fun finishWord() {
        if (en.isNotEmpty()) {
            currentInputConnection?.finishComposingText()
            en.clear()
            view?.showSuggestions(emptyList(), emptyList(), emptyList())
        }
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
        // Apps report cursor changes late: with fast typing (a physical keyboard) the report for the
        // word just placed can arrive after the next word was started. Before treating a report as
        // "the cursor moved away", check the text: if the word being typed is still right before the
        // cursor, the report is an old one.
        fun stillAtWord(shown: String): Boolean = newSelStart == newSelEnd && shown.isNotEmpty() &&
            currentInputConnection?.getTextBeforeCursor(shown.length, 0)?.toString() == shown
        if (en.isNotEmpty() && (candidatesStart == -1 || newSelEnd != candidatesEnd || newSelStart != newSelEnd) && !stillAtWord(en.toString())) {
            currentInputConnection?.finishComposingText()
            en.clear(); enUndo = null
            view?.showSuggestions(emptyList(), emptyList(), emptyList())
        }
        // The cursor moved away from the word being typed (a tap in the text): the word stays as shown
        if (last.composing && (candidatesStart == -1 || newSelEnd != candidatesEnd || newSelStart != newSelEnd) && !stillAtWord(last.preedit.ifEmpty { last.roman })) {
            currentInputConnection?.finishComposingText()
            Engine.reset()
            last = Output()
            view?.showSuggestions(emptyList(), emptyList(), emptyList())
        }
        if (newSelStart != newSelEnd) lastCommitted = null
        if (!last.composing && en.isEmpty()) updateAutoShift()
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
            if (!englishHelp()) { ic.commitText(text, 1); updateAutoShift(); return }
            for (ch in text) {
                if ((ch.isLetter() && ch.code < 0x250) || (ch == '\'' && en.isNotEmpty())) {
                    en.append(ch); enUndo = null
                    showEnglish()
                } else {
                    // a mark or digit ends the word, as typed (makhan.aumi@gmail stays exactly that)
                    if (en.isNotEmpty()) placeEnglish(correct = false)
                    ic.commitText(ch.toString(), 1)
                    enUndo = null
                    view?.showSuggestions(emptyList(), emptyList(), emptyList())
                }
            }
            if (en.isEmpty()) updateAutoShift()
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
                apply(Engine.select(if (hardSel > 0) hardSel else last.selected, ' '))
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
        if (en.isNotEmpty()) {
            placeEnglish(correct = true)
            ic.commitText(" ", 1)
            lastSpaceAt = now
            showNextEnglish()
            updateAutoShift()
            return
        }
        enUndo = null
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
            // Only when the committed word is still right before the cursor (not after a tap
            // elsewhere): otherwise forget the undo and delete normally
            val word = lastCommitted
            val before = word?.let { ic.getTextBeforeCursor(it.length, 0)?.toString() }
            if (word != null && before == word) {
                val out = Engine.backspace()
                if (out.deleteBefore > 0) {
                    // one edit for the app: removing the word and opening its letters again, so it
                    // reports one cursor change (with the word open), not the steps in between
                    ic.beginBatchEdit()
                    editing = true
                    ic.deleteSurroundingText(out.deleteBefore, 0)
                    editing = false
                    last = Output()
                    lastCommitted = null
                    out.committed?.forEach { apply(Engine.key(it)) }
                    ic.endBatchEdit()
                    return
                }
            } else {
                Engine.reset()
                lastCommitted = null
            }
        }
        if (!typingBangla() && en.isNotEmpty()) {
            en.deleteCharAt(en.length - 1)
            if (en.isEmpty()) {
                editing = true; ic.commitText("", 1); editing = false
                view?.showSuggestions(emptyList(), emptyList(), emptyList())
                updateAutoShift()
            } else showEnglish()
            return
        }
        // Right after Space corrected a word: put back what was typed, and leave it as it is
        enUndo?.let { (typed, placed) ->
            enUndo = null
            if (!typingBangla() && ic.getTextBeforeCursor(placed.length + 1, 0)?.toString() == "$placed ") {
                editing = true; ic.deleteSurroundingText(placed.length + 1, 0); editing = false
                en.append(typed); enKeep = typed
                showEnglish()
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
        if (en.isNotEmpty()) { placeEnglish(correct = false); view?.showSuggestions(emptyList(), emptyList(), emptyList()) }
        enUndo = null
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
        if (en.isNotEmpty()) {
            val word = enShown.getOrNull(index) ?: return
            val ic = currentInputConnection ?: return
            learnEnglish(word)
            editing = true; ic.commitText(word, 1); ic.commitText(" ", 1); editing = false
            en.clear(); enUndo = null; enKeep = null
            lastSpaceAt = System.currentTimeMillis()
            showNextEnglish(); updateAutoShift()
            return
        }
        if (!last.composing) return
        apply(Engine.select(index, ' '))
        lastSpaceAt = System.currentTimeMillis()
    }

    override fun onPrediction(word: String) {
        val ic = currentInputConnection ?: return
        if (!typingBangla()) {
            // the sentence's first word gets its capital
            val w = if (view?.shift == Shift.ON) word.replaceFirstChar { it.uppercase() } else word
            learnEnglish(w)
            ic.commitText("$w ", 1)
            showNextEnglish(); updateAutoShift()
            return
        }
        ic.commitText("$word ", 1)
        // what comes after this one
        Engine.reset()
        last = Output()
        view?.showSuggestions(emptyList(), emptyList(), emptyList())
    }

    // ───────────────────────── English help ─────────────────────────

    /** Suggestions and corrections in English: on, unless the field wants plain text */
    private fun englishHelp() = !plainField

    /** What Space should place instead of what was typed, if it's a clear typo (or "dont", "i") */
    private fun englishFix(typed: String): String? {
        if (!prefs.enAutocorrect || typed == enKeep) return null
        // part of an email, a web address or a code (gmail.com, a@b, #tag): never touched
        if (gluedToText(typed.length)) return null
        // i → I and dont → don't come first (they read as Banglish too: দন্ত)
        if (English.isContraction(typed)) English.canonical(typed)?.let { return it }
        // Banglish written in English mode (ami, tomake, bhalobashi) stays as it is
        if (Engine.isBanglish(typed)) return null
        English.canonical(typed)?.let { return it }
        if (typed.length < 3 || English.known(typed)) return null
        if (typed.drop(1).any { it.isUpperCase() }) return null // names, acronyms
        // a capital letter in the middle of a sentence is most likely a name (Rahim, Sylhet)
        if (typed.first().isUpperCase() && !sentenceStart(typed.length)) return null
        // the phone's guesses and the word list's, the most common word wins (teh → the, not tech)
        val guesses = (spell?.answer(typed)?.takeIf { it.typo }?.suggestions?.take(3).orEmpty().map { English.spell(typed, it) } +
            English.corrections(typed, 3)).distinct()
        return English.best(guesses)
    }

    private fun showEnglish(ask: Boolean = true) {
        val ic = currentInputConnection ?: return
        val typed = en.toString()
        editing = true; ic.setComposingText(typed, 1); editing = false
        val fix = englishFix(typed)
        val list = ArrayList<String>()
        if (fix != null) { list += fix; list += typed } else list += typed
        spell?.answer(typed)?.let { a -> if (a.typo) list += a.suggestions.take(2).map { English.spell(typed, it) } }
        list += English.complete(typed, 3)
        if (!English.known(typed)) list += English.corrections(typed, 2)
        enShown = list.filter { it.isNotEmpty() }.distinct().take(6)
        view?.showSuggestions(enShown.map { Candidate(it, "") }, emptyList(), emptyList())
        // Never in a private field: the phone's spell checker is another app's service
        if (ask && !privateField && typed.length >= 3 && !English.known(typed)) spell?.ask(typed)
    }

    /** The word being typed follows a mark with no space (a.b, x@y, #z): an address or a code */
    private fun gluedToText(len: Int): Boolean {
        val before = currentInputConnection?.getTextBeforeCursor(len + 1, 0)?.toString() ?: return false
        if (before.length <= len) return false
        val c = before[0]
        return !c.isWhitespace() && c != '(' && c != '"' && c != '“'
    }

    /** The word being typed starts a sentence (or the text) */
    private fun sentenceStart(len: Int): Boolean {
        val before = currentInputConnection?.getTextBeforeCursor(len + 40, 0)?.toString() ?: return true
        val head = before.dropLast(len).trimEnd()
        return head.isEmpty() || head.last() in ".!?\n।"
    }

    /** Place the English word being typed (corrected if asked and it's a clear typo) */
    private fun placeEnglish(correct: Boolean) {
        val ic = currentInputConnection ?: return
        val typed = en.toString()
        val word = (if (correct) englishFix(typed) else null) ?: typed
        learnEnglish(word)
        editing = true; ic.commitText(word, 1); editing = false
        enUndo = if (word != typed) typed to word else null
        if (typed != enKeep) enKeep = null
        en.clear()
    }

    /** Learn a word, and what it followed. While it is still being typed, it is the last word before
     *  the cursor itself, so what it followed is the one before that. */
    private fun learnEnglish(word: String) {
        if (privateField) return
        val (a, b) = previousWords()
        English.learn(word, if (en.isNotEmpty()) a else b)
    }

    /** The two words before the cursor (call before placing a word, to learn what it followed) */
    private fun previousWords(): Pair<String?, String?> {
        val before = currentInputConnection?.getTextBeforeCursor(60, 0)?.toString() ?: return null to null
        val ws = Regex("[A-Za-z']+").findAll(before).map { it.value }.toList()
        // a sentence end resets
        val tail = before.trimEnd()
        if (tail.endsWith('.') || tail.endsWith('!') || tail.endsWith('?')) return null to null
        return ws.getOrNull(ws.size - 2) to ws.lastOrNull()
    }

    private fun showNextEnglish() {
        if (!prefs.nextWordEnglish) { view?.showSuggestions(emptyList(), emptyList(), emptyList()); return }
        val (a, b) = previousWords()
        view?.showSuggestions(emptyList(), emptyList(), English.nextWords(a, b))
    }

    // ───────────────────────── a physical keyboard ─────────────────────────

    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean {
        val handled = event.isFromSource(android.view.InputDevice.SOURCE_KEYBOARD) && hardKey(keyCode, event)
        // debug builds: what each physical key did, for the automated test
        if (debuggable) android.util.Log.i("EkusheHard", "${KeyEvent.keyCodeToString(keyCode)} t=${event.eventTime} src=${event.source} composing=${last.composing} n=${last.candidates.size} sel=$hardSel bangla=${typingBangla()} handled=$handled")
        if (handled) {
            consumed += keyCode
            return true
        }
        return super.onKeyDown(keyCode, event)
    }
    private val debuggable by lazy { (applicationInfo.flags and android.content.pm.ApplicationInfo.FLAG_DEBUGGABLE) != 0 }

    override fun onKeyUp(keyCode: Int, event: KeyEvent): Boolean {
        if (consumed.remove(keyCode)) {
            if (enterForwarded && (keyCode == KeyEvent.KEYCODE_ENTER || keyCode == KeyEvent.KEYCODE_NUMPAD_ENTER)) {
                enterForwarded = false
                currentInputConnection?.sendKeyEvent(event)
            }
            return true
        }
        return super.onKeyUp(keyCode, event)
    }

    /** A key from a physical keyboard; true if it was handled here (the app doesn't get it) */
    private fun hardKey(keyCode: Int, event: KeyEvent): Boolean {
        val ic = currentInputConnection ?: return false
        // Back, Home, volume and the like (the navigation bar's buttons arrive as keys too) aren't typing
        if (event.isSystem) return false
        if (keyCode == KeyEvent.KEYCODE_SHIFT_LEFT || keyCode == KeyEvent.KEYCODE_SHIFT_RIGHT) {
            // Shift tapped twice in a row with nothing between: Bangla ⇄ English, as on a computer
            if (event.repeatCount == 0) {
                if (lastShiftTap > 0 && event.eventTime - lastShiftTap < 400) { lastShiftTap = 0; toggleLanguage(handOver = false) }
                else lastShiftTap = event.eventTime
            }
            return false
        }
        lastShiftTap = 0
        if (keyCode == KeyEvent.KEYCODE_LANGUAGE_SWITCH || (keyCode == KeyEvent.KEYCODE_SPACE && event.isCtrlPressed)) {
            if (event.repeatCount == 0) toggleLanguage(handOver = false)
            return true
        }
        // shortcuts (Ctrl+C, Alt+Tab …) are the app's; the word being typed stays as shown
        if (event.isCtrlPressed || event.isAltPressed || event.isMetaPressed) { finishWord(); return false }
        val typing = keyCode in TYPING_KEYS || typedChar(event) != null
        if (typing && !hardTyping) { hardTyping = true; updateStripOnly() }
        if (typing && !isInputViewShown && Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) requestShowSelf(0)
        val bn = typingBangla()
        when (keyCode) {
            KeyEvent.KEYCODE_SPACE -> {
                if (!bn && en.isEmpty()) return false
                onSpace(); return true
            }
            KeyEvent.KEYCODE_DEL -> {
                if (bn && (last.composing || canReopen())) { onDelete(); return true }
                if (!bn && en.isNotEmpty()) { onDelete(); return true }
                if (Engine.hasUndo()) { Engine.reset(); lastCommitted = null }
                return false
            }
            KeyEvent.KEYCODE_ENTER, KeyEvent.KEYCODE_NUMPAD_ENTER -> {
                if (!last.composing && en.isEmpty()) return false
                // place the word, then let the app have its Enter (a new line, or Send)
                placeHighlighted()
                ic.sendKeyEvent(event)
                enterForwarded = true
                return true
            }
            KeyEvent.KEYCODE_TAB -> {
                val n = minOf(last.candidates.size, 9)
                if (bn && last.composing && n > 1) {
                    val cur = if (hardSel > 0) hardSel else last.selected
                    hardSel = (cur + if (event.isShiftPressed) n - 1 else 1) % n
                    view?.selected = hardSel
                    return true
                }
                finishWord(); return false
            }
            KeyEvent.KEYCODE_ESCAPE -> {
                if (!last.composing && en.isEmpty()) return false
                placeHighlighted(); return true
            }
            KeyEvent.KEYCODE_DPAD_LEFT, KeyEvent.KEYCODE_DPAD_RIGHT, KeyEvent.KEYCODE_DPAD_UP, KeyEvent.KEYCODE_DPAD_DOWN,
            KeyEvent.KEYCODE_MOVE_HOME, KeyEvent.KEYCODE_MOVE_END, KeyEvent.KEYCODE_PAGE_UP, KeyEvent.KEYCODE_PAGE_DOWN,
            KeyEvent.KEYCODE_FORWARD_DEL -> { finishWord(); return false }
        }
        val ch = typedChar(event) ?: return false
        if (bn) { onText(ch.toString()); return true }
        // English: the app types the key itself, unless Android's layout for this keyboard gives another script
        if (en.isNotEmpty()) finishWord()
        val own = event.getUnicodeChar(event.metaState)
        if (own in 0x20..0x7e) return false
        ic.commitText(ch.toString(), 1)
        return true
    }

    private companion object {
        val TYPING_KEYS = setOf(KeyEvent.KEYCODE_SPACE, KeyEvent.KEYCODE_DEL, KeyEvent.KEYCODE_ENTER, KeyEvent.KEYCODE_NUMPAD_ENTER, KeyEvent.KEYCODE_TAB, KeyEvent.KEYCODE_ESCAPE)
    }

    /** The character a key types on the US layout (a–z, digits, marks), with Shift and Caps Lock */
    private fun typedChar(e: KeyEvent): Char? {
        val own = e.getUnicodeChar(e.metaState)
        if (own in 0x21..0x7e) return own.toChar()
        val us = usKeys.get(e.keyCode, e.metaState and (KeyEvent.META_SHIFT_MASK or KeyEvent.META_CAPS_LOCK_ON))
        return if (us in 0x21..0x7e) us.toChar() else null
    }

    /** The word just placed is still right before the cursor, so Backspace can reopen it */
    private fun canReopen(): Boolean {
        if (!Engine.hasUndo()) return false
        val word = lastCommitted ?: return false
        return currentInputConnection?.getTextBeforeCursor(word.length, 0)?.toString() == word
    }

    /** Place the word being typed as highlighted (Enter, Esc), with no space after it */
    private fun placeHighlighted() {
        if (last.composing) apply(Engine.select(if (hardSel > 0) hardSel else last.selected))
        if (en.isNotEmpty()) { placeEnglish(correct = false); view?.showSuggestions(emptyList(), emptyList(), emptyList()) }
    }

    override fun onShowKeys() {
        keysWanted = true
        updateStripOnly()
    }

    override fun onLanguageToggle() = toggleLanguage(handOver = true)

    /** Bangla ⇄ English. From the on-screen key, English can mean Gboard; from a physical keyboard it never does */
    private fun toggleLanguage(handOver: Boolean) {
        finishWord()
        if (plainField) {
            view?.stripMessage = "এই ঘরে শুধু ইংরেজি · English only here"
            view?.invalidate()
            return
        }
        // English in Gboard: hand over, and come back in Bangla when Gboard's 🌐 key returns here
        if (handOver && bangla && prefs.gboardEnglish) {
            val id = Gboard.enabledId(this)
            if (id != null && handOver(id)) return
            view?.stripMessage = if (Gboard.installed(this)) "Gboard চালু নেই · turn on Gboard in Settings" else "Gboard নেই · Gboard isn't on this phone"
        }
        bangla = !bangla
        prefs.bangla = bangla
        view?.bangla = bangla
        view?.shift = Shift.OFF
        if (!handOver) { view?.stripMessage = if (bangla) "বাংলা" else "English"; view?.invalidate() }
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
