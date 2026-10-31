/*
 * Ekushe for Linux on Fcitx 5: an input method addon on the shared Rust engine (crates/ekushe-ffi),
 * the same engine and the same keys as the IBus engine (platform/linux/ibus/ekushe-engine.c).
 *
 * The word being typed is pre-edit text (underlined in the application), suggestions go to Fcitx's
 * candidate window. Space commits the highlighted word, Enter commits and passes the Enter on, Tab
 * cycles suggestions, Escape keeps the text as shown, Backspace edits the word and, right after a
 * commit, reopens it (when the application reports the text around the cursor). Ctrl+Backspace
 * flips the last word between Bangla and Roman. Double-tap Shift switches to English and back.
 * The status menu opens Studio, the conjunct guide, Settings, the layout's key map and the problem report.
 *
 * Installed as <libdir>/fcitx5/libekushe.so with /usr/share/fcitx5/addon/ekushe.conf and
 * /usr/share/fcitx5/inputmethod/ekushe.conf.
 *
 * SPDX-License-Identifier: MPL-2.0
 */
#include <fcitx-utils/i18n.h>
#include <fcitx-utils/key.h>
#include <fcitx-utils/utf8.h>
#include <fcitx/action.h>
#include <fcitx/addonfactory.h>
#include <fcitx/addonmanager.h>
#include <fcitx/candidatelist.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/inputpanel.h>
#include <fcitx/instance.h>
#include <fcitx/statusarea.h>
#include <fcitx/text.h>
#include <fcitx/userinterfacemanager.h>

#include <chrono>
#include <cstring>
#include <memory>
#include <string>
#include <vector>

#include <spawn.h>
#include <sys/utsname.h>
#include <sys/wait.h>
#include <unistd.h>

extern "C" {
#include "../ekushe_ffi.h"
}

extern char **environ;

namespace {

/* Take ownership of a string from the Rust library. */
std::string take(char *s, bool *ok = nullptr)
{
    if (ok)
        *ok = s != nullptr;
    if (!s)
        return {};
    std::string copy(s);
    ekushe_string_free(s);
    return copy;
}

uint64_t now_ms()
{
    using namespace std::chrono;
    return duration_cast<milliseconds>(steady_clock::now().time_since_epoch()).count();
}

/* Start a program without waiting for it (and without leaving a zombie). */
void spawn_detached(const std::vector<std::string> &args)
{
    pid_t pid = fork();
    if (pid == 0) {
        if (fork() == 0) {
            std::vector<char *> argv;
            for (auto &a : args)
                argv.push_back(const_cast<char *>(a.c_str()));
            argv.push_back(nullptr);
            execvp(argv[0], argv.data());
            _exit(127);
        }
        _exit(0);
    }
    if (pid > 0)
        waitpid(pid, nullptr, 0);
}

/* Studio in its own window (installed next to the IBus engine), else the browser. */
void open_studio(const char *tab)
{
    bool ok = false;
    std::string url = take(ek_studio_url(tab), &ok);
    if (!ok)
        return;
    const char *dirs[] = {"/usr/libexec/ekushe", "/usr/lib/ekushe", "/usr/local/libexec/ekushe"};
    for (const char *d : dirs) {
        std::string window = std::string(d) + "/ekushe-studio-window";
        if (access(window.c_str(), X_OK) == 0) {
            spawn_detached({window, url, tab ? tab : ""});
            return;
        }
    }
    spawn_detached({"xdg-open", url});
}

class EkusheEngine;

class EkusheCandidate : public fcitx::CandidateWord {
public:
    EkusheCandidate(EkusheEngine *engine, int index, const std::string &text)
        : fcitx::CandidateWord(fcitx::Text(text)), engine_(engine), index_(index) {}
    void select(fcitx::InputContext *ic) const override;

private:
    EkusheEngine *engine_;
    int index_;
};

class EkusheEngine : public fcitx::InputMethodEngineV2 {
public:
    explicit EkusheEngine(fcitx::Instance *instance) : instance_(instance)
    {
        ek_init();
        /* Under Fcitx, English is Fcitx's own keyboard (Left Shift or Ctrl+Space switches to it), so
           choosing Ekushe means Bangla. Double-tap Shift still switches inside Ekushe where Fcitx leaves
           Shift alone (Right Shift by default). */
        if (!ek_language())
            ek_toggle_language();
        auto add = [this](fcitx::SimpleAction &a, const char *name, const char *label, const char *icon, auto fn) {
            a.setShortText(label);
            a.setIcon(icon);
            a.connect<fcitx::SimpleAction::Activated>(fn);
            instance_->userInterfaceManager().registerAction(name, &a);
        };
        add(studio_, "ekushe-studio", "Open Ekushe Studio", "accessories-text-editor", [](fcitx::InputContext *) { open_studio(nullptr); });
        add(marks_, "ekushe-marks", "Marks & conjuncts guide (+  ,,  ^ …)", "help-contents", [](fcitx::InputContext *) { open_studio("signs"); });
        add(settings_, "ekushe-settings", "Settings", "preferences-system", [](fcitx::InputContext *) { open_studio("settings"); });
        add(keymap_, "ekushe-keymap", "Key map of the layout", "input-keyboard", [](fcitx::InputContext *) { open_studio("keymap"); });
        add(language_, "ekushe-language", "Bangla / English (double-tap Right Shift)", "input-keyboard", [this](fcitx::InputContext *ic) {
            if (ic)
                finish(ic);
            ek_toggle_language();
            showLanguage(ic);
        });
        add(report_, "ekushe-report", "Report a problem", "dialog-warning", [](fcitx::InputContext *) {
            struct utsname u;
            std::string os = uname(&u) == 0 ? std::string("Linux ") + u.release : "Linux";
            bool ok = false;
            std::string url = take(ek_report_url(os.c_str()), &ok);
            if (ok)
                spawn_detached({"xdg-open", url});
        });
    }

