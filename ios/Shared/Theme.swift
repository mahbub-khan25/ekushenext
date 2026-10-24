import UIKit

extension UIColor {
    convenience init(hex: String) {
        var s = hex.hasPrefix("#") ? String(hex.dropFirst()) : hex
        if s.count == 6 { s = "FF" + s }
        let v = UInt64(s, radix: 16) ?? 0
        self.init(red: CGFloat((v >> 16) & 0xff) / 255, green: CGFloat((v >> 8) & 0xff) / 255, blue: CGFloat(v & 0xff) / 255, alpha: CGFloat((v >> 24) & 0xff) / 255)
    }
    func mixed(with o: UIColor, _ t: CGFloat) -> UIColor {
        var r1: CGFloat = 0, g1: CGFloat = 0, b1: CGFloat = 0, a1: CGFloat = 0, r2: CGFloat = 0, g2: CGFloat = 0, b2: CGFloat = 0, a2: CGFloat = 0
        getRed(&r1, green: &g1, blue: &b1, alpha: &a1); o.getRed(&r2, green: &g2, blue: &b2, alpha: &a2)
        return UIColor(red: r1 + (r2 - r1) * t, green: g1 + (g2 - g1) * t, blue: b1 + (b2 - b1) * t, alpha: a1)
    }
}

/// Keyboard colours: the same themes as the Android keyboard.
struct Palette {
    let background, key, keyPressed, keyShadow, functionKey, text, subtext, hint, accent, onAccent, strip, chip, popup: UIColor
    let dark: Bool
    var keyRadius: CGFloat = 7
    var shadow: CGFloat = 1.2
    var enter: UIColor? = nil
    var onEnter: UIColor? = nil

    private static func p(_ bg: String, _ key: String, _ pressed: String, _ shadow: String, _ fn: String, _ text: String, _ sub: String, _ hint: String, _ accent: String, _ onAccent: String, _ chip: String, _ popup: String, dark: Bool, radius: CGFloat = 7, shadowPt: CGFloat = 1.2, enter: String? = nil, onEnter: String? = nil) -> Palette {
        Palette(background: UIColor(hex: bg), key: UIColor(hex: key), keyPressed: UIColor(hex: pressed), keyShadow: UIColor(hex: shadow), functionKey: UIColor(hex: fn), text: UIColor(hex: text), subtext: UIColor(hex: sub), hint: UIColor(hex: hint), accent: UIColor(hex: accent), onAccent: UIColor(hex: onAccent), strip: UIColor(hex: bg), chip: UIColor(hex: chip), popup: UIColor(hex: popup), dark: dark, keyRadius: radius, shadow: shadowPt, enter: enter.map { UIColor(hex: $0) }, onEnter: onEnter.map { UIColor(hex: $0) })
    }

