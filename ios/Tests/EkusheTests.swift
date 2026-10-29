import XCTest
import UIKit

/// A pretend text box: what an app's text field does with a keyboard's edits.
final class FakeDoc: TextTarget {
    var text = ""
    /// The underlined word being typed, at the end of `text`
    var marked = 0

    var contextBefore: String { text }
    func insert(_ s: String) { if marked > 0 { text.removeLast(marked); marked = 0 }; text += s }
    func deleteBackward() {
        if marked > 0 { text.removeLast(marked); marked = 0; return }
        if !text.isEmpty { text.removeLast() }
    }
    func setMarked(_ s: String) { if marked > 0 { text.removeLast(marked) }; text += s; marked = s.count }
    func unmark() { marked = 0 }
    func moveCursor(_ by: Int) {}
}

final class EkusheTests: XCTestCase {
    static let bundle = Bundle(for: EkusheTests.self)
    var doc: FakeDoc!
    var kb: KeyboardView!
    var typist: Typist!

    override class func setUp() {
        super.setUp()
        UserDefaults.standard.set(true, forKey: "bangla")
        Engine.startAndWait(bundle: bundle)
        English.shared.start(bundle: bundle, sync: true)
    }

    override func setUp() {
        doc = FakeDoc()
        kb = KeyboardView(frame: CGRect(x: 0, y: 0, width: 390, height: 300))
        kb.frame.size.height = kb.preferredHeight
        kb.layoutIfNeeded()
        typist = Typist(target: doc, view: kb)
        kb.listener = typist
        typist.bangla = true
        typist.begin()
    }

    private func type(_ s: String) {
        for c in s {
            if c == " " { kb.tap("SPACE") } else { typist.onText(String(c)) }
        }
    }

    func testTheDictionaryIsMappedFromTheBundle() {
        XCTAssertGreaterThan(Engine.words, 100_000)
        XCTAssertTrue(English.shared.ready)
    }

    func testPhoneticBangla() {
        type("amar sonar bangla ")
        XCTAssertEqual(doc.text, "আমার সোনার বাংলা ")
        type("office ")
        XCTAssertEqual(doc.text, "আমার সোনার বাংলা অফিস ")
    }

    func testBackspaceRightAfterAWordReopensItsLetters() {
        type("amar ")
        typist.onDelete()
        XCTAssertEqual(doc.text, "আমার", "the word is open again (underlined), without the space")
        XCTAssertEqual(kb.candidates.first?.text, "আমার")
        type(" ")
        XCTAssertEqual(doc.text, "আমার ")
    }

    func testShortcut() {
        type(";brb ")
        XCTAssertEqual(doc.text, "এখুনি আসছি ")
    }

    private func english() { typist.onLanguageToggle(); XCTAssertFalse(typist.bangla) }

    func testEnglishTypoFixedBySpaceAndTakenBack() {
        english()
        type("teh ")
        XCTAssertEqual(doc.text, "the ")
        typist.onDelete()
        XCTAssertEqual(doc.text, "teh")
        type(" ")
        XCTAssertEqual(doc.text, "teh ", "a word taken back stays as typed")
        type("dont i ")
        XCTAssertTrue(doc.text.hasSuffix("don't I "), doc.text)
    }

    func testEnglishLeavesAddressesBanglishAndNamesAlone() {
        english()
        type("makhan.aumi")
        typist.onText("@")
        type("gmail.com ")
        XCTAssertEqual(doc.text, "makhan.aumi@gmail.com ")
        type("ami tomake bhalobashi ")
        XCTAssertTrue(doc.text.hasSuffix("ami tomake bhalobashi "), doc.text)
        type("hello ")
        typist.onText("R"); type("ahim ")
        XCTAssertTrue(doc.text.hasSuffix("hello Rahim "), doc.text)
        type("www.ekushenext.com ")
        XCTAssertTrue(doc.text.hasSuffix("www.ekushenext.com "), doc.text)
    }

