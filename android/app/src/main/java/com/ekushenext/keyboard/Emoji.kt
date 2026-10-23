package com.ekushenext.keyboard

/** The emoji panel: a short, well-chosen set in a few groups (the phone's font draws them). */
object Emoji {
    data class Group(val icon: String, val name: String, val items: List<String>)

    private fun split(s: String) = s.trim().split(Regex("\\s+"))

    val groups = listOf(
        Group("😀", "মুখ", split("""
            😀 😃 😄 😁 😆 😅 😂 🤣 😊 😇 🙂 🙃 😉 😌 😍 🥰 😘 😗 😙 😚 😋 😛 😝 😜 🤪 🤨 🧐 🤓 😎 🥳
            😏 😒 😞 😔 😟 😕 🙁 😣 😖 😫 😩 🥺 😢 😭 😤 😠 😡 🤬 🤯 😳 🥵 🥶 😱 😨 😰 😥 😓 🤗 🤔 🤭
            🤫 🤥 😶 😐 😑 😬 🙄 😯 😦 😧 😮 😲 🥱 😴 🤤 😪 😵 🤐 🥴 🤢 🤮 🤧 😷 🤒 🤕
        """)),
        Group("👍", "হাত", split("""
            👍 👎 👌 🤌 ✌️ 🤞 🤟 🤘 🤙 👈 👉 👆 👇 ☝️ ✋ 🤚 🖐️ 🖖 👋 👏 🙌 👐 🤲 🤝 🙏 ✍️ 💪 🫶 🫡 🙋
            🙇 🤦 🤷 💁 🙆 🙅 👪 🧑‍🎓 🧑‍🏫 🧑‍💻 👩‍🍳 🧑‍🌾
        """)),
        Group("❤️", "ভালোবাসা", split("""
            ❤️ 🧡 💛 💚 💙 💜 🖤 🤍 🤎 💔 ❣️ 💕 💞 💓 💗 💖 💘 💝 💟 ♥️ 🌹 🌺 🌸 🌼 🌻 🌷 💐 🍀 ✨ 🎉
            🎊 🎁 🎂 🕯️ 🔥 ⭐ 🌟 💫 🌙 ☀️ 🌈 ☁️ 🌧️ ⛈️ ❄️ 🌊
        """)),
        Group("🍛", "খাবার", split("""
            🍛 🍚 🍲 🍜 🍝 🍞 🥖 🧁 🍰 🎂 🍪 🍫 🍬 🍭 🍮 🍯 🥛 ☕ 🍵 🧃 🥤 🍉 🍌 🍍 🥭 🍎 🍊 🍋 🍇 🍓
            🥥 🥚 🍗 🍖 🐟 🦐 🥗 🌶️ 🧅 🧄 🥔 🍆 🥕 🌽
        """)),
        Group("⚽", "কাজ ও খেলা", split("""
            ⚽ 🏏 🏸 🏓 🏀 🏐 🎾 🏆 🥇 🎯 🎮 🎲 🎵 🎶 🎤 🎧 📱 💻 ⌨️ 🖥️ 📷 📚 📖 ✏️ 📝 📌 📎 🔑 💡 ⏰
            📅 🕌 🛕 ⛪ 🏠 🏫 🏥 🚗 🚌 🚲 🛺 🚆 ✈️ 🚢 ⛵
        """)),
        Group("🇧🇩", "পতাকা ও চিহ্ন", split("""
            🇧🇩 🇮🇳 🇸🇦 🇦🇪 🇲🇾 🇬🇧 🇺🇸 🇨🇦 🇦🇺 🇯🇵 ✅ ❌ ⭕ ❗ ❓ 💯 🔴 🟢 🟡 🔵 ⚪ ⚫ ➕ ➖ ✖️ ➗ ♻️ ⚠️ 🚫 🆗
        """)),
    )
}
