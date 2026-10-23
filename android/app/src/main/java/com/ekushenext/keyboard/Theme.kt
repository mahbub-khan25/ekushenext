package com.ekushenext.keyboard

import android.content.Context
import android.content.res.Configuration
import android.graphics.Color

/** Keyboard colours: the flag's green, on night or on mist. */
data class Palette(
    val background: Int,
    val key: Int,
    val keyPressed: Int,
    val keyShadow: Int,
    val functionKey: Int,
    val text: Int,
    val subtext: Int,
    val hint: Int,
    val accent: Int,
    val onAccent: Int,
    val strip: Int,
    val chip: Int,
    val popup: Int,
    val divider: Int,
    val dark: Boolean,
) {
    companion object {
        val DARK = Palette(
            background = Color.parseColor("#0C1512"),
            key = Color.parseColor("#22302A"),
            keyPressed = Color.parseColor("#33463D"),
            keyShadow = Color.parseColor("#80000000"),
            functionKey = Color.parseColor("#17221D"),
            text = Color.parseColor("#EEF4F1"),
            subtext = Color.parseColor("#A0B6AC"),
            hint = Color.parseColor("#5FD6A3"),
            accent = Color.parseColor("#2FD18E"),
            onAccent = Color.parseColor("#03170F"),
            strip = Color.parseColor("#0C1512"),
            chip = Color.parseColor("#1A2621"),
            popup = Color.parseColor("#2C3D35"),
            divider = Color.parseColor("#22FFFFFF"),
            dark = true,
        )
        val LIGHT = Palette(
            background = Color.parseColor("#DDE5E1"),
            key = Color.parseColor("#FFFFFF"),
            keyPressed = Color.parseColor("#C9D6D0"),
            keyShadow = Color.parseColor("#3304120C"),
            functionKey = Color.parseColor("#C3CFC9"),
            text = Color.parseColor("#0E241B"),
            subtext = Color.parseColor("#5A6F65"),
            hint = Color.parseColor("#0B8A5C"),
            accent = Color.parseColor("#0B8A5C"),
            onAccent = Color.parseColor("#FFFFFF"),
            strip = Color.parseColor("#DDE5E1"),
            chip = Color.parseColor("#EEF3F1"),
            popup = Color.parseColor("#FFFFFF"),
            divider = Color.parseColor("#22000000"),
            dark = false,
        )

        fun of(context: Context): Palette = when (Prefs(context).theme) {
            "light" -> LIGHT
            "dark" -> DARK
            else -> if ((context.resources.configuration.uiMode and Configuration.UI_MODE_NIGHT_MASK) == Configuration.UI_MODE_NIGHT_YES) DARK else LIGHT
        }
    }
}
