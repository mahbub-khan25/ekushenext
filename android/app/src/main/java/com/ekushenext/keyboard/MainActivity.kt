package com.ekushenext.keyboard

import android.content.Intent
import android.content.res.ColorStateList
import android.graphics.Typeface
import android.graphics.drawable.GradientDrawable
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.provider.Settings
import android.text.InputType
import android.util.TypedValue
import android.view.Gravity
import android.view.View
import android.view.ViewGroup
import android.view.inputmethod.InputMethodManager
import android.widget.EditText
import android.widget.ImageView
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.ContextCompat
import com.google.android.material.button.MaterialButton
import com.google.android.material.materialswitch.MaterialSwitch
import org.json.JSONArray
import org.json.JSONObject

/** Setup (turn the keyboard on, choose it, try it) and every setting, in one scrolling page. */
class MainActivity : AppCompatActivity() {
    private lateinit var prefs: Prefs
    private lateinit var root: LinearLayout
    private lateinit var scroll: ScrollView
    private var settingsAnchor: View? = null
    private lateinit var bn: Typeface
    private lateinit var bnBold: Typeface

    private val green get() = ContextCompat.getColor(this, R.color.ek_green)
    private val textColor get() = ContextCompat.getColor(this, R.color.ek_text)
    private val subColor get() = ContextCompat.getColor(this, R.color.ek_sub)
    private val cardColor get() = ContextCompat.getColor(this, R.color.ek_card)
    private val lineColor get() = ContextCompat.getColor(this, R.color.ek_line)
    private fun dp(v: Float) = TypedValue.applyDimension(TypedValue.COMPLEX_UNIT_DIP, v, resources.displayMetrics).toInt()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        prefs = Prefs(this)
        bn = runCatching { Typeface.createFromAsset(assets, "fonts/HindSiliguri-Regular.ttf") }.getOrDefault(Typeface.DEFAULT)
        bnBold = runCatching { Typeface.createFromAsset(assets, "fonts/HindSiliguri-SemiBold.ttf") }.getOrDefault(Typeface.DEFAULT_BOLD)
        Engine.start(this)
        scroll = ScrollView(this).apply { isFillViewport = true }
        root = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL; setPadding(dp(18f), dp(24f), dp(18f), dp(40f)) }
        scroll.addView(root)
        setContentView(scroll)
        build()
    }

    override fun onResume() {
        super.onResume()
        build()
        if (intent?.getStringExtra("open") == "settings") {
            intent.removeExtra("open")
            scroll.post { settingsAnchor?.let { scroll.smoothScrollTo(0, it.top - dp(12f)) } }
        }
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        // back from the keyboard picker
        if (hasFocus) build()
    }

    private fun enabled(): Boolean {
        val imm = getSystemService(INPUT_METHOD_SERVICE) as InputMethodManager
        return imm.enabledInputMethodList.any { it.packageName == packageName }
    }

    private fun selected(): Boolean =
        Settings.Secure.getString(contentResolver, Settings.Secure.DEFAULT_INPUT_METHOD)?.startsWith("$packageName/") == true

    // ───────────────────────── page ─────────────────────────

    private var tryText: String = ""

    private fun build() {
        val keepScroll = scroll.scrollY
        root.findViewWithTag<EditText>("try")?.let { tryText = it.text.toString() }
        root.removeAllViews()
        header()
        setup()
        settingsAnchor = section("টাইপিং", "Typing")
        typing()
        section("কিবোর্ড", "Keyboard")
        keyboard()
        section("স্নিপেট", "Snippets")
        snippets()
        section("গোপনীয়তা", "Privacy")
        privacy()
        section("সাহায্য", "Help")
        help()
        scroll.post { scroll.scrollTo(0, keepScroll) }
    }

    private fun header() {
        val box = LinearLayout(this).apply { orientation = LinearLayout.HORIZONTAL; gravity = Gravity.CENTER_VERTICAL }
        box.addView(ImageView(this).apply {
            setImageResource(R.drawable.logo)
            layoutParams = LinearLayout.LayoutParams(dp(64f), dp(64f))
            clipToOutline = true
        })
        val texts = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL; setPadding(dp(14f), 0, 0, 0) }
        texts.addView(text("একুশে নেক্সট", 28f, bnBold, textColor))
        texts.addView(text("যেভাবে বলেন, সেভাবে লিখুন · Ekushe Next", 14f, bn, subColor))
        box.addView(texts)
        root.addView(box)
        space(18f)
    }

    private fun setup() {
        val on = enabled()
        val chosen = on && selected()
        val card = card()
        card.addView(text(if (chosen) "সব প্রস্তুত! ✓" else "শুরু করুন", 20f, bnBold, if (chosen) green else textColor))
        card.addView(text(if (chosen) "Ekushe Next is your keyboard. Try it below." else "Two quick steps to start typing Bangla", 13f, Typeface.DEFAULT, subColor))
        spaceIn(card, 12f)
        card.addView(step("১", "কিবোর্ডটি চালু করুন", "Turn on Ekushe Next in your phone's keyboard settings", on, "চালু করুন") {
            startActivity(Intent(Settings.ACTION_INPUT_METHOD_SETTINGS))
        })
        spaceIn(card, 10f)
        card.addView(step("২", "একুশে নেক্সট বেছে নিন", "Choose it as the keyboard you type with", chosen, "বেছে নিন", enabledButton = on) {
            (getSystemService(INPUT_METHOD_SERVICE) as InputMethodManager).showInputMethodPicker()
        })
        spaceIn(card, 14f)
        card.addView(text("৩  লিখে দেখুন", 16f, bnBold, textColor))
        card.addView(text("ইংরেজি অক্ষরে উচ্চারণ অনুযায়ী লিখুন, যেমন amar sonar bangla, তারপর Space চাপুন।", 13.5f, bn, subColor))
        spaceIn(card, 8f)
        val edit = EditText(this).apply {
            tag = "try"
            hint = "এখানে লিখুন… amar sonar bangla"
            typeface = bn
            textSize = 18f
            minLines = 3
            gravity = Gravity.TOP or Gravity.START
            inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_FLAG_MULTI_LINE
            setText(tryText)
            background = rounded(lineColor, dp(12f).toFloat(), stroke = true)
            setPadding(dp(14f), dp(12f), dp(14f), dp(12f))
        }
        card.addView(edit)
        spaceIn(card, 10f)
        card.addView(text("টিপ: অক্ষরে চেপে ধরলে বড় হাতের অক্ষর ও চিহ্ন পাবেন (t → ট, ৎ)। শব্দের পরেই ⌫ চাপলে রোমান অক্ষর ফিরে আসে। ◂ Space ▸ টেনে কার্সর সরান।", 12.5f, bn, subColor))
        root.addView(card)
    }

    private fun step(num: String, title: String, sub: String, done: Boolean, action: String, enabledButton: Boolean = true, onClick: () -> Unit): View {
        val row = LinearLayout(this).apply { orientation = LinearLayout.HORIZONTAL; gravity = Gravity.CENTER_VERTICAL }
        row.addView(TextView(this).apply {
            text = if (done) "✓" else num
            typeface = bnBold; textSize = 16f; gravity = Gravity.CENTER
            setTextColor(if (done) ContextCompat.getColor(context, R.color.ek_bg) else green)
            background = GradientDrawable().apply { shape = GradientDrawable.OVAL; if (done) setColor(green) else { setColor(0); setStroke(dp(2f), green) } }
            layoutParams = LinearLayout.LayoutParams(dp(34f), dp(34f))
        })
        val texts = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL; setPadding(dp(12f), 0, dp(8f), 0); layoutParams = LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f) }
        texts.addView(text(title, 16f, bnBold, textColor))
        texts.addView(text(sub, 12.5f, Typeface.DEFAULT, subColor))
        row.addView(texts)
        if (!done) row.addView(MaterialButton(this).apply {
            text = action; typeface = bnBold; isEnabled = enabledButton; isAllCaps = false
            setOnClickListener { onClick() }
        })
        return row
    }

    private fun typing() {
        val cfg = Engine.config()
        val card = card()
        card.addView(choice("টাইপিং মোড", "Typing mode", listOf(
            "Smart" to "স্মার্ট: অভিধানের সেরা শব্দ আগে",
            "PhoneticFirst" to "ফোনেটিক আগে: যা লিখলেন সেটাই আগে",
            "PhoneticOnly" to "শুধু ফোনেটিক: কোনো পরামর্শ নেই",
        ), cfg.optString("typing_mode", "Smart")) { Engine.setConfig(JSONObject().put("typing_mode", it)) })
        divider(card)
        card.addView(switch("বাংলা সংখ্যা", "Digits come out as ০-৯ in Bangla", cfg.optBoolean("bangla_digits", true)) { Engine.setConfig(JSONObject().put("bangla_digits", it)) })
        divider(card)
        card.addView(switch("পরের শব্দের পরামর্শ", "Suggest the next word", cfg.optBoolean("next_word", true)) { Engine.setConfig(JSONObject().put("next_word", it)) })
        divider(card)
        card.addView(switch("ইমোজি পরামর্শ", "Offer emoji for some words (bangladesh → 🇧🇩)", cfg.optBoolean("emoji", true)) { Engine.setConfig(JSONObject().put("emoji", it)) })
        divider(card)
        card.addView(switch("⌫ চাপলে রোমান অক্ষর ফেরে", "Backspace right after a word brings your letters back", cfg.optBoolean("backspace_undo", true)) { Engine.setConfig(JSONObject().put("backspace_undo", it)) })
        divider(card)
        card.addView(switch("দুবার Space-এ দাঁড়ি (।)", "Double space types a dari", prefs.doubleSpaceDari) { prefs.doubleSpaceDari = it })
        root.addView(card)
    }

    private fun keyboard() {
        val card = card()
        card.addView(choice("থিম", "Theme", listOf("system" to "ফোনের মতো (System)", "light" to "হালকা (Light)", "dark" to "গাঢ় (Dark)"), prefs.theme) { prefs.theme = it })
        divider(card)
        card.addView(choice("উচ্চতা", "Height", listOf("0" to "ছোট (Small)", "1" to "সাধারণ (Normal)", "2" to "বড় (Tall)"), prefs.height.toString()) { prefs.height = it.toInt() })
        divider(card)
        card.addView(switch("কীতে বাংলা অক্ষর দেখান", "Show the Bangla letter on each key", prefs.keyHints) { prefs.keyHints = it })
        divider(card)
        card.addView(switch("সংখ্যার সারি", "A row of numbers on top", prefs.numberRow) { prefs.numberRow = it })
        divider(card)
        card.addView(switch("কম্পন", "Vibrate on key press", prefs.vibrate) { prefs.vibrate = it })
        divider(card)
        card.addView(switch("শব্দ", "Sound on key press", prefs.sound) { prefs.sound = it })
        root.addView(card)
    }

    private fun snippets() {
        val card = card()
        card.addView(text("ছোট শর্টকাট লিখে Space চাপলে পুরো লেখা বসে। যেমন ;addr → আপনার ঠিকানা।", 13.5f, bn, subColor))
        card.addView(text("Built in: ;salam ;dhonnobad ;inshaallah ;ekushe …", 12.5f, Typeface.DEFAULT, subColor))
        spaceIn(card, 8f)
        val list: JSONArray = Engine.config().optJSONArray("snippets") ?: JSONArray()
        for (i in 0 until list.length()) {
            val pair = list.getJSONArray(i)
            val k = pair.getString(0); val v = pair.getString(1)
            val row = LinearLayout(this).apply { orientation = LinearLayout.HORIZONTAL; gravity = Gravity.CENTER_VERTICAL; setPadding(0, dp(6f), 0, dp(6f)) }
            row.addView(text(k, 15f, Typeface.MONOSPACE, green).apply { setPadding(0, 0, dp(10f), 0) })
            row.addView(text(v, 15f, bn, textColor).apply { layoutParams = LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f); maxLines = 2 })
            row.addView(MaterialButton(this, null, com.google.android.material.R.attr.borderlessButtonStyle).apply {
                text = "মুছুন"; typeface = bn; isAllCaps = false
                setOnClickListener { Engine.setConfig(JSONObject().put("remove_snippet", k)); build() }
            })
            card.addView(row)
        }
        val key = field(";addr", Typeface.MONOSPACE)
        val value = field("পুরো লেখা, যেমন ধানমন্ডি, ঢাকা-১২০৯", bn)
        spaceIn(card, 6f); card.addView(key); spaceIn(card, 8f); card.addView(value); spaceIn(card, 8f)
        card.addView(MaterialButton(this).apply {
            text = "স্নিপেট যোগ করুন"; typeface = bnBold; isAllCaps = false
            setOnClickListener {
                val k = key.text.toString().trim(); val v = value.text.toString().trim()
                if (k.isNotEmpty() && v.isNotEmpty()) { Engine.setConfig(JSONObject().put("add_snippet", JSONArray().put(k).put(v))); build() }
            }
        })
        root.addView(card)
    }

    private fun privacy() {
        val card = card()
        card.addView(text("🔒 আপনার লেখা, শুধু আপনার", 16f, bnBold, textColor))
        card.addView(text("এই কিবোর্ডের ইন্টারনেট ব্যবহারের অনুমতিই নেই। অভিধান, পরামর্শ, শেখা শব্দ সবকিছু আপনার ফোনেই থাকে। পাসওয়ার্ডের ঘরে কিছু শেখা হয় না।", 13.5f, bn, subColor))
        card.addView(text("This keyboard has no internet permission at all. Nothing you type leaves your phone.", 12.5f, Typeface.DEFAULT, subColor))
        spaceIn(card, 10f)
        card.addView(MaterialButton(this, null, com.google.android.material.R.attr.materialButtonOutlinedStyle).apply {
            text = "শেখা শব্দগুলো মুছে দিন"; typeface = bn; isAllCaps = false
            setOnClickListener {
                AlertDialog.Builder(this@MainActivity)
                    .setMessage("আপনার বেছে নেওয়া যত শব্দ একুশে শিখেছে, সব মুছে যাবে। Forget every learned word?")
                    .setPositiveButton("মুছুন") { _, _ -> Engine.forgetLearned() }
                    .setNegativeButton("না", null).show()
            }
        })
        root.addView(card)
    }

    private fun help() {
        val card = card()
        card.addView(link("📖  ব্যবহার নির্দেশিকা", "The full guide: every typing rule", "https://ekushenext.com/guide/"))
        divider(card)
        card.addView(link("🐞  সমস্যা জানান", "Report a problem (no account needed)",
            Engine.reportUrl("Android ${Build.VERSION.RELEASE} (${Build.MANUFACTURER} ${Build.MODEL})").ifEmpty { "https://ekushenext.com/report/" }))
        divider(card)
        card.addView(link("🌐  ekushenext.com", "support@ekushenext.com", "https://ekushenext.com/"))
        spaceIn(card, 10f)
        val version = runCatching { packageManager.getPackageInfo(packageName, 0).versionName }.getOrNull() ?: Engine.release
        card.addView(text("সংস্করণ $version · অভিধানে ${bnNum(Engine.words)} শব্দ\nফ্রি ও ওপেন সোর্স · কোড MPL-2.0, অভিধান CC BY-SA 4.0, ফন্ট Hind Siliguri (OFL)", 12f, bn, subColor))
        root.addView(card)
    }

    // ───────────────────────── pieces ─────────────────────────

    private fun bnNum(n: Int) = "%,d".format(n).map { if (it in '0'..'9') "০১২৩৪৫৬৭৮৯"[it - '0'] else it }.joinToString("")

    private fun text(s: String, size: Float, face: Typeface, color: Int) = TextView(this).apply {
        text = s; textSize = size; typeface = face; setTextColor(color); setLineSpacing(0f, 1.15f)
    }

    private fun section(bnTitle: String, en: String): View {
        space(26f)
        val row = LinearLayout(this).apply { orientation = LinearLayout.HORIZONTAL; gravity = Gravity.BOTTOM; setPadding(dp(4f), 0, 0, dp(8f)) }
        row.addView(text(bnTitle, 18f, bnBold, textColor))
        row.addView(text("  $en", 13f, Typeface.DEFAULT, subColor))
        root.addView(row)
        return row
    }

    private fun card() = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        setPadding(dp(18f), dp(16f), dp(18f), dp(16f))
        background = rounded(cardColor, dp(20f).toFloat())
        elevation = dp(1f).toFloat()
    }

    private fun rounded(color: Int, radius: Float, stroke: Boolean = false) = GradientDrawable().apply {
        cornerRadius = radius
        if (stroke) { setColor(0); setStroke(dp(1.5f), color) } else setColor(color)
    }

    private fun space(h: Float) = root.addView(View(this), LinearLayout.LayoutParams(1, dp(h)))
    private fun spaceIn(v: LinearLayout, h: Float) = v.addView(View(this), LinearLayout.LayoutParams(1, dp(h)))
    private fun divider(v: LinearLayout) = v.addView(View(this).apply { setBackgroundColor(lineColor) }, LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, dp(1f)).apply { topMargin = dp(10f); bottomMargin = dp(10f) })

    private fun field(hintText: String, face: Typeface) = EditText(this).apply {
        hint = hintText; typeface = face; textSize = 16f; isSingleLine = true
        background = rounded(lineColor, dp(12f).toFloat(), stroke = true)
        setPadding(dp(14f), dp(10f), dp(14f), dp(10f))
    }

    private fun switch(title: String, sub: String, value: Boolean, onChange: (Boolean) -> Unit): View {
        val row = LinearLayout(this).apply { orientation = LinearLayout.HORIZONTAL; gravity = Gravity.CENTER_VERTICAL }
        val texts = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL; layoutParams = LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f) }
        texts.addView(text(title, 15.5f, bnBold, textColor))
        texts.addView(text(sub, 12.5f, Typeface.DEFAULT, subColor))
        row.addView(texts)
        val sw = MaterialSwitch(this).apply {
            isChecked = value
            thumbTintList = null
            setOnCheckedChangeListener { _, b -> onChange(b) }
        }
        row.addView(sw)
        row.setOnClickListener { sw.toggle() }
        return row
    }

    private fun choice(title: String, sub: String, options: List<Pair<String, String>>, current: String, onPick: (String) -> Unit): View {
        val box = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL }
        box.addView(text(title, 15.5f, bnBold, textColor))
        box.addView(text(sub, 12.5f, Typeface.DEFAULT, subColor))
        val chips = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL; setPadding(0, dp(8f), 0, 0) }
        options.forEach { (id, label) ->
            val on = id == current
            chips.addView(TextView(this).apply {
                text = if (on) "●  $label" else "○  $label"
                typeface = if (on) bnBold else bn
                textSize = 15f
                setTextColor(if (on) green else textColor)
                setPadding(dp(12f), dp(9f), dp(12f), dp(9f))
                background = if (on) rounded(lineColor, dp(10f).toFloat()) else null
                setOnClickListener { onPick(id); build() }
            }, LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT).apply { bottomMargin = dp(4f) })
        }
        box.addView(chips)
        return box
    }

    private fun link(title: String, sub: String, url: String): View {
        val row = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL; setPadding(0, dp(4f), 0, dp(4f)) }
        row.addView(text(title, 16f, bnBold, textColor))
        row.addView(text(sub, 12.5f, Typeface.DEFAULT, subColor))
        row.setOnClickListener { runCatching { startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url))) } }
        return row
    }

    @Suppress("unused")
    private fun tint(v: MaterialButton) { v.backgroundTintList = ColorStateList.valueOf(green) }
}
