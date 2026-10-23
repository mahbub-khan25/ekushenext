package com.ekushenext.keyboard

import android.content.Context

/** The keyboard's own settings (the typing engine's settings live in its config.toml). */
class Prefs(context: Context) {
    companion object {
        /** Where the background photo is kept (a private, resized copy) */
        fun photoFile(context: Context) = java.io.File(context.filesDir, "keyboard_photo.jpg")
    }

    private val p = context.applicationContext.getSharedPreferences("keyboard", Context.MODE_PRIVATE)

    var theme: String
        get() = p.getString("theme", "ekushe") ?: "ekushe"
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
    /** Separate key shapes (true) or a flat look (false) */
    var keyBorders: Boolean
        get() = p.getBoolean("key_borders", true)
        set(v) = p.edit().putBoolean("key_borders", v).apply()
    /** A photo of the user's own behind the keys */
    var photo: Boolean
        get() = p.getBoolean("photo", false)
        set(v) = p.edit().putBoolean("photo", v).apply()
    /** How much the photo is dimmed (0–80 %) so the letters stay readable */
    var photoDim: Int
        get() = p.getInt("photo_dim", 35)
        set(v) = p.edit().putInt("photo_dim", v).apply()
    /** Changes whenever the photo is replaced, so the keyboard knows to load it again */
    var photoStamp: Long
        get() = p.getLong("photo_stamp", 0)
        set(v) = p.edit().putLong("photo_stamp", v).apply()
    /** How a key shows it was pressed: classic, 3d, glow, ripple, bounce */
    var pressEffect: String
        get() = p.getString("press_effect", "classic") ?: "classic"
        set(v) = p.edit().putString("press_effect", v).apply()
    var emojiKey: Boolean
        get() = p.getBoolean("emoji_key", true)
        set(v) = p.edit().putBoolean("emoji_key", v).apply()
    /** English is typed in Gboard: the EN key hands over, Gboard's 🌐 key comes back */
    var gboardEnglish: Boolean
        get() = p.getBoolean("gboard_english", true)
        set(v) = p.edit().putBoolean("gboard_english", v).apply()
    var seenTour: Boolean
        get() = p.getBoolean("seen_tour", false)
        set(v) = p.edit().putBoolean("seen_tour", v).apply()
}
