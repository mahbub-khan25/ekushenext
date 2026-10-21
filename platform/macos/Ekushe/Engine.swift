import Foundation

/// What the engine returns after each key (a subset of `SessionOutput`).
struct EngineOutput: Decodable {
    struct Candidate: Decodable { let text: String }
    var preedit: String = ""
    var candidates: [Candidate] = []
    var selected_index: Int = 0
    var committed: String? = nil
    var delete_before: Int? = nil

    static let empty = EngineOutput()
}

/// Thin Swift wrapper over the Rust engine's C functions (one shared engine per process).
enum Engine {
    private static func take(_ p: UnsafeMutablePointer<CChar>?) -> String? {
        guard let p = p else { return nil }
        defer { ekushe_string_free(p) }
        return String(cString: p)
    }

    private static func decode(_ p: UnsafeMutablePointer<CChar>?) -> EngineOutput {
        guard let s = take(p), let data = s.data(using: .utf8),
              let out = try? JSONDecoder().decode(EngineOutput.self, from: data) else { return .empty }
        return out
    }

    static func start() { ek_init() }
    static func key(_ scalar: UInt32) -> EngineOutput { decode(ek_key(scalar)) }
    static func backspace() -> EngineOutput { decode(ek_backspace()) }
    static func select(_ index: Int, trailing: Character?) -> EngineOutput {
        let t = trailing?.unicodeScalars.first?.value ?? 0
        return decode(ek_select(UInt32(max(0, index)), t))
    }
    static func reset() { ek_reset() }
    static var isComposing: Bool { ek_is_composing() }
    static var canUndo: Bool { ek_can_undo() }
    static var isBangla: Bool { ek_language() == 1 }
    @discardableResult static func toggleLanguage() -> Bool { ek_toggle_language() == 1 }
    /// Returns true/false (Bangla/English) when a double Shift tap switched language.
    static func shiftTap() -> Bool? {
        let ms = UInt64(ProcessInfo.processInfo.systemUptime * 1000)
        let r = ek_shift_tap(ms)
        return r == 255 ? nil : r == 1
    }
    static func flipLast() -> (delete: Int, text: String)? {
        guard let s = take(ek_flip_last()), let data = s.data(using: .utf8),
              let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let n = obj["delete"] as? Int, let text = obj["text"] as? String else { return nil }
        return (n, text)
    }
    static func appFocus(_ bundleID: String) { _ = bundleID.withCString { ek_app_focus($0) } }
    static func reloadConfig() { ek_reload_config() }
    static func saveLearned() { ek_save_learned() }
    static func studioURL(tab: String? = nil) -> URL? {
        let s: String?
        if let tab = tab { s = tab.withCString { take(ek_studio_url($0)) } } else { s = take(ek_studio_url(nil)) }
        return s.flatMap(URL.init(string:))
    }
}
