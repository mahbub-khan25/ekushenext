/*
 * Ekushe for Linux: an IBus input method engine on the shared Rust engine (crates/ekushe-ffi).
 *
 * The word being typed is IBus pre-edit text (underlined in the application), suggestions go to
 * the IBus candidate window. Space commits the highlighted word, Enter commits and passes the
 * Enter on, Tab cycles suggestions, Escape keeps the text as shown, Backspace edits the word and,
 * right after a commit, gives back the Roman letters (when the application reports the text
 * around the cursor). Ctrl+Backspace flips the last word between Bangla and Roman. Double-tap
 * Shift switches to English and back. The engine's menu opens Studio and Settings in the browser.
 *
 * Installed as /usr/libexec/ekushe/ibus-engine-ekushe and started by ibus-daemon (--ibus).
 *
 * SPDX-License-Identifier: MPL-2.0
 */
#include <ibus.h>
#include <sys/wait.h>
#include <string.h>

#include "../ekushe_ffi.h"

#define EKUSHE_TYPE_ENGINE (ekushe_engine_get_type())

typedef struct {
    IBusEngine parent;
    IBusLookupTable *table;
    IBusPropList *props;
    IBusProperty *mode_prop; /* "InputMode": GNOME shows its symbol in the top bar (এ / EN) */
    guint selected;
    guint count;
    gchar *last_committed; /* last word committed with a space (what Backspace-undo expects) */
    gboolean shift_clean;
    gboolean secret; /* password or PIN field: keys go straight to the application */
    gboolean private_field; /* nothing learned here (password, PIN or a field marked private) */
    gboolean focused;
    guint note_timer; /* hides the "বাংলা / EN" note shown next to the cursor after switching */
} EkusheEngine;

/* IBUS_INPUT_HINT_PRIVATE (IBus 1.5.24+): the field asks not to be remembered */
#define EKUSHE_HINT_PRIVATE (1 << 13)

typedef struct {
    IBusEngineClass parent;
} EkusheEngineClass;

G_DEFINE_TYPE(EkusheEngine, ekushe_engine, IBUS_TYPE_ENGINE)

/* Take ownership of a string from the Rust library. */
static gchar *take(char *s)
{
    if (!s)
        return NULL;
    gchar *copy = g_strdup(s);
    ekushe_string_free(s);
    return copy;
}

static void hide_all(EkusheEngine *ek)
{
    ibus_engine_hide_preedit_text((IBusEngine *)ek);
    ibus_engine_hide_lookup_table((IBusEngine *)ek);
    ek->count = 0;
    ek->selected = 0;
}

static void commit_string(EkusheEngine *ek, const gchar *text, gboolean remember)
{
    hide_all(ek);
    if (text && *text)
        ibus_engine_commit_text((IBusEngine *)ek, ibus_text_new_from_string(text));
    g_free(ek->last_committed);
    ek->last_committed = (remember && text && g_str_has_suffix(text, " ")) ? g_strdup(text) : NULL;
}

static void show_preedit(EkusheEngine *ek, const gchar *text)
{
    IBusText *t = ibus_text_new_from_string(text);
    guint len = ibus_text_get_length(t);
    ibus_text_append_attribute(t, IBUS_ATTR_TYPE_UNDERLINE, IBUS_ATTR_UNDERLINE_SINGLE, 0, len);
    ibus_engine_update_preedit_text((IBusEngine *)ek, t, len, len > 0);
}