    func testEnglishCompletionsAndNextWords() {
        english()
        type("beau")
        let shown = kb.candidates.map { $0.text }
        XCTAssertTrue(shown.contains("beautiful"), "\(shown)")
        for _ in 0..<4 { typist.onDelete() }
        type("thank ")
        XCTAssertEqual(kb.predictions.first, "you")
    }

    // ── iPad, landscape and split: every key on the keyboard, none on top of another

    private func board(width: CGFloat, pad: Bool, landscape: Bool, split: Bool = false, mode: Mode = .letters) -> KeyboardView {
        UserDefaults.standard.set(split, forKey: "split_pad")
        let v = KeyboardView(frame: CGRect(x: 0, y: 0, width: width, height: 300))
        v.pad = pad; v.landscape = landscape
        v.refreshTheme(dark: false)
        v.mode = mode
        v.frame.size.height = v.preferredHeight
        v.layoutIfNeeded()
        return v
    }

    private func checkGeometry(_ v: KeyboardView, _ what: String) {
        let rects = v.keyRects
        XCTAssertGreaterThan(rects.count, 25, what)
        for (id, r) in rects {
            XCTAssertTrue(v.bounds.insetBy(dx: -0.5, dy: -0.5).contains(r), "\(what): \(id) at \(r) is off the keyboard \(v.bounds)")
            XCTAssertGreaterThan(r.width, 20, "\(what): \(id) too narrow")
        }
        for i in 0..<rects.count { for j in (i + 1)..<rects.count where rects[i].1.insetBy(dx: 1, dy: 1).intersects(rects[j].1.insetBy(dx: 1, dy: 1)) {
            XCTFail("\(what): \(rects[i].0) and \(rects[j].0) overlap")
        } }
    }

    func testIPadLayouts() {
        for (w, land) in [(CGFloat(820), false), (1180, true), (1024, false), (1366, true), (744, false)] {
            for m in [Mode.letters, .symbols, .moreSymbols] { checkGeometry(board(width: w, pad: true, landscape: land, mode: m), "iPad \(w) \(m)") }
        }
        let v = board(width: 1180, pad: true, landscape: true)
        XCTAssertEqual(v.formFactor, .pad)
        let ids = v.keyIds
        XCTAssertTrue(ids.contains("HIDE") && ids.contains("ENTER") && ids.filter { $0 == "SHIFT" }.count == 2, "\(ids)")
        XCTAssertGreaterThan(v.rowH, 60, "iPad keys are taller than a phone's")
        UserDefaults.standard.set(false, forKey: "split_pad")
    }

    func testIPadSplitKeepsTheMiddleClear() {
        for w in [CGFloat(1180), 1366, 1024] {
            for m in [Mode.letters, .symbols, .moreSymbols] {
                let v = board(width: w, pad: true, landscape: true, split: true, mode: m)
                XCTAssertEqual(v.formFactor, .padSplit)
                checkGeometry(v, "split \(w) \(m)")
                let middle = CGRect(x: w * 0.42, y: 0, width: w * 0.16, height: v.bounds.height)
                for (id, r) in v.keyRects { XCTAssertFalse(r.intersects(middle), "split \(w): \(id) in the middle") }
            }
        }
        // upright, the split setting doesn't apply
        XCTAssertEqual(board(width: 820, pad: true, landscape: false, split: true).formFactor, .pad)
        UserDefaults.standard.set(false, forKey: "split_pad")
    }

    func testPhoneLandscapeIsLowerThanUpright() {
        let up = board(width: 390, pad: false, landscape: false), side = board(width: 844, pad: false, landscape: true)
        XCTAssertLessThan(side.preferredHeight, up.preferredHeight - 40)
        checkGeometry(side, "iPhone landscape")
    }

    func testVoiceOverNamesEveryKey() {
        let v = board(width: 390, pad: false, landscape: false)
        _ = picture(v)   // the elements are made when the keyboard is drawn
        let labels = (v.accessibilityElements as? [UIAccessibilityElement] ?? []).compactMap { $0.accessibilityLabel }
        XCTAssertTrue(labels.contains("Delete") && labels.contains("Return") && labels.contains("Shift"), "\(labels)")
        XCTAssertTrue(labels.contains("a, আ"), "a letter says its Bangla sound: \(labels)")
        XCTAssertGreaterThanOrEqual(labels.count, v.keyIds.count)
    }

