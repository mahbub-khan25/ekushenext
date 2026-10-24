import UIKit

/// What the keyboard asks the controller to do.
protocol KeyboardListener: AnyObject {
    func onText(_ text: String)
    func onDelete()
    func onEnter()
    func onSpace()
    func onCandidate(_ index: Int)
    func onPrediction(_ word: String)
    func onLanguageToggle()
    func onGlobe()
    func onCursorMove(_ steps: Int)
    func onSettings()
    func onHide()
    func onEmoji(_ e: String)
    func onKeyFeedback()
}

enum Mode { case letters, symbols, moreSymbols, numbers, emoji }
enum Shift { case off, on, locked }

/// The whole keyboard, drawn in one view, as on Android: the suggestion strip, the keys, the key
/// preview and the long-press choices. A character is typed when the finger lifts (so a slide can
/// still change the choice); a second finger landing types the first finger's key straight away.
final class KeyboardView: UIView {
    weak var listener: KeyboardListener?
    private let prefs = Prefs.shared
    private(set) var palette = Palette.ekushe

    var mode: Mode = .letters { didSet { buildKeys(); setNeedsDisplay() } }
    var shift: Shift = .off { didSet { setNeedsDisplay() } }
    var bangla = true { didSet { setNeedsDisplay() } }
    var enterLabel = "↵" { didSet { setNeedsDisplay() } }
    /// iOS asks for a globe key when the phone has no other way to switch keyboards
    var showsGlobe = false { didSet { if oldValue != showsGlobe { buildKeys(); setNeedsDisplay() } } }
    var candidates: [Candidate] = []
    var tips: [Hint] = []
    var predictions: [String] = []
    var stripMessage: String?

    func showSuggestions(_ c: [Candidate], _ t: [Hint], _ p: [String]) {
        candidates = c; tips = t; predictions = p; stripScroll = 0; stripMessage = nil
        setNeedsDisplay()
    }

    // ── geometry
    let stripH: CGFloat = 46
    var rowH: CGFloat { [48, 54, 60][max(0, min(2, prefs.height))] }
    private let gap: CGFloat = 5
    private let bottomPad: CGFloat = 4
    var preferredHeight: CGFloat { stripH + CGFloat(prefs.numberRow && mode == .letters ? 5 : 4) * rowH + bottomPad }

    private struct Placed { let key: Key; let rect: CGRect }
    private var placed: [Placed] = []
    private var rows: [[Key]] = []
    private var borders = true
    private var effect = "classic"

    override init(frame: CGRect) {
        super.init(frame: frame)
        isMultipleTouchEnabled = true
        isOpaque = true
        contentMode = .redraw
        buildKeys()
    }
    required init?(coder: NSCoder) { fatalError() }

    func refreshTheme(dark: Bool) {
        palette = Palette.byId(prefs.theme, dark: dark)
        borders = prefs.keyBorders
        effect = prefs.pressEffect
        backgroundColor = palette.background
        buildKeys(); setNeedsDisplay()
    }

    private func buildKeys() {
        switch mode {
        case .letters: rows = Layouts.letters(numberRow: prefs.numberRow, emojiKey: prefs.emojiKey, globe: showsGlobe)
        case .symbols: rows = Layouts.symbols(emojiKey: prefs.emojiKey, globe: showsGlobe)
        case .moreSymbols: rows = Layouts.moreSymbols(emojiKey: prefs.emojiKey, globe: showsGlobe)
        case .numbers: rows = Layouts.numbers()
        case .emoji: rows = []
        }
        layoutKeys()
    }

    override func layoutSubviews() {
        super.layoutSubviews()
        layoutKeys()
    }

    private func layoutKeys() {
        guard bounds.width > 0 else { return }
        var out: [Placed] = []
        let top = stripH
        let areaH = bounds.height - stripH - bottomPad
        let rh = areaH / CGFloat(max(1, rows.count))
        let unitsMax = rows.map { $0.reduce(0) { $0 + $1.width } }.max() ?? 10
        let unit = (bounds.width - gap) / (mode == .numbers ? unitsMax : max(10, unitsMax))
        for (r, row) in rows.enumerated() {
            let units = row.reduce(0) { $0 + $1.width }
            var x = (bounds.width - units * unit) / 2
            let y = top + CGFloat(r) * rh
            for k in row {
                let w = k.width * unit
                out.append(Placed(key: k, rect: CGRect(x: x + gap / 2, y: y + gap / 2, width: w - gap, height: rh - gap)))
                x += w
            }
        }
        placed = out
    }

