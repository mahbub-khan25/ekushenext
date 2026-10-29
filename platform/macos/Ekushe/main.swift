import Carbon
import Cocoa
import InputMethodKit

/// Objects the controllers share.
enum AppShared {
    static var server: IMKServer?
    static var candidates: IMKCandidates?
}

let bundleID = Bundle.main.bundleIdentifier ?? "org.ekushe.inputmethod.Ekushe"
let modeID = bundleID + ".Bangla"

/// `Ekushe --register`: add Ekushe to the user's input sources and turn it on (used by the
/// installer, run as the signed-in user).
func registerInputSource() -> Int32 {
    let url = Bundle.main.bundleURL as CFURL
    let status = TISRegisterInputSource(url)
    if status != noErr && status != OSStatus(paramErr) {
        FileHandle.standardError.write("TISRegisterInputSource failed: \(status)\n".data(using: .utf8)!)
    }
    let filter = [kTISPropertyInputSourceID as String: modeID] as CFDictionary
    if let list = TISCreateInputSourceList(filter, true)?.takeRetainedValue() as? [TISInputSource] {
        for source in list { TISEnableInputSource(source) }
        return list.isEmpty ? 1 : 0
    }
    return 1
}

if CommandLine.arguments.contains("--register") {
    exit(registerInputSource())
}

let connection = Bundle.main.infoDictionary?["InputMethodConnectionName"] as? String ?? "Ekushe_1_Connection"
guard let server = IMKServer(name: connection, bundleIdentifier: bundleID) else {
    FileHandle.standardError.write("Ekushe: could not start the input method server\n".data(using: .utf8)!)
    exit(1)
}
AppShared.server = server
AppShared.candidates = IMKCandidates(server: server, panelType: kIMKSingleRowSteppingCandidatePanel)
Engine.start()
NSApplication.shared.run()
