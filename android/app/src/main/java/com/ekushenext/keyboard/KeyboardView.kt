package com.ekushenext.keyboard

import android.annotation.SuppressLint
import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Rect
import android.graphics.Paint
import android.graphics.RectF
import android.graphics.Typeface
import android.os.Handler
import android.os.Looper
import android.util.Log
import android.content.pm.ApplicationInfo
import android.view.HapticFeedbackConstants
import android.view.MotionEvent
import android.view.View
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min

/** What the keyboard asks the input method to do. */
interface KeyboardListener {
    fun onText(text: String)
    fun onDelete()
    fun onEnter()
    fun onSpace()
    fun onCandidate(index: Int)
    fun onPrediction(word: String)
    fun onLanguageToggle()
    fun onSwitchInputMethod()
    fun onCursorMove(steps: Int)
    fun onSettings()
    fun onHide()
    fun onEmojiCommitted(e: String)
    fun onKeyFeedback(view: View, kind: Int)
    /** Show the keys again while a physical keyboard is attached (the ⌨ button in the strip) */
    fun onShowKeys()
    /** The ? button: how to type conjuncts, hasanta and the other marks */
    fun onHelp()
}

enum class Mode { LETTERS, SYMBOLS, MORE_SYMBOLS, NUMBERS, PHONE, EMOJI }
enum class Shift { OFF, ON, LOCKED }

/**
 * The whole keyboard, drawn on one canvas: the suggestion strip, the keys, the key preview and the
 * long-press choices. Characters are typed when a finger lifts (so a slide can still change the
 * choice); a second finger landing types the first finger's key straight away.
 */
@SuppressLint("ViewConstructor")
class KeyboardView(context: Context, private val listener: KeyboardListener) : View(context) {
    private val dp = resources.displayMetrics.density
    private val prefs = Prefs(context)
    var palette: Palette = Palette.of(context)
        private set

    // ── state from the input method
    var mode = Mode.LETTERS
        set(v) { field = v; buildKeys(); invalidate() }
    var shift = Shift.OFF
        set(v) { field = v; invalidate() }
    var bangla = true
        set(v) { field = v; invalidate() }
    var enterLabel = "↵"
        set(v) { field = v; invalidate() }
    /** Suggestions for the word being typed (first = what Space places) */
    var candidates: List<Candidate> = emptyList()
    var tips: List<Hint> = emptyList()
    var predictions: List<String> = emptyList()
    var stripMessage: String? = null
    /** The highlighted suggestion (what Space places); a physical keyboard's Tab moves it */
    var selected = 0
        set(v) { field = v; invalidate() }
    /** A physical keyboard is in use: only the suggestion strip is shown, not the keys */
    var stripOnly = false
        set(v) { if (field != v) { field = v; if (v && mode == Mode.EMOJI) mode = Mode.LETTERS; requestLayout(); invalidate() } }
    /** Room under the strip-only bar (px) for a desktop taskbar that stays on screen over the keyboard */
    var under = 0f
        set(v) { if (field != v) { field = v; if (stripOnly) requestLayout(); invalidate() } }

    fun showSuggestions(c: List<Candidate>, t: List<Hint>, p: List<String>) {
        candidates = c; tips = t; predictions = p; stripScroll = 0f; stripMessage = null; selected = 0
        invalidate()
    }

    // ── geometry
    private val stripH get() = 46 * dp
    private val rowH get() = (when (prefs.height) { 0 -> 48; 2 -> 60; else -> 54 }) * dp
    private val gap get() = 3.5f * dp
    private val bottomPad get() = 6 * dp

    private data class Placed(val key: Key, val rect: RectF)
    private var placed: List<Placed> = emptyList()
    private var rows: List<List<Key>> = emptyList()

    private val keyPaint = Paint(Paint.ANTI_ALIAS_FLAG)
    private val textPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply { textAlign = Paint.Align.CENTER }
    private val glyph = Rect()
    private val hintPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply { textAlign = Paint.Align.RIGHT }
    // The phone's own Bangla font (Noto Sans Bengali on Android), as the rest of the phone uses:
    // clearer on small keys than a display font, digits (১২৩) included
    private val bnFace: Typeface = Typeface.create("sans-serif-medium", Typeface.NORMAL)
    private val bnFaceRegular: Typeface = Typeface.create("sans-serif", Typeface.NORMAL)

    init {
        isHapticFeedbackEnabled = true
        buildKeys()
    }

    // ── a photo behind the keys
    private var photo: Bitmap? = null
    private var photoStamp = -1L
    private val photoPaint = Paint(Paint.FILTER_BITMAP_FLAG or Paint.ANTI_ALIAS_FLAG)
    private var borders = true

    private var effect = "classic"

    fun refreshTheme() {
        borders = prefs.keyBorders
        effect = prefs.pressEffect
        var p = Palette.of(context)
        if (prefs.photo) {
            if (photo == null || photoStamp != prefs.photoStamp) {
                photo = runCatching { BitmapFactory.decodeFile(Prefs.photoFile(context).path) }.getOrNull()
                photoStamp = prefs.photoStamp
            }
            if (photo != null) p = p.overPhoto()
        } else photo = null
        palette = p
        buildKeys(); requestLayout(); invalidate()
    }

    private fun drawPhoto(c: Canvas, b: Bitmap) {
        // fill the keyboard, cropping the photo's edges (centre crop)
        val scale = max(width.toFloat() / b.width, height.toFloat() / b.height)
        val sw = (width / scale).toInt(); val sh = (height / scale).toInt()
        val sx = (b.width - sw) / 2; val sy = (b.height - sh) / 2
        c.drawBitmap(b, Rect(sx, sy, sx + sw, sy + sh), RectF(0f, 0f, width.toFloat(), height.toFloat()), photoPaint)
        val dim = (prefs.photoDim.coerceIn(0, 80) * 255 / 100)
        c.drawColor(if (palette.dark) Color.argb(dim, 0, 0, 0) else Color.argb(dim, 255, 255, 255))
    }