    /// Where each key is (for tests)
    var keyCentres: [String: CGPoint] {
        var d: [String: CGPoint] = [:]
        for p in placed { d[p.key.id] = CGPoint(x: p.rect.midX, y: p.rect.midY) }
        return d
    }

    // ───────────────────────── drawing ─────────────────────────

    private func font(_ size: CGFloat, _ weight: UIFont.Weight = .regular) -> UIFont { .systemFont(ofSize: size, weight: weight) }

    private func draw(_ s: String, centre: CGPoint, font: UIFont, color: UIColor) {
        let a: [NSAttributedString.Key: Any] = [.font: font, .foregroundColor: color]
        let size = (s as NSString).size(withAttributes: a)
        (s as NSString).draw(at: CGPoint(x: centre.x - size.width / 2, y: centre.y - size.height / 2), withAttributes: a)
    }

    private func fill(_ r: CGRect, _ radius: CGFloat, _ c: UIColor) {
        c.setFill()
        UIBezierPath(roundedRect: r, cornerRadius: radius).fill()
    }

    override func draw(_ rect: CGRect) {
        let p = palette
        p.background.setFill()
        UIRectFill(bounds)
        drawStrip()
        if mode == .emoji { drawEmoji(); return }
        for pl in placed { drawKey(pl.key, pl.rect, pressed: pressedKey == pl.key && pressedRect == pl.rect) }
        if let k = pressedKey, let r = pressedRect, k.type == .char, alts == nil, mode != .numbers { drawPreview(k, r) }
        if let a = alts { drawAlts(a) }
    }

    private func isFunction(_ k: Key) -> Bool { k.type != .char && k.type != .space }

    private func keyText(_ k: Key) -> String {
        if k.letter && shift != .off { return k.label.uppercased() }
        // Symbols key: Bangla digits alone in Bangla ("?" next to them sat oddly high), ?123 in English
        if k.type == .symbols && k.label == "?১২৩" { return bangla ? "১২৩" : "?123" }
        if (mode == .symbols || mode == .letters), k.label.count == 1, let c = k.label.first, c.isASCII, c.isNumber { return Layouts.digitLabel(c, bangla: bangla) }
        return k.label
    }