/* Show the engine's last result: commit it, or update the pre-edit and the candidate window. */
static void render(EkusheEngine *ek)
{
    gchar *committed = take(ek_out_committed());
    if (committed) {
        commit_string(ek, committed, TRUE);
        g_free(committed);
        return;
    }
    gchar *pre = take(ek_out_preedit());
    if (!pre || !*pre) {
        hide_all(ek);
        g_free(pre);
        return;
    }
    show_preedit(ek, pre);
    g_free(pre);

    ibus_lookup_table_clear(ek->table);
    ek->count = ek_out_candidate_count();
    for (guint i = 0; i < ek->count; i++) {
        gchar *c = take(ek_out_candidate(i));
        if (c) {
            ibus_lookup_table_append_candidate(ek->table, ibus_text_new_from_string(c));
            g_free(c);
        }
    }
    ek->selected = MIN(ek_out_selected(), ek->count ? ek->count - 1 : 0);
    ibus_lookup_table_set_cursor_pos(ek->table, ek->selected);
    ibus_engine_update_lookup_table((IBusEngine *)ek, ek->table, ek->count > 1);
}

static void commit_selected(EkusheEngine *ek, guint32 trailing)
{
    ekushe_string_free(ek_select(ek->selected, trailing));
    gchar *committed = take(ek_out_committed());
    commit_string(ek, committed, trailing == ' ');
    g_free(committed);
}

/* Finish the word as shown (focus change, arrows, shortcuts). */
static void finish(EkusheEngine *ek)
{
    if (!ek_is_composing())
        return;
    commit_selected(ek, 0);
    ek_reset();
}

static void cycle(EkusheEngine *ek)
{
    ek->selected = (ek->selected + 1) % ek->count;
    ibus_lookup_table_set_cursor_pos(ek->table, ek->selected);
    IBusText *c = ibus_lookup_table_get_candidate(ek->table, ek->selected);
    if (c)
        show_preedit(ek, ibus_text_get_text(c));
    ibus_engine_update_lookup_table((IBusEngine *)ek, ek->table, TRUE);
}

/* TRUE when the text just before the cursor is exactly `expect` (needs surrounding-text support). */
static gboolean text_before_cursor_is(EkusheEngine *ek, const gchar *expect)
{
    IBusEngine *e = (IBusEngine *)ek;
    if (!expect || !(e->client_capabilities & IBUS_CAP_SURROUNDING_TEXT))
        return FALSE;
    IBusText *text = NULL;
    guint cursor = 0, anchor = 0;
    ibus_engine_get_surrounding_text(e, &text, &cursor, &anchor);
    if (!text || cursor != anchor)
        return FALSE;
    glong n = g_utf8_strlen(expect, -1);
    if ((glong)cursor < n)
        return FALSE;
    const gchar *all = ibus_text_get_text(text);
    const gchar *start = g_utf8_offset_to_pointer(all, cursor - n);
    const gchar *end = g_utf8_offset_to_pointer(all, cursor);
    return (gsize)(end - start) == strlen(expect) && strncmp(start, expect, end - start) == 0;
}

static gboolean replace_before_cursor(EkusheEngine *ek, const gchar *expect, const gchar *text)
{
    if (!text_before_cursor_is(ek, expect))
        return FALSE;
    glong n = g_utf8_strlen(expect, -1);
    ibus_engine_delete_surrounding_text((IBusEngine *)ek, -n, n);
    ibus_engine_commit_text((IBusEngine *)ek, ibus_text_new_from_string(text));
    return TRUE;
}

static gboolean undo_last(EkusheEngine *ek)
{
    gchar *expect = g_strdup(ek->last_committed);
    if (!text_before_cursor_is(ek, expect)) {
        g_free(expect);
        ek_reset();
        return FALSE;
    }
    ekushe_string_free(ek_backspace());
    gchar *roman = take(ek_out_committed());
    gboolean ok = roman && replace_before_cursor(ek, expect, roman);
    g_free(ek->last_committed);
    ek->last_committed = NULL;
    g_free(roman);
    g_free(expect);
    return ok;
}

static gboolean flip_last(EkusheEngine *ek)
{
    gchar *expect = g_strdup(ek->last_committed);
    gchar *json = take(ek_flip_last());
    gboolean ok = FALSE;
    if (json && expect) {
        /* {"delete":N,"text":"…"} */
        const gchar *t = strstr(json, "\"text\":\"");
        if (t) {
            t += 8;
            gchar *end = strrchr(t, '"');
            if (end) {
                gchar *raw = g_strndup(t, end - t);
                gchar *text = g_strcompress(raw); /* undo JSON's backslash escapes */
                ok = replace_before_cursor(ek, expect, text);
                if (ok) {
                    g_free(ek->last_committed);
                    ek->last_committed = g_strdup(text);
                }
                g_free(text);
                g_free(raw);
            }
        }
    }
    g_free(json);
    g_free(expect);
    return ok;
}

