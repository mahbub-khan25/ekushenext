import Foundation

/// The exact-spelling marks, as in the Android app and the guide; every example is what the engine
/// gives. The three people mix up (+ ,, `) come first.
enum Marks {
    struct Row { let keys: String; let bangla: String; let what: String; let whatEn: String }

    static let join: [Row] = [
        Row(keys: "as+te", bangla: "আস্তে", what: "+ যুক্তবর্ণ বানায় (স্ত)", whatEn: "+ joins into a conjunct"),
        Row(keys: "as,,te", bangla: "আস্‌তে", what: ",, হসন্ত দেখায়, যুক্ত করে না", whatEn: ",, shows a hasanta, no joining"),
        Row(keys: "as`te", bangla: "আসতে", what: "` অক্ষর আলাদা রাখে", whatEn: "` keeps the letters apart"),
    ]

    static let others: [Row] = [
        Row(keys: "k+t", bangla: "ক্ত", what: "+ যেকোনো দুটি অক্ষর জোড়ে", whatEn: "+ joins any two letters"),
        Row(keys: "allah,,", bangla: "আল্লাহ্", what: ",, শব্দের শেষে হসন্ত", whatEn: ",, a hasanta at the end"),
        Row(keys: "cha^d", bangla: "চাঁদ", what: "^ চন্দ্রবিন্দু", whatEn: "^ chandrabindu"),
        Row(keys: "dhorrmo", bangla: "ধর্ম", what: "rr রেফ", whatEn: "rr reph"),
        Row(keys: "krrishi", bangla: "কৃষি", what: "rri ঋ-কার", whatEn: "rri the ঋ sign"),
        Row(keys: "hothat``", bangla: "হঠাৎ", what: "t`` খণ্ড-ত", whatEn: "t`` khanda-ta"),
        Row(keys: "du:kho", bangla: "দুঃখ", what: ": বিসর্গ", whatEn: ": bisarga"),
        Row(keys: "bakZo", bangla: "বাক্য", what: "Z য-ফলা", whatEn: "Z ya-phala"),
        Row(keys: "/office", bangla: "office", what: "/ ইংরেজি শব্দ, যেমন আছে", whatEn: "/ an English word, as it is"),
    ]

    static let whereBn = "কোথায় পাবেন: কমা (,) চেপে ধরলে ,,  +  ` ; ফুলস্টপ (.) চেপে ধরলে ^  :  /  .. । আলাদা কিবোর্ডে + হলো Shift + =।"
    static let whereEn = "On the keyboard: hold the comma for ,, + `, and the full stop for ^ : / ..; on a physical keyboard, + is Shift + =."
}
