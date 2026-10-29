import XCTest

/// The app itself on a simulator: every screen pictured, and the writing pad typing Bangla from the
/// simulator's own keyboard (letters sent by a keyboard, as a physical one sends them).
final class EkusheUITests: XCTestCase {
    private var app: XCUIApplication!
    private var device: String { UIDevice.current.userInterfaceIdiom == .pad ? "ipad" : "iphone" }

    override func setUp() {
        continueAfterFailure = false
        app = XCUIApplication()
        app.launchArguments += ["-pad_text", ""]
        app.launch()
    }

    private func shot(_ name: String) {
        let img = XCUIScreen.main.screenshot()
        let a = XCTAttachment(screenshot: img); a.name = name; a.lifetime = .keepAlways; add(a)
        if let dir = ProcessInfo.processInfo.environment["SHOTS_DIR"] {
            try? FileManager.default.createDirectory(atPath: dir, withIntermediateDirectories: true)
            try? img.pngRepresentation.write(to: URL(fileURLWithPath: dir).appendingPathComponent("app-\(device)-\(name).png"))
        }
    }

    func testScreensAndThePad() {
        XCTAssertTrue(app.staticTexts["একুশে নেক্সট"].firstMatch.waitForExistence(timeout: 10))
        shot("1-start")
        app.buttons["চিহ্ন"].firstMatch.tap()
        XCTAssertTrue(app.staticTexts["যুক্তবর্ণ, হসন্ত, না আলাদা?"].firstMatch.waitForExistence(timeout: 5))
        shot("3-marks")
        app.buttons["লেখার প্যাড"].firstMatch.tap()
        let pad = app.textViews.firstMatch
        XCTAssertTrue(pad.waitForExistence(timeout: 5))
        pad.tap()
        sleep(1)
        pad.typeText("ami bhalo achi ")
        sleep(1)
        XCTAssertEqual(pad.value as? String, "আমি ভালো আছি ", "letters from a keyboard come out in Bangla")
        pad.typeText("as+te ")
        XCTAssertEqual(pad.value as? String, "আমি ভালো আছি আস্তে ")
        pad.typeText("amar")
        sleep(1)
        shot("2-pad-typing")
    }

    func testLandscape() {
        XCUIDevice.shared.orientation = .landscapeLeft
        sleep(1)
        shot("4-start-landscape")
        app.buttons["লেখার প্যাড"].firstMatch.tap()
        let pad = app.textViews.firstMatch
        XCTAssertTrue(pad.waitForExistence(timeout: 5))
        pad.tap(); sleep(1)
        pad.typeText("ekushe next ")
        XCTAssertEqual(pad.value as? String, "একুশে নেক্সট ")
        shot("5-pad-landscape")
        XCUIDevice.shared.orientation = .portrait
    }
}