/* Show Bangla or English in the top bar (GNOME reads the "InputMode" property's symbol; it must
 * be at most two characters) and in the engine's menu. */
static void show_language(EkusheEngine *ek)
{
    gboolean bn = ek_language() != 0;
    ibus_property_set_symbol(ek->mode_prop, ibus_text_new_from_string(bn ? "এ" : "EN"));
    /* Name only the language in use: a popup or menu shows this label */
    ibus_property_set_label(ek->mode_prop, ibus_text_new_from_string(bn ? "বাংলা" : "English"));
    ibus_engine_update_property((IBusEngine *)ek, ek->mode_prop);
}

static gboolean hide_note(gpointer data)
{
    EkusheEngine *ek = (EkusheEngine *)data;
    ek->note_timer = 0;
    ibus_engine_hide_auxiliary_text((IBusEngine *)ek);
    return G_SOURCE_REMOVE;
}

/* After switching, show the language in use next to the cursor for a moment, like the badge on
 * Windows: the top bar can be hidden (only one keyboard) or slow to update. */
static void show_switch_note(EkusheEngine *ek)
{
    gboolean bn = ek_language() != 0;
    ibus_engine_update_auxiliary_text((IBusEngine *)ek,
        ibus_text_new_from_string(bn ? "এ · বাংলা" : "EN · English"), TRUE);
    if (ek->note_timer)
        g_source_remove(ek->note_timer);
    ek->note_timer = g_timeout_add_full(G_PRIORITY_DEFAULT, 1200, hide_note, g_object_ref(ek), g_object_unref);
}

static gboolean ekushe_process_key_event(IBusEngine *e, guint keyval, guint keycode, guint modifiers)
{
    EkusheEngine *ek = (EkusheEngine *)e;
    (void)keycode;
    gboolean is_shift = keyval == IBUS_KEY_Shift_L || keyval == IBUS_KEY_Shift_R;

    /* Password and PIN fields: type plain characters, no pre-edit, nothing learned */
    if (ek->secret)
        return FALSE;

    if (modifiers & IBUS_RELEASE_MASK) {
        if (is_shift && ek->shift_clean) {
            ek->shift_clean = FALSE;
            guint8 r = ek_shift_tap((guint64)(g_get_monotonic_time() / 1000));
            if (r == 0)
                finish(ek); /* switched to English: close the open word */
            if (r != 255) {
                show_language(ek);
                show_switch_note(ek);
            }
        }
        return FALSE;
    }
    if (is_shift) {
        ek->shift_clean = !(modifiers & (IBUS_CONTROL_MASK | IBUS_MOD1_MASK | IBUS_SUPER_MASK | IBUS_MOD4_MASK));
        return FALSE;
    }
    ek->shift_clean = FALSE;

    gboolean composing = ek_is_composing();
    guint shortcut_mods = IBUS_CONTROL_MASK | IBUS_MOD1_MASK | IBUS_SUPER_MASK | IBUS_MOD4_MASK | IBUS_META_MASK | IBUS_HYPER_MASK;

    if ((modifiers & IBUS_CONTROL_MASK) && keyval == IBUS_KEY_BackSpace && !composing)
        return flip_last(ek);
    /* Shortcuts belong to the application (AltGr arrives as ISO_Level3, not Mod1) */
    if (modifiers & shortcut_mods) {
        if (composing)
            finish(ek);
        return FALSE;
    }
    if (!ek_language())
        return FALSE;

    switch (keyval) {
    case IBUS_KEY_space:
        if (composing) {
            commit_selected(ek, ' ');
            return TRUE;
        }
        return FALSE;
    case IBUS_KEY_Return:
    case IBUS_KEY_KP_Enter:
        if (composing)
            commit_selected(ek, 0);
        return FALSE; /* the application still gets its Enter */
    case IBUS_KEY_BackSpace:
        if (composing) {
            ekushe_string_free(ek_backspace());
            render(ek);
            return TRUE;
        }
        if (ek_can_undo())
            return undo_last(ek);
        return FALSE;
    case IBUS_KEY_Escape:
        if (composing) {
            finish(ek);
            return TRUE;
        }
        return FALSE;
    case IBUS_KEY_Tab:
        if (composing && ek->count > 1) {
            cycle(ek);
            return TRUE;
        }
        if (composing)
            finish(ek);
        return FALSE;
    case IBUS_KEY_Left: case IBUS_KEY_Right: case IBUS_KEY_Up: case IBUS_KEY_Down:
    case IBUS_KEY_Home: case IBUS_KEY_End: case IBUS_KEY_Page_Up: case IBUS_KEY_Page_Down:
    case IBUS_KEY_Delete:
        if (composing)
            finish(ek);
        return FALSE;
    default:
        break;
    }

    gunichar ch = ibus_keyval_to_unicode(keyval);
    if (ch < 0x20 || ch == 0x7F) {
        if (composing)
            finish(ek);
        return FALSE;
    }
    ekushe_string_free(ek_key(ch));
    render(ek);
    return TRUE;
}