    // ── the writing pad: letters from a physical (or Apple's) keyboard go through Ekushe

    private func padType(_ router: PadRouter, _ s: String) { for c in s { _ = router.shouldChange(String(c), rangeLength: 0, translate: true) } }

    func testWritingPadTypesBanglaFromAnyKeyboard() {
        let router = PadRouter(typist: typist)
        padType(router, "ami bhalo achi ")
        XCTAssertEqual(doc.text, "আমি ভালো আছি ")
        // Tab picks the second suggestion: aste → আসতে first, আস্তে second
        padType(router, "aste")
        XCTAssertTrue(router.tab(back: false))
        padType(router, " ")
        XCTAssertTrue(doc.text.hasSuffix("আস্তে "), doc.text)
        // + makes the conjunct straight away
        padType(router, "as+te ")
        XCTAssertTrue(doc.text.hasSuffix("আস্তে আস্তে "), doc.text)
        // Backspace right after a word reopens it
        _ = router.shouldChange("", rangeLength: 1, translate: true)
        XCTAssertTrue(doc.text.hasSuffix("আস্তে"), doc.text)
        XCTAssertTrue(typist.composing)
        XCTAssertTrue(router.escape())
        XCTAssertFalse(typist.composing)
    }

    func testWritingPadShiftTwiceSwitchesLanguageAndBanglaKeyboardsPassThrough() {
        let router = PadRouter(typist: typist)
        let t0 = Date()
        router.shiftTapped(at: t0); router.shiftTapped(at: t0.addingTimeInterval(0.2))
        XCTAssertFalse(typist.bangla)
        padType(router, "hello ")
        XCTAssertEqual(doc.text, "hello ")
        router.shiftTapped(at: t0.addingTimeInterval(5)); router.otherKey(); router.shiftTapped(at: t0.addingTimeInterval(5.1))
        XCTAssertFalse(typist.bangla, "a key between the two Shifts: no switch")
        typist.onLanguageToggle()
        // Ekushe's own keyboard already writes Bangla: the pad takes it as it comes
        XCTAssertTrue(router.shouldChange("আ", rangeLength: 0, translate: false))
        XCTAssertTrue(router.shouldChange("a", rangeLength: 0, translate: false))
        // pasted text goes in as it is
        XCTAssertTrue(router.shouldChange("pasted text", rangeLength: 0, translate: true))
    }

    // ── pictures of the keyboard, for a look at every theme (saved for the CI run)

    private func save(_ img: UIImage, _ name: String) {
        let a = XCTAttachment(image: img); a.name = name; a.lifetime = .keepAlways; add(a)
        if let dir = ProcessInfo.processInfo.environment["SHOTS_DIR"], let png = img.pngData() {
            try? FileManager.default.createDirectory(atPath: dir, withIntermediateDirectories: true)
            try? png.write(to: URL(fileURLWithPath: dir).appendingPathComponent("\(name).png"))
        }
    }

    private func picture(_ v: KeyboardView) -> UIImage {
        UIGraphicsImageRenderer(bounds: v.bounds, format: { let f = UIGraphicsImageRendererFormat(); f.scale = 2; return f }()).image { _ in v.draw(v.bounds) }
    }