    // The flag's green, on night or on mist: what "Follow system" switches between
    static let dark = p("#0C1512", "#22302A", "#33463D", "#80000000", "#17221D", "#EEF4F1", "#A0B6AC", "#5FD6A3", "#2FD18E", "#03170F", "#1A2621", "#2C3D35", dark: true)
    static let light = p("#DDE5E1", "#FFFFFF", "#C9D6D0", "#3304120C", "#C3CFC9", "#0E241B", "#5A6F65", "#0B8A5C", "#0B8A5C", "#FFFFFF", "#EEF3F1", "#FFFFFF", dark: false)
    // একুশে: the Shaheed Minar at dawn, deep green with the red sun
    static let ekushe = p("#06372A", "#0E4D3A", "#176650", "#66000000", "#0A4232", "#F4FAF7", "#A9CFC0", "#FF8A84", "#E23B3B", "#FFFFFF", "#0E4D3A", "#155C46", dark: true)
    // একুশে ডিফল্ট: the familiar phone keyboard look
    static let defaultLight = p("#E8EAEF", "#FFFFFF", "#D3D7DE", "#00000000", "#D2D6DE", "#1F1F1F", "#5F6368", "#80868B", "#3C7BF0", "#FFFFFF", "#F4F5F8", "#FFFFFF", dark: false, radius: 8, shadowPt: 0)
    static let defaultDark = p("#202124", "#3C4043", "#5F6368", "#00000000", "#2E3033", "#E8EAED", "#9AA0A6", "#9AA0A6", "#8AB4F8", "#202124", "#2E3033", "#4A4E52", dark: true, radius: 8, shadowPt: 0)
    // একুশে মসৃণ: soft white keys with a thin shadow on light grey
    static let smoothLight = p("#D1D4DA", "#FFFFFF", "#ECEEF1", "#8E9196", "#ACB1BA", "#000000", "#3C3C43", "#8E8E93", "#007AFF", "#FFFFFF", "#E1E3E7", "#FFFFFF", dark: false, radius: 5.5, shadowPt: 1, enter: "#ACB1BA", onEnter: "#000000")
    static let smoothDark = p("#2B2B2D", "#6B6B6F", "#88888C", "#0D0D0D", "#46464A", "#FFFFFF", "#D0D0D5", "#B4B4BA", "#0A84FF", "#FFFFFF", "#3A3A3D", "#6B6B6F", dark: true, radius: 5.5, shadowPt: 1, enter: "#46464A", onEnter: "#FFFFFF")
    // কৃষ্ণচূড়া, পদ্মা, শাপলা, কাশফুল, রাত
    static let krishnachura = p("#FCE3D3", "#FFF8F2", "#F8CDB4", "#40A1300A", "#F6C3A6", "#3D1208", "#8A4A35", "#D93A16", "#E8501F", "#FFFFFF", "#FFEFE4", "#FFF8F2", dark: false)
    static let padma = p("#0A1C30", "#16304C", "#22436A", "#80000000", "#10263E", "#EAF4FF", "#9DB7D3", "#7CC6FF", "#3AA0FF", "#04192D", "#16304C", "#20406A", dark: true)
    static let shapla = p("#F9E4EC", "#FFFFFF", "#F2C9D8", "#33701838", "#F1CADA", "#3A1426", "#8C5670", "#D2457E", "#DC4F86", "#FFFFFF", "#FDF1F5", "#FFFFFF", dark: false)
    static let kashful = p("#EFEEEA", "#FFFFFF", "#DDDCD6", "#1F000000", "#E0DFD9", "#2A2A28", "#7C7B76", "#9A988F", "#4B4A46", "#FFFFFF", "#F8F7F4", "#FFFFFF", dark: false)
    static let night = p("#000000", "#151515", "#2A2A2A", "#00000000", "#0B0B0B", "#F2F2F2", "#9A9A9A", "#2FD18E", "#2FD18E", "#000000", "#151515", "#222222", dark: true)

    static func byId(_ id: String, dark isDark: Bool) -> Palette {
        switch id {
        case "ekushe": return ekushe
        case "default": return isDark ? defaultDark : defaultLight
        case "smooth": return isDark ? smoothDark : smoothLight
        case "krishnachura": return krishnachura
        case "padma": return padma
        case "shapla": return shapla
        case "kashful": return kashful
        case "night": return night
        case "light": return light
        case "dark": return dark
        default: return isDark ? dark : light
        }
    }
}

/// The theme gallery, in order
let themeChoices: [(id: String, bn: String, en: String)] = [
    ("ekushe", "একুশে", "Ekushe"), ("default", "একুশে ডিফল্ট", "Ekushe Default"), ("smooth", "একুশে মসৃণ", "Ekushe Smooth"),
    ("krishnachura", "কৃষ্ণচূড়া", "Krishnachura"), ("padma", "পদ্মা", "Padma"), ("shapla", "শাপলা", "Shapla"),
    ("kashful", "কাশফুল", "Kashful"), ("night", "রাত", "Night"), ("system", "ফোনের মতো", "Follow system"),
    ("light", "সবুজ দিন", "Classic light"), ("dark", "সবুজ রাত", "Classic dark"),
]