    private func drawKey(_ k: Key, _ slot: CGRect, pressed: Bool) {
        let p = palette
        let radius = p.keyRadius
        let enter = k.type == .enter
        let shiftOn = k.type == .shift && shift != .off
        let base: UIColor = enter ? (p.enter ?? p.accent) : shiftOn ? (shift == .locked ? p.accent : p.keyPressed) : isFunction(k) ? p.functionKey : p.key
        let shaped = borders || pressed || enter || shiftOn || k.type == .space
        let depth: CGFloat = 4.5
        var r = slot
        if effect == "3d" && shaped { r = pressed ? CGRect(x: slot.minX, y: slot.minY + depth, width: slot.width, height: slot.height - depth) : CGRect(x: slot.minX, y: slot.minY, width: slot.width, height: slot.height - depth) }
        if effect == "bounce" && pressed { r = slot.insetBy(dx: slot.width * 0.07, dy: slot.height * 0.08) }
        if shaped {
            let face = (!borders && !pressed && k.type == .space) ? p.functionKey : base
            switch effect {
            case "3d":
                if !pressed { fill(CGRect(x: slot.minX, y: slot.minY + depth, width: slot.width, height: slot.height - depth), radius, face.mixed(with: .black, p.dark ? 0.62 : 0.3)) }
                fill(r, radius, pressed ? face.mixed(with: p.text, 0.1) : face)
                if !pressed { fill(CGRect(x: r.minX + 1.5, y: r.minY + 1, width: r.width - 3, height: 3), 1.5, UIColor.white.withAlphaComponent(p.dark ? 0.1 : 0.55)) }
            case "glow":
                if pressed { fill(r.insetBy(dx: -3, dy: -3), radius + 3, p.accent.withAlphaComponent(0.35)) }
                else if borders && p.shadow > 0 { fill(r.offsetBy(dx: 0, dy: p.shadow), radius, p.keyShadow) }
                fill(r, radius, pressed ? face.mixed(with: p.accent, 0.18) : face)
                if pressed { p.accent.setStroke(); let b = UIBezierPath(roundedRect: r, cornerRadius: radius); b.lineWidth = 2; b.stroke() }
            default:
                if borders && p.shadow > 0 { fill(r.offsetBy(dx: 0, dy: p.shadow), radius, p.keyShadow) }
                fill(r, radius, pressed ? face.mixed(with: p.text, 0.16) : face)
            }
        }
        let fg = enter ? (p.onEnter ?? p.onAccent) : (shift == .locked && k.type == .shift) ? p.onAccent : p.text
        let c = CGPoint(x: r.midX, y: r.midY)
        switch k.type {
        case .space:
            // The language, plus the keyboard's name when the space bar is wide enough for both
            var label = "␣"
            if mode != .numbers {
                let full = bangla ? "বাংলা · একুশে নেক্সট" : "English · Ekushe Next"
                let fits = (full as NSString).size(withAttributes: [.font: font(13)]).width <= r.width - 20
                label = fits ? full : (bangla ? "বাংলা" : "English")
            }
            draw(label, centre: c, font: font(13), color: p.subtext)
        case .lang:
            draw(bangla ? "বাং" : "EN", centre: CGPoint(x: c.x, y: c.y - 6), font: font(14, .semibold), color: bangla ? p.hint : p.subtext)
            draw(bangla ? "EN" : "বাং", centre: CGPoint(x: c.x, y: c.y + 10), font: font(9.5), color: p.subtext)
        case .enter:
            draw(enterLabel, centre: c, font: font(enterLabel.count > 2 ? 14 : 20, .medium), color: fg)
        default:
            let t = keyText(k)
            let size: CGFloat = [.delete, .shift, .emoji, .globe].contains(k.type) ? 20 : isFunction(k) ? 15 : mode == .numbers ? 27 : t.count > 2 ? 14 : 23
            draw(t, centre: c, font: font(size, k.letter ? .regular : .medium), color: fg)
            if k.letter && bangla && mode == .letters && prefs.keyHints, let h = shift != .off ? k.hintShift : k.hint {
                let a: [NSAttributedString.Key: Any] = [.font: font(10), .foregroundColor: p.hint]
                let s = (h as NSString).size(withAttributes: a)
                (h as NSString).draw(at: CGPoint(x: r.maxX - s.width - 4, y: r.minY + 1), withAttributes: a)
            }
            if !k.alts.isEmpty && !k.letter { fill(CGRect(x: r.maxX - 8, y: r.minY + 5, width: 3, height: 3), 1.5, p.subtext) }
        }
    }

    private func drawPreview(_ k: Key, _ r: CGRect) {
        let p = palette
        let w = max(r.width * 1.3, 46), h = r.height * 1.2 + 8
        let left = min(max(2, r.midX - w / 2), bounds.width - w - 2)
        let box = CGRect(x: left, y: max(0, r.minY - h - 2), width: w, height: h)
        fill(box.offsetBy(dx: 0, dy: 2), 10, p.keyShadow)
        fill(box, 10, p.popup)
        draw(keyText(k), centre: CGPoint(x: box.midX, y: box.minY + h * 0.45), font: font(30), color: p.text)
        if k.letter && bangla && mode == .letters, let hint = shift != .off ? k.hintShift : k.hint {
            draw(hint, centre: CGPoint(x: box.midX, y: box.minY + h * 0.82), font: font(15, .semibold), color: p.hint)
        }
    }

    // ── long-press choices
    private struct AltsPopup { let key: Key; let items: [Alt]; let rect: CGRect; let cell: CGFloat; var selected: Int }
    private var alts: AltsPopup?

    private func openAlts(_ k: Key, _ r: CGRect) {
        let items = shift != .off ? (k.shiftAlts ?? k.alts) : k.alts
        guard !items.isEmpty else { return }
        let cell = max(r.width, 46)
        let w = cell * CGFloat(items.count), h: CGFloat = 58
        let left = min(max(2, r.midX - cell / 2), max(2, bounds.width - w - 2))
        alts = AltsPopup(key: k, items: items, rect: CGRect(x: left, y: max(0, r.minY - h - 4), width: w, height: h), cell: cell, selected: 0)
        listener?.onKeyFeedback()
        setNeedsDisplay()
    }

