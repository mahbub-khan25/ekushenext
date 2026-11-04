import Foundation

/// Glide typing, the same as the Android keyboard's (Glide.kt): the drawn line is compared with the
/// line each candidate word makes through the centres of its letters (both cut into the same number
/// of evenly spaced points), and the closest common words win. Points are in key units (Spatial).
enum Glide {
    typealias P = (Float, Float)
    static let n = 32

    static func length(_ pts: [P]) -> Float {
        var s: Float = 0
        if pts.count > 1 { for i in 1..<pts.count { s += hyp(pts[i].0 - pts[i - 1].0, pts[i].1 - pts[i - 1].1) } }
        return s
    }

    @inline(__always) static func hyp(_ a: Float, _ b: Float) -> Float { (a * a + b * b).squareRoot() }

    /// `count` points evenly spaced along a line
    static func resample(_ pts: [P], _ count: Int = n) -> [P] {
        guard let first = pts.first else { return [] }
        if pts.count == 1 { return Array(repeating: first, count: count) }
        let total = length(pts)
        if total < 1e-4 { return Array(repeating: first, count: count) }
        let step = total / Float(count - 1)
        var out: [P] = [first]
        var acc: Float = 0
        var prev = first
        var i = 1
        var cur = pts[1]
        while out.count < count - 1 && i < pts.count {
            let d = hyp(cur.0 - prev.0, cur.1 - prev.1)
            if acc + d >= step && d > 0 {
                let t = (step - acc) / d
                let q = (prev.0 + t * (cur.0 - prev.0), prev.1 + t * (cur.1 - prev.1))
                out.append(q); prev = q; acc = 0
            } else {
                acc += d; prev = cur; i += 1
                if i < pts.count { cur = pts[i] }
            }
        }
        while out.count < count { out.append(pts[pts.count - 1]) }
        return out
    }

    /// The line a word makes through its letters' centres (nil if a letter isn't on the keys)
    static func wordLine(_ word: [UInt8]) -> [P]? {
        var out: [P] = []
        for c in word {
            guard let p = Spatial.center(Character(UnicodeScalar(c))) else { return nil }
            if let l = out.last, l.0 == p.0, l.1 == p.1 { continue }
            out.append(p)
        }
        return out
    }

    /// Letters whose centres are within `r` key widths of a point
    static func keysNear(_ p: P, _ r: Float) -> Set<UInt8> {
        var s = Set<UInt8>()
        for c in "qwertyuiopasdfghjklzxcvbnm".utf8 {
            if let k = Spatial.center(Character(UnicodeScalar(c))), hyp(k.0 - p.0, (k.1 - p.1) * 1.1) <= r { s.insert(c) }
        }
        return s
    }

    static func distance(_ a: [P], _ b: [P]) -> Float {
        var s: Float = 0
        for i in 0..<a.count { s += hyp(a[i].0 - b[i].0, a[i].1 - b[i].1) }
        return s / Float(a.count)
    }

    /// Every letter of the word passes close to the drawn line
    static func covers(_ word: [UInt8], _ dense: [P], _ r: Float) -> Bool {
        for c in word {
            guard let k = Spatial.center(Character(UnicodeScalar(c))) else { return false }
            var best = Float.greatestFiniteMagnitude
            for p in dense { best = min(best, hyp(p.0 - k.0, p.1 - k.1)); if best <= r { break } }
            if best > r { return false }
        }
        return true
    }

    static func score(_ path: [P], _ line: [P], rank: Int, learned: Int) -> Double {
        let shape = Double(distance(path, line))
        let ends = Double(hyp(path[0].0 - line[0].0, path[0].1 - line[0].1) + hyp(path[path.count - 1].0 - line[line.count - 1].0, path[path.count - 1].1 - line[line.count - 1].1))
        return -3.0 * shape - 1.2 * ends + 0.35 * log(1.0 / Double(rank + 50)) + 0.25 * log(1.0 + Double(learned))
    }

    /// Long enough to be a glide (not a sloppy tap)
    static func isGlide(_ pts: [P]) -> Bool { length(pts) >= 0.8 }

    static func lengthFits(_ pathLen: Float, _ wordLen: Float) -> Bool {
        if wordLen < 0.3 { return pathLen < 1.5 }
        let r = pathLen / wordLen
        return r >= 0.55 && r <= 2.2
    }
}
