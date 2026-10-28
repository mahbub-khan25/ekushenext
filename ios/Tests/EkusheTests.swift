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

    // ── pictures of the keyboard, for a look at every theme (saved for the CI run)

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