    private func drawAlts(_ a: AltsPopup) {
        let p = palette
        fill(a.rect.offsetBy(dx: 0, dy: 3), 12, p.keyShadow)
        fill(a.rect, 12, p.popup)
        for (i, alt) in a.items.enumerated() {
            let cx = a.rect.minX + a.cell * (CGFloat(i) + 0.5)
            let on = i == a.selected
            if on { fill(CGRect(x: a.rect.minX + a.cell * CGFloat(i) + 3, y: a.rect.minY + 3, width: a.cell - 6, height: a.rect.height - 6), 9, p.accent) }
            draw(alt.label, centre: CGPoint(x: cx, y: a.rect.minY + (alt.note != nil ? 22 : 29)), font: font(alt.label.count > 2 ? 15 : 20, .semibold), color: on ? p.onAccent : p.text)
            if let n = alt.note { draw(n, centre: CGPoint(x: cx, y: a.rect.minY + 44), font: font(12.5), color: on ? p.onAccent : p.hint) }
        }
    }

    // ── suggestion strip
    private struct Chip { let rect: CGRect; let action: () -> Void; let label: String; let note: String?; let strong: Bool; let tip: Bool }
    private var chips: [Chip] = []
    private var stripScroll: CGFloat = 0
    private var stripContentW: CGFloat = 0
    private struct Tool { let id: String; let rect: CGRect }
    private var tools: [Tool] = []

    private func drawStrip() {
        let p = palette
        p.strip.setFill(); UIRectFill(CGRect(x: 0, y: 0, width: bounds.width, height: stripH))
        var list: [Chip] = []
        var x: CGFloat = 8 - stripScroll
        func chip(_ label: String, _ note: String?, strong: Bool, tip: Bool, _ action: @escaping () -> Void) {
            let f = font(17, strong ? .semibold : .regular)
            var w = (label as NSString).size(withAttributes: [.font: f]).width + 24
            if let n = note { w += (n as NSString).size(withAttributes: [.font: font(12, .bold)]).width + 8 }
            w = max(w, 52)
            list.append(Chip(rect: CGRect(x: x, y: 6, width: w, height: stripH - 12), action: action, label: label, note: note, strong: strong, tip: tip))
            x += w + 6
        }
        if !candidates.isEmpty {
            for (i, c) in candidates.enumerated() { chip(c.text, c.kind == "English" ? "EN" : nil, strong: i == 0, tip: false) { [weak self] in self?.listener?.onCandidate(i) } }
            for t in tips { if let i = candidates.firstIndex(where: { $0.text == t.bangla }) { chip(t.bangla, t.keys, strong: false, tip: true) { [weak self] in self?.listener?.onCandidate(i) } } }
        } else if !predictions.isEmpty {
            for w in predictions { chip(w, nil, strong: false, tip: false) { [weak self] in self?.listener?.onPrediction(w) } }
        }
        stripContentW = x + stripScroll
        chips = list
        if list.isEmpty { drawToolbar(); return }
        let ctx = UIGraphicsGetCurrentContext()
        ctx?.saveGState(); ctx?.clip(to: CGRect(x: 0, y: 0, width: bounds.width, height: stripH))
        for ch in list {
            fill(ch.rect, 10, ch.strong ? p.accent : p.chip)
            if ch.tip { p.accent.setStroke(); let b = UIBezierPath(roundedRect: ch.rect, cornerRadius: 10); b.lineWidth = 1.5; b.stroke() }
            let noteW = ch.note.map { ($0 as NSString).size(withAttributes: [.font: font(12, .bold)]).width + 8 } ?? 0
            draw(ch.label, centre: CGPoint(x: ch.rect.midX - noteW / 2, y: ch.rect.midY), font: font(17, ch.strong ? .semibold : .regular), color: ch.strong ? p.onAccent : p.text)
            if let n = ch.note { draw(n, centre: CGPoint(x: ch.rect.maxX - 10 - noteW / 2 + 4, y: ch.rect.midY), font: font(12, .bold), color: ch.strong ? p.onAccent : p.hint) }
        }
        ctx?.restoreGState()
    }

