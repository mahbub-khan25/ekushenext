import SwiftUI
import UIKit

/// The Ekushe Next app: how to turn the keyboard on, the writing pad (any keyboard, an iPad's
/// physical one included), the marks, and help.
@main
struct EkusheApp: App {
    init() {
        // The pad types with the keyboard's own engine and dictionary, read from inside the keyboard
        // (the app ships it once, in the keyboard)
        let b = Self.keyboardBundle
        Engine.start(bundle: b)
        English.shared.start(bundle: b)
    }

    static var keyboardBundle: Bundle {
        if let u = Bundle.main.builtInPlugInsURL?.appendingPathComponent("EkusheKeyboard.appex"), let b = Bundle(url: u) { return b }
        return .main
    }

    var body: some Scene { WindowGroup { RootView() } }
}

let green = Color(red: 0.04, green: 0.54, blue: 0.36)

/// Three parts: getting started, the writing pad and the marks
struct RootView: View {
    enum Tab: Hashable { case start, pad, marks }
    @State private var tab: Tab = .start

    var body: some View {
        TabView(selection: $tab) {
            NavigationStack { SetupView(openPad: { tab = .pad }).navigationTitle("একুশে নেক্সট").navigationBarTitleDisplayMode(.inline) }
                .tabItem { Label("শুরু", systemImage: "keyboard") }.tag(Tab.start)
            PadScreen(onHelp: { tab = .marks })
                .tabItem { Label("লেখার প্যাড", systemImage: "square.and.pencil") }.tag(Tab.pad)
            NavigationStack { MarksView().navigationTitle("চিহ্নের নিয়ম").navigationBarTitleDisplayMode(.inline) }
                .tabItem { Label("চিহ্ন", systemImage: "character.book.closed") }.tag(Tab.marks)
        }
        .tint(green)
    }
}

/// The marks, the same list as in the keyboard (its ? button) and the guide
struct MarksView: View {
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 18) {
                section("যুক্তবর্ণ, হসন্ত, না আলাদা?", "Conjunct, hasanta or apart: what goes between two consonants", Marks.join)
                section("অন্য চিহ্ন", "The other marks", Marks.others)
                VStack(alignment: .leading, spacing: 6) {
                    Text(Marks.whereBn).font(.subheadline)
                    Text(Marks.whereEn).font(.footnote).foregroundColor(.secondary)
                }
                .padding(18).frame(maxWidth: .infinity, alignment: .leading)
                .background(RoundedRectangle(cornerRadius: 20).fill(Color(.secondarySystemBackground)))
            }
            .padding(18)
            .frame(maxWidth: 720)
            .frame(maxWidth: .infinity)
        }
    }

    private func section(_ title: String, _ sub: String, _ rows: [Marks.Row]) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(title).font(.title3.bold())
            Text(sub).font(.footnote).foregroundColor(.secondary)
            ForEach(rows, id: \.keys) { r in
                HStack(alignment: .firstTextBaseline, spacing: 12) {
                    Text(r.keys).font(.system(.body, design: .monospaced).weight(.medium)).foregroundColor(green)
                        .frame(minWidth: 96, alignment: .leading)
                    Text(r.bangla).font(.title2.weight(.semibold)).frame(minWidth: 84, alignment: .leading)
                    VStack(alignment: .leading, spacing: 1) {
                        Text(r.what).font(.subheadline)
                        Text(r.whatEn).font(.caption).foregroundColor(.secondary)
                    }
                    Spacer(minLength: 0)
                }
                .accessibilityElement(children: .ignore)
                .accessibilityLabel("\(r.keys.map { String($0) }.joined(separator: " ")) gives \(r.bangla). \(r.whatEn)")
                if r.keys != rows.last?.keys { Divider() }
            }
        }
        .padding(18).frame(maxWidth: .infinity, alignment: .leading)
        .background(RoundedRectangle(cornerRadius: 20).fill(Color(.secondarySystemBackground)))
    }
}

struct SetupView: View {
    var openPad: () -> Void
    @State private var tryText = ""
    @Environment(\.scenePhase) private var phase
    @State private var enabled = false

