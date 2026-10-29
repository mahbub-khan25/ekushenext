import Foundation

/// The keyboard's own settings (the typing engine's settings live in its config.toml).
final class Prefs {
    static let shared = Prefs()
    private let d = UserDefaults.standard

    private func bool(_ k: String, _ def: Bool) -> Bool { d.object(forKey: k) == nil ? def : d.bool(forKey: k) }

    var theme: String { get { d.string(forKey: "theme") ?? "default" } set { d.set(newValue, forKey: "theme") } }
    var keyHints: Bool { get { bool("key_hints", true) } set { d.set(newValue, forKey: "key_hints") } }
    var numberRow: Bool { get { bool("number_row", false) } set { d.set(newValue, forKey: "number_row") } }
    var keyBorders: Bool { get { bool("key_borders", true) } set { d.set(newValue, forKey: "key_borders") } }
    var emojiKey: Bool { get { bool("emoji_key", true) } set { d.set(newValue, forKey: "emoji_key") } }
    var sound: Bool { get { bool("sound", true) } set { d.set(newValue, forKey: "sound") } }
    var doubleSpaceDari: Bool { get { bool("double_space_dari", true) } set { d.set(newValue, forKey: "double_space_dari") } }
    /// Keyboard height: 0 small, 1 normal, 2 tall
    var height: Int { get { d.object(forKey: "height") == nil ? 1 : d.integer(forKey: "height") } set { d.set(newValue, forKey: "height") } }
    /// iPad in landscape: the keyboard in two halves, for typing with the thumbs
    var splitPad: Bool { get { bool("split_pad", false) } set { d.set(newValue, forKey: "split_pad") } }
    /// How a key shows it was pressed: classic, 3d, glow, bounce
    var pressEffect: String { get { d.string(forKey: "press_effect") ?? "classic" } set { d.set(newValue, forKey: "press_effect") } }
    var bangla: Bool { get { bool("bangla", true) } set { d.set(newValue, forKey: "bangla") } }
    var recentEmoji: [String] { get { d.stringArray(forKey: "recent_emoji") ?? [] } set { d.set(Array(newValue.prefix(32)), forKey: "recent_emoji") } }
    var enAutocorrect: Bool { get { bool("en_autocorrect", true) } set { d.set(newValue, forKey: "en_autocorrect") } }
    var nextWordEnglish: Bool { get { bool("en_next_word", true) } set { d.set(newValue, forKey: "en_next_word") } }
}