    ~EkusheEngine() override { ek_save_learned(); }

    void activate(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) override
    {
        auto *ic = event.inputContext();
        ek_reload_config();
        syncFieldRules(ic);
        for (auto *a : {&studio_, &marks_, &settings_, &keymap_, &language_, &report_})
            ic->statusArea().addAction(fcitx::StatusGroup::InputMethod, a);
    }

    void deactivate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override
    {
        finish(event.inputContext());
        ek_save_learned();
        reset(entry, event);
    }

    void reset(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) override
    {
        finish(event.inputContext());
        hideAll(event.inputContext());
    }

    std::string subModeLabelImpl(const fcitx::InputMethodEntry &, fcitx::InputContext &) override
    {
        return ek_language() ? "এ" : "EN";
    }

    void keyEvent(const fcitx::InputMethodEntry &, fcitx::KeyEvent &event) override;

    /* A suggestion was clicked */
    void pick(fcitx::InputContext *ic, int index)
    {
        if (index < 0 || (unsigned)index >= count_)
            return;
        selected_ = index;
        commitSelected(ic, ' ');
    }

private:
    bool secret(fcitx::InputContext *ic) const
    {
        auto caps = ic->capabilityFlags();
        return caps.test(fcitx::CapabilityFlag::Password);
    }

    /* The Rust engine is shared by every field: tell it whether this one is private */
    void syncFieldRules(fcitx::InputContext *ic)
    {
        auto caps = ic->capabilityFlags();
        ek_set_private(caps.test(fcitx::CapabilityFlag::Password) || caps.test(fcitx::CapabilityFlag::Sensitive));
    }

    void hideAll(fcitx::InputContext *ic)
    {
        ic->inputPanel().reset();
        ic->updatePreedit();
        ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
        count_ = 0;
        selected_ = 0;
    }

    void commitString(fcitx::InputContext *ic, const std::string &text, bool remember)
    {
        hideAll(ic);
        if (!text.empty())
            ic->commitString(text);
        lastCommitted_ = (remember && !text.empty() && text.back() == ' ') ? text : std::string();
    }

    void showPreedit(fcitx::InputContext *ic, const std::string &text)
    {
        fcitx::Text t;
        t.append(text, fcitx::TextFormatFlag::Underline);
        t.setCursor(text.size());
        if (ic->capabilityFlags().test(fcitx::CapabilityFlag::Preedit))
            ic->inputPanel().setClientPreedit(t);
        else
            ic->inputPanel().setPreedit(t);
        ic->updatePreedit();
    }