static void ekushe_focus_out(IBusEngine *e)
{
    ((EkusheEngine *)e)->focused = FALSE;
    finish((EkusheEngine *)e);
    IBUS_ENGINE_CLASS(ekushe_engine_parent_class)->focus_out(e);
}

static void ekushe_reset(IBusEngine *e)
{
    finish((EkusheEngine *)e);
    IBUS_ENGINE_CLASS(ekushe_engine_parent_class)->reset(e);
}

static void ekushe_disable(IBusEngine *e)
{
    finish((EkusheEngine *)e);
    ek_save_learned();
    IBUS_ENGINE_CLASS(ekushe_engine_parent_class)->disable(e);
}

static void ekushe_enable(IBusEngine *e)
{
    ek_reload_config();
    /* Ask for the text around the cursor (needed for Backspace-undo) */
    ibus_engine_get_surrounding_text(e, NULL, NULL, NULL);
    IBUS_ENGINE_CLASS(ekushe_engine_parent_class)->enable(e);
}

static void ekushe_set_content_type(IBusEngine *e, guint purpose, guint hints)
{
    EkusheEngine *ek = (EkusheEngine *)e;
    gboolean secret = purpose == IBUS_INPUT_PURPOSE_PASSWORD || purpose == IBUS_INPUT_PURPOSE_PIN;
    gboolean private_field = secret || (hints & EKUSHE_HINT_PRIVATE) != 0;
    if (secret == ek->secret && private_field == ek->private_field)
        return;
    /* Close the open word as shown before the field's rules change */
    if (ek->focused)
        finish(ek);
    ek->secret = secret;
    ek->private_field = private_field;
    /* The Rust engine is shared by every field: only the focused one sets it (focus_in resyncs) */
    if (ek->focused)
        ek_set_private(private_field);
}

static void ekushe_focus_in(IBusEngine *e)
{
    EkusheEngine *ek = (EkusheEngine *)e;
    /* The Rust engine is shared by every field: tell it about this one */
    ek->focused = TRUE;
    ek_set_private(ek->private_field);
    ibus_engine_register_properties(e, ek->props);
    show_language(ek);
    IBUS_ENGINE_CLASS(ekushe_engine_parent_class)->focus_in(e);
}

static void ekushe_candidate_clicked(IBusEngine *e, guint index, guint button, guint state)
{
    EkusheEngine *ek = (EkusheEngine *)e;
    (void)button;
    (void)state;
    if (index < ek->count) {
        ek->selected = index;
        commit_selected(ek, ' ');
    }
}