    private fun buildKeys() {
        Layouts.emojiKey = prefs.emojiKey
        rows = when (mode) {
            Mode.LETTERS -> Layouts.letters(prefs.numberRow)
            Mode.SYMBOLS -> Layouts.symbols()
            Mode.MORE_SYMBOLS -> Layouts.moreSymbols()
            Mode.NUMBERS -> Layouts.numbers(false)
            Mode.PHONE -> Layouts.numbers(true)
            Mode.EMOJI -> emptyList()
        }
        layoutKeys()
    }

    private fun rowsCount() = if (mode == Mode.EMOJI) 4 else rows.size

    override fun onMeasure(widthMeasureSpec: Int, heightMeasureSpec: Int) {
        val w = MeasureSpec.getSize(widthMeasureSpec)
        val letterRows = if (prefs.numberRow) 5 else 4
        val h = if (stripOnly) stripH + under else stripH + letterRows * rowH + bottomPad
        setMeasuredDimension(w, h.toInt())
    }

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) = layoutKeys()

    private fun layoutKeys() {
        if (width == 0) return
        val out = ArrayList<Placed>()
        val areaTop = stripH
        val areaH = height - stripH - bottomPad
        val rh = areaH / max(1, rows.size)
        val unitsMax = rows.maxOfOrNull { r -> r.sumOf { it.width.toDouble() } }?.toFloat() ?: 10f
        // the number pad spreads its four columns across; letters keep ten-key widths
        val pad = mode == Mode.NUMBERS || mode == Mode.PHONE
        val unit = (width - gap) / (if (pad) unitsMax else max(10f, unitsMax))
        rows.forEachIndexed { r, row ->
            val units = row.sumOf { it.width.toDouble() }.toFloat()
            var x = (width - units * unit) / 2f
            val y = areaTop + r * rh
            row.forEach { k ->
                val w = k.width * unit
                out += Placed(k, RectF(x + gap / 2, y + gap / 2, x + w - gap / 2, y + rh - gap / 2))
                x += w
            }
        }
        placed = out
        logged = false
    }

    // Debug builds report where each key is, so the automated test can tap them
    private val debuggable = (context.applicationInfo.flags and ApplicationInfo.FLAG_DEBUGGABLE) != 0
    private var logged = false
    private var loggedAt = IntArray(2)
    // The slide-in animation moves the window without redrawing, so look again a few times after
    private var relogLeft = 0
    private val relog = object : Runnable {
        override fun run() {
            logKeys()
            if (--relogLeft > 0) postDelayed(this, 500)
        }
    }
    private fun logKeys() {
        if (!debuggable || (placed.isEmpty() && !stripOnly)) return
        val loc = IntArray(2)
        getLocationOnScreen(loc)
        // again whenever the keyboard has moved (it slides in when it opens)
        if (logged && loc.contentEquals(loggedAt)) return
        logged = true
        loggedAt = loc
        val keys = if (stripOnly) "" else placed.joinToString(",") { (k, r) ->
            val id = if (k.type == KeyType.CHAR) k.text.replace("\\", "\\\\").replace("\"", "\\\"") else k.type.name
            "{\"k\":\"$id\",\"x\":${(loc[0] + r.centerX()).toInt()},\"y\":${(loc[1] + r.centerY()).toInt()}}"
        }
        Log.i("EkusheKeys", "{\"mode\":\"${if (stripOnly) "STRIP_ONLY" else mode.name}\",\"strip\":${loc[1] + (stripH / 2).toInt()},\"keys\":[$keys]}")
    }

    // ───────────────────────── drawing ─────────────────────────

    override fun onDraw(c: Canvas) {
        val p = palette
        val b = photo
        if (b != null) drawPhoto(c, b) else c.drawColor(p.background)
        drawStrip(c)
        if (stripOnly) { drawUnder(c); logged = false; logKeys(); return }
        if (mode == Mode.EMOJI) { drawEmoji(c); return }
        logKeys()
        if (debuggable) { removeCallbacks(relog); relogLeft = 8; postDelayed(relog, 500) }
        for ((k, r) in placed) drawKey(c, k, r, pressedKey == k)
        pressedKey?.let { k -> placed.firstOrNull { it.key == k }?.let { if (k.type == KeyType.CHAR && altsOpen == null) drawPreview(c, k, it.rect) } }
        altsOpen?.let { drawAlts(c, it) }
    }

    /** The room under the strip: the physical keyboard's shortcuts, in case the taskbar steps aside */
    private fun drawUnder(c: Canvas) {
        if (under < 40 * dp) return
        textPaint.color = palette.subtext
        textPaint.textSize = 12.5f * dp
        val t = if (bangla) "Shift দুবার: বাংলা ⇄ English  ·  Tab: পরের শব্দ  ·  Space: বসায়" else "Shift twice: English ⇄ বাংলা  ·  Tab: next word  ·  Space: place it"
        c.drawText(t, width / 2f, stripH + under / 2f + 4.5f * dp, textPaint)
    }

    private fun isFunction(k: Key) = k.type != KeyType.CHAR && k.type != KeyType.SPACE

    private fun keyText(k: Key): String = when {
        k.letter && shift != Shift.OFF -> k.label.uppercase()
        // Symbols key: Bangla digits alone in Bangla ("?" next to them sat oddly high), ?123 in English
        k.type == KeyType.SYMBOLS && k.label == "?১২৩" -> if (bangla) "১২৩" else "?123"
        mode == Mode.SYMBOLS && k.label.length == 1 && k.label[0].isDigit() -> Layouts.digitLabel(k.label[0], bangla)
        mode == Mode.LETTERS && k.label.length == 1 && k.label[0].isDigit() -> Layouts.digitLabel(k.label[0], bangla)
        else -> k.label
    }

    // ── key-press effects
    private class Ripple(val slot: RectF, val x: Float, val y: Float, val start: Long)
    private val ripples = ArrayList<Ripple>()
    private val rippleMs = 360L

    private fun drawKey(c: Canvas, k: Key, slot: RectF, pressed: Boolean) {
        val p = palette
        val radius = p.keyRadius * dp
        val enter = k.type == KeyType.ENTER
        val shiftOn = k.type == KeyType.SHIFT && shift != Shift.OFF
        val base = when {
            enter -> p.enter ?: p.accent
            shiftOn -> if (shift == Shift.LOCKED) p.accent else p.keyPressed
            isFunction(k) -> p.functionKey
            else -> p.key
        }
        // Flat look: only Space, Enter and a held key keep a shape
        val shaped = borders || pressed || enter || shiftOn || k.type == KeyType.SPACE
        val depth = 4.5f * dp
        // where the key's face is drawn (and its label with it)
        val r = when {
            effect == "3d" && shaped -> if (pressed) RectF(slot.left, slot.top + depth, slot.right, slot.bottom) else RectF(slot.left, slot.top, slot.right, slot.bottom - depth)
            effect == "bounce" && pressed -> RectF(slot).apply { inset(slot.width() * 0.07f, slot.height() * 0.08f) }
            else -> slot
        }
        if (shaped) {
            val fill = if (!borders && !pressed && k.type == KeyType.SPACE) p.functionKey else base
            when (effect) {
                "3d" -> {
                    // the key's side: a darker edge below the face, gone when it is pressed down
                    if (!pressed) {
                        keyPaint.color = withAlphaOf(blend(fill, android.graphics.Color.BLACK, if (p.dark) 0.62f else 0.3f), fill)
                        c.drawRoundRect(slot.left, slot.top + depth, slot.right, slot.bottom, radius, radius, keyPaint)
                    }
                    keyPaint.color = if (pressed) withAlphaOf(blend(fill, p.text, 0.1f), fill) else fill
                    c.drawRoundRect(r, radius, radius, keyPaint)
                    // light catching the top of a raised key
                    if (!pressed) {
                        keyPaint.color = Palette.alpha(android.graphics.Color.WHITE, if (p.dark) 0.10f else 0.55f)
                        c.save(); c.clipRect(r.left, r.top, r.right, r.top + radius)
                        c.drawRoundRect(r.left + 1.5f * dp, r.top + 1f * dp, r.right - 1.5f * dp, r.top + 2 * radius, radius, radius, keyPaint)
                        c.restore()
                        keyPaint.color = fill
                        c.drawRoundRect(r.left + 1.5f * dp, r.top + 2.2f * dp, r.right - 1.5f * dp, r.bottom, radius, radius, keyPaint)
                    }
                }
                "glow" -> {
                    if (pressed) {
                        keyPaint.color = Palette.alpha(p.accent, 0.35f)
                        c.drawRoundRect(RectF(r).apply { inset(-3 * dp, -3 * dp) }, radius + 3 * dp, radius + 3 * dp, keyPaint)
                    } else if (borders) {
                        keyPaint.color = p.keyShadow
                        if (p.shadowDp > 0f) c.drawRoundRect(r.left, r.top + p.shadowDp * dp, r.right, r.bottom + p.shadowDp * dp, radius, radius, keyPaint)
                    }
                    keyPaint.color = if (pressed) blend(fill, p.accent, 0.18f) else fill
                    c.drawRoundRect(r, radius, radius, keyPaint)
                    if (pressed) {
                        keyPaint.style = Paint.Style.STROKE; keyPaint.strokeWidth = 2 * dp; keyPaint.color = p.accent
                        c.drawRoundRect(r, radius, radius, keyPaint)
                        keyPaint.style = Paint.Style.FILL
                    }
                }
                else -> {
                    if (borders) {
                        keyPaint.color = p.keyShadow
                        if (p.shadowDp > 0f) c.drawRoundRect(r.left, r.top + p.shadowDp * dp, r.right, r.bottom + p.shadowDp * dp, radius, radius, keyPaint)
                    }
                    // the ripple shows the press itself, so the key keeps its colour
                    keyPaint.color = if (pressed && effect != "ripple") blend(fill, p.text, 0.16f) else fill
                    c.drawRoundRect(r, radius, radius, keyPaint)
                }
            }
        }
        if (effect == "ripple") drawRipples(c, slot, radius)

        val fg = when { enter -> p.onEnter ?: p.onAccent; shift == Shift.LOCKED && k.type == KeyType.SHIFT -> p.onAccent; else -> p.text }
        when (k.type) {
            KeyType.SPACE -> {
                textPaint.typeface = bnFaceRegular
                textPaint.textSize = 14 * dp
                textPaint.color = p.subtext
                // The language, plus the keyboard's name when the space bar is wide enough for both
                val label = when {
                    mode == Mode.NUMBERS || mode == Mode.PHONE -> "␣"
                    else -> {
                        val full = if (bangla) "বাং · একুশে নেক্সট" else "EN · Ekushe Next"
                        if (textPaint.measureText(full) <= r.width() - 20 * dp) full else if (bangla) "বাং" else "EN"
                    }
                }
                c.drawText(label, r.centerX(), r.centerY() + 5 * dp, textPaint)
            }
            KeyType.LANG -> {
                textPaint.typeface = bnFace
                textPaint.textSize = 13.5f * dp
                textPaint.color = if (bangla) p.hint else p.subtext
                c.drawText(if (bangla) "বাং" else "EN", r.centerX(), r.centerY() + 2 * dp, textPaint)
                textPaint.textSize = 9.5f * dp
                textPaint.color = p.subtext
                c.drawText(if (bangla) "EN" else "বাং", r.centerX(), r.centerY() + 15 * dp, textPaint)
            }
            else -> {
                val t = keyText(k)
                val bnLabel = t.any { it.code in 0x0980..0x09FF }
                textPaint.typeface = if (bnLabel) bnFace else Typeface.DEFAULT
                textPaint.color = fg
                textPaint.textSize = when {
                    k.type == KeyType.DELETE || k.type == KeyType.SHIFT || k.type == KeyType.ENTER || k.type == KeyType.EMOJI -> 21 * dp
                    isFunction(k) -> 14.5f * dp
                    (mode == Mode.NUMBERS || mode == Mode.PHONE) && k.type == KeyType.CHAR -> 27 * dp
                    t.length > 2 -> 14 * dp
                    bnLabel -> 21 * dp
                    else -> 22 * dp
                }
                if (bnLabel) {
                    // Bangla letters sit by their own shape, not the font's line height
                    textPaint.getTextBounds(t, 0, t.length, glyph)
                    c.drawText(t, r.centerX(), r.centerY() - glyph.exactCenterY(), textPaint)
                } else {
                    val dy = (textPaint.descent() + textPaint.ascent()) / 2
                    c.drawText(t, r.centerX(), r.centerY() - dy, textPaint)
                }
                // The Bangla letter this key types, small in the corner
                if (k.letter && bangla && mode == Mode.LETTERS && prefs.keyHints) {
                    val h = if (shift != Shift.OFF) k.hintShift else k.hint
                    if (h != null) {
                        hintPaint.typeface = bnFaceRegular
                        hintPaint.textSize = 10.5f * dp
                        hintPaint.color = p.hint
                        c.drawText(h, r.right - 4.5f * dp, r.top + 13 * dp, hintPaint)
                    }
                }
                if (k.alts.isNotEmpty() && !k.letter) {
                    keyPaint.color = p.subtext
                    c.drawCircle(r.right - 6 * dp, r.top + 6 * dp, 1.6f * dp, keyPaint)
                }
            }
        }
    }

    private fun drawPreview(c: Canvas, k: Key, r: RectF) {
        val p = palette
        val w = max(r.width() * 1.25f, 44 * dp)
        val h = r.height() * 1.15f + 8 * dp
        val left = (r.centerX() - w / 2).coerceIn(2 * dp, width - w - 2 * dp)
        val top = max(0f, r.top - h - 2 * dp)
        val rect = RectF(left, top, left + w, top + h)
        keyPaint.color = p.keyShadow
        c.drawRoundRect(rect.left, rect.top + 2 * dp, rect.right, rect.bottom + 2 * dp, 10 * dp, 10 * dp, keyPaint)
        keyPaint.color = p.popup
        c.drawRoundRect(rect, 10 * dp, 10 * dp, keyPaint)
        val t = keyText(k)
        textPaint.typeface = if (t.any { it.code in 0x0980..0x09FF }) bnFace else Typeface.DEFAULT
        textPaint.textSize = 28 * dp
        textPaint.color = p.text
        c.drawText(t, rect.centerX(), rect.top + h * 0.52f, textPaint)
        if (k.letter && bangla && mode == Mode.LETTERS) {
            val hint = if (shift != Shift.OFF) k.hintShift else k.hint
            if (hint != null) {
                textPaint.typeface = bnFace
                textPaint.textSize = 15 * dp
                textPaint.color = p.hint
                c.drawText(hint, rect.centerX(), rect.top + h * 0.86f, textPaint)
            }
        }
    }

    // ── long-press choices
    private class AltsPopup(val key: Key, val items: List<Alt>, val rect: RectF, val cell: Float, var selected: Int)
    private var altsOpen: AltsPopup? = null

    private fun openAlts(k: Key, r: RectF) {
        val items = (if (shift != Shift.OFF) k.shiftAlts ?: k.alts else k.alts)
        if (items.isEmpty()) return
        val cell = max(r.width(), 46 * dp)
        val w = cell * items.size
        val h = 58 * dp
        val left = (r.centerX() - cell / 2).coerceIn(2 * dp, max(2 * dp, width - w - 2 * dp))
        val top = max(0f, r.top - h - 4 * dp)
        altsOpen = AltsPopup(k, items, RectF(left, top, left + w, top + h), cell, 0)
        listener.onKeyFeedback(this, HapticFeedbackConstants.LONG_PRESS)
        invalidate()
    }

    private fun drawAlts(c: Canvas, a: AltsPopup) {
        val p = palette
        keyPaint.color = p.keyShadow
        c.drawRoundRect(a.rect.left, a.rect.top + 3 * dp, a.rect.right, a.rect.bottom + 3 * dp, 12 * dp, 12 * dp, keyPaint)
        keyPaint.color = p.popup
        c.drawRoundRect(a.rect, 12 * dp, 12 * dp, keyPaint)
        a.items.forEachIndexed { i, alt ->
            val cx = a.rect.left + a.cell * (i + 0.5f)
            if (i == a.selected) {
                keyPaint.color = p.accent
                c.drawRoundRect(a.rect.left + a.cell * i + 3 * dp, a.rect.top + 3 * dp, a.rect.left + a.cell * (i + 1) - 3 * dp, a.rect.bottom - 3 * dp, 9 * dp, 9 * dp, keyPaint)
            }
            val on = i == a.selected
            textPaint.typeface = if (alt.label.any { it.code in 0x0980..0x09FF }) bnFace else Typeface.DEFAULT_BOLD
            textPaint.textSize = (if (alt.label.length > 2) 15 else 20) * dp
            textPaint.color = if (on) p.onAccent else p.text
            c.drawText(alt.label, cx, a.rect.top + (if (alt.note != null) 27 else 36) * dp, textPaint)
            alt.note?.let {
                textPaint.typeface = bnFaceRegular
                textPaint.textSize = 12.5f * dp
                textPaint.color = if (on) p.onAccent else p.hint
                c.drawText(it, cx, a.rect.top + 47 * dp, textPaint)
            }
        }
    }

    // ── suggestion strip
    private data class Chip(val rect: RectF, val action: () -> Unit, val label: String, val note: String?, val strong: Boolean, val tip: Boolean)
    private var chips: List<Chip> = emptyList()
    private var stripScroll = 0f
    private var stripContentW = 0f

    private fun drawStrip(c: Canvas) {
        val p = palette
        keyPaint.color = p.strip
        c.drawRect(0f, 0f, width.toFloat(), stripH, keyPaint)
        val list = ArrayList<Chip>()
        val pad = 8 * dp
        var x = pad - stripScroll
        val top = 6 * dp
        val bottom = stripH - 6 * dp
        textPaint.textSize = 17 * dp

        fun chip(label: String, note: String?, strong: Boolean, tip: Boolean, action: () -> Unit) {
            textPaint.typeface = if (strong) bnFace else bnFaceRegular
            val tw = textPaint.measureText(label)
            var w = tw + 24 * dp
            if (note != null) { hintPaint.textSize = 12 * dp; w += hintPaint.measureText(note) + 8 * dp }
            list += Chip(RectF(x, top, x + max(w, 52 * dp), bottom), action, label, note, strong, tip)
            x += max(w, 52 * dp) + 6 * dp
        }
        when {
            candidates.isNotEmpty() -> {
                candidates.forEachIndexed { i, cand ->
                    chip(cand.text, if (cand.kind == "English") "EN" else null, i == selected, false) { listener.onCandidate(i) }
                }
                tips.forEach { t ->
                    val i = candidates.indexOfFirst { it.text == t.bangla }
                    if (i >= 0) chip(t.bangla, t.keys, false, true) { listener.onCandidate(i) }
                }
            }
            predictions.isNotEmpty() -> predictions.forEach { w -> chip(w, null, false, false) { listener.onPrediction(w) } }
        }
        stripContentW = x + stripScroll
        chips = list

        if (list.isEmpty()) { drawToolbar(c); return }
        c.save()
        c.clipRect(0f, 0f, width.toFloat(), stripH)
        for (ch in list) {
            keyPaint.color = when { ch.strong -> p.accent; else -> p.chip }
            c.drawRoundRect(ch.rect, 10 * dp, 10 * dp, keyPaint)
            if (ch.tip) {
                keyPaint.style = Paint.Style.STROKE; keyPaint.strokeWidth = 1.5f * dp; keyPaint.color = p.accent
                c.drawRoundRect(ch.rect, 10 * dp, 10 * dp, keyPaint)
                keyPaint.style = Paint.Style.FILL
            }
            textPaint.typeface = if (ch.strong) bnFace else bnFaceRegular
            textPaint.textSize = 17 * dp
            textPaint.color = if (ch.strong) p.onAccent else p.text
            val noteW = ch.note?.let { hintPaint.textSize = 12 * dp; hintPaint.measureText(it) + 8 * dp } ?: 0f
            c.drawText(ch.label, ch.rect.centerX() - noteW / 2, ch.rect.centerY() + 6 * dp, textPaint)
            ch.note?.let {
                hintPaint.typeface = Typeface.DEFAULT_BOLD
                hintPaint.color = if (ch.strong) p.onAccent else p.hint
                c.drawText(it, ch.rect.right - 10 * dp, ch.rect.centerY() + 4.5f * dp, hintPaint)
            }
        }
        c.restore()
    }

    // Toolbar when nothing is being typed: language, emoji, settings, hide
    private data class Tool(val id: String, val rect: RectF)
    private var tools: List<Tool> = emptyList()

    private fun drawToolbar(c: Canvas) {
        val p = palette
        val s = stripH
        val ids = listOf("lang", "emoji", "settings", "hide")
        val w = 52 * dp
        tools = listOfNotNull(
            Tool("lang", RectF(8 * dp, 6 * dp, 8 * dp + 96 * dp, s - 6 * dp)),
            // a note beside the language switch (password box, English only) takes that space
            if (stripMessage == null) Tool("help", RectF(width - 4 * w - 8 * dp, 0f, width - 3 * w - 8 * dp, s)) else null,
            Tool("emoji", RectF(width - 3 * w - 8 * dp, 0f, width - 2 * w - 8 * dp, s)),
            Tool("settings", RectF(width - 2 * w - 8 * dp, 0f, width - w - 8 * dp, s)),
            Tool("hide", RectF(width - w - 8 * dp, 0f, width - 8 * dp, s)),
        )
        // the language pill
        val pill = tools[0].rect
        keyPaint.color = p.chip
        c.drawRoundRect(pill, pill.height() / 2, pill.height() / 2, keyPaint)
        val half = RectF(if (bangla) pill.left else pill.centerX(), pill.top, if (bangla) pill.centerX() else pill.right, pill.bottom)
        keyPaint.color = p.accent
        c.drawRoundRect(half, pill.height() / 2, pill.height() / 2, keyPaint)
        textPaint.typeface = bnFace
        textPaint.textSize = 14 * dp
        textPaint.color = if (bangla) p.onAccent else p.subtext
        c.drawText("বাংলা", pill.left + pill.width() / 4, pill.centerY() + 5 * dp, textPaint)
        textPaint.typeface = Typeface.DEFAULT_BOLD
        textPaint.color = if (!bangla) p.onAccent else p.subtext
        c.drawText("EN", pill.left + pill.width() * 3 / 4, pill.centerY() + 5 * dp, textPaint)
        stripMessage?.let {
            textPaint.typeface = bnFaceRegular; textPaint.textSize = 13 * dp; textPaint.color = p.subtext
            textPaint.textAlign = Paint.Align.LEFT
            c.drawText(it, pill.right + 12 * dp, pill.centerY() + 5 * dp, textPaint)
            textPaint.textAlign = Paint.Align.CENTER
        }
        textPaint.typeface = Typeface.DEFAULT
        textPaint.textSize = 20 * dp
        textPaint.color = p.subtext
        val glyph = mapOf("help" to "?", "emoji" to if (stripOnly) "⌨" else "☺", "settings" to "⚙", "hide" to "⌄")
        for (t in tools.drop(1)) c.drawText(glyph[t.id]!!, t.rect.centerX(), t.rect.centerY() + 7 * dp, textPaint)
        if (ids.isEmpty()) return
    }

    // ── emoji panel
    private var emojiGroup = 0
    private var emojiScroll = 0f
    private var emojiCells: List<Pair<RectF, String>> = emptyList()
    private var emojiTabs: List<Pair<RectF, Int>> = emptyList()
    private var emojiAbc = RectF()
    private var emojiDel = RectF()

    private fun emojiItems(): List<String> =
        if (emojiGroup == -1) prefs.recentEmoji.split(' ').filter { it.isNotEmpty() } else Emoji.groups[emojiGroup].items

    private fun drawEmoji(c: Canvas) {
        val p = palette
        val tabH = 44 * dp
        val areaTop = stripH
        val areaBottom = height - bottomPad - tabH
        val cols = 8
        val cw = width / cols.toFloat()
        val ch = 46 * dp
        val items = emojiItems()
        val cells = ArrayList<Pair<RectF, String>>()
        c.save()
        c.clipRect(0f, areaTop, width.toFloat(), areaBottom)
        textPaint.typeface = Typeface.DEFAULT
        textPaint.textSize = 27 * dp
        items.forEachIndexed { i, e ->
            val r = RectF((i % cols) * cw, areaTop + (i / cols) * ch - emojiScroll, (i % cols + 1) * cw, areaTop + (i / cols + 1) * ch - emojiScroll)
            if (r.bottom > areaTop && r.top < areaBottom) {
                c.drawText(e, r.centerX(), r.centerY() + 10 * dp, textPaint)
                cells += r to e
            }
        }
        if (items.isEmpty()) {
            textPaint.typeface = bnFaceRegular; textPaint.textSize = 15 * dp; textPaint.color = p.subtext
            c.drawText("এখনো কোনো ইমোজি ব্যবহার করা হয়নি", width / 2f, (areaTop + areaBottom) / 2, textPaint)
        }
        c.restore()
        emojiCells = cells
        // tabs: ABC, recent, the groups, delete
        val tabs = ArrayList<Pair<RectF, Int>>()
        val n = Emoji.groups.size + 3
        val tw = width / n.toFloat()
        val ty = height - bottomPad - tabH
        keyPaint.color = p.functionKey
        c.drawRect(0f, ty, width.toFloat(), height.toFloat(), keyPaint)
        emojiAbc = RectF(0f, ty, tw, ty + tabH)
        emojiDel = RectF(width - tw, ty, width.toFloat(), ty + tabH)
        textPaint.textSize = 14 * dp; textPaint.typeface = Typeface.DEFAULT_BOLD; textPaint.color = p.text
        c.drawText("ABC", emojiAbc.centerX(), emojiAbc.centerY() + 5 * dp, textPaint)
        textPaint.textSize = 20 * dp
        c.drawText("⌫", emojiDel.centerX(), emojiDel.centerY() + 7 * dp, textPaint)
        val icons = listOf("🕘") + Emoji.groups.map { it.icon }
        icons.forEachIndexed { i, ic ->
            val gid = i - 1
            val r = RectF(tw * (i + 1), ty, tw * (i + 2), ty + tabH)
            if (gid == emojiGroup) { keyPaint.color = p.keyPressed; c.drawRoundRect(r.left + 4 * dp, r.top + 5 * dp, r.right - 4 * dp, r.bottom - 5 * dp, 9 * dp, 9 * dp, keyPaint) }
            textPaint.textSize = 20 * dp
            c.drawText(ic, r.centerX(), r.centerY() + 7 * dp, textPaint)
            tabs += r to gid
        }
        emojiTabs = tabs
    }

    private fun emojiMaxScroll(): Float {
        val rowsN = (emojiItems().size + 7) / 8
        val area = height - bottomPad - 44 * dp - stripH
        return max(0f, rowsN * 46 * dp - area)
    }

    fun rememberEmoji(e: String) {
        val list = (listOf(e) + prefs.recentEmoji.split(' ').filter { it.isNotEmpty() && it != e }).take(32)
        prefs.recentEmoji = list.joinToString(" ")
    }

    // ───────────────────────── touch ─────────────────────────

    private val handler = Handler(Looper.getMainLooper())
    private var pressedKey: Key? = null
    private var pressedRect: RectF? = null
    private var downX = 0f
    private var downY = 0f
    private var activePointer = -1
    private var spaceDragging = false
    private var spaceAnchorX = 0f
    private var stripDragging = false
    private var stripDownX = 0f
    private var stripStartScroll = 0f
    private var emojiDragging = false
    private var emojiDownY = 0f
    private var emojiStartScroll = 0f
    private var longFired = false
    private var repeatCount = 0

    private val longPress = Runnable {
        val k = pressedKey ?: return@Runnable
        val r = pressedRect ?: return@Runnable
        when (k.type) {
            KeyType.SPACE -> if (!spaceDragging) { longFired = true; listener.onSwitchInputMethod() }
            KeyType.LANG -> { longFired = true; listener.onSwitchInputMethod() }
            KeyType.CHAR -> openAlts(k, r).also { if (altsOpen != null) longFired = true }
            else -> {}
        }
    }

    private val repeatDelete = object : Runnable {
        override fun run() {
            if (pressedKey?.type != KeyType.DELETE) return
            repeatCount++
            listener.onDelete()
            listener.onKeyFeedback(this@KeyboardView, HapticFeedbackConstants.KEYBOARD_TAP)
            handler.postDelayed(this, if (repeatCount > 12) 35L else 70L)
        }
    }

    private fun keyAt(x: Float, y: Float): Placed? {
        // nearest key in the row, so the gaps never swallow a touch
        val hit = placed.firstOrNull { it.rect.contains(x, y) }
        if (hit != null) return hit
        return placed.filter { y >= it.rect.top - gap && y <= it.rect.bottom + gap }.minByOrNull { abs(it.rect.centerX() - x) }
    }

    @SuppressLint("ClickableViewAccessibility")
    override fun onTouchEvent(e: MotionEvent): Boolean {
        when (e.actionMasked) {
            MotionEvent.ACTION_DOWN, MotionEvent.ACTION_POINTER_DOWN -> {
                val i = e.actionIndex
                val x = e.getX(i)
                val y = e.getY(i)
                // a second finger: type the first finger's key now
                if (e.actionMasked == MotionEvent.ACTION_POINTER_DOWN && pressedKey != null) release(fire = true)
                activePointer = e.getPointerId(i)
                if (y < stripH) { stripDown(x); return true }
                if (stripOnly) return true
                if (mode == Mode.EMOJI) { emojiDragging = false; emojiDownY = y; emojiStartScroll = emojiScroll; downX = x; downY = y; return true }
                val hit = keyAt(x, y) ?: return true
                press(hit, x, y)
            }
            MotionEvent.ACTION_MOVE -> {
                val i = e.findPointerIndex(activePointer)
                if (i < 0) return true
                val x = e.getX(i)
                val y = e.getY(i)
                if (stripPressed) { stripMove(x); return true }
                if (mode == Mode.EMOJI) {
                    if (abs(y - emojiDownY) > 8 * dp) emojiDragging = true
                    if (emojiDragging) { emojiScroll = (emojiStartScroll - (y - emojiDownY)).coerceIn(0f, emojiMaxScroll()); invalidate() }
                    return true
                }
                move(x, y)
            }
            MotionEvent.ACTION_UP, MotionEvent.ACTION_POINTER_UP -> {
                val i = e.actionIndex
                if (e.getPointerId(i) != activePointer) return true
                val x = e.getX(i)
                val y = e.getY(i)
                if (stripPressed) { stripUp(x, y); return true }
                if (mode == Mode.EMOJI) { if (!emojiDragging) emojiTap(x, y); return true }
                release(fire = true)
                activePointer = -1
            }
            MotionEvent.ACTION_CANCEL -> { release(fire = false); stripPressed = false }
        }
        return true
    }

    private fun press(hit: Placed, x: Float, y: Float) {
        val k = hit.key
        pressedKey = k; pressedRect = hit.rect; downX = x; downY = y
        if (effect == "ripple") {
            val now = android.os.SystemClock.uptimeMillis()
            ripples.removeAll { now - it.start > rippleMs }
            ripples += Ripple(hit.rect, x, y, now)
        }
        longFired = false; spaceDragging = false; spaceAnchorX = x; repeatCount = 0
        listener.onKeyFeedback(this, HapticFeedbackConstants.KEYBOARD_TAP)
        when (k.type) {
            KeyType.DELETE -> { listener.onDelete(); handler.postDelayed(repeatDelete, 400) }
            KeyType.SHIFT -> {}
            else -> handler.postDelayed(longPress, if (k.type == KeyType.CHAR) 330L else 500L)
        }
        invalidate()
    }

    private fun move(x: Float, y: Float) {
        val k = pressedKey ?: return
        altsOpen?.let { a ->
            val i = ((x - a.rect.left) / a.cell).toInt().coerceIn(0, a.items.size - 1)
            if (i != a.selected) { a.selected = i; invalidate() }
            return
        }
        if (k.type == KeyType.SPACE) {
            val step = 14 * dp
            if (!spaceDragging && abs(x - downX) > 16 * dp) { spaceDragging = true; handler.removeCallbacks(longPress); spaceAnchorX = x }
            if (spaceDragging) {
                val n = ((x - spaceAnchorX) / step).toInt()
                if (n != 0) { listener.onCursorMove(n); spaceAnchorX += n * step }
            }
            return
        }
        if (k.type == KeyType.DELETE) return
        // sliding to another key changes the choice
        val hit = keyAt(x, y)
        if (hit != null && hit.key != k && hit.key.type == KeyType.CHAR && k.type == KeyType.CHAR) {
            handler.removeCallbacks(longPress)
            pressedKey = hit.key; pressedRect = hit.rect
            handler.postDelayed(longPress, 330L)
            invalidate()
        }
    }

    private var lastShiftTap = 0L

    private fun release(fire: Boolean) {
        handler.removeCallbacks(longPress)
        handler.removeCallbacks(repeatDelete)
        val k = pressedKey
        val a = altsOpen
        pressedKey = null; pressedRect = null; altsOpen = null
        invalidate()
        if (!fire || k == null) return
        if (a != null) { listener.onText(a.items[a.selected].text); afterChar(); return }
        if (longFired || spaceDragging) return
        when (k.type) {
            KeyType.CHAR -> { listener.onText(if (k.letter && shift != Shift.OFF) k.text.uppercase() else k.text); afterChar() }
            KeyType.SPACE -> listener.onSpace()
            KeyType.ENTER -> listener.onEnter()
            KeyType.DELETE -> {}
            KeyType.SHIFT -> {
                val now = System.currentTimeMillis()
                shift = when {
                    shift == Shift.OFF -> if (now - lastShiftTap < 300) Shift.LOCKED else Shift.ON
                    shift == Shift.ON && now - lastShiftTap < 300 -> Shift.LOCKED
                    else -> Shift.OFF
                }
                lastShiftTap = now
            }
            KeyType.SYMBOLS -> mode = Mode.SYMBOLS
            KeyType.MORE_SYMBOLS -> mode = Mode.MORE_SYMBOLS
            KeyType.LETTERS -> mode = Mode.LETTERS
            KeyType.LANG -> listener.onLanguageToggle()
            KeyType.EMOJI -> openEmoji()
        }
    }

    private fun openEmoji() {
        emojiScroll = 0f; emojiGroup = if (prefs.recentEmoji.isNotEmpty()) -1 else 0; mode = Mode.EMOJI
    }

    private fun afterChar() {
        if (shift == Shift.ON) shift = Shift.OFF
    }

    // strip gestures
    private var stripPressed = false
    private fun stripDown(x: Float) { stripPressed = true; stripDragging = false; stripDownX = x; stripStartScroll = stripScroll }
    private fun stripMove(x: Float) {
        if (abs(x - stripDownX) > 8 * dp) stripDragging = true
        if (stripDragging && chips.isNotEmpty()) {
            stripScroll = (stripStartScroll - (x - stripDownX)).coerceIn(0f, max(0f, stripContentW - width + 8 * dp))
            invalidate()
        }
    }
    private fun stripUp(x: Float, y: Float) {
        stripPressed = false
        if (stripDragging) return
        listener.onKeyFeedback(this, HapticFeedbackConstants.KEYBOARD_TAP)
        if (chips.isNotEmpty()) { chips.firstOrNull { it.rect.contains(x, y) }?.action?.invoke(); return }
        when (tools.firstOrNull { it.rect.contains(x, y) }?.id) {
            "lang" -> listener.onLanguageToggle()
            "help" -> listener.onHelp()
            "emoji" -> if (stripOnly) listener.onShowKeys() else openEmoji()
            "settings" -> listener.onSettings()
            "hide" -> listener.onHide()
        }
    }

    private fun emojiTap(x: Float, y: Float) {
        listener.onKeyFeedback(this, HapticFeedbackConstants.KEYBOARD_TAP)
        if (emojiAbc.contains(x, y)) { mode = Mode.LETTERS; return }
        if (emojiDel.contains(x, y)) { listener.onDelete(); return }
        emojiTabs.firstOrNull { it.first.contains(x, y) }?.let { emojiGroup = it.second; emojiScroll = 0f; invalidate(); return }
        emojiCells.firstOrNull { it.first.contains(x, y) }?.let { listener.onEmojiCommitted(it.second); rememberEmoji(it.second) }
    }

    // Settings changed in the app while the keyboard is open (the theme gallery): show them at once
    private val store = context.getSharedPreferences("keyboard", Context.MODE_PRIVATE)
    private val onPrefs = android.content.SharedPreferences.OnSharedPreferenceChangeListener { _, key ->
        if (key in setOf("theme", "key_borders", "photo", "photo_dim", "photo_stamp", "key_hints", "number_row", "height", "press_effect", "emoji_key")) post { refreshTheme() }
    }

    override fun onAttachedToWindow() {
        super.onAttachedToWindow()
        store.registerOnSharedPreferenceChangeListener(onPrefs)
    }

    override fun onDetachedFromWindow() {
        super.onDetachedFromWindow()
        store.unregisterOnSharedPreferenceChangeListener(onPrefs)
        handler.removeCallbacksAndMessages(null)
    }

    private fun drawRipples(c: Canvas, slot: RectF, radius: Float) {
        val now = android.os.SystemClock.uptimeMillis()
        var alive = false
        for (rp in ripples) {
            if (rp.slot !== slot) continue
            val t = ((now - rp.start).toFloat() / rippleMs).coerceIn(0f, 1f)
            if (t >= 1f) continue
            alive = true
            val ease = 1f - (1f - t) * (1f - t) * (1f - t)
            val reach = kotlin.math.hypot(max(rp.x - slot.left, slot.right - rp.x), max(rp.y - slot.top, slot.bottom - rp.y))
            c.save()
            clipPath.reset(); clipPath.addRoundRect(slot, radius, radius, android.graphics.Path.Direction.CW)
            c.clipPath(clipPath)
            keyPaint.color = Palette.alpha(palette.accent, 0.42f * (1f - t * 0.85f))
            c.drawCircle(rp.x, rp.y, reach * (0.15f + 0.85f * ease), keyPaint)
            c.restore()
        }
        if (alive) postInvalidateOnAnimation()
    }
    private val clipPath = android.graphics.Path()

    /** The colour `a` with the transparency of `like` (keys over a photo stay see-through) */
    private fun withAlphaOf(a: Int, like: Int) = (a and 0x00ffffff) or (like and 0xff000000.toInt())

    private fun blend(a: Int, b: Int, t: Float): Int {
        fun ch(x: Int, s: Int) = (x shr s) and 0xff
        val r = (ch(a, 16) + (ch(b, 16) - ch(a, 16)) * t).toInt()
        val g = (ch(a, 8) + (ch(b, 8) - ch(a, 8)) * t).toInt()
        val bl = (ch(a, 0) + (ch(b, 0) - ch(a, 0)) * t).toInt()
        return (0xff shl 24) or (r shl 16) or (g shl 8) or bl
    }

    @Suppress("unused")
    private fun clampF(v: Float, lo: Float, hi: Float) = min(max(v, lo), hi)
}