    /* Show the engine's last result: commit it, or update the pre-edit and the candidates. */
    void render(fcitx::InputContext *ic)
    {
        bool ok = false;
        std::string committed = take(ek_out_committed(), &ok);
        if (ok) {
            commitString(ic, committed, true);
            return;
        }
        std::string pre = take(ek_out_preedit());
        if (pre.empty()) {
            hideAll(ic);
            return;
        }
        showPreedit(ic, pre);
        candidates_.clear();
        count_ = ek_out_candidate_count();
        for (unsigned i = 0; i < count_; i++)
            candidates_.push_back(take(ek_out_candidate(i)));
        selected_ = std::min<unsigned>(ek_out_selected(), count_ ? count_ - 1 : 0);
        showCandidates(ic);
    }

    void showCandidates(fcitx::InputContext *ic)
    {
        if (count_ > 1) {
            auto list = std::make_unique<fcitx::CommonCandidateList>();
            list->setPageSize(9);
            list->setLayoutHint(fcitx::CandidateLayoutHint::Horizontal);
            list->setCursorIncludeUnselected(false);
            for (unsigned i = 0; i < count_; i++)
                list->append<EkusheCandidate>(this, (int)i, candidates_[i]);
            list->setGlobalCursorIndex(selected_);
            ic->inputPanel().setCandidateList(std::move(list));
        } else {
            ic->inputPanel().setCandidateList(nullptr);
        }
        ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
    }

    void commitSelected(fcitx::InputContext *ic, uint32_t trailing)
    {
        ekushe_string_free(ek_select(selected_, trailing));
        std::string committed = take(ek_out_committed());
        commitString(ic, committed, trailing == ' ');
    }

    /* Finish the word as shown (focus change, arrows, shortcuts) */
    void finish(fcitx::InputContext *ic)
    {
        if (!ic || !ek_is_composing())
            return;
        commitSelected(ic, 0);
        ek_reset();
    }

    void cycle(fcitx::InputContext *ic)
    {
        selected_ = (selected_ + 1) % count_;
        showPreedit(ic, candidates_[selected_]);
        showCandidates(ic);
    }

    /* TRUE when the text just before the cursor is exactly `expect` (needs surrounding text) */
    bool textBeforeCursorIs(fcitx::InputContext *ic, const std::string &expect)
    {
        if (expect.empty() || !ic->capabilityFlags().test(fcitx::CapabilityFlag::SurroundingText))
            return false;
        const auto &st = ic->surroundingText();
        if (!st.isValid() || st.cursor() != st.anchor())
            return false;
        const std::string &all = st.text();
        size_t n = fcitx::utf8::length(expect);
        unsigned cursor = st.cursor();
        if (cursor < n || !fcitx::utf8::validate(all))
            return false;
        auto start = fcitx::utf8::nextNChar(all.begin(), cursor - n);
        auto end = fcitx::utf8::nextNChar(all.begin(), cursor);
        return std::string(start, end) == expect;
    }

    /* Backspace right after a word: take the word and its space back and reopen it, with its
       suggestions (another Bangla word, or the English as typed). */
    bool undoLast(fcitx::InputContext *ic)
    {
        std::string expect = lastCommitted_;
        lastCommitted_.clear();
        if (!textBeforeCursorIs(ic, expect)) {
            ek_reset();
            return false;
        }
        bool ok = false;
        take(ek_reopen_last_word(), &ok);
        if (!ok)
            return false;
        int n = fcitx::utf8::length(expect);
        ic->deleteSurroundingText(-n, n);
        render(ic);
        return true;
    }

    bool flipLast(fcitx::InputContext *ic)
    {
        std::string expect = lastCommitted_;
        bool ok = false;
        std::string json = take(ek_flip_last(), &ok);
        if (!ok || expect.empty())
            return false;
        /* {"delete":N,"text":"…"} */
        auto t = json.find("\"text\":\"");
        auto e = json.rfind('"');
        if (t == std::string::npos || e == std::string::npos || e <= t + 8)
            return false;
        std::string text;
        for (size_t i = t + 8; i < e; i++) { /* undo JSON's backslash escapes */
            if (json[i] == '\\' && i + 1 < e)
                i++;
            text += json[i];
        }
        if (!textBeforeCursorIs(ic, expect))
            return false;
        int n = fcitx::utf8::length(expect);
        ic->deleteSurroundingText(-n, n);
        ic->commitString(text);
        lastCommitted_ = text;
        return true;
    }

