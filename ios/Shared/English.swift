import UIKit

/// English typing help, kept small, as on Android: complete a word from the 50,000 most common
/// English words, catch a typo one letter away (with iOS's own spell checker, UITextChecker), learn
/// the words you use, and offer a few likely next words.
final class English {
    static let shared = English()

    private var words: [String] = []
    /// A word without its apostrophe ("dont") → its rank; the word itself is words[rank]
    private var rankOf: [String: Int] = [:]
    private var sorted: [String] = []
    private var exact = Set<String>()
    private var next: [String: [String]] = [:]
    private var learned: [String: Int] = [:]
    private var pairs: [String: Int] = [:]
    private(set) var ready = false
    private var dirty = false
    private let checker = UITextChecker()
    private var file: URL { Engine.folder.appendingPathComponent("en_learned.tsv") }

    private let quiet: Set<String> = ["fuck", "fucking", "fucked", "fucker", "shit", "shitty", "bitch", "bitches", "cunt", "dick", "dicks",
        "pussy", "cock", "cocks", "asshole", "bastard", "slut", "whore", "fag", "faggot", "nigger", "nigga", "retard", "retarded",
        "porn", "sex", "sexy", "nude", "nudes", "boobs", "tits", "penis", "vagina", "rape", "motherfucker", "damn", "crap", "piss"]
    // Typed without the apostrophe, these are almost always meant with it (unlike its, well, were, ill)
    private let contraction: Set<String> = ["im", "ive", "dont", "cant", "wont", "didnt", "doesnt", "isnt", "wasnt", "arent", "werent",
        "couldnt", "wouldnt", "shouldnt", "havent", "hasnt", "hadnt", "thats", "whats", "theres", "youre", "theyre", "youve",
        "weve", "theyve", "youll", "theyll", "shes", "hes", "whos", "heres", "wheres", "hows", "aint"]

    private func plain(_ w: String) -> String { w.lowercased().replacingOccurrences(of: "'", with: "") }

    func start(bundle: Bundle = .main, sync: Bool = false) {
        if ready { return }
        let work = {
            var list: [String] = []
            if let p = bundle.path(forResource: "words", ofType: "txt"), let s = try? String(contentsOfFile: p, encoding: .utf8) {
                list = s.split(separator: "\n").map(String.init).filter { !$0.isEmpty }
            }
            var rank: [String: Int] = [:]
            var exact = Set<String>()
            for (i, w) in list.enumerated() {
                let k = self.plain(w)
                if rank[k] == nil { rank[k] = i }
                exact.insert(w)
            }
            var next: [String: [String]] = [:]
            if let p = bundle.path(forResource: "next", ofType: "tsv"), let s = try? String(contentsOfFile: p, encoding: .utf8) {
                for line in s.split(separator: "\n") where !line.hasPrefix("#") {
                    let parts = line.split(separator: "\t").map(String.init)
                    if parts.count > 1 { next[parts[0], default: []].append(contentsOf: parts.dropFirst()) }
                }
            }
            var learned: [String: Int] = [:], pairs: [String: Int] = [:]
            if let s = try? String(contentsOf: self.file, encoding: .utf8) {
                for line in s.split(separator: "\n") {
                    let p = line.split(separator: "\t").map(String.init)
                    guard p.count == 3, let n = Int(p[2]) else { continue }
                    if p[0] == "w" { learned[p[1]] = n } else if p[0] == "p" { pairs[p[1]] = n }
                }
            }
            DispatchQueue.main.async {
                self.words = list; self.rankOf = rank; self.exact = exact
                self.sorted = rank.keys.sorted(); self.next = next
                self.learned = learned; self.pairs = pairs
                self.ready = true
            }
        }
        if sync {
            work()
            // the main queue runs the hand-over at once in tests
            RunLoop.main.run(until: Date().addingTimeInterval(0.05))
        } else {
            DispatchQueue.global(qos: .userInitiated).async(execute: work)
        }
    }

    /// Words the person keeps on the phone (contacts' names, text replacements), from UILexicon
    func addSupplementary(_ ws: [String]) {
        for w in ws where w.count > 1 && w.allSatisfy({ $0.isLetter || $0 == "'" }) {
            if learned[w] == nil { learned[w] = 3 }
        }
    }

    /// A known word (from the list or what you have typed before)
    func known(_ w: String) -> Bool {
        if !ready { return true }
        return rankOf[plain(w)] != nil || learned[w] != nil || learned[w.lowercased()] != nil
    }

    private func score(_ w: String) -> Double {
        let base = rankOf[plain(w)].map { 1.0 / Double($0 + 50) } ?? 0
        let l = learned[w] ?? learned[w.lowercased()] ?? 0
        return base + Double(l) * 0.004
    }

    /// The spelling to show: the list's (don't, i'm → I'm), in the case the person typed
    func spell(_ typed: String, _ w: String) -> String {
        var out = w
        if out == "i" || out.hasPrefix("i'") { out = "I" + out.dropFirst() }
        if typed.count > 1 && typed.allSatisfy({ $0.isUppercase }) { return out.uppercased() }
        if typed.first?.isUppercase == true { return out.prefix(1).uppercased() + out.dropFirst() }
        return out
    }

    /// "i", or a contraction typed without its apostrophe (dont, im, cant)
    func isContraction(_ typed: String) -> Bool { let l = typed.lowercased(); return l == "i" || contraction.contains(l) }

