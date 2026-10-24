import SwiftUI
import UIKit

/// The Ekushe Next app: how to turn the keyboard on, a box to try it, and help.
@main
struct EkusheApp: App {
    var body: some Scene { WindowGroup { SetupView() } }
}

private let green = Color(red: 0.04, green: 0.54, blue: 0.36)

struct SetupView: View {
    @State private var tryText = ""
    @Environment(\.scenePhase) private var phase
    @State private var enabled = false

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
                    Text("টিপ: অক্ষরে চেপে ধরলে বড় হাতের অক্ষর ও চিহ্ন পাবেন (t → ট, ৎ)। শব্দের পরেই ⌫ চাপলে রোমান অক্ষর ফিরে আসে। Space টেনে কার্সর সরান। ⚙︎ চাপলে কিবোর্ডের সেটিংস।").font(.caption).foregroundColor(.secondary)
                }
                card {
                    Text("🔒 আপনার লেখা, শুধু আপনার").font(.headline)
                    Text("একুশে নেক্সট “Full Access” চায় না, তাই iOS-ই নিশ্চিত করে যে কিবোর্ডটি ইন্টারনেটে কিছু পাঠাতে পারে না। অভিধান, পরামর্শ, শেখা শব্দ সব ফোনেই থাকে। পাসওয়ার্ডের ঘরে iOS নিজের কিবোর্ড দেখায়।").font(.subheadline).foregroundColor(.secondary)
                    Text("Ekushe Next doesn't ask for Full Access, so iOS itself keeps it from sending anything anywhere.").font(.footnote).foregroundColor(.secondary)
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