    /* After switching: the status icon, and a short note next to the cursor */
    void showLanguage(fcitx::InputContext *ic)
    {
        if (!ic)
            return;
        ic->updateUserInterface(fcitx::UserInterfaceComponent::StatusArea);
        instance_->showInputMethodInformation(ic);
    }

    friend class EkusheCandidate;
    fcitx::Instance *instance_;
    fcitx::SimpleAction studio_, marks_, settings_, keymap_, language_, report_;
    std::vector<std::string> candidates_;
    unsigned count_ = 0, selected_ = 0;
    std::string lastCommitted_;
    bool shiftClean_ = false;
};

void EkusheCandidate::select(fcitx::InputContext *ic) const { engine_->pick(ic, index_); }

void EkusheEngine::keyEvent(const fcitx::InputMethodEntry &, fcitx::KeyEvent &event)
{
    auto *ic = event.inputContext();
    const fcitx::Key &key = event.rawKey();
    fcitx::KeySym sym = event.key().sym();
    auto states = event.key().states();
    bool isShift = sym == FcitxKey_Shift_L || sym == FcitxKey_Shift_R;

    /* Password and PIN fields: plain characters, no pre-edit, nothing learned */
    if (secret(ic))
        return;

    if (event.isRelease()) {
        if (isShift && shiftClean_) {
            shiftClean_ = false;
            uint8_t r = ek_shift_tap(now_ms());
            if (r == 0)
                finish(ic); /* switched to English: close the open word */
            if (r != 255)
                showLanguage(ic);
        }
        return;
    }
    const fcitx::KeyStates shortcutMods = fcitx::KeyStates{fcitx::KeyState::Ctrl} | fcitx::KeyState::Alt |
                                          fcitx::KeyState::Super | fcitx::KeyState::Hyper | fcitx::KeyState::Meta;
    if (isShift) {
        shiftClean_ = !(states & shortcutMods);
        return;
    }
    shiftClean_ = false;
    (void)key;

    bool composing = ek_is_composing();
    if ((states & fcitx::KeyState::Ctrl) && sym == FcitxKey_BackSpace && !composing) {
        if (flipLast(ic))
            event.filterAndAccept();
        return;
    }
    /* Shortcuts belong to the application */
    if (states & shortcutMods) {
        if (composing)
            finish(ic);
        return;
    }
    if (!ek_language())
        return;

    switch (sym) {
    case FcitxKey_space:
        if (composing) {
            commitSelected(ic, ' ');
            event.filterAndAccept();
        }
        return;
    case FcitxKey_Return:
    case FcitxKey_KP_Enter:
        if (composing)
            commitSelected(ic, 0);
        return; /* the application still gets its Enter */
    case FcitxKey_BackSpace:
        if (composing) {
            ekushe_string_free(ek_backspace());
            render(ic);
            event.filterAndAccept();
        } else if (ek_can_undo() && undoLast(ic)) {
            event.filterAndAccept();
        }
        return;
    case FcitxKey_Escape:
        if (composing) {
            finish(ic);
            event.filterAndAccept();
        }
        return;
    case FcitxKey_Tab:
        if (composing && count_ > 1) {
            cycle(ic);
            event.filterAndAccept();
        } else if (composing) {
            finish(ic);
        }
        return;
    case FcitxKey_Left: case FcitxKey_Right: case FcitxKey_Up: case FcitxKey_Down:
    case FcitxKey_Home: case FcitxKey_End: case FcitxKey_Page_Up: case FcitxKey_Page_Down:
    case FcitxKey_Delete:
        if (composing)
            finish(ic);
        return;
    default:
        break;
    }

    uint32_t ch = fcitx::Key::keySymToUnicode(sym);
    if (ch < 0x20 || ch == 0x7F) {
        if (composing)
            finish(ic);
        return;
    }
    /* AltGr (Mod5) types a fixed layout's third layer (UniJoy: ঈ ঊ ঐ ঔ ...) */
    if ((states & fcitx::KeyState::Mod5) && ek_fixed_layout())
        ekushe_string_free(ek_key_altgr(ch));
    else
        ekushe_string_free(ek_key(ch));
    render(ic);
    event.filterAndAccept();
}

class EkusheFactory : public fcitx::AddonFactory {
public:
    fcitx::AddonInstance *create(fcitx::AddonManager *manager) override
    {
        return new EkusheEngine(manager->instance());
    }
};

} // namespace

FCITX_ADDON_FACTORY(EkusheFactory);
