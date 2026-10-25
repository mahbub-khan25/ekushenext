/* Ekushe Studio's own window on Linux (GNOME, KDE Plasma, COSMIC, Xfce, Cinnamon, MATE, ...):
 * a GTK window showing Studio's page from 127.0.0.1 with WebKitGTK, instead of a browser tab.
 *
 *   ekushe-studio-window URL [TAB]
 *
 * One window at a time: starting it again brings the open window forward (and opens TAB there).
 * Links to other websites open in the default browser; the window only ever shows Studio.
 * Downloads (Studio's backup file) ask where to save. Exit status 3 means no window could be
 * shown (no display); the caller then falls back to the browser. */
#include <gtk/gtk.h>
#include <webkit2/webkit2.h>
#include <string.h>

#define APP_ID "com.cicadama.EkusheStudio"
#ifndef EKUSHE_VERSION
#define EKUSHE_VERSION "dev"
#endif

static gchar *studio_origin; /* "http://127.0.0.1:PORT" */
static gchar *start_url;
static GtkWindow *window;
static WebKitWebView *view;

static gboolean is_studio(const gchar *uri)
{
    return uri && g_str_has_prefix(uri, studio_origin) &&
           (uri[strlen(studio_origin)] == '/' || uri[strlen(studio_origin)] == '\0');
}

static void open_outside(const gchar *uri)
{
    if (!uri || !(g_str_has_prefix(uri, "https://") || g_str_has_prefix(uri, "http://")))
        return;
    if (!g_app_info_launch_default_for_uri(uri, NULL, NULL)) {
        gchar *argv[] = {"xdg-open", (gchar *)uri, NULL};
        g_spawn_async(NULL, argv, NULL, G_SPAWN_SEARCH_PATH | G_SPAWN_STDOUT_TO_DEV_NULL | G_SPAWN_STDERR_TO_DEV_NULL,
                      NULL, NULL, NULL, NULL);
    }
}

/* Studio stays in the window; everything else goes to the browser. */
static gboolean decide_policy(WebKitWebView *v, WebKitPolicyDecision *decision, WebKitPolicyDecisionType type, gpointer data)
{
    (void)v; (void)data;
    if (type == WEBKIT_POLICY_DECISION_TYPE_NAVIGATION_ACTION ||
        type == WEBKIT_POLICY_DECISION_TYPE_NEW_WINDOW_ACTION) {
        WebKitNavigationAction *action =
            webkit_navigation_policy_decision_get_navigation_action(WEBKIT_NAVIGATION_POLICY_DECISION(decision));
        const gchar *uri = webkit_uri_request_get_uri(webkit_navigation_action_get_request(action));
        if (g_str_has_prefix(uri, "blob:") || g_str_has_prefix(uri, "data:") || g_str_has_prefix(uri, "about:"))
            return FALSE; /* downloads and the page's own content */
        if (type == WEBKIT_POLICY_DECISION_TYPE_NEW_WINDOW_ACTION || !is_studio(uri)) {
            open_outside(uri);
            webkit_policy_decision_ignore(decision);
            return TRUE;
        }
    }
    return FALSE;
}

/* Save Studio's downloads (the backup file) where the user chooses. */
static gboolean decide_destination(WebKitDownload *download, const gchar *suggested, gpointer data)
{
    (void)data;
    GtkFileChooserNative *chooser = gtk_file_chooser_native_new(
        "ব্যাকআপ সংরক্ষণ করুন · Save", window, GTK_FILE_CHOOSER_ACTION_SAVE, "_Save", "_Cancel");
    GtkFileChooser *fc = GTK_FILE_CHOOSER(chooser);
    gtk_file_chooser_set_do_overwrite_confirmation(fc, TRUE);
    const gchar *dl = g_get_user_special_dir(G_USER_DIRECTORY_DOWNLOAD);
    gtk_file_chooser_set_current_folder(fc, dl ? dl : g_get_home_dir());
    gtk_file_chooser_set_current_name(fc, suggested && *suggested ? suggested : "ekushe-next-backup.json");
    if (gtk_native_dialog_run(GTK_NATIVE_DIALOG(chooser)) == GTK_RESPONSE_ACCEPT) {
        gchar *uri = gtk_file_chooser_get_uri(fc);
        webkit_download_set_allow_overwrite(download, TRUE);
        webkit_download_set_destination(download, uri);
        g_free(uri);
    } else {
        webkit_download_cancel(download);
    }
    g_object_unref(chooser);
    return TRUE;
}