static void open_in_browser(const gchar *url)
{
    gchar *argv[] = {"xdg-open", (gchar *)url, NULL};
    g_spawn_async(NULL, argv, NULL, G_SPAWN_SEARCH_PATH | G_SPAWN_STDOUT_TO_DEV_NULL | G_SPAWN_STDERR_TO_DEV_NULL, NULL, NULL, NULL, NULL);
}

/* Studio's own window could not start (no WebKitGTK, no display): use the browser instead. */
static void studio_window_done(GPid pid, gint status, gpointer url)
{
    g_spawn_close_pid(pid);
    if (WIFEXITED(status) && (WEXITSTATUS(status) == 3 || WEXITSTATUS(status) == 127))
        open_in_browser(url);
}

static void open_studio(const char *tab)
{
    gchar *url = take(ek_studio_url(tab));
    if (!url)
        return;
    /* Studio's own window (GTK + WebKitGTK; any desktop); the browser if that isn't installed */
    /* Installed next to this engine (/usr/libexec/ekushe, or /usr/local/... from the AppImage) */
    gchar *self = g_file_read_link("/proc/self/exe", NULL);
    gchar *dir = self ? g_path_get_dirname(self) : g_strdup("/usr/libexec/ekushe");
    gchar *window = g_build_filename(dir, "ekushe-studio-window", NULL);
    g_free(self);
    g_free(dir);
    gchar *argv[] = {window, url, (gchar *)(tab ? tab : ""), NULL};
    GPid pid;
    if (g_file_test(window, G_FILE_TEST_IS_EXECUTABLE) &&
        g_spawn_async(NULL, argv, NULL, G_SPAWN_DO_NOT_REAP_CHILD | G_SPAWN_STDOUT_TO_DEV_NULL | G_SPAWN_STDERR_TO_DEV_NULL,
                      NULL, NULL, &pid, NULL)) {
        g_child_watch_add_full(G_PRIORITY_DEFAULT, pid, studio_window_done, url, g_free);
        url = NULL;
    } else {
        open_in_browser(url);
    }
    g_free(window);
    g_free(url);
}

static void ekushe_property_activate(IBusEngine *e, const gchar *name, guint state)
{
    (void)state;
    if (g_strcmp0(name, "studio") == 0)
        open_studio(NULL);
    else if (g_strcmp0(name, "settings") == 0)
        open_studio("settings");
    else if (g_strcmp0(name, "report") == 0) {
        gchar *os = g_get_os_info(G_OS_INFO_KEY_PRETTY_NAME);
        gchar *url = take(ek_report_url(os ? os : "Linux"));
        g_free(os);
        if (url) {
            open_in_browser(url);
            g_free(url);
        }
    }
    else if (g_strcmp0(name, "InputMode") == 0) {
        finish((EkusheEngine *)e);
        ek_toggle_language();
        show_language((EkusheEngine *)e);
        show_switch_note((EkusheEngine *)e);
    }
}

