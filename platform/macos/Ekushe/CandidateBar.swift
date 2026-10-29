import Cocoa

/// The floating suggestion bar under the word being typed: frosted glass (the system's own blur),
/// light rounded chips, the chosen word in blue and a small EN on English words — the look of the
/// phone keyboards' suggestion strip ("Ekushe Default").
final class CandidateBar {
    static let shared = CandidateBar()

    /// Called with a chip's index when it is clicked
    var onPick: ((Int) -> Void)?

    private let panel: NSPanel
    private let glass: NSVisualEffectView
    private let bar = BarView()
    private static let radius: CGFloat = 12

    private init() {
        panel = NSPanel(contentRect: NSRect(x: 0, y: 0, width: 200, height: 52),
                        styleMask: [.borderless, .nonactivatingPanel], backing: .buffered, defer: true)
        panel.isFloatingPanel = true
        panel.level = .popUpMenu
        panel.hidesOnDeactivate = false
        panel.isOpaque = false
        panel.backgroundColor = .clear
        panel.hasShadow = true
        panel.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .ignoresCycle]
        panel.becomesKeyOnlyIfNeeded = true

        glass = NSVisualEffectView()
        glass.material = .popover
        glass.blendingMode = .behindWindow
        glass.state = .active
        glass.wantsLayer = true
        glass.maskImage = CandidateBar.roundedMask(radius: CandidateBar.radius)
        panel.contentView = glass

        bar.frame = glass.bounds
        bar.autoresizingMask = [.width, .height]
        bar.onPick = { [weak self] i in self?.onPick?(i) }
        glass.addSubview(bar)
    }

    /// A stretchable rounded rectangle, so the blur has the bar's corners
    private static func roundedMask(radius r: CGFloat) -> NSImage {
        let d = r * 2 + 1
        let img = NSImage(size: NSSize(width: d, height: d), flipped: false) { rect in
            NSColor.black.setFill()
            NSBezierPath(roundedRect: rect, xRadius: r, yRadius: r).fill()
            return true
        }
        img.capInsets = NSEdgeInsets(top: r, left: r, bottom: r, right: r)
        img.resizingMode = .stretch
        return img
    }

    /// Show `words` (with `selected` highlighted) under the text at `caret` (screen coordinates,
    /// the line's rectangle as the app reports it).
    func show(_ words: [String], selected: Int, caret: NSRect) {
        bar.words = Array(words.prefix(9))
        bar.selected = selected
        let size = bar.fittingSize
        var frame = NSRect(x: caret.minX - 10, y: caret.minY - size.height - 6, width: size.width, height: size.height)
        let screen = NSScreen.screens.first(where: { $0.frame.intersects(caret) }) ?? NSScreen.main
        if let area = screen?.visibleFrame {
            if frame.minY < area.minY { frame.origin.y = caret.maxY + 6 } // no room below: above the line
            frame.origin.x = min(max(frame.minX, area.minX + 4), area.maxX - frame.width - 4)
            frame.origin.y = min(max(frame.minY, area.minY + 4), area.maxY - frame.height - 4)
        }
        panel.setFrame(frame, display: true)
        bar.needsDisplay = true
        panel.orderFrontRegardless()
    }

    func select(_ i: Int) {
        bar.selected = i
        bar.needsDisplay = true
    }

    func hide() { panel.orderOut(nil) }

    /// `Ekushe --render-bar DIR`: the bar as PNGs over a light and a dark backdrop (for the build's
    /// pictures; the real one is blurred glass over whatever is behind it).
    static func renderPreviews(to dir: String) -> Int32 {
        let words = ["আমার", "আমরা", "আমির", "আমারও", "amar", "আমের"]
        try? FileManager.default.createDirectory(atPath: dir, withIntermediateDirectories: true)
        for (dark, name) in [(false, "light"), (true, "dark")] {
            let view = BarView()
            view.appearance = NSAppearance(named: dark ? .darkAqua : .aqua)
            view.words = words
            view.selected = 0
            let size = view.fittingSize
            let pad: CGFloat = 24
            let host = NSView(frame: NSRect(x: 0, y: 0, width: size.width + pad * 2, height: size.height + pad * 2))
            host.appearance = view.appearance
            host.wantsLayer = true
            host.layer?.backgroundColor = (dark ? NSColor(white: 0.12, alpha: 1) : NSColor(calibratedRed: 0.86, green: 0.89, blue: 0.93, alpha: 1)).cgColor
            let card = NSView(frame: NSRect(x: pad, y: pad, width: size.width, height: size.height))
            card.wantsLayer = true
            card.layer?.cornerRadius = radius
            // a stand-in for the frosted glass
            card.layer?.backgroundColor = (dark ? NSColor(white: 0.22, alpha: 0.9) : NSColor(white: 0.97, alpha: 0.9)).cgColor
            host.addSubview(card)
            view.frame = card.bounds
            card.addSubview(view)
            for scale in [1, 2] {
                guard let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: Int(host.bounds.width) * scale,
                                                 pixelsHigh: Int(host.bounds.height) * scale, bitsPerSample: 8,
                                                 samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
                                                 colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0) else { continue }
                rep.size = host.bounds.size
                host.cacheDisplay(in: host.bounds, to: rep)
                let file = "\(dir)/bar-\(name)@\(scale)x.png"
                try? rep.representation(using: .png, properties: [:])?.write(to: URL(fileURLWithPath: file))
            }
        }
        return 0
    }
}