static void download_started(WebKitWebContext *ctx, WebKitDownload *download, gpointer data)
{
    (void)ctx; (void)data;
    g_signal_connect(download, "decide-destination", G_CALLBACK(decide_destination), NULL);
}

/* Open a tab of Studio in the open window (the page's own tab switcher). */
static void show_tab(const gchar *tab)
{
    if (!tab || !*tab)
        return;
    for (const gchar *c = tab; *c; c++)
        if (!g_ascii_isalnum(*c))
            return; /* tab names are plain words */
    gchar *js = g_strdup_printf("typeof showTab === 'function' && showTab('%s')", tab);
#if WEBKIT_CHECK_VERSION(2, 40, 0)
    webkit_web_view_evaluate_javascript(view, js, -1, NULL, NULL, NULL, NULL, NULL);
#else
    webkit_web_view_run_javascript(view, js, NULL, NULL, NULL);
#endif
    g_free(js);
}

static void build_window(GtkApplication *app)
{
    window = GTK_WINDOW(gtk_application_window_new(app));
    gtk_window_set_title(window, "Ekushe Studio · একুশে স্টুডিও");
    gtk_window_set_default_size(window, 1200, 840);
    if (gtk_icon_theme_has_icon(gtk_icon_theme_get_default(), "ekushe"))
        gtk_window_set_icon_name(window, "ekushe");
    else if (g_file_test("/usr/share/ekushe/ekushe.png", G_FILE_TEST_EXISTS))
        gtk_window_set_icon_from_file(window, "/usr/share/ekushe/ekushe.png", NULL);

    WebKitWebContext *ctx = webkit_web_context_get_default();
    webkit_web_context_set_cache_model(ctx, WEBKIT_CACHE_MODEL_DOCUMENT_VIEWER);
    g_signal_connect(ctx, "download-started", G_CALLBACK(download_started), NULL);

    view = WEBKIT_WEB_VIEW(webkit_web_view_new());
    WebKitSettings *s = webkit_web_view_get_settings(view);
    webkit_settings_set_javascript_can_access_clipboard(s, TRUE);
    webkit_settings_set_enable_developer_extras(s, FALSE);
    webkit_settings_set_allow_file_access_from_file_urls(s, FALSE);
    webkit_settings_set_enable_back_forward_navigation_gestures(s, FALSE);
    g_signal_connect(view, "decide-policy", G_CALLBACK(decide_policy), NULL);

    gtk_container_add(GTK_CONTAINER(window), GTK_WIDGET(view));
    webkit_web_view_load_uri(view, start_url);
    gtk_widget_show_all(GTK_WIDGET(window));
}

/* First start: open the window. Later starts (forwarded here by GTK): bring it forward. */
static int command_line(GApplication *gapp, GApplicationCommandLine *cmd, gpointer data)
{
    (void)data;
    gint argc;
    gchar **argv = g_application_command_line_get_arguments(cmd, &argc);
    if (!window) {
        build_window(GTK_APPLICATION(gapp));
    } else {
        if (argc > 2)
            show_tab(argv[2]);
        gtk_window_present(window);
    }
    g_strfreev(argv);
    return 0;
}

int main(int argc, char **argv)
{
    if (argc > 1 && strcmp(argv[1], "--version") == 0) {
        g_print("ekushe-studio-window %s (WebKitGTK %u.%u)\n", EKUSHE_VERSION,
                webkit_get_major_version(), webkit_get_minor_version());
        return 0;
    }
    if (argc < 2 || !g_str_has_prefix(argv[1], "http://127.0.0.1:")) {
        g_printerr("usage: ekushe-studio-window http://127.0.0.1:PORT/... [TAB]\n");
        return 2;
    }
    /* The dock and window list match this window to ekushe-studio.desktop by this name */
    g_set_prgname("ekushe-studio");
    g_set_application_name("Ekushe Studio");
    if (!gtk_init_check(&argc, &argv))
        return 3; /* no display: the caller opens a browser instead */

    start_url = g_strdup(argv[1]);
    const gchar *rest = argv[1] + strlen("http://127.0.0.1:");
    gsize port_len = strspn(rest, "0123456789");
    studio_origin = g_strdup_printf("http://127.0.0.1:%.*s", (int)port_len, rest);

    GtkApplication *app = gtk_application_new(APP_ID, G_APPLICATION_HANDLES_COMMAND_LINE);
    g_signal_connect(app, "command-line", G_CALLBACK(command_line), NULL);
    int status = g_application_run(G_APPLICATION(app), argc, argv);
    g_object_unref(app);
    return status;
}