    private func drawToolbar() {
        let p = palette
        let w: CGFloat = 52
        tools = [
            Tool(id: "lang", rect: CGRect(x: 8, y: 6, width: 96, height: stripH - 12)),
            Tool(id: "emoji", rect: CGRect(x: bounds.width - 3 * w - 8, y: 0, width: w, height: stripH)),
            Tool(id: "settings", rect: CGRect(x: bounds.width - 2 * w - 8, y: 0, width: w, height: stripH)),
            Tool(id: "hide", rect: CGRect(x: bounds.width - w - 8, y: 0, width: w, height: stripH)),
        ]
        let pill = tools[0].rect
        fill(pill, pill.height / 2, p.chip)
        fill(CGRect(x: bangla ? pill.minX : pill.midX, y: pill.minY, width: pill.width / 2, height: pill.height), pill.height / 2, p.accent)
        draw("বাংলা", centre: CGPoint(x: pill.minX + pill.width / 4, y: pill.midY), font: font(14, .semibold), color: bangla ? p.onAccent : p.subtext)
        draw("EN", centre: CGPoint(x: pill.minX + pill.width * 3 / 4, y: pill.midY), font: font(14, .bold), color: bangla ? p.subtext : p.onAccent)
        if let m = stripMessage {
            let a: [NSAttributedString.Key: Any] = [.font: font(13), .foregroundColor: p.subtext]
            let s = (m as NSString).size(withAttributes: a)
            (m as NSString).draw(at: CGPoint(x: pill.maxX + 12, y: pill.midY - s.height / 2), withAttributes: a)
        }
        let glyph = ["emoji": "☺︎", "settings": "⚙︎", "hide": "⌄"]
        for t in tools.dropFirst() { draw(glyph[t.id]!, centre: CGPoint(x: t.rect.midX, y: t.rect.midY), font: font(21), color: p.subtext) }
    }

    // ── emoji panel
    private var emojiGroup = 0
    private var emojiScroll: CGFloat = 0
    private var emojiCells: [(CGRect, String)] = []
    private var emojiTabs: [(CGRect, Int)] = []
    private var emojiAbc = CGRect.zero
    private var emojiDel = CGRect.zero
    private let tabH: CGFloat = 44

    private func emojiItems() -> [String] { emojiGroup == -1 ? prefs.recentEmoji : Emoji.groups[emojiGroup].items }

    private func drawEmoji() {
        let p = palette
        let areaTop = stripH, areaBottom = bounds.height - bottomPad - tabH
        let cols = 8
        let cw = bounds.width / CGFloat(cols), chH: CGFloat = 46
        var cells: [(CGRect, String)] = []
        let ctx = UIGraphicsGetCurrentContext()
        ctx?.saveGState(); ctx?.clip(to: CGRect(x: 0, y: areaTop, width: bounds.width, height: areaBottom - areaTop))
        let items = emojiItems()
        for (i, e) in items.enumerated() {
            let r = CGRect(x: CGFloat(i % cols) * cw, y: areaTop + CGFloat(i / cols) * chH - emojiScroll, width: cw, height: chH)
            if r.maxY > areaTop && r.minY < areaBottom { draw(e, centre: CGPoint(x: r.midX, y: r.midY), font: font(28), color: p.text); cells.append((r, e)) }
        }
        if items.isEmpty { draw("এখনো কোনো ইমোজি ব্যবহার করা হয়নি", centre: CGPoint(x: bounds.width / 2, y: (areaTop + areaBottom) / 2), font: font(15), color: p.subtext) }
        ctx?.restoreGState()
        emojiCells = cells
        let n = Emoji.groups.count + 3
        let tw = bounds.width / CGFloat(n)
        let ty = bounds.height - bottomPad - tabH
        p.functionKey.setFill(); UIRectFill(CGRect(x: 0, y: ty, width: bounds.width, height: bounds.height - ty))
        emojiAbc = CGRect(x: 0, y: ty, width: tw, height: tabH)
        emojiDel = CGRect(x: bounds.width - tw, y: ty, width: tw, height: tabH)
        draw("ABC", centre: CGPoint(x: emojiAbc.midX, y: emojiAbc.midY), font: font(14, .bold), color: p.text)
        draw("⌫", centre: CGPoint(x: emojiDel.midX, y: emojiDel.midY), font: font(20), color: p.text)
        var tabs: [(CGRect, Int)] = []
        for (i, ic) in (["🕘"] + Emoji.groups.map { $0.icon }).enumerated() {
            let gid = i - 1
            let r = CGRect(x: tw * CGFloat(i + 1), y: ty, width: tw, height: tabH)
            if gid == emojiGroup { fill(r.insetBy(dx: 4, dy: 5), 9, p.keyPressed) }
            draw(ic, centre: CGPoint(x: r.midX, y: r.midY), font: font(20), color: p.text)
            tabs.append((r, gid))
        }
        emojiTabs = tabs
    }