    func testDeviceSnapshots() {
        UserDefaults.standard.set("default", forKey: "theme")
        UserDefaults.standard.set(true, forKey: "bangla")
        let shots: [(String, CGFloat, Bool, Bool, Bool, Mode)] = [
            ("20-iphone-landscape", 844, false, true, false, .letters),
            ("21-ipad-portrait", 820, true, false, false, .letters),
            ("22-ipad-landscape", 1180, true, true, false, .letters),
            ("23-ipad-split", 1180, true, true, true, .letters),
            ("24-ipad-symbols", 1180, true, true, false, .symbols),
            ("25-ipad-split-symbols", 1180, true, true, true, .symbols),
            ("26-ipad-pro-13-portrait", 1032, true, false, false, .letters),
            ("27-ipad-mini-portrait", 744, true, false, false, .letters),
        ]
        let doc = FakeDoc()   // the typist holds its text box weakly
        for (name, w, pad, land, split, m) in shots {
            let v = board(width: w, pad: pad, landscape: land, split: split, mode: m)
            let t = Typist(target: doc, view: v); v.listener = t; t.begin()
            save(picture(v), name)
        }
        // typing on an iPad, and the pad's strip
        let v = board(width: 1180, pad: true, landscape: true)
        let t = Typist(target: doc, view: v); v.listener = t; t.begin()
        for c in "aste" { t.onText(String(c)) }
        XCTAssertTrue(t.moveHighlight())
        XCTAssertEqual(v.selected, 1)
        save(picture(v), "28-ipad-typing-tab")
        let strip = KeyboardView(frame: CGRect(x: 0, y: 0, width: 1180, height: 60))
        strip.pad = true; strip.stripOnly = true; strip.toolIds = []
        strip.refreshTheme(dark: true); strip.frame.size.height = strip.preferredHeight; strip.layoutIfNeeded()
        let doc2 = FakeDoc()
        let st = Typist(target: doc2, view: strip); strip.listener = st; st.begin()
        save(picture(strip), "29-pad-strip-idle")
        for c in "amar" { st.onText(String(c)) }
        save(picture(strip), "30-pad-strip-typing")
        UserDefaults.standard.set(false, forKey: "split_pad")
    }

    private func snapshot(_ name: String, theme: String, dark: Bool = false, setup: ((KeyboardView, Typist) -> Void)? = nil) {
        UserDefaults.standard.set(theme, forKey: "theme")
        UserDefaults.standard.set(true, forKey: "bangla")
        let v = KeyboardView(frame: CGRect(x: 0, y: 0, width: 390, height: 300))
        v.frame.size.height = v.preferredHeight
        v.refreshTheme(dark: dark)
        v.layoutIfNeeded()
        let d = FakeDoc()
        let t = Typist(target: d, view: v)
        v.listener = t
        t.begin()
        setup?(v, t)
        let img = UIGraphicsImageRenderer(bounds: v.bounds, format: { let f = UIGraphicsImageRendererFormat(); f.scale = 3; return f }()).image { _ in v.draw(v.bounds) }
        let a = XCTAttachment(image: img); a.name = name; a.lifetime = .keepAlways; add(a)
        if let dir = ProcessInfo.processInfo.environment["SHOTS_DIR"], let png = img.pngData() {
            try? FileManager.default.createDirectory(atPath: dir, withIntermediateDirectories: true)
            try? png.write(to: URL(fileURLWithPath: dir).appendingPathComponent("\(name).png"))
        }
    }

    func testSnapshots() {
        snapshot("01-ekushe", theme: "ekushe")
        snapshot("02-ekushe-typing", theme: "ekushe") { _, t in for c in "amar" { t.onText(String(c)) } }
        for (i, th) in ["default", "smooth", "krishnachura", "padma", "shapla", "kashful", "night", "system"].enumerated() {
            snapshot(String(format: "%02d-%@", i + 3, th), theme: th)
        }
        snapshot("11-default-dark", theme: "default", dark: true)
        snapshot("12-smooth-dark", theme: "smooth", dark: true)
        snapshot("13-symbols", theme: "ekushe") { v, _ in v.mode = .symbols }
        snapshot("14-numbers", theme: "ekushe") { v, _ in v.mode = .numbers }
        snapshot("15-emoji", theme: "ekushe") { v, _ in v.openEmoji() }
        snapshot("16-english", theme: "ekushe") { _, t in t.onLanguageToggle(); for c in "beau" { t.onText(String(c)) } }
        UserDefaults.standard.set("default", forKey: "theme")
        UserDefaults.standard.set(true, forKey: "bangla")
    }
}
