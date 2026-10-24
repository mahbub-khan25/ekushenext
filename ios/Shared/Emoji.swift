import Foundation

/// The emoji panel: a short, well-chosen set in a few groups (the system font draws them).
enum Emoji {
    struct Group { let icon: String; let name: String; let items: [String] }
    static let groups: [Group] = [
        Group(icon: "😀", name: "মুখ", items: "😀 😃 😄 😁 😆 😅 😂 🤣 😊 😇 🙂 🙃 😉 😌 😍 🥰 😘 😗 😙 😚 😋 😛 😝 😜 🤪 🤨 🧐 🤓 😎 🥳 😏 😒 😞 😔 😟 😕 🙁 😣 😖 😫 😩 🥺 😢 😭 😤 😠 😡 🤬 🤯 😳 🥵 🥶 😱 😨 😰 😥 😓 🤗 🤔 🤭 🤫 🤥 😶 😐 😑 😬 🙄 😯 😦 😧 😮 😲 🥱 😴 🤤 😪 😵 🤐 🥴 🤢 🤮 🤧 😷 🤒 🤕".split(separator: " ").map(String.init)),
        Group(icon: "👍", name: "হাত", items: "👍 👎 👌 🤌 ✌️ 🤞 🤟 🤘 🤙 👈 👉 👆 👇 ☝️ ✋ 🤚 🖐️ 🖖 👋 👏 🙌 👐 🤲 🤝 🙏 ✍️ 💪 🫶 🫡 🙋 🙇 🤦 🤷 💁 🙆 🙅 👪 🧑‍🎓 🧑‍🏫 🧑‍💻 👩‍🍳 🧑‍🌾".split(separator: " ").map(String.init)),
        Group(icon: "❤️", name: "ভালোবাসা", items: "❤️ 🧡 💛 💚 💙 💜 🖤 🤍 🤎 💔 ❣️ 💕 💞 💓 💗 💖 💘 💝 💟 ♥️ 🌹 🌺 🌸 🌼 🌻 🌷 💐 🍀 ✨ 🎉 🎊 🎁 🎂 🕯️ 🔥 ⭐ 🌟 💫 🌙 ☀️ 🌈 ☁️ 🌧️ ⛈️ ❄️ 🌊".split(separator: " ").map(String.init)),
        Group(icon: "🍛", name: "খাবার", items: "🍛 🍚 🍲 🍜 🍝 🍞 🥖 🧁 🍰 🎂 🍪 🍫 🍬 🍭 🍮 🍯 🥛 ☕ 🍵 🧃 🥤 🍉 🍌 🍍 🥭 🍎 🍊 🍋 🍇 🍓 🥥 🥚 🍗 🍖 🐟 🦐 🥗 🌶️ 🧅 🧄 🥔 🍆 🥕 🌽".split(separator: " ").map(String.init)),
        Group(icon: "⚽", name: "কাজ ও খেলা", items: "⚽ 🏏 🏸 🏓 🏀 🏐 🎾 🏆 🥇 🎯 🎮 🎲 🎵 🎶 🎤 🎧 📱 💻 ⌨️ 🖥️ 📷 📚 📖 ✏️ 📝 📌 📎 🔑 💡 ⏰ 📅 🕌 🛕 ⛪ 🏠 🏫 🏥 🚗 🚌 🚲 🛺 🚆 ✈️ 🚢 ⛵".split(separator: " ").map(String.init)),
        Group(icon: "🇧🇩", name: "পতাকা ও চিহ্ন", items: "🇧🇩 🇮🇳 🇸🇦 🇦🇪 🇲🇾 🇬🇧 🇺🇸 🇨🇦 🇦🇺 🇯🇵 ✅ ❌ ⭕ ❗ ❓ 💯 🔴 🟢 🟡 🔵 ⚪ ⚫ ➕ ➖ ✖️ ➗ ♻️ ⚠️ 🚫 🆗".split(separator: " ").map(String.init)),
    ]
}