    private func emojiMaxScroll() -> CGFloat {
        let rowsN = (emojiItems().count + 7) / 8
        return max(0, CGFloat(rowsN) * 46 - (bounds.height - bottomPad - tabH - stripH))
    }

    func openEmoji() {
        emojiScroll = 0; emojiGroup = prefs.recentEmoji.isEmpty ? 0 : -1; mode = .emoji
    }

    private func rememberEmoji(_ e: String) {
        prefs.recentEmoji = [e] + prefs.recentEmoji.filter { $0 != e }
    }

    // ───────────────────────── touch ─────────────────────────

    private var activeTouch: UITouch?
    private var pressedKey: Key?
    private var pressedRect: CGRect?
    private var downPoint = CGPoint.zero
    private var spaceDragging = false
    private var spaceAnchorX: CGFloat = 0
    private var longFired = false
    private var longTimer: Timer?
    private var repeatTimer: Timer?
    private var repeatCount = 0
    private var stripPressed = false
    private var stripDragging = false
    private var stripStartScroll: CGFloat = 0
    private var emojiDragging = false
    private var emojiStartScroll: CGFloat = 0
    private var lastShiftTap = Date.distantPast

    private func keyAt(_ pt: CGPoint) -> Placed? {
        if let hit = placed.first(where: { $0.rect.contains(pt) }) { return hit }
        // the nearest key in the row, so the gaps never swallow a touch
        return placed.filter { pt.y >= $0.rect.minY - gap && pt.y <= $0.rect.maxY + gap }.min { abs($0.rect.midX - pt.x) < abs($1.rect.midX - pt.x) }
    }

    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
        for t in touches {
            // a second finger: type the first finger's key now
            if activeTouch != nil, pressedKey != nil { release(fire: true) }
            activeTouch = t
            let pt = t.location(in: self)
            downPoint = pt
            if pt.y < stripH { stripPressed = true; stripDragging = false; stripStartScroll = stripScroll; continue }
            if mode == .emoji { emojiDragging = false; emojiStartScroll = emojiScroll; continue }
            if let hit = keyAt(pt) { press(hit, pt) }
        }
    }

    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let t = activeTouch, touches.contains(t) else { return }
        let pt = t.location(in: self)
        if stripPressed {
            if abs(pt.x - downPoint.x) > 8 { stripDragging = true }
            if stripDragging && !chips.isEmpty {
                stripScroll = min(max(0, stripStartScroll - (pt.x - downPoint.x)), max(0, stripContentW - bounds.width + 8))
                setNeedsDisplay()
            }
            return
        }
        if mode == .emoji {
            if abs(pt.y - downPoint.y) > 8 { emojiDragging = true }
            if emojiDragging { emojiScroll = min(max(0, emojiStartScroll - (pt.y - downPoint.y)), emojiMaxScroll()); setNeedsDisplay() }
            return
        }
        guard let k = pressedKey else { return }
        if var a = alts {
            let i = min(max(0, Int((pt.x - a.rect.minX) / a.cell)), a.items.count - 1)
            if i != a.selected { a.selected = i; alts = a; setNeedsDisplay() }
            return
        }
        if k.type == .space {
            let step: CGFloat = 12
            if !spaceDragging && abs(pt.x - downPoint.x) > 14 { spaceDragging = true; longTimer?.invalidate(); spaceAnchorX = pt.x }
            if spaceDragging {
                let n = Int((pt.x - spaceAnchorX) / step)
                if n != 0 { listener?.onCursorMove(n); spaceAnchorX += CGFloat(n) * step }
            }
            return
        }
        if k.type == .delete { return }
        // sliding to another key changes the choice
        if let hit = keyAt(pt), hit.key != k, hit.key.type == .char, k.type == .char {
            pressedKey = hit.key; pressedRect = hit.rect
            schedule(long: 0.33)
            setNeedsDisplay()
        }
    }

    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let t = activeTouch, touches.contains(t) else { return }
        let pt = t.location(in: self)
        activeTouch = nil
        if stripPressed { stripPressed = false; if !stripDragging { stripTap(pt) }; return }
        if mode == .emoji { if !emojiDragging { emojiTap(pt) }; return }
        release(fire: true)
    }

    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) {
        activeTouch = nil; stripPressed = false
        release(fire: false)
    }

    private func schedule(long seconds: TimeInterval) {
        longTimer?.invalidate()
        longTimer = Timer.scheduledTimer(withTimeInterval: seconds, repeats: false) { [weak self] _ in self?.longPress() }
    }

    private func press(_ hit: Placed, _ pt: CGPoint) {
        let k = hit.key
        pressedKey = k; pressedRect = hit.rect
        longFired = false; spaceDragging = false; spaceAnchorX = pt.x; repeatCount = 0
        listener?.onKeyFeedback()
        switch k.type {
        case .delete:
            listener?.onDelete()
            repeatTimer?.invalidate()
            repeatTimer = Timer.scheduledTimer(withTimeInterval: 0.4, repeats: false) { [weak self] _ in self?.repeatDelete() }
        case .shift: break
        default: schedule(long: k.type == .char ? 0.33 : 0.5)
        }
        setNeedsDisplay()
    }

    private func repeatDelete() {
        guard pressedKey?.type == .delete else { return }
        repeatCount += 1
        listener?.onDelete()
        repeatTimer = Timer.scheduledTimer(withTimeInterval: repeatCount > 12 ? 0.035 : 0.07, repeats: false) { [weak self] _ in self?.repeatDelete() }
    }

    private func longPress() {
        guard let k = pressedKey, let r = pressedRect else { return }
        switch k.type {
        case .space, .lang: if !spaceDragging { longFired = true; listener?.onGlobe() }
        case .char: openAlts(k, r); if alts != nil { longFired = true }
        default: break
        }
    }

    private func release(fire: Bool) {
        longTimer?.invalidate(); repeatTimer?.invalidate()
        let k = pressedKey, a = alts
        pressedKey = nil; pressedRect = nil; alts = nil
        setNeedsDisplay()
        guard fire, let k else { return }
        if let a { listener?.onText(a.items[a.selected].text); afterChar(); return }
        if longFired || spaceDragging { return }
        switch k.type {
        case .char: listener?.onText(k.letter && shift != .off ? k.text.uppercased() : k.text); afterChar()
        case .space: listener?.onSpace()
        case .enter: listener?.onEnter()
        case .delete: break
        case .shift:
            let now = Date()
            let quick = now.timeIntervalSince(lastShiftTap) < 0.3
            switch shift {
            case .off: shift = quick ? .locked : .on
            case .on: shift = quick ? .locked : .off
            case .locked: shift = .off
            }
            lastShiftTap = now
        case .symbols: mode = .symbols
        case .moreSymbols: mode = .moreSymbols
        case .letters: mode = .letters
        case .lang: listener?.onLanguageToggle()
        case .emoji: openEmoji()
        case .globe: listener?.onGlobe()
        }
    }

    private func afterChar() { if shift == .on { shift = .off } }

    private func stripTap(_ pt: CGPoint) {
        listener?.onKeyFeedback()
        if !chips.isEmpty { chips.first(where: { $0.rect.contains(pt) })?.action(); return }
        switch tools.first(where: { $0.rect.contains(pt) })?.id {
        case "lang": listener?.onLanguageToggle()
        case "emoji": openEmoji()
        case "settings": listener?.onSettings()
        case "hide": listener?.onHide()
        default: break
        }
    }

    private func emojiTap(_ pt: CGPoint) {
        listener?.onKeyFeedback()
        if emojiAbc.contains(pt) { mode = .letters; return }
        if emojiDel.contains(pt) { listener?.onDelete(); return }
        if let tab = emojiTabs.first(where: { $0.0.contains(pt) }) { emojiGroup = tab.1; emojiScroll = 0; setNeedsDisplay(); return }
        if let cell = emojiCells.first(where: { $0.0.contains(pt) }) { rememberEmoji(cell.1); listener?.onEmoji(cell.1) }
    }

    // ── for tests: press a key as a finger would
    func tap(_ id: String) {
        guard let p = placed.first(where: { $0.key.id == id }) else { return }
        press(p, CGPoint(x: p.rect.midX, y: p.rect.midY))
        release(fire: true)
    }
}
