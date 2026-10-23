package com.ekushenext.keyboard

import android.content.Context

/** The keyboard's own settings (the typing engine's settings live in its config.toml). */
class Prefs(context: Context) {
    private val p = context.applicationContext.getSharedPreferences("keyboard", Context.MODE_PRIVATE)

    var theme: String
        get() = p.getString("theme", "system") ?: "system"
        set(v) = p.edit().putString("theme", v).apply()
    var vibrate: Boolean
        get() = p.getBoolean("vibrate", true)
        set(v) = p.edit().putBoolean("vibrate", v).apply()
    var sound: Boolean
        get() = p.getBoolean("sound", false)
        set(v) = p.edit().putBoolean("sound", v).apply()
    var numberRow: Boolean
        get() = p.getBoolean("number_row", false)
        set(v) = p.edit().putBoolean("number_row", v).apply()
    var keyHints: Boolean
        get() = p.getBoolean("key_hints", true)
        set(v) = p.edit().putBoolean("key_hints", v).apply()
    var doubleSpaceDari: Boolean
        get() = p.getBoolean("double_space_dari", true)
        set(v) = p.edit().putBoolean("double_space_dari", v).apply()
    /** Keyboard height: 0 small, 1 normal, 2 tall */
    var height: Int
        get() = p.getInt("height", 1)
        set(v) = p.edit().putInt("height", v).apply()
    /** The language last used (true: Bangla) */
    var bangla: Boolean
        get() = p.getBoolean("bangla", true)
        set(v) = p.edit().putBoolean("bangla", v).apply()
    var recentEmoji: String
        get() = p.getString("recent_emoji", "") ?: ""
        set(v) = p.edit().putString("recent_emoji", v).apply()
    var seenTour: Boolean
        get() = p.getBoolean("seen_tour", false)
        set(v) = p.edit().putBoolean("seen_tour", v).apply()
}
