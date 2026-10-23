package com.ekushenext.keyboard

import android.content.Context
import android.content.res.Configuration
import android.graphics.Color
import android.os.Build

/** Keyboard colours. */
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
    /** The same colours made see-through, for keys over a photo */
    fun overPhoto(): Palette = copy(
        background = Color.TRANSPARENT,
        key = alpha(key, if (dark) 0.62f else 0.78f),
        keyPressed = alpha(keyPressed, 0.85f),
        functionKey = alpha(functionKey, if (dark) 0.55f else 0.7f),
        keyShadow = alpha(keyShadow, 0.5f),
        strip = Color.TRANSPARENT,
        chip = alpha(chip, 0.75f),
    )

    companion object {
        private fun c(s: String) = Color.parseColor(s)
        fun alpha(color: Int, a: Float) = Color.argb((Color.alpha(color) * a).toInt(), Color.red(color), Color.green(color), Color.blue(color))

        // The flag's green, on night or on mist: what "Follow system" switches between
        val DARK = Palette(
            background = c("#0C1512"), key = c("#22302A"), keyPressed = c("#33463D"), keyShadow = c("#80000000"),
            functionKey = c("#17221D"), text = c("#EEF4F1"), subtext = c("#A0B6AC"), hint = c("#5FD6A3"),
            accent = c("#2FD18E"), onAccent = c("#03170F"), strip = c("#0C1512"), chip = c("#1A2621"),
            popup = c("#2C3D35"), divider = c("#22FFFFFF"), dark = true,
        )
        val LIGHT = Palette(
            background = c("#DDE5E1"), key = c("#FFFFFF"), keyPressed = c("#C9D6D0"), keyShadow = c("#3304120C"),
            functionKey = c("#C3CFC9"), text = c("#0E241B"), subtext = c("#5A6F65"), hint = c("#0B8A5C"),
            accent = c("#0B8A5C"), onAccent = c("#FFFFFF"), strip = c("#DDE5E1"), chip = c("#EEF3F1"),
            popup = c("#FFFFFF"), divider = c("#22000000"), dark = false,
        )

        // একুশে: the Shaheed Minar at dawn, deep green with the red sun
        val EKUSHE = Palette(
            background = c("#06372A"), key = c("#0E4D3A"), keyPressed = c("#176650"), keyShadow = c("#66000000"),
            functionKey = c("#0A4232"), text = c("#F4FAF7"), subtext = c("#A9CFC0"), hint = c("#FF8A84"),
            accent = c("#E23B3B"), onAccent = c("#FFFFFF"), strip = c("#06372A"), chip = c("#0E4D3A"),
            popup = c("#155C46"), divider = c("#22FFFFFF"), dark = true,
        )
        // কৃষ্ণচূড়া: flame-tree blossoms, warm red and orange
        val KRISHNACHURA = Palette(
            background = c("#FCE3D3"), key = c("#FFF8F2"), keyPressed = c("#F8CDB4"), keyShadow = c("#40A1300A"),
            functionKey = c("#F6C3A6"), text = c("#3D1208"), subtext = c("#8A4A35"), hint = c("#D93A16"),
            accent = c("#E8501F"), onAccent = c("#FFFFFF"), strip = c("#FCE3D3"), chip = c("#FFEFE4"),
            popup = c("#FFF8F2"), divider = c("#22000000"), dark = false,
        )
        // পদ্মা: the river at dusk, calm blues
        val PADMA = Palette(
            background = c("#0A1C30"), key = c("#16304C"), keyPressed = c("#22436A"), keyShadow = c("#80000000"),
            functionKey = c("#10263E"), text = c("#EAF4FF"), subtext = c("#9DB7D3"), hint = c("#7CC6FF"),
            accent = c("#3AA0FF"), onAccent = c("#04192D"), strip = c("#0A1C30"), chip = c("#16304C"),
            popup = c("#20406A"), divider = c("#22FFFFFF"), dark = true,
        )
        // শাপলা: the water lily, soft pink and white
        val SHAPLA = Palette(
            background = c("#F9E4EC"), key = c("#FFFFFF"), keyPressed = c("#F2C9D8"), keyShadow = c("#33701838"),
            functionKey = c("#F1CADA"), text = c("#3A1426"), subtext = c("#8C5670"), hint = c("#D2457E"),
            accent = c("#DC4F86"), onAccent = c("#FFFFFF"), strip = c("#F9E4EC"), chip = c("#FDF1F5"),
            popup = c("#FFFFFF"), divider = c("#22000000"), dark = false,
        )
        // কাশফুল: kash flowers in autumn, airy and quiet
        val KASHFUL = Palette(
            background = c("#EFEEEA"), key = c("#FFFFFF"), keyPressed = c("#DDDCD6"), keyShadow = c("#1F000000"),
            functionKey = c("#E0DFD9"), text = c("#2A2A28"), subtext = c("#7C7B76"), hint = c("#9A988F"),
            accent = c("#4B4A46"), onAccent = c("#FFFFFF"), strip = c("#EFEEEA"), chip = c("#F8F7F4"),
            popup = c("#FFFFFF"), divider = c("#1A000000"), dark = false,
        )
        // রাত: pure black, kind to OLED screens and batteries
        val NIGHT = Palette(
            background = c("#000000"), key = c("#151515"), keyPressed = c("#2A2A2A"), keyShadow = Color.TRANSPARENT,
            functionKey = c("#0B0B0B"), text = c("#F2F2F2"), subtext = c("#9A9A9A"), hint = c("#2FD18E"),
            accent = c("#2FD18E"), onAccent = c("#000000"), strip = c("#000000"), chip = c("#151515"),
            popup = c("#222222"), divider = c("#22FFFFFF"), dark = true,
        )

        fun systemDark(context: Context) =
            (context.resources.configuration.uiMode and Configuration.UI_MODE_NIGHT_MASK) == Configuration.UI_MODE_NIGHT_YES

        /** "Match my phone": Android 12's wallpaper colours */
        fun dynamic(context: Context): Palette {
            if (Build.VERSION.SDK_INT < Build.VERSION_CODES.S) return if (systemDark(context)) DARK else LIGHT
            val r = context.resources
            @Suppress("NewApi") fun k(id: Int) = r.getColor(id, context.theme)
            return if (systemDark(context)) Palette(
                background = k(android.R.color.system_neutral1_900), key = k(android.R.color.system_neutral2_800),
                keyPressed = k(android.R.color.system_neutral2_700), keyShadow = c("#66000000"),
                functionKey = k(android.R.color.system_neutral1_800), text = k(android.R.color.system_neutral1_50),
                subtext = k(android.R.color.system_neutral2_200), hint = k(android.R.color.system_accent1_200),
                accent = k(android.R.color.system_accent1_300), onAccent = k(android.R.color.system_accent1_900),
                strip = k(android.R.color.system_neutral1_900), chip = k(android.R.color.system_neutral2_800),
                popup = k(android.R.color.system_neutral2_700), divider = c("#22FFFFFF"), dark = true,
            ) else Palette(
                background = k(android.R.color.system_neutral2_100), key = k(android.R.color.system_neutral1_10),
                keyPressed = k(android.R.color.system_neutral2_200), keyShadow = c("#26000000"),
                functionKey = k(android.R.color.system_accent2_100), text = k(android.R.color.system_neutral1_900),
                subtext = k(android.R.color.system_neutral2_600), hint = k(android.R.color.system_accent1_600),
                accent = k(android.R.color.system_accent1_600), onAccent = k(android.R.color.system_accent1_0),
                strip = k(android.R.color.system_neutral2_100), chip = k(android.R.color.system_neutral1_50),
                popup = k(android.R.color.system_neutral1_10), divider = c("#22000000"), dark = false,
            )
        }

        fun byId(context: Context, id: String): Palette = when (id) {
            "ekushe" -> EKUSHE
            "krishnachura" -> KRISHNACHURA
            "padma" -> PADMA
            "shapla" -> SHAPLA
            "kashful" -> KASHFUL
            "night" -> NIGHT
            "dynamic" -> dynamic(context)
            "light" -> LIGHT
            "dark" -> DARK
            else -> if (systemDark(context)) DARK else LIGHT
        }

        /** The colours to draw with right now (not yet made see-through for a photo) */
        fun of(context: Context): Palette = byId(context, Prefs(context).theme)
    }
}

/** One choice in the theme gallery */
data class ThemeChoice(val id: String, val bn: String, val en: String)

object Themes {
    fun choices(): List<ThemeChoice> = buildList {
        add(ThemeChoice("ekushe", "একুশে", "Ekushe"))
        add(ThemeChoice("krishnachura", "কৃষ্ণচূড়া", "Krishnachura"))
        add(ThemeChoice("padma", "পদ্মা", "Padma"))
        add(ThemeChoice("shapla", "শাপলা", "Shapla"))
        add(ThemeChoice("kashful", "কাশফুল", "Kashful"))
        add(ThemeChoice("night", "রাত", "Night"))
        add(ThemeChoice("system", "ফোনের মতো", "Follow system"))
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) add(ThemeChoice("dynamic", "ফোনের রঙে", "Match my phone"))
        add(ThemeChoice("light", "সবুজ দিন", "Classic light"))
        add(ThemeChoice("dark", "সবুজ রাত", "Classic dark"))
    }
}