    init(openPad: @escaping () -> Void = {}) { self.openPad = openPad }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 22) {
                HStack(spacing: 14) {
                    Image("Logo").resizable().frame(width: 64, height: 64).clipShape(RoundedRectangle(cornerRadius: 15))
                    VStack(alignment: .leading, spacing: 2) {
                        Text("একুশে নেক্সট").font(.system(size: 30, weight: .bold))
                        Text("যেভাবে বলেন, সেভাবে লিখুন · Ekushe Next").font(.subheadline).foregroundColor(.secondary)
                    }
                }
                card {
                    Text(enabled ? "সব প্রস্তুত! ✓" : "শুরু করুন").font(.title3.bold()).foregroundColor(enabled ? green : .primary)
                    Text(enabled ? "Ekushe Next is on. Try it below." : "Two quick steps to start typing Bangla").font(.footnote).foregroundColor(.secondary)
                    step("১", "কিবোর্ডটি চালু করুন", "Settings → Keyboards → turn on Ekushe Next. It doesn't need “Allow Full Access”.", done: enabled) {
                        Button("সেটিংস খুলুন") { if let u = URL(string: UIApplication.openSettingsURLString) { UIApplication.shared.open(u) } }
                            .buttonStyle(.borderedProminent).tint(green)
                    }
                    step("২", "একুশে নেক্সট বেছে নিন", "In any app, touch and hold 🌐 (or tap it) on the keyboard and choose Ekushe Next.", done: false) { EmptyView() }
                    Text("৩  লিখে দেখুন").font(.headline).padding(.top, 4)
                    Text("ইংরেজি অক্ষরে উচ্চারণ অনুযায়ী লিখুন, যেমন amar sonar bangla, তারপর Space চাপুন।").font(.subheadline).foregroundColor(.secondary)
                    TextField("এখানে লিখুন… amar sonar bangla", text: $tryText, axis: .vertical)
                        .lineLimit(3...6).padding(12)
                        .overlay(RoundedRectangle(cornerRadius: 12).stroke(Color.secondary.opacity(0.4)))
                    Text("টিপ: অক্ষরে চেপে ধরলে বড় হাতের অক্ষর ও চিহ্ন পাবেন (t → ট, ৎ)। শব্দের পরেই ⌫ চাপলে শব্দটি পরামর্শসহ আবার খোলে। Space টেনে কার্সর সরান। কিবোর্ডের ⚙︎ সেটিংস, ? চিহ্নের নিয়ম।").font(.caption).foregroundColor(.secondary)
                }
                card {
                    Text("⌨️  আলাদা কিবোর্ড? লেখার প্যাড").font(.headline)
                    Text("আইপ্যাডের কিবোর্ড কভার বা ব্লুটুথ কিবোর্ডের কী অ্যাপল অন্য কোনো কিবোর্ডকে পড়তে দেয় না, তাই সেখানে ফোনেটিক বাংলা লিখুন একুশের লেখার প্যাডে: একই পরামর্শ, Tab দিয়ে বাছাই, Shift দুবার বাংলা ⇄ English। লেখা শেষে কপি বা শেয়ার করুন।").font(.subheadline).foregroundColor(.secondary)
                    Text("Apple doesn't let other keyboards read a physical keyboard, so type phonetic Bangla on one in Ekushe's writing pad, then copy or share it.").font(.footnote).foregroundColor(.secondary)
                    Button("লেখার প্যাড খুলুন · Open the pad", action: openPad).buttonStyle(.borderedProminent).tint(green)
                }
                card {
                    Text("🔒 আপনার লেখা, শুধু আপনার").font(.headline)
                    Text("একুশে নেক্সট “Full Access” চায় না, তাই iOS-ই নিশ্চিত করে যে কিবোর্ডটি ইন্টারনেটে কিছু পাঠাতে পারে না। অভিধান, পরামর্শ, শেখা শব্দ সব ফোনেই থাকে। পাসওয়ার্ডের ঘরে iOS নিজের কিবোর্ড দেখায়।").font(.subheadline).foregroundColor(.secondary)
                    Text("Ekushe Next doesn't ask for Full Access, so iOS itself keeps it from sending anything anywhere. That's also why its settings live inside the keyboard (⚙︎): without Full Access the keyboard and this app can't share them.").font(.footnote).foregroundColor(.secondary)
                }
                card {
                    link("📖  ব্যবহার নির্দেশিকা", "The full guide", "https://ekushenext.com/guide/")
                    Divider()
                    link("🐞  সমস্যা জানান", "Report a problem (no account needed)", reportURL)
                    Divider()
                    link("🌐  ekushenext.com", "support@ekushenext.com", "https://ekushenext.com/")
                    Text("সংস্করণ \(version) · ফ্রি ও ওপেন সোর্স · কোড MPL-2.0, অভিধান CC BY-SA 4.0").font(.caption).foregroundColor(.secondary).padding(.top, 6)
                }
            }
            .padding(18)
            .frame(maxWidth: 720)
            .frame(maxWidth: .infinity)
        }
        .onAppear(perform: check)
        .onChange(of: phase) { _ in check() }
    }

    private var version: String { Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "" }
    private var reportURL: String {
        let os = "iOS \(UIDevice.current.systemVersion) (\(UIDevice.current.model))"
        var c = URLComponents(string: "https://ekushenext.com/report/")!
        c.queryItems = [URLQueryItem(name: "v", value: version), URLQueryItem(name: "os", value: os), URLQueryItem(name: "from", value: "app")]
        return c.url?.absoluteString ?? "https://ekushenext.com/report/"
    }

    /// Is the keyboard on? iOS lists the keyboards that are on in the preferences
    private func check() {
        let ids = UserDefaults.standard.object(forKey: "AppleKeyboards") as? [String] ?? []
        enabled = ids.contains { $0.hasPrefix((Bundle.main.bundleIdentifier ?? "") + ".") }
    }

    @ViewBuilder private func card<C: View>(@ViewBuilder _ content: () -> C) -> some View {
        VStack(alignment: .leading, spacing: 10, content: content)
            .padding(18).frame(maxWidth: .infinity, alignment: .leading)
            .background(RoundedRectangle(cornerRadius: 20).fill(Color(.secondarySystemBackground)))
    }

    @ViewBuilder private func step<A: View>(_ n: String, _ title: String, _ sub: String, done: Bool, @ViewBuilder action: () -> A) -> some View {
        HStack(alignment: .center, spacing: 12) {
            Text(done ? "✓" : n).font(.headline).frame(width: 34, height: 34)
                .foregroundColor(done ? .white : green)
                .background(Circle().fill(done ? green : Color.clear)).overlay(Circle().stroke(green, lineWidth: 2))
            VStack(alignment: .leading, spacing: 2) {
                Text(title).font(.headline)
                Text(sub).font(.footnote).foregroundColor(.secondary)
            }
            Spacer(minLength: 4)
            if !done { action() }
        }
    }

    private func link(_ title: String, _ sub: String, _ url: String) -> some View {
        Button { if let u = URL(string: url) { UIApplication.shared.open(u) } } label: {
            VStack(alignment: .leading, spacing: 2) {
                Text(title).font(.headline).foregroundColor(.primary)
                Text(sub).font(.footnote).foregroundColor(.secondary)
            }.frame(maxWidth: .infinity, alignment: .leading)
        }
    }
}
