import Foundation

/// One suggestion: the word, and what kind it is (Dictionary, Phonetic, English, Emoji, Snippet …).
struct Candidate: Equatable {
    let text: String
    let kind: String
}

/// A tip on how to type a word on offer directly (হেঁসে ← he^se).
struct Hint: Equatable {
    let bangla: String
    let keys: String
}

/// What a key did: the word being typed, the suggestions, and any text to place in the app.
struct Output {
    var preedit = ""
    var roman = ""
    var candidates: [Candidate] = []
    var selected = 0
    var hints: [Hint] = []
    var next: [String] = []
    var committed: String? = nil
    var deleteBefore = 0

    var composing: Bool { !roman.isEmpty }
    /// Hints that are tips for one of the suggestions
    var tips: [Hint] {
        var out: [Hint] = []
        for h in hints { if candidates.contains(where: { $0.text == h.bangla }) { out.append(h) } else { break } }
        return out
    }

    static func parse(_ json: String?) -> Output {
        guard let json, let data = json.data(using: .utf8),
              let o = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any] else { return Output() }
        var out = Output()
        out.preedit = o["preedit"] as? String ?? ""
        out.roman = o["roman_buffer"] as? String ?? ""
        out.candidates = (o["candidates"] as? [[String: Any]] ?? []).map { Candidate(text: $0["text"] as? String ?? "", kind: $0["kind"] as? String ?? "") }
        out.selected = o["selected_index"] as? Int ?? 0
        out.hints = (o["sign_hints"] as? [[String: Any]] ?? []).map { Hint(bangla: $0["result_bangla"] as? String ?? "", keys: $0["key_hint"] as? String ?? "") }
        out.next = o["next_word_predictions"] as? [String] ?? []
        out.committed = o["committed"] as? String
        out.deleteBefore = o["delete_before"] as? Int ?? 0
        return out
    }
}

/// The Rust engine (crates/ekushe-mobile). The dictionary is mapped from the bundle on a
/// background queue once per process; until then typing works with the small built-in list.
enum Engine {
    private(set) static var words: Int32 = 0
    private static var started = false
    private static let queue = DispatchQueue(label: "ekushe.engine")

    /// The keyboard's own folder for settings and learned words.
    static var folder: URL {
        let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        return base.appendingPathComponent("Ekushe", isDirectory: true)
    }

    static func start(bundle: Bundle = .main, done: (() -> Void)? = nil) {
        if started { done?(); return }
        started = true
        let dict = bundle.path(forResource: "ekushe", ofType: "dict") ?? ""
        let dir = folder.path
        queue.async {
            words = ekm_init(dict, dir)
            if let done { DispatchQueue.main.async(execute: done) }
        }
    }

    /// Wait for the dictionary (tests)
    static func startAndWait(bundle: Bundle) {
        if started { queue.sync {}; return }
        started = true
        let dict = bundle.path(forResource: "ekushe", ofType: "dict") ?? ""
        words = ekm_init(dict, folder.path)
    }

    private static func take(_ p: UnsafeMutablePointer<CChar>?) -> String? {
        guard let p else { return nil }
        defer { ekm_free(p) }
        return String(cString: p)
    }

    static func key(_ c: Character) -> Output {
        guard let u = c.unicodeScalars.first?.value else { return Output() }
        return Output.parse(take(ekm_key(u)))
    }
    static func backspace() -> Output { Output.parse(take(ekm_backspace())) }
    static func select(_ i: Int, trailing: Character? = nil) -> Output {
        Output.parse(take(ekm_select(UInt32(max(0, i)), trailing?.unicodeScalars.first?.value ?? 0)))
    }
    static func commit(trailing: Character? = nil) -> Output { Output.parse(take(ekm_commit(trailing?.unicodeScalars.first?.value ?? 0))) }
    static func reset() { ekm_reset() }
    static func setPrivate(_ on: Bool) { ekm_set_private(on) }
    static func hasUndo() -> Bool { ekm_has_undo() }
    /// A Bangla word in English letters ("ami", "tomake")? English typing leaves those alone.
    static func isBanglish(_ w: String) -> Bool { words > 0 && ekm_is_banglish(w) }
    static func config() -> [String: Any] {
        guard let s = take(ekm_get_config()), let d = s.data(using: .utf8) else { return [:] }
        return (try? JSONSerialization.jsonObject(with: d)) as? [String: Any] ?? [:]
    }
    @discardableResult
    static func setConfig(_ change: [String: Any]) -> Bool {
        guard let d = try? JSONSerialization.data(withJSONObject: change), let s = String(data: d, encoding: .utf8) else { return false }
        return ekm_set_config(s)
    }
    static func save() { queue.async { ekm_save() } }
    static func forgetLearned() { ekm_forget_learned() }
    static var release: String { take(ekm_release()) ?? "" }
    static func reportUrl(os: String) -> String { take(ekm_report_url(os)) ?? "https://ekushenext.com/report/" }
}