static void ekushe_engine_init(EkusheEngine *ek)
{
    ek->table = ibus_lookup_table_new(9, 0, TRUE, FALSE);
    g_object_ref_sink(ek->table);
    ibus_lookup_table_set_orientation(ek->table, IBUS_ORIENTATION_HORIZONTAL);

    ek->props = ibus_prop_list_new();
    g_object_ref_sink(ek->props);
    ibus_prop_list_append(ek->props, ibus_property_new("studio", PROP_TYPE_NORMAL,
        ibus_text_new_from_string("Open Ekushe Studio"), "accessories-text-editor",
        ibus_text_new_from_string("Writing pad, conjunct guide, spell check, Bijoy converter"),
        TRUE, TRUE, PROP_STATE_UNCHECKED, NULL));
    ibus_prop_list_append(ek->props, ibus_property_new("settings", PROP_TYPE_NORMAL,
        ibus_text_new_from_string("Settings"), "preferences-system",
        ibus_text_new_from_string("Typing mode, layout, snippets"), TRUE, TRUE, PROP_STATE_UNCHECKED, NULL));
    ek->mode_prop = ibus_property_new("InputMode", PROP_TYPE_NORMAL,
        ibus_text_new_from_string("বাংলা"), NULL,
        ibus_text_new_from_string("Double-tap Shift to switch between Bangla and English"), TRUE, TRUE, PROP_STATE_UNCHECKED, NULL);
    ibus_property_set_symbol(ek->mode_prop, ibus_text_new_from_string("এ"));
    g_object_ref_sink(ek->mode_prop);
    ibus_prop_list_append(ek->props, ek->mode_prop);
    ibus_prop_list_append(ek->props, ibus_property_new("report", PROP_TYPE_NORMAL,
        ibus_text_new_from_string("Report a problem"), "dialog-warning",
        ibus_text_new_from_string("Tell us what went wrong (opens ekushenext.com)"), TRUE, TRUE, PROP_STATE_UNCHECKED, NULL));
}

static void ekushe_engine_finalize(GObject *obj)
{
    EkusheEngine *ek = (EkusheEngine *)obj;
    g_clear_object(&ek->table);
    g_clear_object(&ek->props);
    g_clear_object(&ek->mode_prop);
    g_free(ek->last_committed);
    G_OBJECT_CLASS(ekushe_engine_parent_class)->finalize(obj);
}

static void ekushe_engine_class_init(EkusheEngineClass *klass)
{
    IBusEngineClass *engine = IBUS_ENGINE_CLASS(klass);
    G_OBJECT_CLASS(klass)->finalize = ekushe_engine_finalize;
    engine->process_key_event = ekushe_process_key_event;
    engine->focus_in = ekushe_focus_in;
    engine->focus_out = ekushe_focus_out;
    engine->reset = ekushe_reset;
    engine->enable = ekushe_enable;
    engine->disable = ekushe_disable;
    engine->candidate_clicked = ekushe_candidate_clicked;
    engine->property_activate = ekushe_property_activate;
    engine->set_content_type = ekushe_set_content_type;
}

static void on_disconnected(IBusBus *bus, gpointer data)
{
    (void)bus;
    (void)data;
    ek_save_learned();
    ibus_quit();
}

int main(int argc, char **argv)
{
    gboolean ibus_mode = argc > 1 && strcmp(argv[1], "--ibus") == 0;
    if (argc > 1 && strcmp(argv[1], "--version") == 0) {
        g_print("ibus-engine-ekushe " EKUSHE_VERSION "\n");
        return 0;
    }
    ibus_init();
    ek_init();
    ek_use_start_language(); /* English at login unless the user chose Bangla */
    IBusBus *bus = ibus_bus_new();
    if (!ibus_bus_is_connected(bus)) {
        g_printerr("ibus-engine-ekushe: IBus is not running\n");
        return 1;
    }
    g_signal_connect(bus, "disconnected", G_CALLBACK(on_disconnected), NULL);
    IBusFactory *factory = ibus_factory_new(ibus_bus_get_connection(bus));
    ibus_factory_add_engine(factory, "ekushe", EKUSHE_TYPE_ENGINE);
    if (ibus_mode) {
        ibus_bus_request_name(bus, "org.freedesktop.IBus.Ekushe", 0);
    } else {
        /* Started by hand (development): register the component ourselves */
        IBusComponent *c = ibus_component_new("org.freedesktop.IBus.Ekushe", "Ekushe Bangla", EKUSHE_VERSION,
            "MPL-2.0", "Ekushe Next Contributors", "https://github.com/mahbub-khan25/ekushenext", "", "ekushe");
        ibus_component_add_engine(c, ibus_engine_desc_new("ekushe", "Ekushe Bangla", "Bangla phonetic typing",
            "bn", "MPL-2.0", "Ekushe Next Contributors", "", "us"));
        ibus_bus_register_component(bus, c);
    }
    ibus_main();
    return 0;
}