/// The chips themselves
final class BarView: NSView {
    var words: [String] = []
    var selected = 0
    var onPick: ((Int) -> Void)?

    private let pad: CGFloat = 7
    private let gap: CGFloat = 6
    private let chipH: CGFloat = 38
    private let wordFont = NSFont.systemFont(ofSize: 17)
    private let tagFont = NSFont.systemFont(ofSize: 10, weight: .semibold)

    override var isFlipped: Bool { true }
    override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }

    private var dark: Bool { effectiveAppearance.bestMatch(from: [.aqua, .darkAqua]) == .darkAqua }

    private func isEnglish(_ s: String) -> Bool { !s.isEmpty && s.unicodeScalars.allSatisfy { $0.isASCII } }

    private func chipWidths() -> [CGFloat] {
        words.map { w in
            let ww = (w as NSString).size(withAttributes: [.font: wordFont]).width
            let tag = isEnglish(w) ? ("EN" as NSString).size(withAttributes: [.font: tagFont]).width + 6 : 0
            return max(46, ceil(ww + tag + 28))
        }
    }

    override var fittingSize: NSSize {
        let w = chipWidths().reduce(0, +) + gap * CGFloat(max(0, words.count - 1)) + pad * 2
        return NSSize(width: max(120, w), height: chipH + pad * 2)
    }

    private func chipRects() -> [NSRect] {
        var x = pad
        return chipWidths().map { w in
            defer { x += w + gap }
            return NSRect(x: x, y: pad, width: w, height: chipH)
        }
    }

    override func draw(_ dirtyRect: NSRect) {
        // A light tint over the glass: the soft grey of the phone keyboards' bar
        (dark ? NSColor(srgbRed: 32 / 255, green: 33 / 255, blue: 36 / 255, alpha: 0.45)
              : NSColor(srgbRed: 232 / 255, green: 234 / 255, blue: 239 / 255, alpha: 0.55)).setFill()
        NSBezierPath(roundedRect: bounds, xRadius: 12, yRadius: 12).fill()

        let accent = dark ? NSColor(srgbRed: 138 / 255, green: 180 / 255, blue: 248 / 255, alpha: 1)
                          : NSColor(srgbRed: 60 / 255, green: 123 / 255, blue: 240 / 255, alpha: 1)
        let onAccent = dark ? NSColor(srgbRed: 32 / 255, green: 33 / 255, blue: 36 / 255, alpha: 1) : .white
        let chip = dark ? NSColor(white: 1, alpha: 0.09) : NSColor(white: 1, alpha: 0.78)
        let ink = dark ? NSColor(srgbRed: 232 / 255, green: 234 / 255, blue: 237 / 255, alpha: 1)
                       : NSColor(srgbRed: 31 / 255, green: 31 / 255, blue: 31 / 255, alpha: 1)
        let muted = dark ? NSColor(srgbRed: 154 / 255, green: 160 / 255, blue: 166 / 255, alpha: 1)
                         : NSColor(srgbRed: 95 / 255, green: 99 / 255, blue: 104 / 255, alpha: 1)

        for (i, (word, r)) in zip(words, chipRects()).enumerated() {
            let sel = i == selected
            (sel ? accent : chip).setFill()
            NSBezierPath(roundedRect: r, xRadius: 10, yRadius: 10).fill()
            let wa: [NSAttributedString.Key: Any] = [.font: wordFont, .foregroundColor: sel ? onAccent : ink]
            let ws = (word as NSString).size(withAttributes: wa)
            let en = isEnglish(word)
            let ta: [NSAttributedString.Key: Any] = [.font: tagFont, .foregroundColor: (sel ? onAccent : muted).withAlphaComponent(0.8)]
            let tw = en ? ("EN" as NSString).size(withAttributes: ta).width + 6 : 0
            var x = r.midX - (ws.width + tw) / 2
            (word as NSString).draw(at: NSPoint(x: x, y: r.midY - ws.height / 2), withAttributes: wa)
            if en {
                x += ws.width + 6
                let ts = ("EN" as NSString).size(withAttributes: ta)
                ("EN" as NSString).draw(at: NSPoint(x: x, y: r.midY - ts.height / 2 + 1), withAttributes: ta)
            }
        }
    }

    override func mouseDown(with event: NSEvent) {
        let p = convert(event.locationInWindow, from: nil)
        if let i = chipRects().firstIndex(where: { $0.contains(p) }) { onPick?(i) }
    }
}