    /// The list's spelling of a typed word, if it is known in another form ("dont" → "don't", "i" → "I")
    func canonical(_ typed: String) -> String? {
        guard ready else { return nil }
        let low = typed.lowercased()
        if low == "i" { return "I" }
        if exact.contains(low) && !contraction.contains(low) { return nil }
        if low == "lets" { return nil }
        guard let r = rankOf[plain(typed)] else { return nil }
        let w = spell(typed, words[r])
        return w != typed ? w : nil
    }

    private func lowerBound(_ key: String) -> Int {
        var lo = 0, hi = sorted.count
        while lo < hi { let mid = (lo + hi) / 2; if sorted[mid] < key { lo = mid + 1 } else { hi = mid } }
        return lo
    }

    /// Words that start with what was typed, best first
    func complete(_ typed: String, max: Int = 3) -> [String] {
        guard ready, !typed.isEmpty else { return [] }
        let p = plain(typed)
        var found: [(String, Double)] = []
        for (w, n) in learned where n > 0 && w.lowercased().hasPrefix(p) && w.count > typed.count { found.append((w, score(w))) }
        var i = lowerBound(p), scanned = 0
        while i < sorted.count && sorted[i].hasPrefix(p) && scanned < 30_000 {
            let key = sorted[i]
            if key.count > p.count, let r = rankOf[key], !quiet.contains(words[r]) { found.append((words[r], score(words[r]))) }
            i += 1; scanned += 1
        }
        var seen = Set<String>()
        return found.sorted { $0.1 > $1.1 }.map { spell(typed, $0.0) }.filter { seen.insert($0).inserted }.prefix(max).map { $0 }
    }

    /// Known words one letter away from a typo, best first, with iOS's own spell checker's guesses
    func corrections(_ typed: String, max: Int = 3) -> [String] {
        guard ready, typed.count >= 2, !known(typed) else { return [] }
        let p = Array(plain(typed))
        var cand = Set<String>()
        let letters = Array("abcdefghijklmnopqrstuvwxyz")
        for i in 0...p.count {
            if i < p.count { var q = p; q.remove(at: i); cand.insert(String(q)) }
            if i < p.count - 1 { var q = p; q.swapAt(i, i + 1); cand.insert(String(q)) }
            for c in letters {
                if i < p.count { var q = p; q[i] = c; cand.insert(String(q)) }
                var q = p; q.insert(c, at: i); cand.insert(String(q))
            }
        }
        var out: [String] = cand.compactMap { k in rankOf[k].map { words[$0] } }
            .filter { !quiet.contains($0) && (rankOf[plain($0)] ?? Int.max) < 20_000 }
        // iOS's spell checker
        let ns = typed as NSString
        let miss = checker.rangeOfMisspelledWord(in: typed, range: NSRange(location: 0, length: ns.length), startingAt: 0, wrap: false, language: "en_US")
        if miss.location != NSNotFound {
            out += (checker.guesses(forWordRange: miss, in: typed, language: "en_US") ?? []).prefix(3).filter { !$0.contains(" ") && !quiet.contains($0.lowercased()) }
        }
        var seen = Set<String>()
        return out.sorted { score($0) > score($1) }.map { spell(typed, $0) }.filter { seen.insert($0).inserted }.prefix(max).map { $0 }
    }

    /// The most likely of some guesses: the most common word
    func best(_ guesses: [String]) -> String? {
        guesses.enumerated().max { a, b in score(a.element) - Double(a.offset) * 1e-9 < score(b.element) - Double(b.offset) * 1e-9 }?.element
    }

    /// Likely words after the last one or two: what you wrote before, then the basics
    func nextWords(_ prev2: String?, _ prev: String?, max: Int = 3) -> [String] {
        guard let prev, !prev.isEmpty else { return [] }
        let a = prev.lowercased()
        var out = pairs.filter { $0.key.hasPrefix(a + " ") }.sorted { $0.value > $1.value }.map { String($0.key.split(separator: " ").last ?? "") }
        if let p2 = prev2, let l = next["\(p2.lowercased()) \(a)"] { out += l }
        if let l = next[a] { out += l }
        var seen = Set<String>()
        return out.map { $0 == "i" || $0.hasPrefix("i'") ? "I" + $0.dropFirst() : $0 }.filter { !$0.isEmpty && seen.insert($0).inserted }.prefix(max).map { $0 }
    }

    /// Remember a word that was written (not in private fields)
    func learn(_ word: String, prev: String?) {
        guard ready, word.count >= 2, word.contains(where: { $0.isLetter }) else { return }
        learned[word, default: 0] += 1
        if let prev, !prev.isEmpty { pairs["\(prev.lowercased()) \(word.lowercased())", default: 0] += 1 }
        dirty = true
    }

    func save() {
        guard dirty else { return }
        dirty = false
        var text = ""
        for (k, v) in learned.sorted(by: { $0.value > $1.value }).prefix(5000) { text += "w\t\(k)\t\(v)\n" }
        for (k, v) in pairs.sorted(by: { $0.value > $1.value }).prefix(5000) { text += "p\t\(k)\t\(v)\n" }
        try? FileManager.default.createDirectory(at: Engine.folder, withIntermediateDirectories: true)
        try? text.write(to: file, atomically: true, encoding: .utf8)
    }

    func forget() {
        learned.removeAll(); pairs.removeAll(); dirty = false
        try? FileManager.default.removeItem(at: file)
    }
}
