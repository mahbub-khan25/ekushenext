import Foundation

/// Typo correction that knows the keyboard, the same as the Android keyboard's (Spatial.kt): a letter
/// typed instead of the one next to it ("thw" for "the") costs less than one from across the keys,
/// and where the finger actually landed counts.
///
/// Positions are in key units on the QWERTY letters: q's centre is (0.5, 0.5), a's (1.0, 1.5), z's
/// (2.0, 2.5); one unit is one key's width (or one row's height).
enum Spatial {
    private static let centers: [Character: (Float, Float)] = {
        var d: [Character: (Float, Float)] = [:]
        for (i, c) in "qwertyuiop".enumerated() { d[c] = (Float(i) + 0.5, 0.5) }
        for (i, c) in "asdfghjkl".enumerated() { d[c] = (Float(i) + 1.0, 1.5) }
        for (i, c) in "zxcvbnm".enumerated() { d[c] = (Float(i) + 2.0, 2.5) }
        return d
    }()

    static func center(_ c: Character) -> (Float, Float)? { centers[Character(c.lowercased())] }

    /// Which letters a word has, one bit each (a quick first check before the full comparison)
    static func letters(_ w: [UInt8]) -> Int32 {
        var m: Int32 = 0
        for c in w where c >= 97 && c <= 122 { m |= 1 << Int32(c - 97) }
        return m
    }

    /// How far a tap (or the typed key's centre) is from a key, in key widths
    static func distance(_ tap: (Float, Float)?, _ typed: UInt8, _ key: UInt8) -> Float {
        guard let k = centers[Character(UnicodeScalar(key))] else { return 9 }
        guard let t = tap ?? centers[Character(UnicodeScalar(typed))] else { return 9 }
        // rows are a little taller than keys are wide, and misses between rows are rarer
        let dx = t.0 - k.0, dy = (t.1 - k.1) * 1.25
        return (dx * dx + dy * dy).squareRoot()
    }

    /// One letter typed where another was meant: a neighbour is a cheap mistake, a far key isn't
    static func substitution(_ tap: (Float, Float)?, _ typed: UInt8, _ meant: UInt8) -> Float {
        if typed == meant { return 0 }
        return min(1, max(0.2, (distance(tap, typed, meant) - 0.3) * 0.8))
    }

    private static func deleteCost(_ t: [UInt8], _ i: Int) -> Float {
        if i > 0 && t[i - 1] == t[i] { return 0.45 }
        if i > 0 && distance(nil, t[i - 1], t[i]) < 1.2 { return 0.65 }
        return 0.8
    }

    private static func insertCost(_ c: [UInt8], _ j: Int) -> Float {
        if j > 0 && c[j - 1] == c[j] { return 0.45 }
        return 0.9
    }

    /// What it costs to turn what was typed into `cand` (lowercase a–z): letters swapped, missing,
    /// extra or mistyped. Stops early above `limit` (returns a number above it).
    static func cost(_ t: [UInt8], _ taps: [(Float, Float)?], _ c: [UInt8], limit: Float) -> Float {
        let n = t.count, m = c.count
        if n == 0 { return Float(m) * 0.9 }
        let big = limit + 1
        var prev2 = [Float](repeating: 0, count: m + 1)
        var prev = [Float](repeating: 0, count: m + 1)
        var cur = [Float](repeating: 0, count: m + 1)
        if m > 0 { for j in 1...m { prev[j] = prev[j - 1] + insertCost(c, j - 1) } }
        var lastMin: Float = 0
        for i in 1...n {
            let ti = t[i - 1]
            let tap = i - 1 < taps.count ? taps[i - 1] : nil
            cur[0] = prev[0] + deleteCost(t, i - 1)
            var rowMin = cur[0]
            if m > 0 {
                for j in 1...m {
                    let cj = c[j - 1]
                    var best = prev[j - 1] + substitution(tap, ti, cj)
                    best = min(best, prev[j] + deleteCost(t, i - 1))
                    best = min(best, cur[j - 1] + insertCost(c, j - 1))
                    if i > 1 && j > 1 && ti == c[j - 2] && t[i - 2] == cj { best = min(best, prev2[j - 2] + 0.55) }
                    cur[j] = best
                    rowMin = min(rowMin, best)
                }
            }
            // a swap reaches back two rows, so stop only when both are past the limit
            if rowMin > limit && lastMin > limit { return big }
            lastMin = rowMin
            (prev2, prev, cur) = (prev, cur, prev2)
        }
        return prev[m]
    }

    /// How far a word may be from what was typed and still be offered as its correction
    static func limit(_ length: Int) -> Float {
        switch length {
        case ...2: return 0.45
        case 3: return 0.7
        case 4...5: return 1.25
        case 6...8: return 1.75
        default: return 2.25
        }
    }

    /// A word's likelihood: common words and the person's own words first, close words before far ones
    static func score(rank: Int, learned: Int, cost: Float) -> Double {
        log(1.0 / Double(rank + 50)) + log(1.0 + Double(learned)) * 0.6 - 3.2 * Double(cost)
    }

    /// Could `c` start the way `t` does (the same first key, a neighbour, or two letters swapped)?
    static func startsLike(_ t: [UInt8], _ firstTap: (Float, Float)?, _ c: [UInt8]) -> Bool {
        let a = t[0], b = c[0]
        if a == b { return true }
        if distance(firstTap, a, b) < 1.3 { return true }
        if t.count > 1 && c.count > 1 && t[1] == b && c[1] == a { return true }
        if c.count > 1 && c[1] == a { return true }
        if t.count > 1 && t[1] == b { return true }
        return false
    }
}
