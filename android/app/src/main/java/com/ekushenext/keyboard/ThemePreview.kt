package com.ekushenext.keyboard

import android.annotation.SuppressLint
import android.content.Context
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.Path
import android.graphics.RectF
import android.view.View

/** A tiny keyboard drawn in a theme's colours, for the theme gallery. */
@SuppressLint("ViewConstructor")
class ThemePreview(context: Context, private val palette: Palette, private val night: Palette? = null, private val borders: Boolean = true) : View(context) {
    private val dp = resources.displayMetrics.density
    private val paint = Paint(Paint.ANTI_ALIAS_FLAG)
    private val clip = Path()

    override fun onMeasure(widthMeasureSpec: Int, heightMeasureSpec: Int) {
        val w = MeasureSpec.getSize(widthMeasureSpec)
        setMeasuredDimension(w, (w * 0.62f).toInt())
    }

    override fun onDraw(c: Canvas) {
        val w = width.toFloat(); val h = height.toFloat()
        clip.reset(); clip.addRoundRect(RectF(0f, 0f, w, h), 12 * dp, 12 * dp, Path.Direction.CW)
        c.save(); c.clipPath(clip)
        draw(c, palette, w, h)
        night?.let { n ->
            // "Follow system": day on the left, night on the right, split on a slant
            val slant = Path().apply { moveTo(w * 0.62f, 0f); lineTo(w, 0f); lineTo(w, h); lineTo(w * 0.38f, h); close() }
            c.save(); c.clipPath(slant); draw(c, n, w, h); c.restore()
        }
        c.restore()
    }

    private fun draw(c: Canvas, p: Palette, w: Float, h: Float) {
        c.drawColor(p.background)
        val pad = 5 * dp
        val stripH = h * 0.17f
        // language pill in the strip
        paint.color = p.chip
        val pill = RectF(pad, pad * 0.8f, pad + w * 0.28f, stripH)
        c.drawRoundRect(pill, pill.height() / 2, pill.height() / 2, paint)
        paint.color = p.accent
        c.drawRoundRect(RectF(pill.left, pill.top, pill.centerX(), pill.bottom), pill.height() / 2, pill.height() / 2, paint)
        val top = stripH + pad * 0.6f
        val rowH = (h - top - pad * 0.6f) / 4
        val gap = 2.2f * dp
        val unit = (w - 2 * pad) / 10
        val rows = listOf(10, 9, 7)
        rows.forEachIndexed { r, n ->
            val y = top + r * rowH
            var x = pad + (10 - n) * unit / 2
            if (r == 2) {
                key(c, p, RectF(pad + gap / 2, y + gap / 2, pad + 1.5f * unit - gap / 2, y + rowH - gap / 2), p.functionKey)
                x = pad + 1.5f * unit
            }
            repeat(n) {
                val rect = RectF(x + gap / 2, y + gap / 2, x + unit - gap / 2, y + rowH - gap / 2)
                key(c, p, rect, p.key)
                // the small Bangla letter in the corner
                paint.color = p.hint
                c.drawCircle(rect.right - 2.6f * dp, rect.top + 2.6f * dp, 1.1f * dp, paint)
                paint.color = p.text
                c.drawRoundRect(RectF(rect.centerX() - 1.6f * dp, rect.centerY() - 0.8f * dp, rect.centerX() + 1.6f * dp, rect.centerY() + 1.6f * dp), dp, dp, paint)
                x += unit
            }
            if (r == 2) key(c, p, RectF(x + gap / 2, y + gap / 2, w - pad - gap / 2, y + rowH - gap / 2), p.functionKey)
        }
        val y = top + 3 * rowH
        key(c, p, RectF(pad + gap / 2, y + gap / 2, pad + 1.5f * unit - gap / 2, y + rowH - gap / 2), p.functionKey)
        key(c, p, RectF(pad + 1.5f * unit + gap / 2, y + gap / 2, pad + 8f * unit - gap / 2, y + rowH - gap / 2), if (borders) p.key else p.functionKey, always = true)
        key(c, p, RectF(pad + 8f * unit + gap / 2, y + gap / 2, w - pad - gap / 2, y + rowH - gap / 2), p.enter ?: p.accent, always = true)
    }

    private fun key(c: Canvas, p: Palette, r: RectF, color: Int, always: Boolean = false) {
        if (!borders && !always) return
        val rad = p.keyRadius / 7f * 3 * dp
        if (borders && p.shadowDp > 0f) {
            paint.color = p.keyShadow
            c.drawRoundRect(r.left, r.top + 0.8f * dp, r.right, r.bottom + 0.8f * dp, rad, rad, paint)
        }
        paint.color = color
        c.drawRoundRect(r, rad, rad, paint)
    }
}
