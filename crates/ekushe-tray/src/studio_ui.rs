//! Studio web UI (HTML/CSS/JS) served by the running `ekushe.exe` tray app on 127.0.0.1.
//!
//! Frosted-glass design: translucent panels over a slowly drifting backdrop taken from the
//! Bangladesh flag (green field, red sun) and krishnachura. Light, dark or follow-the-system theme;
//! the glass strength follows the "popup transparency" setting. Bangla interface text uses Hind
//! Siliguri; the user's chosen font (Kalpurush by default) is used for the writing itself.
//!
//! Tabs: writing pad, word X-ray and conjunct builder, conjunct and word guide, spell checker,
//! Bijoy ⇄ Unicode converter, settings. `crates/ekushe-ffi/examples/studio_preview.rs` serves this
//! page on any OS for design work.

pub const STUDIO_HTML: &str = r##"<!DOCTYPE html>
<html lang="bn">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0, viewport-fit=cover">
<meta name="color-scheme" content="light dark">
<title>একুশে স্টুডিও</title>
<link rel="icon" type="image/png" href="/logo.png">
<style>
  @font-face { font-family: 'Kalpurush'; src: url('/fonts/Kalpurush.ttf') format('truetype'); font-display: swap; }
  @font-face { font-family: 'SolaimanLipi'; src: url('/fonts/SolaimanLipi.ttf') format('truetype'); font-display: swap; }
  @font-face { font-family: 'Siyam Rupali'; src: url('/fonts/SiyamRupali.ttf') format('truetype'); font-display: swap; }
  @font-face { font-family: 'Hind Siliguri'; src: url('/fonts/HindSiliguri-Regular.ttf') format('truetype'); font-weight: 400; font-display: swap; }
  @font-face { font-family: 'Hind Siliguri'; src: url('/fonts/HindSiliguri-Bold.ttf') format('truetype'); font-weight: 600 700; font-display: swap; }

  :root {
    --font-ui: 'Segoe UI Variable Text', 'Segoe UI', -apple-system, BlinkMacSystemFont, 'Hind Siliguri', 'Noto Sans Bengali', sans-serif;
    --font-bn: 'Kalpurush', 'SolaimanLipi', 'Hind Siliguri', sans-serif;
    --glass-alpha: 0.5;

    --green: #0B6B4A;
    --forest: #06281D;
    --red: #E0303F;
    --coral: #F26B4F;
    --mustard: #E9B949;
    --mist: #EEF4F1;

    --page: #E6EFEA;
    --ink: #0E241B;
    --ink-soft: #4C6359;
    --ink-faint: #7C9089;
    --accent: var(--green);
    --accent-ink: #FFFFFF;
    --accent-soft: rgba(11, 107, 74, 0.12);
    --danger: #C4202E;
    --glass-rgb: 255, 255, 255;
    --etch: rgba(255, 255, 255, 0.30);
    --etch-strong: rgba(255, 255, 255, 0.58);
    --line: rgba(14, 36, 27, 0.10);
    --line-bright: rgba(255, 255, 255, 0.75);
    --shadow: 0 24px 60px -28px rgba(6, 40, 29, 0.45);
    --sun: 0.85;
  }
  :root[data-theme="dark"] {
    --page: #04140E;
    --ink: #EAF3EE;
    --ink-soft: #A7BDB3;
    --ink-faint: #71877D;
    --accent: #43C796;
    --accent-ink: #03170F;
    --accent-soft: rgba(67, 199, 150, 0.16);
    --danger: #FF6B75;
    --glass-rgb: 14, 30, 24;
    --etch: rgba(255, 255, 255, 0.05);
    --etch-strong: rgba(255, 255, 255, 0.10);
    --line: rgba(255, 255, 255, 0.08);
    --line-bright: rgba(255, 255, 255, 0.12);
    --shadow: 0 24px 60px -24px rgba(0, 0, 0, 0.7);
    --sun: 0.55;
  }
  @media (prefers-color-scheme: dark) {
    :root:not([data-theme="light"]) {
      --page: #04140E;
      --ink: #EAF3EE;
      --ink-soft: #A7BDB3;
      --ink-faint: #71877D;
      --accent: #43C796;
      --accent-ink: #03170F;
      --accent-soft: rgba(67, 199, 150, 0.16);
      --danger: #FF6B75;
      --glass-rgb: 14, 30, 24;
      --etch: rgba(255, 255, 255, 0.05);
      --etch-strong: rgba(255, 255, 255, 0.10);
      --line: rgba(255, 255, 255, 0.08);
      --line-bright: rgba(255, 255, 255, 0.12);
      --shadow: 0 24px 60px -24px rgba(0, 0, 0, 0.7);
      --sun: 0.55;
    }
  }

  * { box-sizing: border-box; margin: 0; padding: 0; }
  html, body { height: 100%; }
  body {
    font-family: var(--font-ui);
    font-size: 15px;
    line-height: 1.55;
    color: var(--ink);
    background: var(--page);
    -webkit-font-smoothing: antialiased;
    overflow-x: hidden;
  }
  button, input, select, textarea { font: inherit; color: inherit; }
  button { cursor: pointer; background: none; border: 0; }
  :focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }

  /* ── Backdrop: the flag's red sun and green field, plus krishnachura, drifting behind the glass ── */
  .backdrop { position: fixed; inset: 0; z-index: 0; overflow: hidden; pointer-events: none; }
  .blob { position: absolute; border-radius: 50%; filter: blur(70px); will-change: transform; }
  .blob.sun { width: 46vmax; height: 46vmax; left: 52%; top: -8%; background: radial-gradient(circle, var(--red) 0%, rgba(224,48,63,0.6) 45%, transparent 70%); opacity: var(--sun); mix-blend-mode: normal; animation: drift-a 38s ease-in-out infinite alternate; }
  .blob.field { width: 70vmax; height: 50vmax; left: -18%; bottom: -22%; background: radial-gradient(ellipse, #0F8A5F 0%, rgba(11,107,74,0.55) 50%, transparent 72%); animation: drift-b 46s ease-in-out infinite alternate; }
  .blob.flower { width: 30vmax; height: 30vmax; left: 8%; top: 4%; background: radial-gradient(circle, var(--coral) 0%, transparent 68%); opacity: 0.55; animation: drift-c 52s ease-in-out infinite alternate; }
  .blob.light { width: 34vmax; height: 34vmax; right: -10%; bottom: 2%; background: radial-gradient(circle, var(--mustard) 0%, transparent 66%); opacity: 0.45; animation: drift-a 60s ease-in-out infinite alternate-reverse; }
  :root[data-theme="dark"] .blob.field { background: radial-gradient(ellipse, #0B6B4A 0%, rgba(6,40,29,0.7) 55%, transparent 75%); }
  :root[data-theme="dark"] .blob.light { opacity: 0.18; }
  :root[data-theme="dark"] .blob.flower { opacity: 0.3; }
  @media (prefers-color-scheme: dark) {
    :root:not([data-theme="light"]) .blob.field { background: radial-gradient(ellipse, #0B6B4A 0%, rgba(6,40,29,0.7) 55%, transparent 75%); }
    :root:not([data-theme="light"]) .blob.light { opacity: 0.18; }
    :root:not([data-theme="light"]) .blob.flower { opacity: 0.3; }
  }
  @keyframes drift-a { to { transform: translate(-9vmax, 7vmax) scale(1.08); } }
  @keyframes drift-b { to { transform: translate(8vmax, -6vmax) scale(1.05); } }
  @keyframes drift-c { to { transform: translate(6vmax, 9vmax) scale(0.92); } }
  @media (prefers-reduced-motion: reduce) { .blob { animation: none; } }

  /* ── Glass ── */
  .glass {
    background: rgba(var(--glass-rgb), var(--glass-alpha));
    -webkit-backdrop-filter: blur(30px) saturate(170%);
    backdrop-filter: blur(30px) saturate(170%);
    border: 1px solid var(--line-bright);
    box-shadow: inset 0 1px 0 var(--line-bright), var(--shadow);
  }

  .app {
    position: relative; z-index: 1;
    display: grid;
    grid-template-columns: 232px minmax(0, 1fr);
    gap: 18px;
    max-width: 1380px;
    min-height: 100vh;
    margin: 0 auto;
    padding: 18px;
  }

  /* ── Side rail ── */
  .rail {
    position: sticky; top: 18px;
    height: calc(100vh - 36px);
    border-radius: 26px;
    padding: 18px 12px 14px;
    display: flex; flex-direction: column; gap: 4px;
  }
  .brand { display: flex; align-items: center; gap: 11px; padding: 2px 8px 18px; }
  .brand img { width: 38px; height: 38px; border-radius: 11px; }
  .brand-name { font-family: 'Hind Siliguri', var(--font-ui); font-weight: 700; font-size: 1.12rem; line-height: 1.1; }
  .brand-sub { font-size: 0.78rem; color: var(--ink-faint); }
  .nav-btn {
    display: flex; align-items: center; gap: 12px;
    width: 100%; padding: 9px 12px;
    border-radius: 14px;
    text-align: left;
    color: var(--ink-soft);
    transition: background-color .15s ease, color .15s ease;
  }
  .nav-btn svg { width: 20px; height: 20px; flex: none; }
  .nav-btn .lbl { display: flex; flex-direction: column; line-height: 1.2; }
  .nav-btn .bn { font-family: 'Hind Siliguri', var(--font-ui); font-weight: 600; font-size: 0.97rem; }
  .nav-btn .en { font-size: 0.74rem; color: var(--ink-faint); }
  .nav-btn:hover { background: var(--etch); color: var(--ink); }
  .nav-btn.active { background: var(--etch-strong); color: var(--ink); box-shadow: inset 0 0 0 1px var(--line); }
  .nav-btn.active svg { color: var(--accent); }
  .rail-foot { margin-top: auto; padding: 12px 8px 2px; border-top: 1px solid var(--line); font-size: 0.8rem; color: var(--ink-faint); }
  .rail-foot b { color: var(--ink-soft); font-weight: 600; }
  .about-card { display: flex; gap: 8px; align-items: center; width: 100%; margin-top: 12px; padding: 10px 4px; border: 0; border-top: 1px solid var(--line); background: none; color: var(--ink-faint); font: inherit; font-size: 0.72rem; line-height: 1.45; text-align: left; cursor: pointer; border-radius: 0 0 12px 12px; }
  .about-card:hover, .about-card:focus-visible { background: var(--accent-soft); outline: none; }
  .about-card img { width: 26px; height: 26px; border-radius: 9px; flex: none; }
  .about-card b { color: var(--ink-soft); font-weight: 600; font-size: 0.82rem; }
  .about-card > span { min-width: 0; white-space: nowrap; }
  .about-dlg { margin: auto; position: fixed; inset: 0; height: fit-content; width: min(420px, 92vw); border: 1px solid var(--line); border-radius: 18px; padding: 26px 24px 20px; background: var(--page); color: var(--ink); font: inherit; text-align: center; line-height: 1.7; }
  .about-dlg::backdrop { background: rgba(0, 0, 0, .35); }
  .about-dlg img { width: 72px; height: 72px; border-radius: 18px; }
  .about-dlg h3 { margin: 10px 0 0; font-size: 1.35rem; }
  .about-dlg .ver { color: var(--ink-faint); font-size: .85rem; }
  .about-dlg dl { display: grid; grid-template-columns: auto 1fr; gap: 6px 14px; margin: 18px 0 8px; text-align: left; }
  .about-dlg dt { color: var(--ink-faint); }
  .about-dlg dd { margin: 0; }
  .about-dlg a { color: var(--accent); }

  /* ── Main sheet ── */
  .sheet { border-radius: 30px; padding: 22px 28px 30px; min-width: 0; }
  .topbar { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; margin-bottom: 20px; }
  .page-title h1 { font-family: 'Hind Siliguri', var(--font-ui); font-size: 1.7rem; font-weight: 700; line-height: 1.2; letter-spacing: -0.01em; }
  .page-title p { color: var(--ink-soft); font-size: 0.92rem; margin-top: 2px; max-width: 62ch; }
  .top-actions { display: flex; align-items: center; gap: 8px; flex: none; }

  .lang-switch { display: inline-flex; padding: 3px; border-radius: 999px; background: var(--etch); box-shadow: inset 0 0 0 1px var(--line); }
  .lang-switch span { padding: 5px 14px; border-radius: 999px; font-weight: 600; font-size: 0.88rem; color: var(--ink-soft); transition: all .18s ease; }
  .lang-switch.bangla .opt-bn, .lang-switch.english .opt-en { background: var(--accent); color: var(--accent-ink); box-shadow: 0 4px 14px -6px var(--accent); }
  .icon-btn { width: 36px; height: 36px; border-radius: 12px; display: grid; place-items: center; color: var(--ink-soft); background: var(--etch); box-shadow: inset 0 0 0 1px var(--line); }
  .icon-btn:hover { color: var(--ink); background: var(--etch-strong); }
  .icon-btn svg { width: 18px; height: 18px; }

  .panel { display: none; }
  .panel.active { display: block; animation: rise .22s ease; }
  @keyframes rise { from { opacity: 0; transform: translateY(4px); } }
  @media (prefers-reduced-motion: reduce) { .panel.active { animation: none; } }

  /* ── Building blocks ── */
  [hidden] { display: none !important; }
  .etched { background: var(--etch); border-radius: 20px; box-shadow: inset 0 0 0 1px var(--line); padding: 16px 18px; }
  .section-title { font-family: 'Hind Siliguri', var(--font-ui); font-weight: 600; font-size: 1.05rem; margin-bottom: 2px; }
  .hint { color: var(--ink-soft); font-size: 0.86rem; }
  .stack { display: grid; gap: 14px; }
  .row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
  .grow { flex: 1; min-width: 0; }

  .field, textarea, select, input[type="text"] {
    width: 100%;
    background: var(--etch-strong);
    border: 1px solid var(--line);
    border-radius: 14px;
    padding: 11px 14px;
    outline: none;
    transition: border-color .15s ease, box-shadow .15s ease;
  }
  textarea { resize: vertical; font-family: var(--font-bn); font-size: 1.25rem; line-height: 1.7; }
  select { appearance: none; padding-right: 36px; background-image: linear-gradient(45deg, transparent 50%, var(--ink-soft) 50%), linear-gradient(135deg, var(--ink-soft) 50%, transparent 50%); background-position: calc(100% - 19px) 50%, calc(100% - 14px) 50%; background-size: 5px 5px; background-repeat: no-repeat; }
  textarea:focus, select:focus, input[type="text"]:focus { border-color: var(--accent); box-shadow: 0 0 0 4px var(--accent-soft); }
  ::placeholder { color: var(--ink-faint); }

  .btn { display: inline-flex; align-items: center; gap: 7px; padding: 9px 16px; border-radius: 12px; font-weight: 600; font-size: 0.92rem; background: var(--etch-strong); box-shadow: inset 0 0 0 1px var(--line); transition: transform .1s ease, background-color .15s ease; }
  .btn:hover { background: rgba(var(--glass-rgb), 0.9); }
  .btn:active { transform: scale(0.98); }
  .btn.primary { background: var(--accent); color: var(--accent-ink); box-shadow: 0 8px 22px -12px var(--accent); }
  .btn svg { width: 16px; height: 16px; }

  .chip { display: inline-flex; align-items: center; gap: 6px; padding: 5px 12px; border-radius: 999px; font-size: 0.88rem; background: var(--etch); box-shadow: inset 0 0 0 1px var(--line); color: var(--ink-soft); transition: background-color .12s ease, color .12s ease; white-space: nowrap; }
  .chip:hover { color: var(--ink); background: var(--etch-strong); }
  .chip.primary { background: var(--ink); color: var(--page); box-shadow: none; }
  .chip-bar { display: flex; flex-wrap: wrap; gap: 6px; }
  .scroll-x { flex-wrap: nowrap; overflow-x: auto; padding-bottom: 4px; scrollbar-width: thin; }
  kbd, code { font-family: var(--font-ui); font-size: 0.86em; padding: 1px 7px; border-radius: 7px; background: var(--etch-strong); box-shadow: inset 0 -1px 0 var(--line), inset 0 0 0 1px var(--line); color: var(--ink); font-weight: 600; }

  /* ── Writing pad ── */
  .composer { padding: 18px; border-radius: 24px; }
  .roman-wrap { position: relative; }
  .roman-wrap svg { position: absolute; left: 16px; top: 50%; transform: translateY(-50%); width: 20px; height: 20px; color: var(--ink-faint); }
  #romanInput { font-size: 1.15rem; padding: 14px 16px 14px 46px; border-radius: 16px; }
  .candidates { display: flex; flex-wrap: wrap; gap: 6px; min-height: 44px; margin-top: 12px; }
  .cand { display: inline-flex; align-items: baseline; gap: 6px; padding: 6px 14px 7px; border-radius: 14px; font-family: var(--font-bn); font-size: 1.22rem; background: var(--etch-strong); box-shadow: inset 0 0 0 1px var(--line); }
  .cand .n { font-family: var(--font-ui); font-size: 0.72rem; color: var(--ink-faint); font-weight: 600; }
  .cand .tag { font-family: var(--font-ui); font-size: 0.7rem; color: var(--ink-faint); }
  .cand:hover { background: rgba(var(--glass-rgb), 0.95); }
  .cand.top { background: var(--accent); color: var(--accent-ink); box-shadow: 0 10px 24px -14px var(--accent); }
  .cand.top .n, .cand.top .tag { color: inherit; opacity: 0.75; }
  .cand-empty { color: var(--ink-faint); font-size: 0.88rem; align-self: center; }
  .hints { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 8px; }
  .hints:empty { display: none; }
  .hint-chip { display: inline-flex; align-items: baseline; gap: 6px; padding: 3px 10px; border-radius: 10px; font-size: 0.8rem; color: var(--ink-soft); background: var(--etch); }
  .hint-chip b { font-family: var(--font-bn); font-size: 1.02rem; color: var(--ink); font-weight: 400; }
  .hint-chip:hover { background: var(--etch-strong); }
  .tips { display: flex; flex-wrap: wrap; gap: 6px 18px; margin-top: 12px; font-size: 0.82rem; color: var(--ink-faint); }

  .pad-card { margin-top: 14px; border-radius: 24px; padding: 6px; }
  .toolbar { display: flex; flex-wrap: wrap; gap: 2px; padding: 4px 6px 6px; }
  .tool { display: inline-flex; align-items: center; gap: 6px; padding: 6px 10px; border-radius: 10px; font-size: 0.84rem; color: var(--ink-soft); }
  .tool svg { width: 16px; height: 16px; }
  .tool:hover { background: var(--etch-strong); color: var(--ink); }
  #banglaPad { min-height: 190px; border-radius: 18px; font-size: 1.4rem; padding: 16px 18px; background: rgba(var(--glass-rgb), 0.55); }
  .pad-actions { display: flex; justify-content: space-between; align-items: center; gap: 8px; flex-wrap: wrap; padding: 10px 8px 6px; }
  .count { font-size: 0.8rem; color: var(--ink-faint); }

  .letters-head { display: flex; align-items: center; justify-content: space-between; gap: 12px; flex-wrap: wrap; margin: 26px 0 12px; }
  .seg { display: inline-flex; padding: 3px; border-radius: 12px; background: var(--etch); box-shadow: inset 0 0 0 1px var(--line); flex-wrap: wrap; }
  .seg .chip { background: transparent; box-shadow: none; border-radius: 9px; }
  .seg .chip.primary { background: var(--etch-strong); color: var(--ink); box-shadow: 0 1px 3px rgba(0,0,0,0.08), inset 0 0 0 1px var(--line); }
  .quick-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(92px, 1fr)); gap: 8px; }
  .sign-btn { display: flex; flex-direction: column; align-items: center; gap: 1px; padding: 10px 6px 9px; border-radius: 16px; background: var(--etch); box-shadow: inset 0 0 0 1px var(--line); transition: transform .12s ease, background-color .12s ease; }
  .sign-btn:hover { transform: translateY(-2px); background: var(--etch-strong); }
  .sign-btn .bn { font-family: var(--font-bn); font-size: 1.65rem; line-height: 1.25; }
  .sign-btn .rm { font-size: 0.78rem; font-weight: 600; color: var(--accent); }
  .sign-btn .dc { font-size: 0.7rem; color: var(--ink-faint); text-align: center; }

  /* ── X-ray ── */
  .two-col { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; }
  .xray-tokens { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 14px; }
  .xray-token { display: flex; flex-direction: column; align-items: center; padding: 10px 12px 8px; border-radius: 16px; background: var(--etch-strong); box-shadow: inset 0 0 0 1px var(--line); min-width: 64px; }
  .xt-char { font-family: var(--font-bn); font-size: 1.9rem; line-height: 1.2; }
  .xt-key { font-size: 0.78rem; font-weight: 600; color: var(--accent); }
  .marks-card { margin: 0 0 14px; padding: 12px 14px; border-radius: 14px; background: var(--accent-soft); box-shadow: inset 0 0 0 1px var(--line); }
  .marks-card h3 { margin: 0 0 8px; font-size: 0.98rem; }
  .marks-card .sub { margin: 0 0 10px; font-size: 0.85rem; color: var(--ink-soft); }
  .marks-join { display: grid; grid-template-columns: repeat(auto-fit, minmax(170px, 1fr)); gap: 8px; margin: 0 0 12px; }
  .marks-join div { padding: 9px 11px; border-radius: 10px; background: var(--etch); box-shadow: inset 0 0 0 1px var(--line); font-size: 0.88rem; }
  .marks-join b { display: block; font-size: 1.25rem; margin-bottom: 2px; color: var(--accent); }
  .marks-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 8px; }
  .mark-item { display: flex; align-items: center; gap: 10px; padding: 7px 12px; border-radius: 10px; background: var(--etch); font-size: 0.9rem; white-space: nowrap; }
  .mark-item b { min-width: 2.4em; text-align: center; font-size: 1.25rem; color: var(--accent); }
  .mark-item kbd { margin-right: 4px; }
  .mark-item span { display: block; margin-top: 2px; color: var(--ink-soft); font-size: 0.8rem; }
  .xt-name { font-size: 0.7rem; color: var(--ink-faint); }
  #xraySummary { margin-top: 12px; color: var(--ink-soft); font-size: 0.92rem; }
  .matrix-label { font-size: 0.82rem; color: var(--ink-soft); margin: 12px 0 6px; }
  .consonant-matrix { display: grid; grid-template-columns: repeat(auto-fill, minmax(38px, 1fr)); gap: 5px; }
  .c-btn { padding: 4px 0; border-radius: 10px; font-family: var(--font-bn); font-size: 1.2rem; text-align: center; background: var(--etch); cursor: pointer; transition: background-color .12s ease; }
  .c-btn:hover { background: var(--etch-strong); }
  .c-btn.selected { background: var(--accent); color: var(--accent-ink); }
  .fusion-preview { display: flex; align-items: center; justify-content: space-between; gap: 14px; flex-wrap: wrap; margin-top: 16px; padding: 14px 16px; border-radius: 18px; background: var(--etch-strong); box-shadow: inset 0 0 0 1px var(--line); }
  #fusionFormula { font-family: var(--font-bn); color: var(--ink-faint); font-size: 1rem; }
  .fusion-big { font-family: var(--font-bn); font-size: 3rem; line-height: 1.1; }
  #fusionRoman { font-size: 0.86rem; color: var(--accent); font-weight: 600; }

  /* ── Guide ── */
  .search-wrap { position: relative; margin: 12px 0 14px; }
  .search-wrap svg { position: absolute; left: 15px; top: 50%; transform: translateY(-50%); width: 18px; height: 18px; color: var(--ink-faint); }
  #signSearch { padding-left: 44px; }
  #signsToast { min-height: 1.4em; font-size: 0.86rem; color: var(--accent); font-weight: 600; }
  .atlas-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 10px; }
  .atlas-card { padding: 14px; border-radius: 20px; background: var(--etch); box-shadow: inset 0 0 0 1px var(--line); display: flex; flex-direction: column; gap: 10px; }
  .atlas-card-top { display: flex; align-items: center; gap: 12px; cursor: pointer; }
  .atlas-glyph { font-family: var(--font-bn); font-size: 2rem; min-width: 64px; height: 58px; display: grid; place-items: center; border-radius: 16px; background: var(--etch-strong); }
  .atlas-meta { font-size: 0.86rem; color: var(--ink-soft); }
  .atlas-meta .key { color: var(--accent); font-weight: 700; font-size: 0.95rem; }
  .series-tag { font-size: 0.72rem; color: var(--ink-faint); margin-left: 6px; }
  .word-list { display: flex; flex-wrap: wrap; gap: 5px; }
  .word-pill { display: inline-flex; align-items: baseline; gap: 5px; padding: 3px 9px; border-radius: 10px; background: var(--etch-strong); cursor: pointer; font-size: 0.78rem; }
  .word-pill:hover { box-shadow: inset 0 0 0 1px var(--accent); }
  .word-pill .wb { font-family: var(--font-bn); font-size: 1rem; }
  .word-pill .wr { color: var(--accent); font-weight: 600; }
  .word-pill .wm { color: var(--ink-faint); }

  /* ── Spell ── */
  .issue { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; padding: 12px 14px; border-radius: 16px; background: var(--etch); box-shadow: inset 0 0 0 1px var(--line); }
  .issue .wrong { font-family: var(--font-bn); font-size: 1.2rem; color: var(--danger); text-decoration: underline wavy; text-underline-offset: 5px; }
  .issue .kind { font-size: 0.78rem; color: var(--ink-faint); }
  .issue .fix { font-family: var(--font-bn); font-size: 1.05rem; }
  .ok-note { display: flex; align-items: center; gap: 8px; color: var(--accent); font-weight: 600; }

  /* ── Settings ── */
  .settings-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 14px; }
  .setting { display: grid; gap: 6px; margin-top: 12px; }
  .setting > label { font-weight: 600; font-size: 0.9rem; }
  .setting small, .toggle small { color: var(--ink-faint); font-size: 0.8rem; }
  .theme-seg .chip { padding: 6px 14px; }
  input[type="range"] { width: 100%; accent-color: var(--accent); }
  .toggle { display: flex; align-items: flex-start; justify-content: space-between; gap: 14px; padding: 12px 0; border-top: 1px solid var(--line); cursor: pointer; }
  .toggle:first-of-type { border-top: 0; }
  .toggle span { display: grid; gap: 1px; }
  .toggle b { font-weight: 600; font-size: 0.92rem; }
  .switch { position: relative; flex: none; width: 42px; height: 24px; }
  .switch input { position: absolute; opacity: 0; inset: 0; cursor: pointer; }
  .switch i { position: absolute; inset: 0; border-radius: 999px; background: var(--etch-strong); box-shadow: inset 0 0 0 1px var(--line); transition: background-color .18s ease; }
  .switch i::after { content: ""; position: absolute; top: 3px; left: 3px; width: 18px; height: 18px; border-radius: 50%; background: #fff; box-shadow: 0 1px 3px rgba(0,0,0,0.25); transition: transform .18s ease; }
  .switch input:checked + i { background: var(--accent); }
  .switch input:checked + i::after { transform: translateX(18px); }
  .switch input:focus-visible + i { outline: 2px solid var(--accent); outline-offset: 2px; }
  .font-sample { font-family: var(--font-bn); font-size: 1.35rem; padding: 6px 2px 0; }
  .snip-chip code { margin-right: 2px; }
  .snip-chip b { font-family: var(--font-bn); font-weight: 400; font-size: 1rem; color: var(--ink); }

  /* ── Toast ── */
  .toast { position: fixed; left: 50%; bottom: 28px; z-index: 50; transform: translate(-50%, 20px); opacity: 0; padding: 10px 18px; border-radius: 14px; font-weight: 600; font-size: 0.9rem; pointer-events: none; transition: opacity .2s ease, transform .2s ease; }
  .toast.show { opacity: 1; transform: translate(-50%, 0); }

  /* ── Small screens: the rail becomes a bottom tab bar ── */
  @media (max-width: 1020px) { .two-col { grid-template-columns: 1fr; } }
  @media (max-width: 820px) {
    .app { grid-template-columns: 1fr; padding: 12px 12px 96px; gap: 12px; }
    .rail { position: fixed; z-index: 40; left: 10px; right: 10px; bottom: 10px; top: auto; height: auto; flex-direction: row; justify-content: space-around; padding: 6px; border-radius: 22px; gap: 0; }
    .brand, .rail-foot { display: none; }
    .nav-btn { flex-direction: column; gap: 2px; padding: 7px 2px; width: auto; flex: 1; align-items: center; }
    .nav-btn .lbl { align-items: center; }
    .nav-btn .bn { font-size: 0.7rem; font-weight: 600; text-align: center; }
    .nav-btn .en { display: none; }
    .sheet { padding: 18px 16px 24px; border-radius: 24px; }
    .topbar { flex-direction: column-reverse; align-items: stretch; }
    .top-actions { justify-content: space-between; }
    .page-title h1 { font-size: 1.4rem; }
    .toast { bottom: 96px; }
    .rail { --glass-alpha: 0.82; }
    .nav-btn .bn { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 100%; font-size: 0.66rem; }
    .tips { display: none; }
    .hints .hint-chip:nth-child(n+5) { display: none; }
    .composer { padding: 12px; }
  }
  /* Key map of a fixed layout (and its printed cheat sheet) */
  .ocr-drop { text-align: center; padding: 28px 18px; border: 2px dashed var(--line); }
  .ocr-drop.over { box-shadow: inset 0 0 0 2px var(--accent); }
  .ocr-ico { width: 34px; height: 34px; color: var(--accent); }
  .ocr-preview img { max-width: 100%; max-height: 420px; border-radius: 12px; box-shadow: 0 0 0 1px var(--line); }
  #ocrText { width: 100%; font-size: 1.05rem; line-height: 1.7; }
  .ocr-fix { display: inline-block; margin: 2px 4px 2px 0; padding: 1px 8px; border-radius: 999px; background: var(--etch-strong); cursor: pointer; border: 0; font: inherit; color: var(--ink); }
  .ocr-fix s { color: var(--ink-faint); }
  .welcome-choices { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 10px; margin-top: 12px; }
  .welcome-choice { text-align: left; padding: 14px; border-radius: 14px; border: 1px solid var(--line); background: var(--etch-strong); color: var(--ink); cursor: pointer; font: inherit; display: flex; flex-direction: column; gap: 4px; }
  .welcome-choice:hover, .welcome-choice:focus-visible { box-shadow: inset 0 0 0 2px var(--accent); }
  .welcome-choice span { font-size: .85rem; color: var(--ink-soft); }
  .welcome-list { margin: 10px 0 0; padding-left: 20px; line-height: 1.9; }
  .lay-ed { margin: auto; position: fixed; inset: 0; width: min(1100px, 96vw); height: min(860px, 94vh); border: 1px solid var(--line); border-radius: 16px; padding: 18px 20px; background: var(--page); color: var(--ink); font: inherit; overflow: auto; }
  .lay-ed::backdrop { background: rgba(0, 0, 0, .4); }
  .lay-kb { display: flex; flex-direction: column; gap: 6px; margin: 10px 0; user-select: none; }
  .lay-row { display: flex; gap: 6px; }
  .lay-key { flex: 0 0 auto; width: 62px; height: 58px; border: 1px solid var(--line, rgba(127,127,127,.3)); border-radius: 9px; background: var(--card, rgba(255,255,255,.6)); color: inherit; cursor: pointer; position: relative; font: inherit; padding: 0; }
  .lay-key .k { position: absolute; left: 6px; top: 3px; font-size: .72rem; opacity: .6; font-family: ui-monospace, monospace; }
  .lay-key .n { position: absolute; left: 0; right: 0; bottom: 6px; text-align: center; font-family: var(--font-bn); font-size: 1.25rem; }
  .lay-key .s { position: absolute; right: 6px; top: 2px; font-family: var(--font-bn); font-size: .85rem; opacity: .75; }
  .lay-key.sel { outline: 3px solid var(--accent, #0b6b4f); }
  .lay-key.link { background: rgba(11, 107, 79, .14); }
  .lay-pal { display: flex; flex-wrap: wrap; gap: 4px; margin: 6px 0; }
  .lay-pal button { min-width: 36px; height: 36px; border: 1px solid var(--line, rgba(127,127,127,.3)); border-radius: 8px; background: transparent; color: inherit; font-family: var(--font-bn); font-size: 1.1rem; cursor: pointer; }
  .lay-pal button:hover { background: rgba(11, 107, 79, .12); }
  .don-prog { margin: 4px 0 10px; }
  .don-bar, .don-meter { height: 8px; border-radius: 6px; background: rgba(127,127,127,.18); overflow: hidden; margin-top: 6px; }
  .don-bar i, .don-meter i { display: block; height: 100%; width: 0; background: var(--accent, #0b6b4f); transition: width .2s; }
  .don-meter i { transition: width .06s; }
  .don-meter i.loud { background: #d97706; }
  .don-grid { display: flex; flex-wrap: wrap; gap: 5px; margin: 8px 0 2px; }
  .don-grid button { width: 30px; height: 30px; border-radius: 8px; border: 1px solid var(--line, rgba(127,127,127,.35)); background: transparent; color: inherit; font: inherit; font-size: .8rem; cursor: pointer; }
  .don-grid button.rec { background: rgba(37, 99, 235, .18); border-color: rgba(37, 99, 235, .55); }
  .don-grid button.sent { background: var(--accent, #0b6b4f); color: #fff; border-color: transparent; }
  .don-grid button.cur { outline: 2px solid var(--accent, #0b6b4f); outline-offset: 1px; }
  .ocr-send { margin: auto; position: fixed; inset: 0; height: fit-content; border: 1px solid var(--line); border-radius: 16px; padding: 20px 22px; max-width: 520px; background: var(--page); color: var(--ink); font: inherit; line-height: 1.7; }
  .ocr-send::backdrop { background: rgba(0, 0, 0, .35); }
  .ocr-send h3 { margin: 0 0 8px; }
  .ocr-send ul { margin: 8px 0; padding-left: 20px; }
  .ocr-send .warn { color: var(--danger); }
  .link-btn { background: none; border: 0; padding: 0; color: var(--accent); font: inherit; cursor: pointer; text-decoration: underline; }
  .keymap-back { position: fixed; inset: 0; background: rgba(0,0,0,.45); display: grid; place-items: center; z-index: 50; padding: 16px; }
  .keymap { background: var(--page); color: var(--ink); border-radius: 20px; padding: 18px; max-width: 980px; width: 100%; max-height: 92vh; overflow: auto; box-shadow: 0 30px 80px -30px rgba(0,0,0,.6); }
  .keymap h3 { margin: 0 0 4px; }
  .keymap .kb { display: grid; gap: 6px; margin-top: 12px; }
  .keymap .kb-row { display: flex; gap: 6px; }
  .keymap .key { position: relative; flex: 0 0 auto; width: 62px; height: 62px; border-radius: 10px; background: var(--etch-strong); box-shadow: inset 0 0 0 1px var(--line); }
  .keymap .key .n { position: absolute; left: 0; right: 0; top: 18px; text-align: center; font-size: 1.35rem; }
  .keymap .key .s { position: absolute; left: 6px; top: 3px; font-size: .82rem; color: var(--ink-soft); }
  .keymap .key .a { position: absolute; right: 6px; bottom: 3px; font-size: .78rem; color: var(--accent); }
  .keymap .key .e { position: absolute; right: 6px; top: 3px; font-size: .62rem; color: var(--ink-faint); font-family: system-ui, sans-serif; }
  .keymap .key.link { box-shadow: inset 0 0 0 2px var(--accent); }
  .keymap .key .n.small { font-size: .7rem; top: 24px; font-family: system-ui, sans-serif; color: var(--ink-soft); }
  .keymap .key.link .n { color: var(--accent); font-size: .95rem; top: 21px; }
  .keymap .legend { display: flex; gap: 16px; flex-wrap: wrap; margin-top: 12px; font-size: .85rem; color: var(--ink-soft); }
  @media print {
    body > *:not(.keymap-back) { display: none !important; }
    .keymap-back { position: static; background: none; padding: 0; }
    .keymap { box-shadow: none; max-height: none; overflow: visible; }
    .keymap .no-print { display: none !important; }
  }
</style>
</head>
<body>
<div class="backdrop" aria-hidden="true">
  <div class="blob field"></div><div class="blob flower"></div><div class="blob sun"></div><div class="blob light"></div>
</div>

<svg width="0" height="0" style="position:absolute" aria-hidden="true">
  <defs>
    <symbol id="i-pen" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z"/></symbol>
    <symbol id="i-scan" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M3 7V5a2 2 0 0 1 2-2h2M17 3h2a2 2 0 0 1 2 2v2M21 17v2a2 2 0 0 1-2 2h-2M7 21H5a2 2 0 0 1-2-2v-2"/><path d="M7 12h10"/></symbol>
    <symbol id="i-book" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M4 19.5V5a2 2 0 0 1 2-2h13v16H6.5A2.5 2.5 0 0 0 4 21.5Z"/><path d="M8 7h7M8 11h5"/></symbol>
    <symbol id="i-check" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M4 7h9M4 12h6M4 17h5"/><path d="m14 16 3 3 5-6"/></symbol>
    <symbol id="i-swap" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M7 4 3 8l4 4"/><path d="M3 8h13a4 4 0 0 1 4 4"/><path d="m17 20 4-4-4-4"/><path d="M21 16H8a4 4 0 0 1-4-4"/></symbol>
    <symbol id="i-image" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="16" rx="2.5"/><path d="M7 9h6M7 13h10M7 17h4"/></symbol>
    <symbol id="i-gear" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M4 6h10M18 6h2M4 12h4M12 12h8M4 18h12M20 18h0"/><circle cx="16" cy="6" r="2"/><circle cx="10" cy="12" r="2"/><circle cx="18" cy="18" r="2"/></symbol>
    <symbol id="i-fix" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 1 0 3-6.7"/><path d="M3 4v4h4"/><path d="m9 12 2 2 4-4"/></symbol>
    <symbol id="i-sun" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/></symbol>
    <symbol id="i-moon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5Z"/></symbol>
    <symbol id="i-copy" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="12" height="12" rx="2.5"/><path d="M5 15V5a2 2 0 0 1 2-2h8"/></symbol>
    <symbol id="i-keys" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="6" width="20" height="12" rx="3"/><path d="M6 10h.01M10 10h.01M14 10h.01M18 10h.01M7 14h10"/></symbol>
    <symbol id="i-search" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/></symbol>
    <symbol id="i-spark" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3v4M12 17v4M3 12h4M17 12h4M6 6l2.5 2.5M15.5 15.5 18 18M18 6l-2.5 2.5M8.5 15.5 6 18"/></symbol>
  </defs>
</svg>

<div class="app">
  <nav class="rail glass" aria-label="Studio">
    <div class="brand">
      <img src="/logo.png" alt="">
      <div><div class="brand-name">একুশে</div><div class="brand-sub">Ekushe Next Studio</div></div>
    </div>
    <button class="nav-btn active" data-tab="playground"><svg><use href="#i-pen"/></svg><span class="lbl"><span class="bn">লেখার প্যাড</span><span class="en">Writing pad</span></span></button>
    <button class="nav-btn" data-tab="xray"><svg><use href="#i-scan"/></svg><span class="lbl"><span class="bn">শব্দ এক্স-রে</span><span class="en">Word X-ray</span></span></button>
    <button class="nav-btn" data-tab="signs"><svg><use href="#i-book"/></svg><span class="lbl"><span class="bn">যুক্তবর্ণ</span><span class="en">Conjunct guide</span></span></button>
    <button class="nav-btn" data-tab="spell"><svg><use href="#i-check"/></svg><span class="lbl"><span class="bn">বানান</span><span class="en">Spell check</span></span></button>
    <button class="nav-btn" data-tab="converter"><svg><use href="#i-swap"/></svg><span class="lbl"><span class="bn">রূপান্তর</span><span class="en">Bijoy ⇄ Unicode</span></span></button>
    <button class="nav-btn" data-tab="ocr"><svg><use href="#i-image"/></svg><span class="lbl"><span class="bn">ছবি থেকে লেখা</span><span class="en">Picture to text</span></span></button>
    <button class="nav-btn" data-tab="fix" id="navFix" style="display:none"><svg><use href="#i-fix"/></svg><span class="lbl"><span class="bn">ভুল শব্দ</span><span class="en">A word was wrong</span></span></button>
    <button class="nav-btn" data-tab="settings"><svg><use href="#i-gear"/></svg><span class="lbl"><span class="bn">সেটিংস</span><span class="en">Settings</span></span></button>
    <div class="rail-foot">বাংলা ⇄ English বদলাতে<br><b id="hotkeyBadge">Double-Tap Shift</b>
      <button class="about-card" onclick="document.getElementById('aboutDlg').showModal()" title="একুশে নেক্সট সম্পর্কে (About)">
        <img src="/logo.png" alt="">
        <span><b>একুশে নেক্সট</b><br>সংস্করণ <span class="aboutVer">—</span><br>মাহ্‌বুব আফজাল খান<br>Mahbub Afzal Khan<br>support@ekushenext.com</span>
      </button>
    </div>
  </nav>
  <dialog class="about-dlg" id="aboutDlg" aria-labelledby="aboutTitle">
    <img src="/logo.png" alt="">
    <h3 id="aboutTitle">একুশে নেক্সট</h3>
    <div class="ver">Ekushe Next</div>
    <dl>
      <dt>সংস্করণ<br><small>Version</small></dt><dd class="aboutVer">—</dd>
      <dt>তৈরি করেছেন<br><small>Developed by</small></dt><dd>মাহ্‌বুব আফজাল খান<br>Mahbub Afzal Khan</dd>
      <dt>যোগাযোগ<br><small>Contact</small></dt><dd><a href="mailto:support@ekushenext.com">support@ekushenext.com</a></dd>
      <dt>ওয়েবসাইট<br><small>Website</small></dt><dd><a href="https://ekushenext.com" target="_blank" rel="noopener">ekushenext.com</a></dd>
      <dt>লাইসেন্স<br><small>License</small></dt><dd>ওপেন সোর্স, MPL-2.0</dd>
    </dl>
    <button class="btn" onclick="document.getElementById('aboutDlg').close()">বন্ধ করুন</button>
  </dialog>

  <main class="sheet glass">
    <div class="topbar">
      <div class="page-title"><h1 id="pageTitle">লেখার প্যাড</h1><p id="pageSub">রোমান হরফে লিখুন, একুশে বাংলা করে দেবে।</p></div>
      <div class="top-actions">
        <button class="lang-switch bangla" id="modePill" onclick="toggleMode()" title="বাংলা / English" aria-label="Switch language">
          <span class="opt-bn">বাংলা</span><span class="opt-en">EN</span>
        </button>
        <button class="icon-btn" id="themeBtn" onclick="cycleTheme()" title="Theme" aria-label="Change theme"><svg><use href="#i-sun"/></svg></button>
      </div>
    </div>

    <!-- Windows: another Bangla keyboard program is running next to Ekushe -->
    <div class="etched" id="conflictBox" hidden role="status" style="margin-bottom:16px;border-left:4px solid var(--accent, #c0392b)">
      <div class="section-title">অন্য বাংলা কীবোর্ড চালু আছে</div><div class="hint" id="conflictMsg"></div>
    </div>

    <!-- Linux: the package is installed but Ekushe isn't one of the user's keyboards yet -->
    <div class="etched" id="addKeyboardBox" hidden style="margin-bottom:16px;display:flex;align-items:center;justify-content:space-between;gap:16px;flex-wrap:wrap">
      <div><div class="section-title">একুশে এখনো আপনার কিবোর্ডের তালিকায় নেই</div><div class="hint" id="addKeyboardMsg">যোগ করলে একুশেই আপনার কিবোর্ড হবে, সব অ্যাপে। বাংলা ⇄ English বদলাতে দুবার Shift চাপুন; ওপরের বারে এ বা EN দেখাবে।</div></div>
      <button class="btn primary" id="addKeyboardBtn" onclick="addKeyboard()">কিবোর্ড যোগ করুন</button>
    </div>
    <div class="etched" id="removeKeyboardBox" hidden style="margin-bottom:16px;display:flex;align-items:center;justify-content:space-between;gap:16px;flex-wrap:wrap">
      <div><div class="section-title">একুশে আপনার কিবোর্ডের তালিকায় আছে</div><div class="hint" id="removeKeyboardMsg">একুশে সরিয়ে ফেলার আগে এখান থেকে বাদ দিন: কিবোর্ডের তালিকা ও ইনপুট-মেথড সেটিং আগের মতো হয়ে যাবে। টার্মিনালেও হয়: ekushe-studio --remove-keyboard</div></div>
      <button class="btn" id="removeKeyboardBtn" onclick="removeKeyboard()">কিবোর্ড বাদ দিন</button>
    </div>

    <!-- Writing pad -->
    <section class="panel active" id="tab-playground" data-title="লেখার প্যাড" data-sub="রোমান হরফে লিখুন, একুশে বাংলা করে দেবে। Space চাপলে প্রথম শব্দটি বসবে।">
      <!-- First run: how do you type Bangla? One answer sets the keyboard up for it -->
      <div class="etched welcome" id="welcomeBox" hidden style="margin-bottom:16px">
        <div id="welcomeAsk">
          <div class="section-title">স্বাগতম! আপনি কীভাবে বাংলা লেখেন?</div>
          <div class="hint">একটি বেছে নিন, একুশে সেভাবেই সেজে যাবে। পরে সেটিংসে যেকোনো সময় বদলানো যায়।</div>
          <div class="welcome-choices">
            <button class="welcome-choice" onclick="welcomeChoose('phonetic')"><b>ইংরেজি হরফে, উচ্চারণ করে</b><span>amar sonar bangla → আমার সোনার বাংলা (অভ্রের মতো)</span></button>
            <button class="welcome-choice" onclick="welcomeChoose('fixed')"><b>নির্দিষ্ট কী-বোর্ড লেআউটে, মুখস্থ হাতে</b><span>জাতীয় লেআউট, অথবা আপনার নিজের লেআউট ফাইল; লেখা হয় ইউনিকোডে</span></button>
            <button class="welcome-choice" onclick="welcomeChoose('new')"><b>এখনো শিখিনি</b><span>সবচেয়ে সহজ পথ: যেমন বলেন, তেমন লিখুন</span></button>
          </div>
          <div class="row" style="justify-content:flex-end;margin-top:8px"><button class="btn" onclick="welcomeDone()">পরে দেখব</button></div>
        </div>
        <div id="welcomeFixed" hidden>
          <div class="section-title"><span id="welcomeLayoutName">জাতীয়</span> লেআউট চালু হয়েছে ✓</div>
          <div class="hint">বাংলা ⇄ English বদলাতে <b id="welcomeHotkey">দুবার Shift</b>। আপনার পুরনো অভ্যাসের জন্য আরও কয়েকটি জিনিস:</div>
          <ul class="welcome-list">
            <li><button class="link-btn" onclick="welcomeGo('settings', 'layoutFileInput')">নিজের লেআউট ফাইল যোগ করুন</button>: অন্য কোনো লেআউটে অভ্যস্ত হলে তার লেআউট ফাইল (.ekushe-layout) দিন; একুশের সব সুবিধা তাতেও কাজ করবে।</li>
            <li><button class="link-btn" onclick="showKeyMap()">কী-ম্যাপ দেখুন ও প্রিন্ট করুন</button>: কোন কী-তে কোন অক্ষর, এক পাতায়।</li>
            <li><button class="link-btn" onclick="welcomeGo('converter', 'fileConvInput')">পুরনো বিজয় (SutonnyMJ) ফাইল ইউনিকোডে বদলান</button>: Word বা টেক্সট ফাইল, লেখার ধরন ঠিক রেখে।</li>
            <li id="welcomeBijoyOut" hidden><button class="link-btn" onclick="welcomeBijoyOutput()">ডিজাইনের অ্যাপে বিজয় ফন্টে লিখুন</button>: Photoshop, Illustrator, PageMaker-এ SutonnyMJ দিয়ে লিখতে চাইলে চালু করুন; অন্য সব অ্যাপে ইউনিকোডই থাকবে।</li>
            <li>ভুল বানান লিখলে প্রমিত বানানটি পাশে দেখাবে; Space চাপলে আপনার লেখাই বসবে।</li>
          </ul>
          <div class="row" style="justify-content:flex-end;margin-top:8px"><button class="btn primary" onclick="document.getElementById('welcomeBox').hidden = true">ঠিক আছে</button></div>
        </div>
      </div>
      <div class="composer etched">
        <div class="roman-wrap">
          <svg><use href="#i-keys"/></svg>
          <input type="text" id="romanInput" placeholder="এখানে লিখুন — যেমন amar sonar bangla" oninput="onRomanInput()" autocomplete="off" spellcheck="false">
        </div>
        <div class="candidates" id="candidateBar"><span class="cand-empty">লিখতে শুরু করলে শব্দের পরামর্শ এখানে দেখাবে</span></div>
        <div class="hints" id="hintBar"></div>
        <div class="tips">
          <span><kbd>/office</kbd> ইংরেজি শব্দ রাখে</span>
          <span><kbd>;salam</kbd> স্নিপেট বসায়</span>
          <span><kbd>Ctrl</kbd>+<kbd>Backspace</kbd> শেষ শব্দ বাংলা ⇄ ইংরেজি</span>
        </div>
      </div>

      <div class="pad-card etched">
        <div class="toolbar" role="toolbar" aria-label="Writing tools">
          <button class="tool" onclick="convertPadDigitsToBangla()" title="2026 → ২০২৬">১২৩ বাংলা সংখ্যা</button>
          <button class="tool" onclick="convertPadDigitsToEnglish()" title="২০২৬ → 2026">123 ইংরেজি সংখ্যা</button>
          <button class="tool" onclick="insertTakaCurrency()" title="৳ ১,০০০/-">৳ টাকা</button>
          <button class="tool" onclick="insertTodayBanglaDate()">আজকের তারিখ</button>
          <button class="tool" onclick="smartPolishBanglaPad()" title="ফুলস্টপকে দাড়ি করে, বাড়তি ফাঁকা সরায়"><svg><use href="#i-spark"/></svg>দাড়ি ও ফাঁকা ঠিক করুন</button>
          <button class="tool" onclick="inspectPadWordInXray()"><svg><use href="#i-scan"/></svg>শেষ শব্দের এক্স-রে</button>
        </div>
        <textarea id="banglaPad" rows="5" placeholder="আপনার লেখা এখানে জমা হবে" oninput="updateCount()">আমি বাংলায় গান গাই। সৃষ্টি ও স্মৃতি আমাদের অনুপ্রেরণা।</textarea>
        <div class="pad-actions">
          <span class="count" id="padCount"></span>
          <div class="row">
            <button class="btn" onclick="clearPad()">মুছুন</button>
            <button class="btn primary" onclick="copyPad()"><svg><use href="#i-copy"/></svg>কপি করুন</button>
          </div>
        </div>
      </div>

      <div class="letters-head">
        <div><div class="section-title">এক ক্লিকে যুক্তবর্ণ ও চিহ্ন</div><div class="hint">ক্লিক করলে প্যাডে বসবে। নিচের রোমান অক্ষরগুলো টাইপ করেও একই জিনিস পাবেন।</div></div>
        <div class="seg">
          <button class="chip primary" onclick="filterQuickGrid('ALL', this)">সব</button>
          <button class="chip" onclick="filterQuickGrid('Conjunct', this)">যুক্তবর্ণ</button>
          <button class="chip" onclick="filterQuickGrid('Fola', this)">ফলা</button>
          <button class="chip" onclick="filterQuickGrid('Kar', this)">কার</button>
          <button class="chip" onclick="filterQuickGrid('Mark', this)">চিহ্ন</button>
        </div>
      </div>
      <div class="quick-grid" id="quickSignsGrid"></div>
    </section>

    <!-- Word X-ray and conjunct builder -->
    <section class="panel" id="tab-xray" data-title="শব্দ এক্স-রে" data-sub="যেকোনো শব্দ ভেঙে দেখুন কোন অক্ষর কীভাবে টাইপ করতে হয়, বা দুটি অক্ষর বেছে যুক্তবর্ণ বানান।">
      <div class="two-col">
        <div class="etched">
          <div class="section-title">শব্দ ভেঙে দেখুন</div>
          <div class="hint" style="margin-bottom:10px">বাংলা শব্দ লিখুন বা পেস্ট করুন।</div>
          <input type="text" id="xrayInput" value="সৃষ্টি" placeholder="যেমন: বিজ্ঞান" oninput="runWordXray()" style="font-family:var(--font-bn);font-size:1.25rem">
          <div class="chip-bar" style="margin-top:10px">
            <button class="chip" onclick="setXrayWord('সৃষ্টি')">সৃষ্টি</button>
            <button class="chip" onclick="setXrayWord('স্মৃতি')">স্মৃতি</button>
            <button class="chip" onclick="setXrayWord('বিজ্ঞান')">বিজ্ঞান</button>
            <button class="chip" onclick="setXrayWord('শিক্ষা')">শিক্ষা</button>
            <button class="chip" onclick="setXrayWord('শ্রেষ্ঠ')">শ্রেষ্ঠ</button>
            <button class="chip" onclick="setXrayWord('আশ্চর্য')">আশ্চর্য</button>
          </div>
          <div id="xraySummary"></div>
          <div class="xray-tokens" id="xrayTokens"></div>
        </div>
        <div class="etched">
          <div class="section-title">দুই ক্লিকে যুক্তবর্ণ</div>
          <div class="hint">প্রথম অক্ষর, তারপর দ্বিতীয় অক্ষর বেছে নিন।</div>
          <div class="matrix-label">প্রথম অক্ষর</div>
          <div class="consonant-matrix" id="cMatrix1"></div>
          <div class="matrix-label">দ্বিতীয় অক্ষর</div>
          <div class="consonant-matrix" id="cMatrix2"></div>
          <div class="fusion-preview">
            <div>
              <div id="fusionFormula">ষ + ্ + ট =</div>
              <div class="fusion-big" id="fusionResult">ষ্ট</div>
              <div id="fusionRoman"></div>
            </div>
            <button class="btn primary" onclick="insertBuiltConjunct()">প্যাডে বসান ও কপি করুন</button>
          </div>
        </div>
      </div>
    </section>

    <!-- Conjunct and word guide -->
    <section class="panel" id="tab-signs" data-title="যুক্তবর্ণ ও শব্দ" data-sub="উচ্চারণের ধারা অনুযায়ী সাজানো। যুক্তবর্ণ বা উদাহরণ শব্দে ক্লিক করলে কপি হবে ও প্যাডে বসবে।">
      <div class="marks-card" id="marksCard">
        <h3>বিশেষ চিহ্ন: ঠিক যেটা চান, সেটাই প্রথমে</h3>
        <p class="sub">এই চিহ্নগুলো লিখলে একুশে বোঝে আপনি ইচ্ছে করেই চাইছেন, তাই সেই বানানটিই এক নম্বরে আসে। তালিকার কোনো শব্দে এমন চিহ্ন থাকলে পপআপে তার লেখার উপায়ও দেখায় (যেমন হেঁসে: <kbd>he^se</kbd>)।</p>
        <p class="sub"><b>যুক্তবর্ণ, হসন্ত, না আলাদা?</b> দুটি ব্যঞ্জনের মাঝে কোন চিহ্ন দেবেন:</p>
        <div class="marks-join">
          <div><b>আস্তে</b><kbd>as+te</kbd> &nbsp;<kbd>+</kbd> যুক্তবর্ণ বানায় (স্ত)</div>
          <div><b>আস্‌তে</b><kbd>as,,te</kbd> &nbsp;<kbd>,,</kbd> হসন্ত দেখায়, যুক্ত করে না</div>
          <div><b>আসতে</b><kbd>as`te</kbd> &nbsp;<kbd>`</kbd> অক্ষর আলাদা রাখে</div>
        </div>
        <div class="marks-grid">
          <div class="mark-item"><b>হ্</b><div><kbd>,,</kbd> হসন্ত (দেখা যায়)<span>allah,, → আল্লাহ্</span></div></div>
          <div class="mark-item"><b>চাঁ</b><div><kbd>^</kbd> চন্দ্রবিন্দু<span>he^se → হেঁসে</span></div></div>
          <div class="mark-item"><b>কৃ</b><div><kbd>rri</kbd> ঋ-কার<span>krrishi → কৃষি</span></div></div>
          <div class="mark-item"><b>র্ম</b><div><kbd>rr</kbd> রেফ<span>dhorrmo → ধর্ম</span></div></div>
          <div class="mark-item"><b>ৎ</b><div><kbd>t``</kbd> খণ্ড-ত<span>hothat`` → হঠাৎ</span></div></div>
          <div class="mark-item"><b>দুঃ</b><div><kbd>:</kbd> বিসর্গ<span>du:kho → দুঃখ</span></div></div>
          <div class="mark-item"><b>ক্য</b><div><kbd>Z</kbd> য-ফলা<span>bakZo → বাক্য</span></div></div>
          <div class="mark-item"><b>ক্ত</b><div><kbd>+</kbd> যুক্ত করুন<span>k+t → ক্ত</span></div></div>
          <div class="mark-item"><b>সত</b><div><kbd>`</kbd> আলাদা রাখুন<span>as`te → আসতে</span></div></div>
        </div>
      </div>
      <div class="chip-bar scroll-x" id="atlasFilterBar">
        <button class="chip primary" onclick="filterAtlasGrid('ALL', this)">সব</button>
        <button class="chip" onclick="filterAtlasGrid('K-Series', this)">ক-ধারা</button>
        <button class="chip" onclick="filterAtlasGrid('Ch-Series', this)">চ-ধারা</button>
        <button class="chip" onclick="filterAtlasGrid('T-Series', this)">ত/ট-ধারা</button>
        <button class="chip" onclick="filterAtlasGrid('N-Series', this)">ন/ণ-ধারা</button>
        <button class="chip" onclick="filterAtlasGrid('P-Series', this)">প/ম-ধারা</button>
        <button class="chip" onclick="filterAtlasGrid('Sh-Series', this)">শ/ষ/স-ধারা</button>
        <button class="chip" onclick="filterAtlasGrid('D-Series', this)">দ/ধ-ধারা</button>
        <button class="chip" onclick="filterAtlasGrid('L-Series', this)">ল/হ-ধারা</button>
        <button class="chip" onclick="filterAtlasGrid('Fola', this)">ফলা</button>
        <button class="chip" onclick="filterAtlasGrid('Kar', this)">কার</button>
        <button class="chip" onclick="filterAtlasGrid('Mark', this)">চিহ্ন</button>
      </div>
      <div class="search-wrap">
        <svg><use href="#i-search"/></svg>
        <input type="text" id="signSearch" placeholder="খুঁজুন: রক্ত, rokto, blood, জ্ঞ, kkh" oninput="loadSigns(this.value)">
      </div>
      <div id="signsToast" role="status"></div>
      <div class="atlas-grid" id="atlasGrid"></div>
    </section>

    <!-- Spell check -->
    <section class="panel" id="tab-spell" data-title="বানান পরীক্ষা" data-sub="বাংলা একাডেমির প্রমিত বানান অনুযায়ী ভুল ও অপ্রমিত শব্দ দেখায়।">
      <textarea id="spellInput" rows="5">আমি বাংলাদেস এ বাস করি। মানুস মাত্রই জিবন ভালোবাসে।</textarea>
      <div class="row" style="margin-top:10px"><button class="btn primary" onclick="runSpellCheck()"><svg><use href="#i-check"/></svg>বানান পরীক্ষা করুন</button></div>
      <div class="stack" id="spellResults" style="margin-top:16px"></div>
    </section>

    <!-- Converter -->
    <section class="panel" id="tab-converter" data-title="রূপান্তর" data-sub="বিজয় (SutonnyMJ) আর ইউনিকোডের মধ্যে, বা বাংলিশ থেকে বাংলায়। উইন্ডোজে যেকোনো অ্যাপে লেখা বেছে Ctrl+Alt+C চাপলেও হয়।">
      <div class="seg" id="convModes" style="margin-bottom:12px">
        <button class="chip primary" onclick="runConvert('bijoy2uni', this)">বিজয় → ইউনিকোড</button>
        <button class="chip" onclick="runConvert('uni2bijoy', this)">ইউনিকোড → বিজয়</button>
        <button class="chip" onclick="runConvert('banglish', this)">বাংলিশ → বাংলা</button>
      </div>
      <div class="two-col">
        <textarea id="convInput" rows="7" placeholder="এখানে লেখা পেস্ট করুন" oninput="runConvert()">Avwg evsjvq Mvb MvB| evsjv†`k Avgvi ¯^‡`k|</textarea>
        <textarea id="convOutput" rows="7" placeholder="ফলাফল" readonly></textarea>
      </div>
      <div class="row" style="margin-top:10px;justify-content:flex-end"><button class="btn primary" onclick="copyText(document.getElementById('convOutput').value)"><svg><use href="#i-copy"/></svg>ফলাফল কপি করুন</button></div>
      <div class="etched" style="margin-top:18px">
        <div class="section-title">পুরো ফাইল রূপান্তর</div>
        <div class="hint">Word (.docx) বা টেক্সট (.txt) ফাইল দিন, রূপান্তর করা নতুন ফাইল ডাউনলোড হবে; আসল ফাইল যেমন ছিল তেমনই থাকে। Word ফাইলে লেখার ধরন (বোল্ড, রং, টেবিল) ঠিক থাকে। বিজয় থেকে ইউনিকোডে শুধু SutonnyMJ-জাতীয় (…MJ) ফন্টের লেখা বদলায়, তাই ইংরেজি অংশ যেমন আছে তেমনই থাকে।</div>
        <div class="row" style="gap:8px;flex-wrap:wrap;margin-top:10px;align-items:center">
          <select id="fileConvMode" style="max-width:260px">
            <option value="bijoy2uni">বিজয় → ইউনিকোড</option>
            <option value="uni2bijoy">ইউনিকোড → বিজয় (SutonnyMJ)</option>
          </select>
          <button class="btn primary" onclick="document.getElementById('fileConvInput').click()">ফাইল বেছে নিন</button>
          <input type="file" id="fileConvInput" accept=".docx,.txt" style="display:none" onchange="convertFile(this)">
        </div>
        <div id="fileConvMsg" role="status" class="hint" style="margin-top:8px"></div>
      </div>
    </section>

    <section class="panel" id="tab-ocr" data-title="ছবি থেকে লেখা" data-sub="ছাপা বাংলা লেখার ছবি বা স্ক্যান দিন, একুশে লেখাটি পড়ে ইউনিকোডে দেবে। ছবি আপনার কম্পিউটারেই থাকে; আপনি নিজে „একুশেকে পাঠান“ না চাপলে কোথাও পাঠানো হয় না।">
      <div class="etched ocr-drop" id="ocrDrop" tabindex="0">
        <svg class="ocr-ico"><use href="#i-image"/></svg>
        <div class="section-title">ছবি এখানে ছেড়ে দিন, বেছে নিন, বা Ctrl+V দিয়ে পেস্ট করুন</div>
        <div class="hint">স্ক্যান, ফোনে তোলা ছবি, স্ক্রিনশট (PNG, JPG) বা স্ক্যান করা PDF; একসাথে কয়েকটি পাতাও দেওয়া যায়। লেখা যত সোজা আর পরিষ্কার, ফল তত ভালো।</div>
        <div class="row" style="gap:8px;flex-wrap:wrap;justify-content:center;margin-top:10px">
          <button class="btn primary" onclick="document.getElementById('ocrFile').click()">ছবি বেছে নিন</button>
          <label class="hint" style="display:flex;align-items:center;gap:6px"><input type="checkbox" id="ocrEnglish"> লেখায় ইংরেজি শব্দও আছে</label>
          <label class="hint" style="display:flex;align-items:center;gap:6px" title="ছবি, নকশা, দাগ বা আরবির মতো অন্য লিপি থেকে ভুলভাল অক্ষর আসে; সেগুলো বাদ যায়"><input type="checkbox" id="ocrJunk" checked> ছবি ও নকশার আজেবাজে অক্ষর বাদ দিন</label>
          <label class="hint" style="display:flex;align-items:center;gap:6px">PDF-এর পাতা <input id="ocrPages" placeholder="সব (যেমন ১-১০, ১৫)" style="width:150px"></label>
        </div>
        <input type="file" id="ocrFile" accept="image/*,application/pdf,.pdf" multiple style="display:none" onchange="ocrFiles(this.files); this.value=''">
      </div>
      <div class="row" style="gap:10px;align-items:center;margin:10px 2px">
        <div id="ocrProgress" class="hint" role="status"></div>
        <button class="btn" id="ocrStop" hidden onclick="ocrStopNow = true">থামান</button>
      </div>
      <div class="two-col ocr-out" id="ocrOut" hidden>
        <div class="ocr-preview"><img id="ocrImg" alt=""></div>
        <div>
          <textarea id="ocrText" rows="16" spellcheck="false"></textarea>
          <div id="ocrFixes" class="hint" style="margin-top:8px"></div>
          <div id="ocrDropped" class="hint" style="margin-top:6px"></div>
          <div class="row" style="margin-top:10px;gap:8px;justify-content:flex-end;flex-wrap:wrap">
            <button class="btn" onclick="ocrJoinLines()" title="ছাপার লাইন ভাঙা বাদ দিয়ে অনুচ্ছেদ হিসেবে">লাইন জোড়া দিন</button>
            <button class="btn" onclick="ocrToPad()">লেখার প্যাডে নিন</button>
            <button class="btn" onclick="ocrSave()">.txt হিসেবে রাখুন</button>
            <button class="btn" id="ocrLearnBtn" hidden onclick="ocrSaveForLearning()" title="লেখা ঠিক করার পর চাপুন: ছবি আর ঠিক করা লেখা আপনার কম্পিউটারে জমা থাকে (কোথাও পাঠানো হয় না), পরে একুশের ছবি পড়া আরও ভালো করতে কাজে লাগে">শেখার জন্য রাখুন</button>
            <button class="btn" id="ocrSendBtn" hidden onclick="ocrSendAsk()" title="লেখা ঠিক করার পর চাপুন: পাতার ছবি আর ঠিক করা লেখা একুশের কাছে যায়, যাতে ছবি থেকে বাংলা পড়া আরও ভালো করা যায়">একুশেকে পাঠান</button>
            <button class="btn primary" onclick="copyText(document.getElementById('ocrText').value)"><svg><use href="#i-copy"/></svg>কপি করুন</button>
          </div>
        </div>
      </div>
    </section>

    <dialog class="lay-ed" id="layEd" aria-labelledby="layEdTitle">
      <div class="row" style="justify-content:space-between;align-items:center;gap:10px;flex-wrap:wrap">
        <h3 id="layEdTitle" style="margin:0">নিজে লেআউট বানান</h3>
        <div class="row" style="gap:8px">
          <button class="btn" onclick="document.getElementById('layEd').close()">বন্ধ</button>
          <button class="btn primary" onclick="layEdSave()">সংরক্ষণ ও চালু করুন</button>
        </div>
      </div>
      <div class="row" style="gap:10px;flex-wrap:wrap;margin-top:10px;align-items:flex-end">
        <label>নাম <input type="text" id="layEdName" maxlength="40" placeholder="আমার লেআউট" style="width:200px"></label>
        <label>শুরু করুন <select id="layEdFrom" onchange="layEdStart(this.value)"></select></label>
        <span class="hint">একটি কী-তে ক্লিক করুন, তারপর নিচ থেকে অক্ষর বেছে নিন।</span>
      </div>
      <div class="row" style="gap:6px;margin-top:10px;flex-wrap:wrap" role="radiogroup" aria-label="স্তর">
        <button class="btn" data-layer="n" onclick="layEdLayer('n')">সাধারণ</button>
        <button class="btn" data-layer="s" onclick="layEdLayer('s')">Shift</button>
        <button class="btn" data-layer="a" onclick="layEdLayer('a')">AltGr (ডান Alt)</button>
      </div>
      <div class="lay-kb" id="layEdKb"></div>
      <div class="row" style="gap:10px;align-items:center;flex-wrap:wrap">
        <b id="layEdSelKey">কোনো কী বাছা হয়নি</b>
        <input type="text" id="layEdText" maxlength="8" style="width:120px;font-family:var(--font-bn);font-size:1.1rem" oninput="layEdSetText(this.value)" aria-label="এই কী যা লিখবে">
        <button class="btn" onclick="layEdSetText('');document.getElementById('layEdText').value=''">খালি করুন</button>
      </div>
      <div class="lay-pal" id="layEdPal"></div>
      <div class="row" style="gap:16px;flex-wrap:wrap;margin-top:8px;align-items:center">
        <label>যুক্তবর্ণের লিংক কী <select id="layEdLink" onchange="layEdDraw()"></select></label>
        <label style="display:flex;gap:6px;align-items:center"><input type="checkbox" id="layEdKarBefore"> ি ে ৈ সবসময় অক্ষরের আগে টাইপ হয়</label>
        <label style="display:flex;gap:6px;align-items:center"><input type="checkbox" id="layEdRephAfter"> রেফ অক্ষরের পরে টাইপ হয়</label>
      </div>
      <div id="layEdWarn" class="hint" style="margin-top:8px"></div>
      <div style="margin-top:10px">
        <b>এখানে টাইপ করে দেখুন</b> <span class="hint">(ইংরেজি কীবোর্ডে; AltGr = ডান Alt)</span>
        <div tabindex="0" id="layEdTry" onkeydown="layEdKey(event)" style="min-height:64px;margin-top:6px;padding:10px 12px;border:1px solid var(--line, rgba(127,127,127,.3));border-radius:10px;font-family:var(--font-bn);font-size:1.3rem;white-space:pre-wrap;cursor:text"></div>
        <div class="row" style="gap:8px;margin-top:6px"><button class="btn" onclick="layEdKeys=[];layEdRun()">মুছে আবার দেখুন</button><span class="hint" id="layEdTryMsg"></span></div>
      </div>
    </dialog>
    <dialog class="ocr-send" id="donateDlg">
      <h3>একুশেকে কণ্ঠ দেবেন?</h3>
      <div>আপনি যে বাক্যগুলো পড়বেন, তার রেকর্ডিং দিয়ে শুধু একুশের নিজের ভয়েস টাইপিং মডেল শেখানো হবে। রেকর্ডিং কখনো প্রকাশ করা বা অন্য কাউকে দেওয়া হবে না।</div>
      <ul>
        <li>যা যাবে: প্রতিটি বাক্যের রেকর্ডিং, বাক্যটি, নিচের ঐচ্ছিক তথ্য, একুশের সংস্করণ আর কম্পিউটারের ধরন।</li>
        <li>যা যাবে না: আপনার নাম, ইমেইল বা অন্য কোনো ফাইল। আপনার কম্পিউটার একটি এলোমেলো কোড বানাবে।</li>
        <li>শুধু আপনি „পাঠান“ চাপলে যায়। সেই কোড দিয়ে যেকোনো সময় সব মুছে ফেলা যাবে।</li>
      </ul>
      <div class="row" style="gap:8px;flex-wrap:wrap;margin:8px 0">
        <select id="donateRegion" aria-label="অঞ্চল"><option value="">অঞ্চল (ঐচ্ছিক)</option><option>ঢাকা</option><option>চট্টগ্রাম</option><option>রাজশাহী</option><option>খুলনা</option><option>বরিশাল</option><option>সিলেট</option><option>রংপুর</option><option>ময়মনসিংহ</option><option>পশ্চিমবঙ্গ</option><option>অন্য কোথাও</option></select>
        <select id="donateAge" aria-label="বয়স"><option value="">বয়স (ঐচ্ছিক)</option><option>১৮–২৯</option><option>৩০–৪৪</option><option>৪৫–৫৯</option><option>৬০+</option></select>
        <select id="donateMic" aria-label="মাইক"><option value="">মাইক (ঐচ্ছিক)</option><option>হেডসেট</option><option>ল্যাপটপের মাইক</option><option>আলাদা মাইক</option><option>ইয়ারফোন</option></select>
      </div>
      <label style="display:flex;gap:8px;align-items:flex-start;margin-top:6px"><input type="checkbox" id="donateAdult"> <span>আমার বয়স ১৮ বছর বা তার বেশি।</span></label>
      <label style="display:flex;gap:8px;align-items:flex-start;margin-top:4px"><input type="checkbox" id="donateAgree"> <span>আমি রাজি: আমার পাঠানো রেকর্ডিং দিয়ে একুশের ভয়েস টাইপিং শেখানো যাবে।</span></label>
      <div class="row" style="margin-top:14px;gap:8px;justify-content:flex-end">
        <button class="btn" onclick="document.getElementById('donateDlg').close()">থাক</button>
        <button class="btn primary" onclick="donateAgree()">রাজি, শুরু করি</button>
      </div>
    </dialog>
    <dialog class="ocr-send" id="ocrSendDlg">
      <h3>পাতাটি একুশেকে পাঠাবেন?</h3>
      <div>আপনার ঠিক করা লেখা দিয়ে একুশের ছবি পড়া আরও ভালো করা হবে। যা যাবে:</div>
      <ul>
        <li>পড়া পাতার ছবি (<span id="ocrSendPages"></span>)</li>
        <li>আপনার ঠিক করা লেখা, আর একুশে প্রথমে যা পড়েছিল</li>
        <li>একুশের সংস্করণ আর কম্পিউটারের ধরন (যেমন Windows)</li>
      </ul>
      <div>আপনার নাম, ঠিকানা বা অন্য কোনো ফাইল যায় না।</div>
      <div class="warn">পরিচয়পত্র, চিঠি, ব্যাংক বা অন্য ব্যক্তিগত কাগজের ছবি পাঠাবেন না।</div>
      <div id="ocrSendSame" class="hint" hidden style="margin-top:6px">লেখাটি এখনও বদলানো হয়নি। ভুলগুলো ঠিক করে পাঠালে সবচেয়ে বেশি কাজে লাগে।</div>
      <div class="row" style="margin-top:14px;gap:8px;justify-content:flex-end">
        <button class="btn" onclick="document.getElementById('ocrSendDlg').close()">থাক</button>
        <button class="btn primary" id="ocrSendGo" onclick="ocrSend()">পাঠান</button>
      </div>
    </dialog>

    <!-- A word was wrong -->
    <section class="panel" id="tab-fix" data-title="ভুল শব্দ ঠিক করুন" data-sub="একুশে কোনো শব্দ ভুল বসালে এখানে ঠিকটা বেছে দিন। একুশে সঙ্গে সঙ্গে শিখে নেবে, পরের বার ঠিক শব্দটাই আসবে।">
      <div class="row" style="justify-content:space-between;gap:12px;flex-wrap:wrap;margin-bottom:12px">
        <div class="hint">সাম্প্রতিক শব্দগুলো (নতুনটা আগে)। এগুলো শুধু এই কম্পিউটারের মেমোরিতে থাকে, একুশে বন্ধ করলে মুছে যায়। পাসওয়ার্ডের মতো লেখা বা গোপন ঘরের লেখা এখানে আসে না।</div>
        <div class="row" style="gap:8px"><button class="btn" onclick="loadRecent()">রিফ্রেশ</button><button class="btn" onclick="clearRecent()">তালিকা মুছুন</button></div>
      </div>
      <div class="stack" id="recentList"></div>
      <div class="etched" id="reportedBox" style="margin-top:18px;display:none">
        <div class="row" style="justify-content:space-between;gap:12px;flex-wrap:wrap">
          <div><div class="section-title">আপনার ঠিক করা শব্দ</div><div class="hint">এই কম্পিউটারেই রাখা আছে, কোথাও পাঠানো হয়নি। চাইলে কপি করে সমস্যা জানানোর ফর্মে দিন, তাহলে সবার জন্য একুশে আরও ভালো হবে।</div></div>
          <div class="row" style="gap:8px"><button class="btn primary" onclick="copyReported()"><svg><use href="#i-copy"/></svg>কপি করুন</button><a class="btn" id="reportLink2" style="text-decoration:none;color:inherit" href="https://ekushenext.com/report/" target="_blank" rel="noopener">সমস্যা জানান</a></div>
        </div>
        <div class="stack" id="reportedList" style="margin-top:12px"></div>
      </div>
    </section>

    <!-- Settings -->
    <section class="panel" id="tab-settings" data-title="সেটিংস" data-sub="পরিবর্তন সঙ্গে সঙ্গে চালু ও সংরক্ষিত হয়।">
      <div class="settings-grid">
        <div class="etched">
          <div class="section-title">চেহারা</div>
          <div class="setting">
            <label>থিম</label>
            <div class="seg theme-seg" id="themeSeg">
              <button class="chip" data-theme-opt="system" onclick="setTheme('system')">সিস্টেম</button>
              <button class="chip" data-theme-opt="light" onclick="setTheme('light')">হালকা</button>
              <button class="chip" data-theme-opt="dark" onclick="setTheme('dark')">গাঢ়</button>
            </div>
          </div>
          <div class="setting">
            <label for="cfgOpacity">কাচের স্বচ্ছতা <span id="cfgOpacityVal">92%</span></label>
            <input type="range" id="cfgOpacity" min="70" max="100" value="92" oninput="onOpacityChange(this.value)">
            <small>স্টুডিও আর টাইপ করার সময়ের পপ-আপ, দুটোতেই কাজ করে।</small>
          </div>
          <div class="setting">
            <label for="cfgFont">বাংলা ফন্ট</label>
            <select id="cfgFont" onchange="changeBanglaFont(this.value)">
              <option value="Kalpurush">কালপুরুষ</option>
              <option value="SolaimanLipi">সোলাইমানলিপি</option>
              <option value="Siyam Rupali">সিয়াম রূপালী</option>
              <option value="Hind Siliguri">হিন্দ শিলিগুড়ি</option>
            </select>
            <div class="font-sample">আমার সোনার বাংলা, আমি তোমায় ভালোবাসি।</div>
          </div>
        </div>

        <div class="etched">
          <div class="section-title">টাইপিং</div>
          <div class="setting">
            <label for="cfgHotkey">বাংলা ⇄ English বদলানোর কী</label>
            <select id="cfgHotkey" onchange="saveSettings()">
              <option value="DoubleTapShift">দুবার Shift (প্রস্তাবিত)</option>
              <option value="CtrlSpace">Ctrl + Space</option>
              <option value="F12">F12</option>
              <option value="CtrlAltB">Ctrl + Alt + B</option>
            </select>
          </div>
          <div class="setting">
            <label for="cfgMode">টাইপিং মোড</label>
            <select id="cfgMode" onchange="saveSettings()">
              <option value="Smart">স্মার্ট — অভিধানের সেরা শব্দ আগে</option>
              <option value="PhoneticFirst">ফোনেটিক আগে — যা টাইপ করলেন সেটাই আগে</option>
              <option value="PhoneticOnly">শুধু ফোনেটিক — কোনো পপ-আপ নেই</option>
            </select>
          </div>
          <div class="setting">
            <label for="cfgLayout">কীবোর্ড লেআউট</label>
            <select id="cfgLayout" onchange="saveSettings()">
              <option value="AvroPhonetic">একুশে ফোনেটিক (অভ্র-সামঞ্জস্যপূর্ণ)</option>
              <option value="NationalJatiya">জাতীয়</option>
              <option value="Probhat">প্রভাত</option>
              <option value="Bornona">বর্ণনা</option>
              <option value="MunirOptima">মুনীর অপটিমা</option>
            </select>
            <small>ফিক্সড লেআউটে যেভাবে হাতে অভ্যাস সেভাবেই লিখুন: কার আগে বা পরে, যুক্তবর্ণের জন্য লিংক কী। একুশে সবসময় সঠিক ইউনিকোডে রাখে।</small>
          </div>
          <div class="setting" id="ownLayoutBox">
            <label>নিজের লেআউট</label>
            <small>নিজের পছন্দমতো লেআউট বানান, বা অন্য কোনো লেআউটের একুশে লেআউট ফাইল (.ekushe-layout) যোগ করুন, তালিকায় চলে আসবে। ফাইলে শুধু কোন কী-তে কোন অক্ষর, তা থাকে; একুশে যোগ করার আগে যাচাই করে নেয়।</small>
            <div class="row" style="gap:8px;flex-wrap:wrap">
              <button class="btn" onclick="document.getElementById('layoutFileInput').click()">লেআউট ফাইল যোগ করুন</button>
              <button class="btn primary" onclick="layEdOpen()">নিজে লেআউট বানান</button>
              <button class="btn" onclick="exportLayout()">এই লেআউট ফাইল হিসেবে নিন</button>
              <input type="file" id="layoutFileInput" accept=".ekushe-layout,.json,application/json" style="display:none" onchange="importLayoutFile(this)">
            </div>
            <div class="chip-bar" id="customLayoutList"></div>
            <div id="layoutFileMsg" role="status" class="hint"></div>
          </div>
          <div id="fixedLayoutOptions">
            <label class="toggle">
              <span><b>প্রমিত বানানের ইঙ্গিত</b><small>ফিক্সড লেআউটে কোনো শব্দের প্রমিত বানান আলাদা হলে পাশে দেখায় (যেমন সরকারী → সরকারি)। স্পেস চাপলে যেমন লিখেছেন তেমনই থাকে; Tab চেপে ইঙ্গিতটি নিতে পারেন। শব্দের পরেই Backspace চাপলে শব্দটি আবার খোলে।</small></span>
              <span class="switch"><input type="checkbox" id="cfgSpellHint" checked onchange="saveSettings()"><i></i></span>
            </label>
            <label class="toggle">
              <span><b>শব্দের পরামর্শ</b><small>ফিক্সড লেআউটে টাইপ করার সময় ওই অক্ষর দিয়ে শুরু হওয়া প্রচলিত শব্দ দেখায় (বাং → বাংলা, বাংলাদেশ)।</small></span>
              <span class="switch"><input type="checkbox" id="cfgWordSugg" onchange="saveSettings()"><i></i></span>
            </label>
            <div class="row" style="gap:8px;flex-wrap:wrap;margin-top:8px">
              <button class="btn" onclick="showKeyMap()">কী-ম্যাপ দেখুন</button>
            </div>
          </div>
        </div>

        <div class="etched">
          <div class="section-title">আচরণ</div>
          <label class="toggle">
            <span><b>ছোট পপ-আপ</b><small>টাইপ করার সময়ের পপ-আপে যুক্তবর্ণের সারি লুকায়</small></span>
            <span class="switch"><input type="checkbox" id="cfgCompact" onchange="saveSettings()"><i></i></span>
          </label>
          <div id="bijoyOutBox" hidden>
            <label class="toggle">
              <span><b>ডিজাইন অ্যাপে বিজয় (SutonnyMJ) লেখা</b><small>চালু থাকলে নিচের অ্যাপগুলোতে একুশে ইউনিকোডের বদলে বিজয় লেখা লেখে, SutonnyMJ ফন্টের সঙ্গে ব্যবহারের জন্য (Photoshop, Illustrator, InDesign, CorelDRAW, QuarkXPress)। অন্য সব অ্যাপে ইউনিকোডই থাকে। Ctrl+Alt+V চাপলে ক্লিপবোর্ডের লেখা অ্যাপ অনুযায়ী ঠিক রূপে বসে।</small></span>
              <span class="switch"><input type="checkbox" id="cfgBijoyOut" onchange="saveSettings()"><i></i></span>
            </label>
            <div class="setting">
              <label for="cfgBijoyApps">বিজয় লেখার অ্যাপ</label>
              <textarea id="cfgBijoyApps" rows="2" onchange="saveSettings()" placeholder="photoshop.exe, indesign.exe"></textarea>
              <small>প্রোগ্রামের নাম, কমা দিয়ে আলাদা করে।</small>
            </div>
          </div>
          <label class="toggle">
            <span><b>টার্মিনালে ইংরেজি</b><small>টার্মিনাল (Windows Terminal, CMD, PowerShell, ম্যাকের Terminal) ইংরেজিতে শুরু হয়; দুবার Shift চাপলে সেখানেও বাংলা</small></span>
            <span class="switch"><input type="checkbox" id="cfgTerminal" checked onchange="saveSettings()"><i></i></span>
          </label>
          <label class="toggle">
            <span><b>শুরুতে বাংলা</b><small>বন্ধ থাকলে কম্পিউটার চালু (লগ ইন) হলে ইংরেজিতে শুরু হয়; দুবার Shift চাপলে বাংলা। চালু করলে শুরু থেকেই বাংলা।</small></span>
            <span class="switch"><input type="checkbox" id="cfgStartBn" onchange="saveSettings()"><i></i></span>
          </label>
          <label class="toggle" id="cfgUpdatesRow" style="display:none">
            <span><b>নতুন সংস্করণের খবর</b><small>চালু করলে দিনে একবার দেখে নেয় নতুন সংস্করণ এসেছে কি না (শুধু GitHub-কে সংস্করণের নম্বর জিজ্ঞেস করে, আপনার কিছু পাঠায় না)। <span id="appVersion"></span></small></span>
            <span class="switch"><input type="checkbox" id="cfgUpdates" onchange="saveSettings()"><i></i></span>
          </label>
          <div class="row" id="checkNowRow" style="display:none;gap:8px;margin-top:6px"><button class="btn" onclick="fetch('/api/update/check',{method:'POST'})">নতুন সংস্করণ এখনই দেখুন</button></div>
        </div>

        <div class="etched">
          <div class="section-title">স্নিপেট</div>
          <div class="hint">শর্টকাট লিখে Space চাপলে পুরো লেখাটা বসে যায়। ক্লিক করলে প্যাডে বসবে। তারিখ আর সময়ও লেখা যায়: <b>;aj</b> আজকের তারিখ, <b>;bar</b> বার, <b>;bongabdo</b> বাংলা তারিখ, <b>;somoy</b> এখনকার সময়।</div>
          <div class="setting">
            <input type="text" id="newSnipKey" placeholder="শর্টকাট, যেমন ;addr">
            <input type="text" id="newSnipVal" placeholder="পুরো লেখা, যেমন ধানমন্ডি, ঢাকা-১২০৯" style="font-family:var(--font-bn)">
            <div><button class="btn primary" onclick="addCustomSnippet()">স্নিপেট যোগ করুন</button></div>
          </div>
          <div class="chip-bar" id="snippetsList" style="margin-top:12px"></div>
        </div>
      </div>
      <div id="settingsSavedMsg" role="status" class="hint" style="margin-top:12px"></div>
      <div class="etched" id="voiceBox" hidden style="margin-top:18px">
        <div class="section-title">🎤 ভয়েস টাইপিং <small class="hint" style="font-weight:400">(পরীক্ষামূলক)</small></div>
        <div class="hint" style="margin:4px 0 8px"><b>পরীক্ষামূলক:</b> ভয়েস টাইপিং এখনো শেখার পথে; কিছু শব্দ ভুল আসবে, তাই লেখা একবার দেখে নিন। কেমন কাজ করছে তা <a href="https://ekushenext.com/report/" target="_blank" rel="noopener">সমস্যা জানান</a> পাতায় জানালে আমরা এটিকে আরও ভালো করতে পারব।</div>
        <div class="hint">যেকোনো অ্যাপে <b id="voiceHotkey">Ctrl+Alt+Space</b> চাপুন; টুং শব্দ আর “🔴 শুনছি” দেখলে বাংলায় বলুন। একটু থামলেই সেটুকু লেখা হয়ে যায়, আর একুশে শুনতে থাকে; শেষ হলে আবার শর্টকাট চাপুন (<b>Esc</b> চাপলে বাতিল)। সব কাজ আপনার কম্পিউটারেই হয়, কণ্ঠ বা লেখা কোথাও পাঠানো হয় না (আপনি নিজে নিচের „একুশেকে কণ্ঠ দিন“ থেকে না পাঠালে)। “দাঁড়ি”, “কমা”, “নতুন লাইন”, “নতুন অনুচ্ছেদ” বললে চিহ্ন বা নতুন লাইন বসে।</div>
        <div id="voiceModelMsg" class="hint" style="margin-top:10px"></div>
        <div id="voiceModels" style="margin-top:8px"></div>
        <div id="voiceSpeed" class="hint" style="margin-top:6px"></div>
        <div id="voiceOptions" style="margin-top:10px">
          <label class="toggle">
            <span><b>বলা চিহ্ন</b><small>“দাঁড়ি” → ।, “কমা” → ,, “প্রশ্নবোধক চিহ্ন” → ?, “নতুন লাইন” → নতুন লাইন</small></span>
            <span class="switch"><input type="checkbox" id="cfgVoicePunct" checked onchange="saveSettings()"><i></i></span>
          </label>
          <label class="toggle">
            <span><b>সংখ্যা অঙ্কে</b><small>“দুই হাজার পাঁচশো টাকা” → ২৫০০ টাকা (ছোট সংখ্যা যেমন “এক কাপ চা” কথায় থাকে)</small></span>
            <span class="switch"><input type="checkbox" id="cfgVoiceNum" checked onchange="saveSettings()"><i></i></span>
          </label>
          <label class="toggle">
            <span><b>থামলে নিজে থেকে শেষ</b><small>চালু করলে কথা থামালেই ভয়েস টাইপিং বন্ধ হয়; বন্ধ থাকলে শর্টকাট বা Esc না চাপা পর্যন্ত শুনতে থাকে</small></span>
            <span class="switch"><input type="checkbox" id="cfgVoiceAutoStop" onchange="saveSettings()"><i></i></span>
          </label>
          <label class="toggle">
            <span><b>গ্রাফিক্স কার্ড ব্যবহার</b><small>থাকলে দ্রুত হয়; বন্ধ করলে শুধু প্রসেসরে চলে</small></span>
            <span class="switch"><input type="checkbox" id="cfgVoiceGpu" checked onchange="saveSettings()"><i></i></span>
          </label>
          <label class="toggle">
            <span><b>শব্দ</b><small>মাইক চালু ও বন্ধ হলে মৃদু টুং শব্দ</small></span>
            <span class="switch"><input type="checkbox" id="cfgVoiceSounds" checked onchange="saveSettings()"><i></i></span>
          </label>
          <div class="setting">
            <label for="cfgVoiceQuality">গতি ও নির্ভুলতা</label>
            <select id="cfgVoiceQuality" onchange="saveSettings()">
              <option value="fast">দ্রুত (সাধারণ কম্পিউটারের জন্য)</option>
              <option value="best">সবচেয়ে নির্ভুল (একটু ধীর)</option>
            </select>
          </div>
        </div>
        <div id="voiceRecentBox" hidden style="margin-top:12px">
          <div class="section-title" style="font-size:.95rem">শেষ কয়েকটি বলা লেখা</div>
          <div class="hint">কোনো শব্দ ভুল এলে এখানে ঠিক করে “শিখুন” চাপুন; একুশে পরের বার সেটি ঠিক লিখবে (শুধু আপনার কম্পিউটারে)।</div>
          <div id="voiceRecent"></div>
        </div>
        <div id="voiceLearnedBox" hidden style="margin-top:10px">
          <div class="hint">একুশে যা শিখেছে (ক্লিক করলে ভুলে যাবে):</div>
          <div class="chip-bar" id="voiceLearned"></div>
        </div>
      </div>
      <div class="etched" id="donateBox" hidden style="margin-top:18px">
        <div class="section-title">🎙️ একুশেকে কণ্ঠ দিন <small class="hint" style="font-weight:400">(ইচ্ছা হলে)</small></div>
        <div class="hint">কয়েকটি বাক্য পড়ে রেকর্ড করে পাঠালে একুশের ভয়েস টাইপিং আপনার মতো কণ্ঠ আর মাইকে আরও ভালো বুঝবে। প্রতিটি অংশে ২০টি বাক্য, ৪–৫ মিনিট লাগে। আপনি নিজে „পাঠান“ না চাপলে কিছুই যায় না।</div>
        <div id="donateStart" style="margin-top:10px"><button class="btn primary" onclick="donateAsk()">শুরু করুন</button></div>
        <div id="donateMain" hidden style="margin-top:10px">
          <div class="don-prog"><div class="row" style="justify-content:space-between;gap:8px"><b>আপনার অবদান</b><span class="hint" id="donateTotal"></span></div><div class="don-bar"><i id="donateTotalBar"></i></div></div>
          <div class="setting">
            <label for="donateMicSel">মাইক</label>
            <select id="donateMicSel" onchange="donateSetMic(this.value)"><option value="">কম্পিউটারের ঠিক করা মাইক</option></select>
            <small>যে মাইকে সাধারণত ভয়েস টাইপিং করেন সেটি বাছুন।</small>
          </div>
          <div class="setting">
            <label for="donateBatch">কোন অংশ পড়বেন</label>
            <select id="donateBatch" onchange="donateOpen(+this.value)"></select>
            <small id="donateNote"></small>
          </div>
          <div id="donateSentence" style="font-family:var(--font-bn);font-size:1.45rem;line-height:1.7;padding:14px 16px;margin:8px 0;border:1px solid var(--line, rgba(127,127,127,.25));border-radius:12px"></div>
          <div class="hint" id="donatePos"></div>
          <div class="don-grid" id="donateGrid" aria-label="এই অংশের বাক্যগুলো"></div>
          <div class="don-meter" aria-hidden="true"><i id="donateMeter"></i></div>
          <div class="row" style="gap:8px;flex-wrap:wrap;margin-top:8px">
            <button class="btn" onclick="donateMove(-1)" title="আগের বাক্য">◀</button>
            <button class="btn primary" id="donateRec" onclick="donateRecord()">● রেকর্ড</button>
            <button class="btn" id="donatePlay" onclick="donatePlay()" disabled>▶ শুনুন</button>
            <button class="btn" onclick="donateMove(1)" title="পরের বাক্য">▶▶</button>
          </div>
          <div class="hint" id="donateLevel" style="margin-top:6px">Space = রেকর্ড শুরু/থামান · ← → আগের/পরের বাক্য · P = শুনুন</div>
          <div class="row" style="gap:8px;flex-wrap:wrap;margin-top:12px;align-items:center">
            <button class="btn primary" id="donateSendBtn" onclick="donateSend()" disabled>পাঠান</button>
            <span class="hint" id="donateCount"></span>
          </div>
          <div class="hint" style="margin-top:12px">আপনার কোড: <b id="donateCode"></b> — লিখে রাখুন। পরে যা পাঠিয়েছেন সব মুছে ফেলতে চাইলে এই কোডটি লাগবে।
            <button class="btn" style="margin-left:6px" onclick="donateDelete()">আমার পাঠানো সব মুছুন</button>
            <button class="btn" style="margin-left:6px" onclick="donateStop()">বন্ধ করুন</button></div>
        </div>
      </div>
      <div class="etched" id="backupBox" style="margin-top:18px;display:flex;align-items:center;justify-content:space-between;gap:16px;flex-wrap:wrap">
        <div><div class="section-title">ব্যাকআপ ও ফিরিয়ে আনা</div><div class="hint">সেটিংস, আপনার শর্টকাট, যোগ করা কীবোর্ড লেআউট আর একুশের শেখা শব্দ এক ফাইলে রাখুন। নতুন কম্পিউটারে (উইন্ডোজ, ম্যাক বা লিনাক্স) ফাইলটি দিলেই সব ফিরে আসবে, আগের কিছু মুছবে না।</div><div id="backupMsg" role="status" class="hint" style="margin-top:6px"></div></div>
        <div class="row" style="gap:8px">
          <button class="btn primary" onclick="downloadBackup()">ব্যাকআপ নিন</button>
          <button class="btn" onclick="document.getElementById('restoreFile').click()">ফিরিয়ে আনুন</button>
          <input type="file" id="restoreFile" accept=".json,application/json" style="display:none" onchange="restoreBackup(this)">
        </div>
      </div>
      <div class="etched" style="margin-top:18px;display:flex;align-items:center;justify-content:space-between;gap:16px;flex-wrap:wrap">
        <div><div class="section-title">কোনো সমস্যা হচ্ছে?</div><div class="hint">কী হয়েছে লিখে পাঠান, কোনো অ্যাকাউন্ট লাগবে না। আপনার লেখা কিছু পাঠানো হয় না।</div></div>
        <a class="btn primary" id="reportLink" href="https://ekushenext.com/report/" target="_blank" rel="noopener">সমস্যা জানান</a>
      </div>
    </section>
  </main>
</div>

<div class="toast glass" id="toast" role="status" aria-live="polite"></div>

<script>
// Every Studio API call carries this page's key; other websites cannot read or change settings.
// The server only puts the key in the page when Ekushe opened it with a one-time launch code; the tab
// keeps it (session storage is per tab and per port) so reloading still works.
const EKUSHE_TOKEN = (() => {
  const served = "__EKUSHE_TOKEN__";
  let t = served;
  try {
    if (served) sessionStorage.setItem('ekushe_key', served);
    else t = sessionStorage.getItem('ekushe_key') || '';
  } catch (e) {}
  // Take the used-up launch code out of the address bar (keep ?tab=)
  const q = new URLSearchParams(window.location.search);
  if (q.has('code')) {
    q.delete('code');
    const rest = q.toString();
    history.replaceState(null, '', window.location.pathname + (rest ? '?' + rest : '') + window.location.hash);
  }
  return t;
})();
// Shown when this page has no valid key: opened without a launch code, or Ekushe restarted since
function showNeedLaunch() {
  if (document.getElementById('needLaunch')) return;
  try { sessionStorage.removeItem('ekushe_key'); } catch (e) {}
  const d = document.createElement('div');
  d.id = 'needLaunch';
  d.setAttribute('role', 'alert');
  d.style.cssText = 'position:fixed;inset:0;z-index:9999;display:grid;place-items:center;background:var(--bg,#f4f6f5);color:var(--ink,#0e241b);font:16px/1.6 var(--font-ui,system-ui),sans-serif;text-align:center;padding:24px';
  const box = document.createElement('div');
  box.style.maxWidth = '640px';
  const h = document.createElement('h2');
  h.textContent = 'স্টুডিও একুশে নেক্সট থেকে খুলুন';
  const p1 = document.createElement('p');
  p1.textContent = 'নিরাপত্তার জন্য স্টুডিও শুধু একুশের মেনু থেকে খোলা যায়। উইন্ডোজে ট্রে আইকনে রাইট-ক্লিক করে "একুশে নেক্সট স্টুডিও খুলুন" বেছে নিন; ম্যাক ও লিনাক্সে কিবোর্ড মেনু থেকে "Open Ekushe Studio"।';
  const p2 = document.createElement('p');
  p2.style.opacity = '0.7';
  p2.textContent = 'For your security, Studio only opens from Ekushe\'s menu: right-click the tray icon on Windows, or use the keyboard menu on Mac and Linux.';
  box.append(h, p1, p2);
  d.append(box);
  document.body.append(d);
}
if (!EKUSHE_TOKEN) {
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', showNeedLaunch);
  else showNeedLaunch();
}
const nativeFetch = window.fetch.bind(window);
window.fetch = (url, opts = {}) => {
  if (typeof url === 'string' && url.startsWith('/api/')) {
    opts = Object.assign({}, opts);
    opts.headers = Object.assign({}, opts.headers || {}, { 'X-Ekushe-Token': EKUSHE_TOKEN });
  }
  const p = nativeFetch(url, opts);
  if (typeof url === 'string' && url.startsWith('/api/')) {
    p.then((r) => { if (r.status === 403) showNeedLaunch(); }).catch(() => {});
  }
  return p;
};

let allSignItems = [];
let currentSearchItems = [];
let activeQuickKind = 'ALL';
let activeAtlasKind = 'ALL';
let activeConvMode = 'bijoy2uni';
let selectedC1 = 'ষ';
let selectedC2 = 'ট';

const BANGLA_CONSONANTS = [
  {c:'ক', r:'k'}, {c:'খ', r:'kh'}, {c:'গ', r:'g'}, {c:'ঘ', r:'gh'}, {c:'ঙ', r:'Ng'},
  {c:'চ', r:'c'}, {c:'ছ', r:'ch'}, {c:'জ', r:'j'}, {c:'ঝ', r:'jh'}, {c:'ঞ', r:'NG'},
  {c:'ট', r:'T'}, {c:'ঠ', r:'Th'}, {c:'ড', r:'D'}, {c:'ঢ', r:'Dh'}, {c:'ণ', r:'N'},
  {c:'ত', r:'t'}, {c:'থ', r:'th'}, {c:'দ', r:'d'}, {c:'ধ', r:'dh'}, {c:'ন', r:'n'},
  {c:'প', r:'p'}, {c:'ফ', r:'ph'}, {c:'ব', r:'b'}, {c:'ভ', r:'bh'}, {c:'ম', r:'m'},
  {c:'য', r:'z'}, {c:'র', r:'r'}, {c:'ল', r:'l'}, {c:'শ', r:'sh'}, {c:'ষ', r:'Sh'},
  {c:'স', r:'s'}, {c:'হ', r:'h'}
];

const CHAR_ROMAN_MAP = {
  'অ':'o','আ':'a','ই':'i','ঈ':'I','উ':'u','ঊ':'U','ঋ':'rri','এ':'e','ঐ':'OI','ও':'O','ঔ':'OU',
  'ক':'k','খ':'kh','গ':'g','ঘ':'gh','ঙ':'ng','চ':'c','ছ':'ch','জ':'j','ঝ':'jh','ঞ':'NG',
  'ট':'T','ঠ':'Th','ড':'D','ঢ':'Dh','ণ':'N','ত':'t','থ':'th','দ':'d','ধ':'dh','ন':'n',
  'প':'p','ফ':'ph','ব':'b/w','ভ':'bh','ম':'m','য':'z/y','র':'r','ল':'l','শ':'sh','ষ':'Sh/sh','স':'s','হ':'h',
  'ড়':'R','ঢ়':'Rh','য়':'y','ৎ':'t`','ং':'ng','ঃ':':','ঁ':'^',
  'া':'a','ি':'i','ী':'I / ii','ু':'u','ূ':'U / uu','ৃ':'ri','ে':'e','ৈ':'OI','ো':'o','ৌ':'OU','্':'+'
};
const KAR_NAMES = {'া':'আ-কার','ি':'ই-কার','ী':'ঈ-কার','ু':'উ-কার','ূ':'ঊ-কার','ৃ':'ঋ-কার','ে':'এ-কার','ৈ':'ঐ-কার','ো':'ও-কার','ৌ':'ঔ-কার'};

const esc = (s) => String(s).replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
const bnDigits = (n) => String(n).replace(/[0-9]/g, d => '০১২৩৪৫৬৭৮৯'[+d]);

// ── Theme: system (default), light or dark ──
function storedTheme() { try { return localStorage.getItem('ekushe_theme') || 'system'; } catch (e) { return 'system'; } }
function setTheme(mode) {
  const html = document.documentElement;
  if (mode === 'system') html.removeAttribute('data-theme'); else html.setAttribute('data-theme', mode);
  try { localStorage.setItem('ekushe_theme', mode); } catch (e) {}
  document.querySelectorAll('[data-theme-opt]').forEach(b => b.classList.toggle('primary', b.dataset.themeOpt === mode));
  const dark = mode === 'dark' || (mode === 'system' && matchMedia('(prefers-color-scheme: dark)').matches);
  const use = document.querySelector('#themeBtn use');
  if (use) use.setAttribute('href', dark ? '#i-moon' : '#i-sun');
}
function cycleTheme() {
  const order = ['system', 'light', 'dark'];
  const next = order[(order.indexOf(storedTheme()) + 1) % order.length];
  setTheme(next);
  toast({system: 'থিম: সিস্টেম অনুযায়ী', light: 'থিম: হালকা', dark: 'থিম: গাঢ়'}[next]);
}
setTheme(storedTheme());
matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => setTheme(storedTheme()));

let toastTimer = null;
function toast(msg) {
  const t = document.getElementById('toast');
  t.textContent = msg;
  t.classList.add('show');
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => t.classList.remove('show'), 1800);
}
function copyText(text) {
  // Some app windows (Studio's own window on Linux) refuse the clipboard API: copy the old way then
  const fallback = () => {
    const ta = document.createElement('textarea');
    ta.value = text; ta.style.position = 'fixed'; ta.style.opacity = '0';
    document.body.appendChild(ta); ta.select();
    let ok = false;
    try { ok = document.execCommand('copy'); } catch (e) {}
    ta.remove();
    toast(ok ? 'কপি হয়েছে' : 'কপি করা যায়নি');
  };
  if (navigator.clipboard && window.isSecureContext) navigator.clipboard.writeText(text).then(() => toast('কপি হয়েছে'), fallback);
  else fallback();
}

// 70–100 % → glass from airy to nearly solid
function applyOpacityVisuals(pct) {
  const clamped = Math.max(70, Math.min(100, parseInt(pct || 92, 10)));
  const alpha = 0.22 + (clamped - 70) / 30 * 0.6;
  document.documentElement.style.setProperty('--glass-alpha', alpha.toFixed(2));
  const v = document.getElementById('cfgOpacityVal');
  const s = document.getElementById('cfgOpacity');
  if (v) v.textContent = bnDigits(clamped) + '%';
  if (s) s.value = clamped;
}
let opacitySaveTimer = null;
function onOpacityChange(pct) {
  applyOpacityVisuals(pct);
  clearTimeout(opacitySaveTimer);
  opacitySaveTimer = setTimeout(saveSettings, 250);
}

function applyCssFont(fontName) {
  document.documentElement.style.setProperty('--font-bn', `'${fontName}', 'Kalpurush', 'SolaimanLipi', 'Hind Siliguri', sans-serif`);
  const sel = document.getElementById('cfgFont');
  if (sel) sel.value = fontName;
}
async function changeBanglaFont(fontName) {
  applyCssFont(fontName);
  await saveSettings();
}

function showTab(id) {
  const panel = document.getElementById('tab-' + id);
  if (!panel) return;
  document.querySelectorAll('.panel').forEach(p => p.classList.toggle('active', p === panel));
  document.querySelectorAll('.nav-btn').forEach(b => {
    const on = b.dataset.tab === id;
    b.classList.toggle('active', on);
    if (on) b.setAttribute('aria-current', 'page'); else b.removeAttribute('aria-current');
  });
  document.getElementById('pageTitle').textContent = panel.dataset.title;
  document.getElementById('pageSub').textContent = panel.dataset.sub;
  if (id === 'spell') runSpellCheck();
  if (id === 'converter') runConvert();
  if (id === 'xray') runWordXray();
  if (id === 'playground') document.getElementById('romanInput').focus({ preventScroll: true });
  if (id === 'fix') loadRecent();
}
document.querySelectorAll('.nav-btn').forEach(b => b.addEventListener('click', () => {
  showTab(b.dataset.tab);
  history.replaceState(null, '', '?tab=' + b.dataset.tab);
}));

async function refreshState() {
  const res = await fetch('/api/state');
  const st = await res.json();
  const kb = st.keyboard;
  document.getElementById('addKeyboardBox').hidden = !kb || kb.added;
  document.getElementById('removeKeyboardBox').hidden = !kb || !kb.added;
  if (kb && !kb.added) {
    document.getElementById('addKeyboardBtn').hidden = !kb.can_add;
    if (!kb.can_add) document.getElementById('addKeyboardMsg').textContent = kb.steps;
  }
  welcomeMaybeShow(st);
  document.getElementById('conflictBox').hidden = !st.conflicts;
  document.getElementById('conflictMsg').textContent = st.conflicts || '';
  const pill = document.getElementById('modePill');
  pill.className = 'lang-switch ' + (st.language === 'Bangla' ? 'bangla' : 'english');
  document.getElementById('hotkeyBadge').textContent = st.hotkey_display;
  document.getElementById('cfgHotkey').value = st.hotkey_id;
  document.getElementById('cfgMode').value = st.typing_mode;
  document.getElementById('cfgLayout').value = st.active_layout;
  const cmpEl = document.getElementById('cfgCompact');
  if (cmpEl && typeof st.compact_popup === 'boolean') cmpEl.checked = st.compact_popup;
  const trmEl = document.getElementById('cfgTerminal');
  if (trmEl && typeof st.auto_english_in_terminals === 'boolean') trmEl.checked = st.auto_english_in_terminals;
  if (typeof st.start_bangla === 'boolean') document.getElementById('cfgStartBn').checked = st.start_bangla;
  if (typeof st.fixed_spelling_hint === 'boolean') document.getElementById('cfgSpellHint').checked = st.fixed_spelling_hint;
  if (typeof st.fixed_word_suggestions === 'boolean') document.getElementById('cfgWordSugg').checked = st.fixed_word_suggestions;
  if (typeof st.bijoy_output === 'boolean') {
    document.getElementById('bijoyOutBox').hidden = false;
    document.getElementById('cfgBijoyOut').checked = st.bijoy_output;
    const appsEl = document.getElementById('cfgBijoyApps');
    if (document.activeElement !== appsEl) appsEl.value = st.bijoy_output_apps || '';
  }
  document.getElementById('fixedLayoutOptions').hidden = !st.active_layout || st.active_layout === 'AvroPhonetic';
  applyVoiceState(st.voice);
  if (st.app_version) { appInfo.app = st.app_version; document.querySelectorAll('.aboutVer').forEach(e => e.textContent = st.app_version); }
  if (typeof st.check_for_updates === 'boolean' && st.app_version) {
    document.getElementById('cfgUpdatesRow').style.display = '';
    document.getElementById('checkNowRow').style.display = '';
    document.getElementById('cfgUpdates').checked = st.check_for_updates;
    document.getElementById('appVersion').textContent = 'এখন আছে: ' + st.app_version;
  }
  if (Array.isArray(st.snippets)) {
    const sBox = document.getElementById('snippetsList');
    sBox.innerHTML = '';
    st.snippets.forEach(([k, v]) => {
      const b = document.createElement('button');
      b.className = 'chip snip-chip';
      b.innerHTML = `<code>${esc(k)}</code><b>${esc(v)}</b>`;
      b.onclick = () => { insertIntoPad(v + ' '); toast('প্যাডে বসানো হয়েছে'); };
      sBox.appendChild(b);
    });
  }
  if (st.font_family) applyCssFont(st.font_family);
  if (st.popup_opacity) applyOpacityVisuals(st.popup_opacity);
  if (st.report_url && /^https:\/\/ekushenext\.com\//.test(st.report_url)) { document.getElementById('reportLink').href = st.report_url; document.getElementById('reportLink2').href = st.report_url; }
}

async function toggleMode() {
  await fetch('/api/toggle', { method: 'POST' });
  await refreshState();
}

// ── Writing pad ──
let lastCandidates = [];
let lastQuery = '';
async function onRomanInput() {
  const val = document.getElementById('romanInput').value;
  const cBar = document.getElementById('candidateBar');
  const hBar = document.getElementById('hintBar');
  if (!val.trim()) {
    lastCandidates = [];
    cBar.innerHTML = '<span class="cand-empty">লিখতে শুরু করলে শব্দের পরামর্শ এখানে দেখাবে</span>';
    hBar.innerHTML = '';
    return;
  }
  const res = await fetch('/api/suggest?q=' + encodeURIComponent(val));
  const data = await res.json();
  if (document.getElementById('romanInput').value !== val) return; // a newer keystroke won
  lastCandidates = data.candidates;
  lastQuery = val;
  cBar.innerHTML = '';
  data.candidates.forEach((c, idx) => {
    const el = document.createElement('button');
    el.className = 'cand' + (idx === 0 ? ' top' : '');
    el.innerHTML = `<span class="n">${idx + 1}</span>${esc(c.text)}${c.annotation ? `<span class="tag">${esc(c.annotation)}</span>` : ''}`;
    el.onclick = () => commitCandidate(idx);
    cBar.appendChild(el);
  });
  hBar.innerHTML = '';
  data.sign_hints.slice(0, 8).forEach(h => {
    const el = document.createElement('button');
    el.className = 'hint-chip';
    el.title = h.label;
    el.innerHTML = `<b>${esc(h.result_bangla)}</b>${esc(h.key_hint)}`;
    el.onclick = () => { const r = document.getElementById('romanInput'); r.value = h.key_hint; r.focus(); onRomanInput(); };
    hBar.appendChild(el);
  });
}
function commitCandidate(idx) {
  const c = lastCandidates[idx];
  if (!c) return;
  insertIntoPad(c.text + ' ');
  const r = document.getElementById('romanInput');
  r.value = '';
  r.focus();
  onRomanInput();
}
// Space commits in typing order even when it is pressed before the word's suggestions arrive:
// the word is taken off the input at once (so the next word can be typed straight away) and
// placed in the pad when its own top suggestion is known.
let commitChain = Promise.resolve();
document.getElementById('romanInput').addEventListener('keydown', (e) => {
  const r = e.currentTarget;
  if ((e.key === ' ' || e.key === 'Enter') && r.value.trim()) {
    e.preventDefault();
    const word = r.value;
    const known = lastQuery === word ? lastCandidates : null;
    r.value = '';
    onRomanInput();
    commitChain = commitChain.then(async () => {
      let cands = known;
      if (!cands) {
        try { cands = (await (await fetch('/api/suggest?q=' + encodeURIComponent(word))).json()).candidates; }
        catch (err) { cands = []; }
      }
      insertIntoPad((cands[0] ? cands[0].text : word) + ' ');
    });
  } else if (/^[1-9]$/.test(e.key) && e.altKey && lastCandidates[+e.key - 1]) {
    e.preventDefault();
    commitCandidate(+e.key - 1);
  }
});

function updateCount() {
  const t = document.getElementById('banglaPad').value.trim();
  const words = t ? t.split(/\s+/).length : 0;
  document.getElementById('padCount').textContent = `${bnDigits(words)} শব্দ`;
}
function insertIntoPad(str) {
  const pad = document.getElementById('banglaPad');
  pad.value += str;
  updateCount();
}
function clearPad() {
  document.getElementById('banglaPad').value = '';
  updateCount();
  document.getElementById('romanInput').focus();
}
function copyPad() { copyText(document.getElementById('banglaPad').value); }

function convertPadDigitsToBangla() {
  const pad = document.getElementById('banglaPad');
  pad.value = pad.value.replace(/[0-9]/g, d => '০১২৩৪৫৬৭৮৯'[+d]);
}
function convertPadDigitsToEnglish() {
  const pad = document.getElementById('banglaPad');
  const map = {'০':'0','১':'1','২':'2','৩':'3','৪':'4','৫':'5','৬':'6','৭':'7','৮':'8','৯':'9'};
  pad.value = pad.value.replace(/[০-৯]/g, d => map[d]);
}
function insertTakaCurrency() {
  const pad = document.getElementById('banglaPad');
  const replaced = pad.value.replace(/([0-9০-৯]{4,})/g, m => '৳ ' + m + '/-');
  if (replaced !== pad.value) pad.value = replaced; else insertIntoPad('৳ ১,০০০/- ');
}
function insertTodayBanglaDate() {
  const now = new Date();
  const months = ['জানুয়ারি','ফেব্রুয়ারি','মার্চ','এপ্রিল','মে','জুন','জুলাই','আগস্ট','সেপ্টেম্বর','অক্টোবর','নভেম্বর','ডিসেম্বর'];
  insertIntoPad(`${bnDigits(now.getDate())} ${months[now.getMonth()]} ${bnDigits(now.getFullYear())} `);
}
function smartPolishBanglaPad() {
  const pad = document.getElementById('banglaPad');
  let t = pad.value;
  t = t.replace(/য়/g, 'য়').replace(/ড়/g, 'ড়').replace(/ঢ়/g, 'ঢ়');
  t = t.replace(/([ঀ-৿])\./g, '$1।');
  t = t.replace(/[ \t]{2,}/g, ' ');
  pad.value = t;
  toast('দাড়ি ও ফাঁকা ঠিক করা হয়েছে');
}
function inspectPadWordInXray() {
  const words = document.getElementById('banglaPad').value.trim().split(/\s+/);
  const last = (words[words.length - 1] || '').replace(/[।,.!?]/g, '');
  document.getElementById('xrayInput').value = last || 'সৃষ্টি';
  showTab('xray');
}

// ── Word X-ray and conjunct builder ──
function setXrayWord(w) {
  document.getElementById('xrayInput').value = w;
  runWordXray();
}
function runWordXray() {
  const word = (document.getElementById('xrayInput').value || '').trim();
  const box = document.getElementById('xrayTokens');
  const sum = document.getElementById('xraySummary');
  box.innerHTML = '';
  if (!word) { sum.textContent = ''; return; }
  let matchedRoman = '';
  for (const it of allSignItems) {
    const found = (it.words || []).find(w => w.bangla === word);
    if (found) { matchedRoman = found.roman; break; }
  }
  const chars = Array.from(word);
  const parts = [];
  let i = 0;
  while (i < chars.length) {
    if (i + 2 < chars.length && chars[i + 1] === '্') {
      if (i + 4 < chars.length && chars[i + 3] === '্') {
        parts.push({ glyph: chars.slice(i, i + 5).join(''), formula: `${chars[i]}+${chars[i+2]}+${chars[i+4]}`, type: 'যুক্তবর্ণ' });
        i += 5; continue;
      }
      const cluster = chars.slice(i, i + 3).join('');
      const known = allSignItems.find(it => it.bangla === cluster);
      const keyHint = known ? known.primary_key : ((CHAR_ROMAN_MAP[chars[i]] || '') + (CHAR_ROMAN_MAP[chars[i+2]] || ''));
      parts.push({ glyph: cluster, formula: keyHint, type: 'যুক্তবর্ণ' });
      i += 3; continue;
    }
    const ch = chars[i];
    const isKar = ch >= 'া' && ch <= 'ৌ';
    parts.push({ glyph: ch, formula: CHAR_ROMAN_MAP[ch] || ch, type: isKar ? (KAR_NAMES[ch] || 'কার') : 'বর্ণ' });
    i++;
  }
  sum.innerHTML = matchedRoman
    ? `টাইপ করুন <kbd>${esc(matchedRoman)}</kbd>`
    : `${bnDigits(parts.length)}টি অংশ`;
  parts.forEach(p => {
    const d = document.createElement('div');
    d.className = 'xray-token';
    d.innerHTML = `<span class="xt-char">${esc(p.glyph)}</span><span class="xt-key">${esc(p.formula)}</span><span class="xt-name">${esc(p.type)}</span>`;
    box.appendChild(d);
  });
}
function initConjunctBuilder() {
  const m1 = document.getElementById('cMatrix1');
  const m2 = document.getElementById('cMatrix2');
  m1.innerHTML = ''; m2.innerHTML = '';
  BANGLA_CONSONANTS.forEach(item => {
    [[m1, 1], [m2, 2]].forEach(([m, which]) => {
      const b = document.createElement('button');
      const sel = which === 1 ? selectedC1 : selectedC2;
      b.className = 'c-btn' + (item.c === sel ? ' selected' : '');
      b.textContent = item.c;
      b.title = item.r;
      b.onclick = () => { if (which === 1) selectedC1 = item.c; else selectedC2 = item.c; initConjunctBuilder(); };
      m.appendChild(b);
    });
  });
  updateFusionPreview();
}
function updateFusionPreview() {
  const fused = selectedC1 + '্' + selectedC2;
  document.getElementById('fusionFormula').textContent = `${selectedC1} + ্ + ${selectedC2}`;
  document.getElementById('fusionResult').textContent = fused;
  const known = allSignItems.find(it => it.bangla === fused);
  const r1 = (BANGLA_CONSONANTS.find(x => x.c === selectedC1) || {r:''}).r;
  const r2 = (BANGLA_CONSONANTS.find(x => x.c === selectedC2) || {r:''}).r;
  const hint = known ? [known.primary_key].concat(known.alt_keys || []).join(' / ') : r1 + r2;
  document.getElementById('fusionRoman').textContent = 'টাইপ করুন: ' + hint;
}
function insertBuiltConjunct() {
  const fused = selectedC1 + '্' + selectedC2;
  insertIntoPad(fused);
  copyText(fused);
}

// ── Guide ──
function isBanglaConsonant(ch) {
  if (!ch) return false;
  const code = ch.charCodeAt(0);
  return (code >= 0x0995 && code <= 0x09B9) || code === 0x09DC || code === 0x09DD || code === 0x09DF;
}
function insertSignItemIntoPad(item) {
  const pad = document.getElementById('banglaPad');
  const lastChar = pad.value.slice(-1);
  let inserted = item.bangla;
  if ((item.kind === 'Kar' || item.kind === 'Fola') && !isBanglaConsonant(lastChar)) {
    inserted = item.display_bangla.split('/')[0].trim();
  }
  insertIntoPad(inserted);
  if (navigator.clipboard) navigator.clipboard.writeText(item.bangla);
  toast(`${inserted} কপি হয়েছে · টাইপ: ${item.primary_key}`);
}
function insertWordExample(banglaWord, romanKey, meaning, ev) {
  if (ev) ev.stopPropagation();
  const pad = document.getElementById('banglaPad');
  insertIntoPad((pad.value && !pad.value.endsWith(' ') ? ' ' : '') + banglaWord + ' ');
  if (navigator.clipboard) navigator.clipboard.writeText(banglaWord);
  toast(`${banglaWord} কপি হয়েছে · টাইপ: ${romanKey}`);
}
function markActive(chipEl) {
  if (chipEl && chipEl.parentElement) {
    chipEl.parentElement.querySelectorAll('.chip').forEach(c => c.classList.remove('primary'));
    chipEl.classList.add('primary');
  }
}
function filterQuickGrid(kind, chipEl) { activeQuickKind = kind; markActive(chipEl); renderQuickGrid(); }
function filterAtlasGrid(kind, chipEl) { activeAtlasKind = kind; markActive(chipEl); renderAtlasGrid(); }

function renderQuickGrid() {
  const qGrid = document.getElementById('quickSignsGrid');
  qGrid.innerHTML = '';
  const items = activeQuickKind === 'ALL' ? allSignItems : allSignItems.filter(it => it.kind === activeQuickKind || it.series === activeQuickKind);
  items.slice(0, 36).forEach(item => {
    const b = document.createElement('button');
    b.className = 'sign-btn';
    b.title = item.decomposition;
    b.innerHTML = `<span class="bn">${esc(item.display_bangla)}</span><span class="rm">${esc(item.primary_key)}</span><span class="dc">${esc(item.decomposition.split('(')[0])}</span>`;
    b.onclick = () => insertSignItemIntoPad(item);
    qGrid.appendChild(b);
  });
}
function renderAtlasGrid() {
  const atlas = document.getElementById('atlasGrid');
  atlas.innerHTML = '';
  const items = activeAtlasKind === 'ALL' ? currentSearchItems : currentSearchItems.filter(it => it.series === activeAtlasKind || it.kind === activeAtlasKind);
  if (!items.length) {
    atlas.innerHTML = '<div class="hint">কিছু পাওয়া যায়নি। অন্য বানানে বা রোমান হরফে খুঁজে দেখুন।</div>';
    return;
  }
  items.forEach(item => {
    const card = document.createElement('div');
    card.className = 'atlas-card';
    const alts = item.alt_keys && item.alt_keys.length ? ' / ' + item.alt_keys.join(', ') : '';
    const top = document.createElement('div');
    top.className = 'atlas-card-top';
    top.title = 'ক্লিক করে কপি করুন';
    top.innerHTML = `<div class="atlas-glyph">${esc(item.display_bangla)}</div><div class="atlas-meta"><div><span class="key">${esc(item.primary_key + alts)}</span><span class="series-tag">${esc(item.series || item.kind)}</span></div><div>${esc(item.decomposition)}</div></div>`;
    top.onclick = () => insertSignItemIntoPad(item);
    card.appendChild(top);
    const list = document.createElement('div');
    list.className = 'word-list';
    (item.words || []).forEach(w => {
      const p = document.createElement('span');
      p.className = 'word-pill';
      p.innerHTML = `<span class="wb">${esc(w.bangla)}</span><span class="wr">${esc(w.roman)}</span><span class="wm">${esc(w.meaning)}</span>`;
      p.onclick = (ev) => insertWordExample(w.bangla, w.roman, w.meaning, ev);
      list.appendChild(p);
    });
    card.appendChild(list);
    atlas.appendChild(card);
  });
}
let signsReq = 0;
async function loadSigns(q) {
  const my = ++signsReq;
  const res = await fetch('/api/signs?q=' + encodeURIComponent(q || ''));
  const items = await res.json();
  if (my !== signsReq) return;
  currentSearchItems = items;
  renderAtlasGrid();
  if (!q) {
    allSignItems = items;
    renderQuickGrid();
    initConjunctBuilder();
    runWordXray();
  }
}

// ── Spell check ──
async function runSpellCheck() {
  const text = document.getElementById('spellInput').value;
  const res = await fetch('/api/spell', { method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify({ text }) });
  const issues = await res.json();
  const box = document.getElementById('spellResults');
  box.innerHTML = '';
  if (!issues.length) {
    box.innerHTML = '<div class="ok-note"><svg width="18" height="18"><use href="#i-check"/></svg>কোনো বানান ভুল পাওয়া যায়নি</div>';
    return;
  }
  issues.forEach(iss => {
    const div = document.createElement('div');
    div.className = 'issue';
    const kind = iss.kind === 'non_standard' ? 'অপ্রমিত বানান' : 'সম্ভাব্য ভুল';
    div.innerHTML = `<div><div class="wrong">${esc(iss.word)}</div><div class="kind">${kind}</div></div><div class="chip-bar grow"></div>`;
    const bar = div.querySelector('.chip-bar');
    iss.suggestions.forEach(s => {
      const b = document.createElement('button');
      b.className = 'chip fix';
      b.textContent = s;
      b.onclick = () => replaceSpell(iss.word, s);
      bar.appendChild(b);
    });
    box.appendChild(div);
  });
}
function replaceSpell(wrong, right) {
  const el = document.getElementById('spellInput');
  el.value = el.value.replace(wrong, right);
  runSpellCheck();
}

// ── Converter ──
let convTimer = null;
async function runConvert(mode, chipEl) {
  if (mode) { activeConvMode = mode; markActive(chipEl || document.querySelector(`#convModes [onclick*="${mode}"]`)); }
  clearTimeout(convTimer);
  convTimer = setTimeout(async () => {
    const text = document.getElementById('convInput').value;
    const res = await fetch('/api/convert', { method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify({ mode: activeConvMode, text }) });
    const out = await res.json();
    document.getElementById('convOutput').value = out.result;
  }, mode ? 0 : 150);
}

// ── Whole-file conversion (.txt, .docx) ──
// Windows-1252 bytes 0x80-0x9F (old Bijoy .txt files are saved in this code page)
const CP1252_HIGH = '€\u0081‚ƒ„…†‡ˆ‰Š‹Œ\u008DŽ\u008F\u0090‘’“”•–—˜™š›œ\u009DžŸ';
function encode1252(text) {
  const out = new Uint8Array(text.length);
  for (let i = 0; i < text.length; i++) {
    const c = text.charCodeAt(i);
    if (c < 0x80 || (c >= 0xA0 && c <= 0xFF)) out[i] = c;
    else { const k = CP1252_HIGH.indexOf(text[i]); out[i] = k >= 0 ? 0x80 + k : 0x3F; }
  }
  return out;
}
// Convert many pieces of text in one request, kept apart by a private-use character
async function convertPieces(mode, pieces) {
  const SEP = '\uE001';
  const out = [];
  for (let i = 0; i < pieces.length; i += 400) {
    const chunk = pieces.slice(i, i + 400);
    const res = await fetch('/api/convert', { method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify({ mode, text: chunk.join(SEP) }) });
    if (!res.ok) throw new Error('convert');
    const parts = (await res.json()).result.split(SEP);
    if (parts.length !== chunk.length) throw new Error('pieces');
    out.push(...parts);
  }
  return out;
}
const CRC_TABLE = (() => { const t = new Uint32Array(256); for (let n = 0; n < 256; n++) { let c = n; for (let k = 0; k < 8; k++) c = c & 1 ? 0xEDB88320 ^ (c >>> 1) : c >>> 1; t[n] = c >>> 0; } return t; })();
function crc32(b) { let c = 0xFFFFFFFF; for (let i = 0; i < b.length; i++) c = CRC_TABLE[(c ^ b[i]) & 0xFF] ^ (c >>> 8); return (c ^ 0xFFFFFFFF) >>> 0; }
async function inflateRaw(bytes) {
  const ds = new DecompressionStream('deflate-raw');
  const buf = await new Response(new Blob([bytes]).stream().pipeThrough(ds)).arrayBuffer();
  return new Uint8Array(buf);
}
// Read a .zip (a .docx is one): [{name, data}]
async function readZip(buf) {
  const v = new DataView(buf), u8 = new Uint8Array(buf);
  let eocd = -1;
  for (let i = u8.length - 22; i >= Math.max(0, u8.length - 66000); i--) if (v.getUint32(i, true) === 0x06054b50) { eocd = i; break; }
  if (eocd < 0) throw new Error('not a zip');
  const count = v.getUint16(eocd + 10, true);
  let p = v.getUint32(eocd + 16, true);
  const dec = new TextDecoder();
  const files = [];
  for (let n = 0; n < count; n++) {
    if (v.getUint32(p, true) !== 0x02014b50) throw new Error('bad zip');
    const method = v.getUint16(p + 10, true), csize = v.getUint32(p + 20, true);
    const nlen = v.getUint16(p + 28, true), xlen = v.getUint16(p + 30, true), clen = v.getUint16(p + 32, true);
    const local = v.getUint32(p + 42, true);
    const name = dec.decode(u8.subarray(p + 46, p + 46 + nlen));
    const start = local + 30 + v.getUint16(local + 26, true) + v.getUint16(local + 28, true);
    const raw = u8.subarray(start, start + csize);
    const data = method === 0 ? raw.slice() : method === 8 ? await inflateRaw(raw) : null;
    if (!data) throw new Error('unsupported zip');
    files.push({ name, data });
    p += 46 + nlen + xlen + clen;
  }
  return files;
}
// Write a .zip with the files stored (Word opens it like any other)
function writeZip(files) {
  const enc = new TextEncoder();
  const parts = [], central = [];
  let offset = 0;
  for (const f of files) {
    const name = enc.encode(f.name), crc = crc32(f.data), size = f.data.length;
    const h = new DataView(new ArrayBuffer(30));
    h.setUint32(0, 0x04034b50, true); h.setUint16(4, 20, true); h.setUint16(6, 0x0800, true);
    h.setUint32(14, crc, true); h.setUint32(18, size, true); h.setUint32(22, size, true); h.setUint16(26, name.length, true);
    parts.push(new Uint8Array(h.buffer), name, f.data);
    const c = new DataView(new ArrayBuffer(46));
    c.setUint32(0, 0x02014b50, true); c.setUint16(4, 20, true); c.setUint16(6, 20, true); c.setUint16(8, 0x0800, true);
    c.setUint32(16, crc, true); c.setUint32(20, size, true); c.setUint32(24, size, true); c.setUint16(28, name.length, true);
    c.setUint32(42, offset, true);
    central.push(new Uint8Array(c.buffer), name);
    offset += 30 + name.length + size;
  }
  const csize = central.reduce((a, b) => a + b.length, 0);
  const e = new DataView(new ArrayBuffer(22));
  e.setUint32(0, 0x06054b50, true); e.setUint16(8, files.length, true); e.setUint16(10, files.length, true);
  e.setUint32(12, csize, true); e.setUint32(16, offset, true);
  return new Blob([...parts, ...central, new Uint8Array(e.buffer)], { type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document' });
}
const W_NS = 'http://schemas.openxmlformats.org/wordprocessingml/2006/main';
function runFont(run) {
  const rPr = run.getElementsByTagNameNS(W_NS, 'rPr')[0];
  const f = rPr && rPr.getElementsByTagNameNS(W_NS, 'rFonts')[0];
  if (!f) return '';
  return ['ascii', 'hAnsi', 'cs', 'eastAsia'].map(a => f.getAttributeNS(W_NS, a) || '').join(' ');
}
function setRunFont(doc, run, font) {
  let rPr = run.getElementsByTagNameNS(W_NS, 'rPr')[0];
  if (!rPr) { rPr = doc.createElementNS(W_NS, 'w:rPr'); run.insertBefore(rPr, run.firstChild); }
  let f = rPr.getElementsByTagNameNS(W_NS, 'rFonts')[0];
  if (!f) { f = doc.createElementNS(W_NS, 'w:rFonts'); rPr.insertBefore(f, rPr.firstChild); }
  for (const a of ['ascii', 'hAnsi', 'cs', 'eastAsia']) f.setAttributeNS(W_NS, 'w:' + a, font);
  if (font !== 'SutonnyMJ') { const cs = doc.createElementNS(W_NS, 'w:lang'); cs.setAttributeNS(W_NS, 'w:bidi', 'bn-BD'); if (!rPr.getElementsByTagNameNS(W_NS, 'lang')[0]) rPr.appendChild(cs); }
}
// Convert one Word XML part; returns [newBytes, runsConverted]
async function convertWordXml(bytes, mode) {
  const doc = new DOMParser().parseFromString(new TextDecoder().decode(bytes), 'application/xml');
  if (doc.getElementsByTagName('parsererror').length) return [bytes, 0];
  const runs = [...doc.getElementsByTagNameNS(W_NS, 'r')].filter(r => {
    const text = [...r.getElementsByTagNameNS(W_NS, 't')].map(t => t.textContent).join('');
    if (!text.trim()) return false;
    if (mode === 'bijoy2uni') return /MJ\b|SutonnyMJ|Bijoy/i.test(runFont(r));
    return /[\u0980-\u09FF]/.test(text);
  });
  if (!runs.length) return [bytes, 0];
  const pieces = runs.map(r => [...r.getElementsByTagNameNS(W_NS, 't')].map(t => t.textContent).join(''));
  const done = await convertPieces(mode, pieces);
  const font = mode === 'bijoy2uni' ? (document.getElementById('cfgFont') && document.getElementById('cfgFont').value || 'Kalpurush') : 'SutonnyMJ';
  runs.forEach((r, i) => {
    const ts = [...r.getElementsByTagNameNS(W_NS, 't')];
    ts[0].textContent = done[i];
    ts[0].setAttribute('xml:space', 'preserve');
    ts.slice(1).forEach(t => t.parentNode.removeChild(t));
    setRunFont(doc, r, font);
  });
  return [new TextEncoder().encode(new XMLSerializer().serializeToString(doc)), runs.length];
}
function download(blob, name) {
  const a = document.createElement('a');
  a.href = URL.createObjectURL(blob);
  a.download = name;
  document.body.appendChild(a);
  a.click();
  setTimeout(() => { URL.revokeObjectURL(a.href); a.remove(); }, 1500);
}
async function convertFile(input) {
  const msg = document.getElementById('fileConvMsg');
  const file = input.files && input.files[0];
  input.value = '';
  if (!file) return;
  const mode = document.getElementById('fileConvMode').value;
  const suffix = mode === 'bijoy2uni' ? '-unicode' : '-bijoy';
  const base = file.name.replace(/\.[^.]+$/, '');
  if (file.size > 40 * 1024 * 1024) { msg.textContent = 'ফাইলটি অনেক বড় (৪০ MB-এর বেশি)।'; return; }
  msg.textContent = 'রূপান্তর হচ্ছে…';
  try {
    const buf = await file.arrayBuffer();
    if (/\.txt$/i.test(file.name)) {
      const bytes = new Uint8Array(buf);
      const utf8 = bytes[0] === 0xEF && bytes[1] === 0xBB && bytes[2] === 0xBF;
      let text;
      try { text = new TextDecoder('utf-8', { fatal: true }).decode(bytes); }
      catch (e) { text = new TextDecoder('windows-1252').decode(bytes); }
      if (utf8) text = text.replace(/^\uFEFF/, '');
      const lines = text.split('\n');
      const out = (await convertPieces(mode, lines)).join('\n');
      const blob = mode === 'uni2bijoy'
        ? new Blob([encode1252(out)], { type: 'text/plain' })
        : new Blob(['\uFEFF' + out], { type: 'text/plain;charset=utf-8' });
      download(blob, base + suffix + '.txt');
      msg.textContent = `হয়ে গেছে: ${base + suffix}.txt ডাউনলোড হয়েছে।`;
      return;
    }
    const files = await readZip(buf);
    let total = 0;
    for (const f of files) {
      if (/^word\/(document|header\d*|footer\d*|footnotes|endnotes)\.xml$/.test(f.name)) {
        const [data, n] = await convertWordXml(f.data, mode);
        f.data = data;
        total += n;
      }
    }
    if (!total) {
      msg.textContent = mode === 'bijoy2uni'
        ? 'ফাইলে SutonnyMJ-জাতীয় (…MJ) ফন্টের কোনো লেখা পাওয়া যায়নি। লেখাটি কপি করে উপরের বাক্সে দিন।'
        : 'ফাইলে বাংলা ইউনিকোড লেখা পাওয়া যায়নি।';
      return;
    }
    download(writeZip(files), base + suffix + '.docx');
    const bn = (n) => String(n).replace(/[0-9]/g, d => '০১২৩৪৫৬৭৮৯'[d]);
    msg.textContent = `হয়ে গেছে: ${bn(total)}টি অংশ বদলানো হয়েছে, ${base + suffix}.docx ডাউনলোড হয়েছে।` + (mode === 'uni2bijoy' ? ' খোলার কম্পিউটারে SutonnyMJ ফন্ট থাকতে হবে।' : '');
  } catch (e) {
    msg.textContent = 'ফাইলটি পড়া যায়নি। এটি .docx বা .txt তো? (' + e.message + ')';
  }
}

// ── A word was wrong ──
let reportedText = '';
async function loadRecent() {
  let data;
  try {
    const res = await fetch('/api/recent');
    if (!res.ok) return false;
    data = await res.json();
  } catch (e) { return false; }
  const box = document.getElementById('recentList');
  box.innerHTML = '';
  if (!data.recent.length) {
    box.innerHTML = '<div class="hint">এখনো কোনো শব্দ নেই। যেকোনো অ্যাপে বাংলায় লিখুন, তারপর এখানে ফিরে আসুন।</div>';
  }
  data.recent.forEach(r => {
    const div = document.createElement('div');
    div.className = 'issue';
    div.innerHTML = `<div><div class="wrong" style="color:var(--ink);text-decoration:none">${esc(r.word)}</div><div class="kind">লিখেছিলেন: ${esc(r.roman)}</div></div><div class="chip-bar grow"></div>`;
    const bar = div.querySelector('.chip-bar');
    r.others.forEach(o => {
      const b = document.createElement('button');
      b.className = 'chip fix';
      b.textContent = o;
      b.onclick = () => fixWord(r, o);
      bar.appendChild(b);
    });
    const inp = document.createElement('input');
    inp.type = 'text';
    inp.placeholder = 'অন্য শব্দ (বাংলা বা রোমানে)';
    inp.setAttribute('aria-label', 'ঠিক শব্দটি লিখুন');
    inp.style.cssText = 'min-width:12em;flex:1';
    inp.onkeydown = (e) => { if (e.key === 'Enter' && inp.value.trim()) fixWord(r, inp.value.trim()); };
    const ok = document.createElement('button');
    ok.className = 'btn';
    ok.textContent = 'ঠিক করুন';
    ok.onclick = () => { if (inp.value.trim()) fixWord(r, inp.value.trim()); };
    bar.appendChild(inp);
    bar.appendChild(ok);
    box.appendChild(div);
  });
  reportedText = data.reported || '';
  const lines = reportedText.split('\n').filter(l => l.trim());
  document.getElementById('reportedBox').style.display = lines.length ? '' : 'none';
  const rl = document.getElementById('reportedList');
  rl.innerHTML = '';
  lines.slice(-20).reverse().forEach(l => {
    const [roman, wrong, right] = l.split('\t');
    const d = document.createElement('div');
    d.className = 'hint';
    d.innerHTML = `${esc(roman || '')}: <s>${esc(wrong || '')}</s> → <b>${esc(right || '')}</b>`;
    rl.appendChild(d);
  });
  return true;
}
async function fixWord(r, right) {
  const res = await fetch('/api/wrong', { method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify({ roman: r.roman, wrong: r.word, right }) });
  if (!res.ok) { toast('শব্দটি আর তালিকায় নেই'); await loadRecent(); return; }
  const out = await res.json();
  toast(`শিখে নিয়েছে: ${out.fixed.roman} → ${out.fixed.right}`);
  await loadRecent();
}
async function clearRecent() {
  await fetch('/api/recent/clear', { method: 'POST' });
  await loadRecent();
}
function copyReported() {
  copyText('Ekushe Next: words that were wrong (typed / placed / meant / word before)\n' + reportedText);
}
// Only the Windows app keeps recent words; elsewhere the tab stays hidden.
loadRecent().then(ok => { if (ok) document.getElementById('navFix').style.display = ''; });

// ── Linux: add Ekushe to the keyboards ──
async function addKeyboard() {
  const res = await fetch('/api/add-keyboard', { method: 'POST' });
  const out = res.ok ? await res.json() : { ok: false, message: 'নিজে থেকে যোগ করা গেল না।' };
  if (out.ok && !out.relogin) {
    toast(out.message);
    document.getElementById('addKeyboardBox').hidden = true;
  } else {
    // A re-login is needed, or it didn't work: keep the box with what to do next
    document.getElementById('addKeyboardMsg').textContent = out.message;
    document.getElementById('addKeyboardBtn').hidden = out.ok;
  }
}

async function removeKeyboard() {
  const res = await fetch('/api/remove-keyboard', { method: 'POST' });
  const out = res.ok ? await res.json() : { ok: false, message: 'বাদ দেওয়া গেল না।' };
  if (out.ok && !out.relogin) {
    toast(out.message);
    document.getElementById('removeKeyboardBox').hidden = true;
    document.getElementById('addKeyboardBox').hidden = false;
  } else {
    document.getElementById('removeKeyboardMsg').textContent = out.message;
    document.getElementById('removeKeyboardBtn').hidden = out.ok;
  }
}

// ── Keyboard layouts: built-in and the user's own layout files ──
const BUILT_IN_LAYOUT_NAMES = {
  AvroPhonetic: 'একুশে ফোনেটিক (অভ্র-সামঞ্জস্যপূর্ণ)', NationalJatiya: 'জাতীয়',
  Probhat: 'প্রভাত', Bornona: 'বর্ণনা', MunirOptima: 'মুনীর অপটিমা',
};
async function refreshLayouts() {
  const res = await fetch('/api/layouts');
  if (!res.ok) return;
  const data = await res.json();
  const sel = document.getElementById('cfgLayout');
  sel.innerHTML = '';
  const list = document.getElementById('customLayoutList');
  list.innerHTML = '';
  for (const l of data.layouts) {
    const o = document.createElement('option');
    o.value = l.value;
    o.textContent = l.custom ? `${l.name} (নিজের)` : (BUILT_IN_LAYOUT_NAMES[l.value] || l.name);
    sel.appendChild(o);
    if (l.custom) {
      const chip = document.createElement('span');
      chip.className = 'chip';
      chip.title = [l.author && ('তৈরি করেছেন: ' + l.author), l.description].filter(Boolean).join(' — ');
      chip.textContent = l.name + ' ';
      const x = document.createElement('button');
      x.textContent = '✕';
      x.setAttribute('aria-label', l.name + ' সরান');
      x.onclick = () => removeLayout(l.id, l.name);
      chip.appendChild(x);
      list.appendChild(chip);
    }
  }
  sel.value = data.active;
}
// The engine's reasons, in Bangla (the English detail stays for anything unusual)
function layoutErrorBn(e) {
  if (/isn't an Ekushe layout/.test(e)) return 'এটি একুশের লেআউট ফাইল নয়।';
  if (/newer version/.test(e)) return 'ফাইলটি একুশের নতুন সংস্করণের জন্য তৈরি; একুশে আপডেট করে আবার চেষ্টা করুন।';
  if (/fewer than 10/.test(e)) return 'ফাইলে ১০টির কম বাংলা অক্ষর আছে, লেআউটটি অসম্পূর্ণ মনে হচ্ছে।';
  if (/too large/.test(e)) return 'ফাইলটি অনেক বড়, এটি লেআউট ফাইল নয়।';
  if (/no name/.test(e)) return 'লেআউটের কোনো নাম নেই।';
  if (/can't type/.test(e)) return 'ফাইলে এমন চিহ্ন আছে যা কীবোর্ড দিয়ে লেখার নয়। (' + e + ')';
  if (/single key/.test(e)) return 'ফাইলে একটি কী-এর জায়গায় একাধিক অক্ষর আছে। (' + e + ')';
  return e ? 'ফাইলে ভুল আছে। (' + e + ')' : 'ফাইলটি পড়া যায়নি।';
}
async function importLayoutFile(input) {
  const msg = document.getElementById('layoutFileMsg');
  const file = input.files && input.files[0];
  input.value = '';
  if (!file) return;
  if (file.size > 64 * 1024) { msg.textContent = 'ফাইলটি অনেক বড়, এটি লেআউট ফাইল নয়।'; return; }
  const res = await fetch('/api/layouts/import', { method: 'POST', headers: {'Content-Type': 'application/json'}, body: await file.text() });
  const out = await res.json().catch(() => ({}));
  if (!res.ok) { msg.textContent = 'লেআউট যোগ করা যায়নি: ' + layoutErrorBn(out.error || ''); return; }
  await refreshLayouts();
  document.getElementById('cfgLayout').value = out.value;
  await saveSettings();
  msg.textContent = `"${out.imported}" যোগ হয়েছে ও চালু আছে।`;
  toast('লেআউট যোগ হয়েছে');
}
async function removeLayout(id, name) {
  if (!confirm(`"${name}" লেআউটটি সরাবেন?`)) return;
  const res = await fetch('/api/layouts/remove', { method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify({ id }) });
  document.getElementById('layoutFileMsg').textContent = res.ok ? `"${name}" সরানো হয়েছে।` : 'সরানো যায়নি।';
  await refreshLayouts();
  await refreshState();
}
async function exportLayout() {
  const msg = document.getElementById('layoutFileMsg');
  const value = document.getElementById('cfgLayout').value;
  const res = await fetch('/api/layouts/export?id=' + encodeURIComponent(value));
  if (!res.ok) { msg.textContent = 'ফোনেটিক টাইপিংয়ের কোনো কী-লেআউট নেই; আগে একটি ফিক্সড লেআউট বেছে নিন।'; return; }
  const text = await res.text();
  const a = document.createElement('a');
  a.href = URL.createObjectURL(new Blob([text], { type: 'application/json' }));
  a.download = value.replace(/^Custom:/, '') + '.ekushe-layout';
  document.body.appendChild(a);
  a.click();
  setTimeout(() => { URL.revokeObjectURL(a.href); a.remove(); }, 1000);
  msg.textContent = 'লেআউট ফাইল ডাউনলোড হয়েছে। এটি বদলে নিজের লেআউট বানাতে পারেন।';
}
const layoutsReady = refreshLayouts();
// ── Layout editor: make a fixed layout by clicking keys (saved as an ordinary layout file) ──
const LAY_ROWS = ['`1234567890-=', 'qwertyuiop[]\\', "asdfghjkl;'", 'zxcvbnm,./'];
const LAY_SHIFT = { '`': '~', '1': '!', '2': '@', '3': '#', '4': '$', '5': '%', '6': '^', '7': '&', '8': '*', '9': '(', '0': ')', '-': '_', '=': '+', '[': '{', ']': '}', '\\': '|', ';': ':', "'": '"', ',': '<', '.': '>', '/': '?' };
const layShift = k => /[a-z]/.test(k) ? k.toUpperCase() : (LAY_SHIFT[k] || k);
const LAY_PAL = [
  'অ আ ই ঈ উ ঊ ঋ এ ঐ ও ঔ',
  'া ি ী ু ূ ৃ ে ৈ ো ৌ ৗ',
  'ক খ গ ঘ ঙ চ ছ জ ঝ ঞ ট ঠ ড ঢ ণ ত থ দ ধ ন প ফ ব ভ ম য র ল শ ষ স হ ড় ঢ় য় ৎ',
  'ং ঃ ঁ ্ । ॥ ‘ ’ “ ”',
  '০ ১ ২ ৩ ৪ ৫ ৬ ৭ ৮ ৯ ৳',
  'ক্ষ জ্ঞ ্য ্র র্',
];
const LAY_NEEDED = 'ক খ গ ঘ ঙ চ ছ জ ঝ ঞ ট ঠ ড ঢ ণ ত থ দ ধ ন প ফ ব ভ ম য র ল শ ষ স হ ড় ঢ় য় া ি ী ু ূ ৃ ে ৈ ো ৌ ং ঃ ঁ'.split(' ');
let layEd = { keys: {}, altgr: {}, after_link: {}, layer: 'n', sel: null };
let layEdKeys = [];
async function layEdOpen() {
  const from = document.getElementById('layEdFrom');
  const r = await (await fetch('/api/layouts')).json().catch(() => ({ layouts: [] }));
  from.innerHTML = '<option value="">খালি থেকে</option>' + (r.layouts || []).filter(l => l.value !== 'AvroPhonetic')
    .map(l => `<option value="${l.value}">${l.name} থেকে কপি করে</option>`).join('');
  const link = document.getElementById('layEdLink');
  link.innerHTML = '<option value="">নেই</option>' + LAY_ROWS.join('').split('').concat(LAY_ROWS.join('').split('').map(layShift))
    .map(k => `<option value="${k.replace(/"/g, '&quot;')}">${k}</option>`).join('');
  layEd = { keys: {}, altgr: {}, after_link: {}, layer: 'n', sel: null };
  document.getElementById('layEdName').value = '';
  document.getElementById('layEdKarBefore').checked = false;
  document.getElementById('layEdRephAfter').checked = false;
  document.getElementById('layEdPal').innerHTML = LAY_PAL.map(row => row.split(' ').map(c =>
    `<button type="button" onclick="layEdPick('${c}')">${c === '্' ? '◌্' : /^[ািীুূৃেৈোৌৗংঃঁ]$/.test(c) ? '◌' + c : c}</button>`).join('')).join('<span style="flex-basis:100%"></span>');
  layEdKeys = []; layEdLayer('n'); layEdRun();
  document.getElementById('layEd').showModal();
}
async function layEdStart(value) {
  layEd.keys = {}; layEd.altgr = {}; layEd.after_link = {};
  if (value) {
    const f = await (await fetch('/api/layouts/export?id=' + encodeURIComponent(value))).json().catch(() => null);
    if (f) {
      layEd.keys = f.keys || {}; layEd.altgr = f.altgr || {}; layEd.after_link = f.after_link || {};
      document.getElementById('layEdLink').value = f.link_key || '';
      document.getElementById('layEdKarBefore').checked = !!f.kar_before;
      document.getElementById('layEdRephAfter').checked = !!f.reph_after;
      if (!document.getElementById('layEdName').value) document.getElementById('layEdName').value = value.startsWith('Custom:') ? (f.name || '') : (f.name || '') + ' (আমার)';
    }
  }
  layEdDraw(); layEdRun();
}
function layEdLayer(l) {
  layEd.layer = l;
  document.querySelectorAll('#layEd [data-layer]').forEach(b => b.classList.toggle('primary', b.dataset.layer === l));
  layEdDraw();
}
// the key as stored: Shift turns q into Q and 1 into !
function layEdSlot(k) { return layEd.layer === 's' ? layShift(k) : k; }
function layEdMap() { return layEd.layer === 'a' ? layEd.altgr : layEd.keys; }
function layEdDraw() {
  const kb = document.getElementById('layEdKb'), link = document.getElementById('layEdLink').value;
  const esc = t => String(t || '').replace(/&/g, '&amp;').replace(/</g, '&lt;');
  kb.innerHTML = LAY_ROWS.map((row, r) => `<div class="lay-row" style="padding-left:${r * 18}px">` + row.split('').map(k => {
    const slot = layEdSlot(k), main = layEdMap()[slot];
    const other = layEd.layer === 'n' ? layEd.keys[layShift(k)] : layEd.layer === 's' ? layEd.keys[k] : '';
    const cls = ['lay-key', layEd.sel === k ? 'sel' : '', (link && (link === slot)) ? 'link' : ''].join(' ');
    return `<button type="button" class="${cls}" onclick="layEdSelect('${k.replace(/\\/g, '\\\\').replace(/'/g, "\\'")}')"><span class="k">${esc(slot)}</span><span class="s">${esc(other)}</span><span class="n">${esc(main)}</span></button>`;
  }).join('') + '</div>').join('');
  if (layEd.sel) {
    const slot = layEdSlot(layEd.sel);
    document.getElementById('layEdSelKey').textContent = `কী: ${layEd.layer === 'a' ? 'AltGr + ' : ''}${slot}`;
    document.getElementById('layEdText').value = layEdMap()[slot] || '';
  }
  layEdWarn();
}
function layEdSelect(k) { layEd.sel = k; layEdDraw(); document.getElementById('layEdText').focus(); }
function layEdSetText(t) {
  if (!layEd.sel) { toast('আগে একটি কী বেছে নিন'); return; }
  const slot = layEdSlot(layEd.sel), m = layEdMap();
  if (t) m[slot] = t; else delete m[slot];
  const v = document.getElementById('layEdText').value;
  layEdDraw(); document.getElementById('layEdText').value = v;
  layEdRun();
}
function layEdPick(c) {
  document.getElementById('layEdText').value = c;
  layEdSetText(c);
}
function layEdWarn() {
  const all = new Set();
  // by parts: ো is ে + া, য় is য + ়, so a layout typing the parts can type them
  for (const m of [layEd.keys, layEd.altgr, layEd.after_link]) for (const v of Object.values(m)) for (const c of [...v.normalize('NFD')]) all.add(c);
  const missing = LAY_NEEDED.filter(c => ![...c.normalize('NFD')].every(x => all.has(x)));
  const n = Object.values(layEd.keys).filter(v => /[ঀ-৿]/.test(v)).length;
  const w = [];
  if (n < 10) w.push(`এখনও ${bnNum(10 - n)}টি বাংলা অক্ষর কম: অন্তত ১০টি কী-তে বাংলা দিন।`);
  if (missing.length) w.push('যা এখনও কোনো কী দিয়ে লেখা যায় না: ' + missing.map(c => /^[ািীুূৃেৈোৌংঃঁ]$/.test(c) ? '◌' + c : c).join(' '));
  if (!document.getElementById('layEdLink').value && !all.has('্')) w.push('যুক্তবর্ণের জন্য একটি লিংক কী বা ্ (হসন্ত) দিন।');
  document.getElementById('layEdWarn').textContent = w.length ? '⚠ ' + w.join(' ') : '✓ সব অক্ষর লেখা যায়।';
}
function layEdFile() {
  return JSON.stringify({
    format: 'ekushe-layout', version: 1,
    name: document.getElementById('layEdName').value.trim() || 'আমার লেআউট',
    link_key: document.getElementById('layEdLink').value || null,
    reph_after: document.getElementById('layEdRephAfter').checked,
    kar_before: document.getElementById('layEdKarBefore').checked,
    keys: layEd.keys, altgr: layEd.altgr, after_link: layEd.after_link,
  }, null, 2);
}
function layEdKey(e) {
  if (e.ctrlKey && !e.altKey) return;   // Ctrl+C and the like
  const altgr = e.getModifierState && e.getModifierState('AltGraph') || (e.ctrlKey && e.altKey);
  let k = null;
  if (e.key === 'Backspace') k = '\b'; else if (e.key === 'Enter') k = '\n'; else if (e.key === ' ') k = ' ';
  else if (e.code && /^(Key|Digit)/.test(e.code)) { const b = e.code.slice(-1).toLowerCase(); k = e.shiftKey ? layShift(b) : b; }
  else if (e.key && e.key.length === 1 && e.key.charCodeAt(0) < 127) k = e.key;
  if (!k) return;
  e.preventDefault();
  layEdKeys.push([k, !!altgr]); layEdRun();
}
let layEdTimer = null;
function layEdRun() {
  clearTimeout(layEdTimer);
  layEdTimer = setTimeout(async () => {
    const box = document.getElementById('layEdTry'), msg = document.getElementById('layEdTryMsg');
    if (!layEdKeys.length) { box.textContent = ''; msg.textContent = 'বাক্সে ক্লিক করে টাইপ করুন'; return; }
    const res = await fetch('/api/layouts/try', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ file: layEdFile(), keys: layEdKeys }) });
    const r = await res.json().catch(() => ({}));
    if (res.ok) { box.textContent = r.text; msg.textContent = ''; }
    else msg.textContent = layoutErrorBn(r.error || '');
  }, 80);
}
async function layEdSave() {
  if (!document.getElementById('layEdName').value.trim()) { toast('লেআউটের একটি নাম দিন'); document.getElementById('layEdName').focus(); return; }
  const res = await fetch('/api/layouts/import', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: layEdFile() });
  const out = await res.json().catch(() => ({}));
  if (!res.ok) { toast('সংরক্ষণ হয়নি: ' + layoutErrorBn(out.error || '')); return; }
  await refreshLayouts();
  document.getElementById('cfgLayout').value = out.value;
  await saveSettings();
  document.getElementById('layEd').close();
  document.getElementById('layoutFileMsg').textContent = `"${out.imported}" সংরক্ষণ হয়েছে ও চালু আছে। „এই লেআউট ফাইল হিসেবে নিন“ দিয়ে অন্য কম্পিউটারে বা ট্যাবলেটে নিতে পারেন।`;
  toast('লেআউট সংরক্ষণ হয়েছে');
}

// ── Key map of the layout in use (and a printable cheat sheet) ──
const KEY_ROWS = [
  ['`~','1!','2@','3#','4$','5%','6^','7&','8*','9(','0)','-_','=+'],
  ['qQ','wW','eE','rR','tT','yY','uU','iI','oO','pP','[{',']}','\\|'],
  ['aA','sS','dD','fF','gG','hH','jJ','kK','lL',';:',"'\""],
  ['zZ','xX','cC','vV','bB','nN','mM',',<','.>','/?'],
];
function keyText(t) {
  if (t === undefined) return '';
  return t.replace(/\u200C/g, 'ZWNJ').replace(/\u200D/g, 'ZWJ').replace(/^([\u09BE-\u09CD\u0981-\u0983\u09D7])/, '◌$1');
}
// ── Picture to text (Bangla OCR) ──
// Tesseract (WebAssembly) reads the picture in this page; Ekushe's dictionary then cleans the text.
let ocrWorker = null, ocrBusy = false, ocrChanges = [], ocrStopNow = false, ocrDroppedAll = [], ocrPageData = [], ocrFirstText = '';
function ocrStatus(t) { document.getElementById('ocrProgress').textContent = t; }
function ocrLoadScript(src) {
  return new Promise((ok, fail) => { const s = document.createElement('script'); s.src = src; s.onload = ok; s.onerror = fail; document.head.appendChild(s); });
}
async function ocrWorkerReady() {
  if (ocrWorker) return ocrWorker;
  ocrStatus('বাংলা পড়ার মডেল চালু হচ্ছে…');
  if (!window.Tesseract) await ocrLoadScript('/ocr/tesseract.min.js');
  // WebAssembly SIMD makes it about twice as fast where the browser has it
  const simd = WebAssembly.validate(new Uint8Array([0,97,115,109,1,0,0,0,1,5,1,96,0,1,123,3,2,1,0,10,10,1,8,0,65,0,253,15,253,98,11]));
  const base = location.origin + '/ocr/';
  // three Bangla models, each unsure word read with all of them: one trained on real scanned books
  // (old and new), Ekushe's fine-tune for modern fonts, and Tesseract's own (old books and archive
  // scans: 25% → 16% words wrong; modern pages a little better too)
  ocrWorker = await Tesseract.createWorker(['benreal', 'ben', 'benold'], 1, {
    workerPath: base + 'worker.min.js', workerBlobURL: false,
    corePath: base + (simd ? 'tesseract-core-simd-lstm.wasm.js' : 'tesseract-core-lstm.wasm.js'),
    langPath: base.slice(0, -1), gzip: false, cacheMethod: 'none',
    logger: m => { if (m.status === 'recognizing text') ocrStatus('পড়া হচ্ছে… ' + Math.round(m.progress * 100) + '%'); },
  });
  return ocrWorker;
}

// Picture clean-up before reading: grey, even lighting, letters at a size the model reads best,
// and straight lines. (Measured on rendered, scanned and photographed pages: a phone photo goes
// from about 1 wrong word in 3 to about 1 in 10.)
function ocrGray(img, maxSide) {
  const f = Math.min(1, maxSide / Math.max(img.naturalWidth || img.width, img.naturalHeight || img.height));
  const w = Math.round((img.naturalWidth || img.width) * f), h = Math.round((img.naturalHeight || img.height) * f);
  const c = document.createElement('canvas'); c.width = w; c.height = h;
  const x = c.getContext('2d'); x.fillStyle = '#fff'; x.fillRect(0, 0, w, h); x.drawImage(img, 0, 0, w, h);
  const d = x.getImageData(0, 0, w, h).data, g = new Uint8ClampedArray(w * h);
  for (let i = 0, j = 0; j < g.length; i += 4, j++) g[j] = (d[i] * 77 + d[i + 1] * 150 + d[i + 2] * 29) >> 8;
  return { g, w, h };
}
function ocrMinMax(src, w, h, r, isMax) {
  // separable square max/min filter (a grey-level dilation/erosion)
  const tmp = new Uint8ClampedArray(w * h), out = new Uint8ClampedArray(w * h);
  const pick = isMax ? Math.max : Math.min;
  for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
    let v = src[y * w + x];
    for (let k = Math.max(0, x - r); k <= Math.min(w - 1, x + r); k++) v = pick(v, src[y * w + k]);
    tmp[y * w + x] = v;
  }
  for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
    let v = tmp[y * w + x];
    for (let k = Math.max(0, y - r); k <= Math.min(h - 1, y + r); k++) v = pick(v, tmp[k * w + x]);
    out[y * w + x] = v;
  }
  return out;
}
function ocrOtsu(g) {
  const hist = new Array(256).fill(0); for (const v of g) hist[v]++;
  let sum = 0; for (let i = 0; i < 256; i++) sum += i * hist[i];
  let sb = 0, wb = 0, best = 0, t = 128;
  for (let i = 0; i < 256; i++) {
    wb += hist[i]; if (!wb) continue; const wf = g.length - wb; if (!wf) break;
    sb += i * hist[i]; const mb = sb / wb, mf = (sum - sb) / wf, v = wb * wf * (mb - mf) * (mb - mf);
    if (v > best) { best = v; t = i; }
  }
  return t;
}
function ocrLetterHeight(g, w, h, t) {
  // median height of ink blobs (connected components)
  const lab = new Int32Array(w * h).fill(-1), hs = [];
  const stack = [];
  for (let s = 0; s < w * h; s++) {
    if (g[s] > t || lab[s] >= 0) continue;
    let top = h, bot = 0, n = 0; stack.push(s); lab[s] = 1;
    while (stack.length) {
      const p = stack.pop(), y = (p / w) | 0, x = p - y * w; n++;
      if (y < top) top = y; if (y > bot) bot = y;
      for (let dy = -1; dy <= 1; dy++) for (let dx = -1; dx <= 1; dx++) {
        const yy = y + dy, xx = x + dx; if (yy < 0 || xx < 0 || yy >= h || xx >= w) continue;
        const q = yy * w + xx; if (g[q] <= t && lab[q] < 0) { lab[q] = 1; stack.push(q); }
      }
    }
    const bh = bot - top + 1; if (bh > 2 && bh < 120 && n > 4) hs.push(bh);
  }
  hs.sort((a, b) => a - b); return hs.length ? hs[hs.length >> 1] : 0;
}
async function ocrPrepare(img) {
  // work on a copy at most 2600 px wide for the measurements
  let { g, w, h } = ocrGray(img, 2600);
  // even out the lighting: divide by the page background (a closing on a quarter-size copy)
  const sw = Math.max(1, w >> 2), sh = Math.max(1, h >> 2), sc = document.createElement('canvas');
  sc.width = sw; sc.height = sh;
  const small = new Uint8ClampedArray(sw * sh);
  for (let y = 0; y < sh; y++) for (let x = 0; x < sw; x++) small[y * sw + x] = g[(y * 4) * w + x * 4];
  const bg = ocrMinMax(ocrMinMax(small, sw, sh, 3, true), sw, sh, 3, false);
  for (let y = 0; y < h; y++) {
    // smooth (bilinear) background, so the division leaves no blocks
    const fy = Math.min(sh - 1, Math.max(0, y / 4 - 0.5)), y0 = Math.floor(fy), y1 = Math.min(sh - 1, y0 + 1), wy = fy - y0;
    for (let x = 0; x < w; x++) {
      const fx = Math.min(sw - 1, Math.max(0, x / 4 - 0.5)), x0 = Math.floor(fx), x1 = Math.min(sw - 1, x0 + 1), wx = fx - x0;
      const b = (bg[y0 * sw + x0] * (1 - wx) + bg[y0 * sw + x1] * wx) * (1 - wy) + (bg[y1 * sw + x0] * (1 - wx) + bg[y1 * sw + x1] * wx) * wy;
      g[y * w + x] = Math.min(255, (g[y * w + x] * 255) / Math.max(1, b || 255));
    }
  }
  const t = ocrOtsu(g);
  // letters about 26 px tall read best (as accurate as larger, and twice as fast)
  const lh = ocrLetterHeight(g, w, h, t);
  const f = lh ? Math.max(0.5, Math.min(4, 26 / lh)) : 1;
  // straighten: the angle where text lines line up best
  let bestA = 0, bestV = -1;
  const qw = Math.max(1, w >> 2), qh = Math.max(1, h >> 2);
  for (let a = -5; a <= 5.001; a += 0.25) {
    const rad = a * Math.PI / 180, cs = Math.cos(rad), sn = Math.sin(rad), rows = new Float64Array(qh * 2);
    for (let y = 0; y < qh; y += 1) for (let x = 0; x < qw; x += 2) {
      if (g[(y * 4) * w + x * 4] > t) continue;
      const ry = Math.round((x - qw / 2) * sn + (y - qh / 2) * cs + qh);
      if (ry >= 0 && ry < rows.length) rows[ry]++;
    }
    let m = 0; for (const v of rows) m += v; m /= rows.length;
    let v = 0; for (const r of rows) v += (r - m) * (r - m);
    if (v > bestV) { bestV = v; bestA = a; }
  }
  const out = document.createElement('canvas');
  out.width = Math.round(w * f); out.height = Math.round(h * f);
  const src = document.createElement('canvas'); src.width = w; src.height = h;
  const id = src.getContext('2d').createImageData(w, h);
  for (let i = 0, j = 0; j < g.length; i += 4, j++) { id.data[i] = id.data[i + 1] = id.data[i + 2] = g[j]; id.data[i + 3] = 255; }
  src.getContext('2d').putImageData(id, 0, 0);
  const ctx = out.getContext('2d'); ctx.fillStyle = '#fff'; ctx.fillRect(0, 0, out.width, out.height);
  ctx.imageSmoothingEnabled = true; ctx.imageSmoothingQuality = 'high';
  ctx.translate(out.width / 2, out.height / 2); ctx.rotate(bestA * Math.PI / 180);
  ctx.drawImage(src, -out.width / 2, -out.height / 2, out.width, out.height);
  return out;
}

// PDFs: a page that carries its own text (typed, including Bijoy/SutonnyMJ PDFs) gives that text;
// a scanned page is drawn at about 300 dpi with pdf.js and read like a picture. Pages are opened
// one at a time, so a whole book doesn't fill the memory.
async function ocrPdfOpen(file) {
  const pdfjs = await import('/ocr/pdf.min.mjs');
  pdfjs.GlobalWorkerOptions.workerSrc = '/ocr/pdf.worker.min.mjs';
  // wasmUrl: the JBIG2 / JPEG 2000 decoders most scanned book PDFs need
  return pdfjs.getDocument({ data: new Uint8Array(await file.arrayBuffer()), wasmUrl: '/ocr/wasm/' }).promise;
}
async function ocrPdfText(page) {
  // the page's text, with its line breaks
  // raw text: normalising would drop the soft hyphen, which is ল-ফলা in Bijoy fonts
  const tc = await page.getTextContent({ disableNormalization: true });
  let out = '', lastY = null;
  for (const it of tc.items) {
    if (!('str' in it)) continue;
    const y = it.transform ? Math.round(it.transform[5]) : null;
    if (lastY !== null && y !== null && Math.abs(y - lastY) > 2 && !out.endsWith('\n')) out += '\n';
    out += it.str; if (it.hasEOL) out += '\n';
    lastY = y;
  }
  return out;
}
async function ocrPdfDraw(page) {
  const v1 = page.getViewport({ scale: 1 });
  const vp = page.getViewport({ scale: Math.min(4, 2480 / v1.width) });
  const c = document.createElement('canvas'); c.width = Math.round(vp.width); c.height = Math.round(vp.height);
  const x = c.getContext('2d'); x.fillStyle = '#fff'; x.fillRect(0, 0, c.width, c.height);
  await page.render({ canvasContext: x, viewport: vp }).promise;
  return c;
}
// "১-১০, ১৫" or "1-10,15" → page numbers (all pages when empty)
function ocrPageList(spec, n) {
  spec = (spec || '').replace(/[০-৯]/g, d => '০১২৩৪৫৬৭৮৯'.indexOf(d)).replace(/[–—]/g, '-').trim();
  if (!spec) return Array.from({ length: n }, (_, i) => i + 1);
  const out = [];
  for (const part of spec.split(/[,،\s]+/)) {
    const m = part.match(/^(\d+)(?:-(\d+))?$/); if (!m) continue;
    const a = Math.max(1, +m[1]), b = Math.min(n, m[2] ? +m[2] : +m[1]);
    for (let i = a; i <= b; i++) if (!out.includes(i)) out.push(i);
  }
  return out.length ? out : Array.from({ length: n }, (_, i) => i + 1);
}
function ocrImage(file) {
  return new Promise((ok, fail) => { const i = new Image(); i.onload = () => ok(i); i.onerror = fail; i.src = URL.createObjectURL(file); });
}
// OCR's lines with each word's confidence; null between paragraphs
function ocrLines(data) {
  const lines = [];
  for (const b of data.blocks || []) for (const p of b.paragraphs || []) {
    if (lines.length) lines.push(null);
    for (const l of p.lines || []) lines.push((l.words || []).map(w => [w.text, w.confidence]));
  }
  return lines;
}
async function ocrPost(path, body) {
  const res = await fetch(path, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
  if (!res.ok) throw new Error('Studio ' + res.status);
  return res.json();
}
async function ocrFiles(files) {
  files = Array.from(files || []).filter(f => f.type.startsWith('image/') || f.type === 'application/pdf' || /\.pdf$/i.test(f.name));
  if (!files.length || ocrBusy) return;
  ocrBusy = true; ocrStopNow = false;
  const stop = document.getElementById('ocrStop'); stop.hidden = false;
  const texts = []; ocrChanges = []; ocrDroppedAll = []; ocrPageData = [];
  const english = document.getElementById('ocrEnglish').checked;
  const junk = document.getElementById('ocrJunk').checked;
  let typed = 0, read = 0;
  try {
    // every page in order: pictures as they are, PDF pages by number (opened when their turn comes)
    const pages = [];
    for (const f of files) {
      if (f.type === 'application/pdf' || /\.pdf$/i.test(f.name)) {
        ocrStatus('PDF খোলা হচ্ছে…');
        const doc = await ocrPdfOpen(f);
        for (const n of ocrPageList(document.getElementById('ocrPages').value, doc.numPages)) pages.push({ doc, n });
      } else pages.push({ file: f });
    }
    let worker = null;
    for (let k = 0; k < pages.length && !ocrStopNow; k++) {
      const pg = pages[k], head = pages.length > 1 ? `পাতা ${bnNum(k + 1)}/${bnNum(pages.length)}: ` : '';
      let text = null;
      let img;
      if (pg.doc) {
        const page = await pg.doc.getPage(pg.n);
        // typed PDFs: use the page's own text when it is real Bangla (or Bijoy text)
        const own = await ocrPdfText(page);
        if (own.replace(/\s/g, '').length >= 20) {
          const tl = await ocrPost('/api/ocr/textlayer', { text: own });
          if (tl.use) { text = tl.text; typed++; }
        }
        if (text === null) img = await ocrPdfDraw(page);
        page.cleanup();
      } else img = await ocrImage(pg.file);
      if (img) {
        document.getElementById('ocrImg').src = img.src || img.toDataURL('image/jpeg', 0.7);
        document.getElementById('ocrOut').hidden = false;
        if (!worker) {
          worker = await ocrWorkerReady();
          // Bangla pages: no Latin letters, so a smudged line is never read as English nonsense
          // automatic page layout: columns, pictures and captions are found and read apart
          await worker.setParameters({
            tessedit_char_blacklist: english ? '' : 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz',
            tessedit_pageseg_mode: '3',
          });
        }
        ocrStatus(head + 'ছবি পরিষ্কার করা হচ্ছে…');
        await new Promise(r => setTimeout(r, 30));
        const canvas = await ocrPrepare(img);
        const { data } = await worker.recognize(canvas, {}, { text: true, blocks: true });
        ocrStatus(head + 'অভিধান দিয়ে বানান মেলানো হচ্ছে…');
        const lines = ocrLines(data);
        // kept for "save for learning": the picture as read and where each line was
        const boxes = [];
        for (const b of data.blocks || []) for (const pa of b.paragraphs || []) for (const l of pa.lines || []) boxes.push({ text: l.text, bbox: l.bbox });
        ocrPageData.push({ canvas, boxes });
        const out = await ocrPost('/api/ocr/correct', lines.length ? { lines, junk } : { text: data.text || '' });
        const offset = texts.reduce((n, t) => n + t.length + 2, 0);
        for (const c of out.changes || []) ocrChanges.push({ at: c.at + offset, from: c.from, to: c.to });
        for (const d of out.dropped || []) ocrDroppedAll.push(d);
        text = out.text || '';
        read++;
      } else document.getElementById('ocrOut').hidden = false;
      texts.push(text.trim());
      document.getElementById('ocrText').value = texts.join('\n\n');
    }
    ocrShowFixes();
    ocrFirstText = document.getElementById('ocrText').value;
    document.getElementById('ocrLearnBtn').hidden = !ocrPageData.length;
    document.getElementById('ocrSendBtn').hidden = !ocrPageData.length;
    const n = texts.length;
    let msg = n > 1 ? `${bnNum(n)}টি পাতা পড়া হয়েছে।` : 'পড়া হয়েছে। লেখাটি দেখে নিন, দরকার হলে এখানেই ঠিক করুন।';
    if (typed) msg += ` ${bnNum(typed)}টি পাতা PDF-এর নিজের লেখা থেকে নেওয়া (ছবি পড়ার দরকার হয়নি)।`;
    if (ocrStopNow) msg = `থামানো হয়েছে; ${bnNum(n)}টি পাতা পড়া হয়েছে।`;
    ocrStatus(msg);
  } catch (e) {
    ocrStatus('ছবিটি পড়া যায়নি: ' + (e && e.message ? e.message : e));
  } finally {
    ocrBusy = false; stop.hidden = true;
  }
}
function bnNum(n) { return String(n).replace(/[0-9]/g, d => '০১২৩৪৫৬৭৮৯'[d]); }
function ocrShowFixes() {
  const dbox = document.getElementById('ocrDropped');
  dbox.innerHTML = '';
  if (ocrDroppedAll.length) {
    const d = document.createElement('details');
    d.innerHTML = `<summary>ছবি, নকশা বা দাগ থেকে আসা ${bnNum(ocrDroppedAll.length)} জায়গার অক্ষর বাদ দেওয়া হয়েছে (দেখুন)</summary><div class="ocr-dropped"></div>`;
    d.querySelector('.ocr-dropped').textContent = ocrDroppedAll.join('  ‖  ');
    dbox.appendChild(d);
  }
  const box = document.getElementById('ocrFixes');
  box.innerHTML = '';
  if (!ocrChanges.length) return;
  box.append(`অভিধান দিয়ে ${bnNum(ocrChanges.length)}টি শব্দ ঠিক করা হয়েছে (ক্লিক করলে আগের মতো হবে): `);
  ocrChanges.forEach((c, i) => {
    const b = document.createElement('button'); b.className = 'ocr-fix';
    b.innerHTML = `<s></s> → <b></b>`; b.querySelector('s').textContent = c.from; b.querySelector('b').textContent = c.to;
    b.onclick = () => ocrUndo(i, b);
    box.appendChild(b);
  });
}
function ocrUndo(i, btn) {
  const c = ocrChanges[i], ta = document.getElementById('ocrText'), t = ta.value;
  // find the corrected word near where it was put (the text may have been edited since)
  let at = t.indexOf(c.to, Math.max(0, c.at - 40));
  if (at < 0) at = t.indexOf(c.to);
  if (at < 0) { btn.remove(); return; }
  ta.value = t.slice(0, at) + c.from + t.slice(at + c.to.length);
  const d = c.from.length - c.to.length;
  ocrChanges.forEach(o => { if (o.at > c.at) o.at += d; });
  btn.remove();
}
// The page's line breaks out, paragraphs (blank lines) kept
function ocrJoinLines() {
  const ta = document.getElementById('ocrText');
  ta.value = ta.value.split(/\n\s*\n/).map(p => p.replace(/\s*\n\s*/g, ' ').trim()).filter(Boolean).join('\n\n');
  ocrChanges = []; document.getElementById('ocrFixes').innerHTML = '';
}
function ocrToPad() {
  const out = document.getElementById('banglaPad');
  if (out) { out.value = (out.value ? out.value + '\n' : '') + document.getElementById('ocrText').value; if (typeof updateCount === 'function') updateCount(); }
  showTab('playground');
}
// Save the picture(s) as read with the corrected text, on this computer, as material to train the
// reader on later. Nothing is sent anywhere.
async function ocrSaveForLearning() {
  const id = new Date().toISOString().replace(/[-:]/g, '').replace('T', '-').slice(0, 15) + '-' + Math.random().toString(36).slice(2, 6);
  try {
    for (let i = 0; i < ocrPageData.length; i++) {
      const jpg = ocrPageData[i].canvas.toDataURL('image/jpeg', 0.85).split(',')[1];
      await ocrPost('/api/ocr/sample', { id, page: i + 1, jpg, lines: ocrPageData[i].boxes });
    }
    const r = await ocrPost('/api/ocr/sample', { id, corrected: document.getElementById('ocrText').value, original: ocrFirstText });
    toast('রাখা হয়েছে: ' + (r.dir || ''));
  } catch (e) {
    toast('রাখা যায়নি: ' + (e && e.message ? e.message : e));
  }
}
// Send the corrected page(s) to Ekushe (only after the person agrees in the dialog), to train the
// reader on real pages. Goes to ekushenext.com/api/ocr-sample; nothing is sent otherwise.
const OCR_SEND_URL = 'https://ekushenext.com/api/ocr-sample';
let appInfo = { app: '', os: /Windows/.test(navigator.userAgent) ? 'Windows' : /Mac/.test(navigator.userAgent) ? 'macOS' : /Linux/.test(navigator.userAgent) ? 'Linux' : '' };
function ocrSendAsk() {
  if (!ocrPageData.length) return;
  document.getElementById('ocrSendPages').textContent = ocrPageData.length > 10 ? 'প্রথম ১০টি পাতা' : ocrPageData.length > 1 ? `${bnNum(ocrPageData.length)}টি পাতা` : 'একটি পাতা';
  document.getElementById('ocrSendSame').hidden = document.getElementById('ocrText').value !== ocrFirstText;
  document.getElementById('ocrSendGo').disabled = false;
  document.getElementById('ocrSendDlg').showModal();
}
// A page picture small enough to send (long side at most 2600 px, under 3.5 MB)
function ocrJpeg(canvas) {
  let c = canvas;
  const long = Math.max(c.width, c.height);
  if (long > 2600) {
    const k = 2600 / long, d = document.createElement('canvas');
    d.width = Math.round(c.width * k); d.height = Math.round(c.height * k);
    d.getContext('2d').drawImage(c, 0, 0, d.width, d.height); c = d;
  }
  for (const q of [0.85, 0.7, 0.55]) {
    const b = c.toDataURL('image/jpeg', q).split(',')[1];
    if (b.length * 0.75 < 3.5e6) return { jpg: b, scale: c.width / canvas.width };
  }
  return null;
}
async function ocrSend() {
  const go = document.getElementById('ocrSendGo'); go.disabled = true;
  try {
    const pages = [];
    for (const p of ocrPageData.slice(0, 10)) {
      const j = ocrJpeg(p.canvas);
      if (!j) continue;
      const k = j.scale;
      pages.push({ jpg: j.jpg, lines: p.boxes.map(b => ({ text: b.text, bbox: b.bbox && { x0: b.bbox.x0 * k, y0: b.bbox.y0 * k, x1: b.bbox.x1 * k, y1: b.bbox.y1 * k } })) });
    }
    const res = await fetch(OCR_SEND_URL, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ app: appInfo.app, os: appInfo.os, corrected: document.getElementById('ocrText').value, original: ocrFirstText, pages }),
    });
    const r = await res.json().catch(() => ({}));
    document.getElementById('ocrSendDlg').close();
    if (r.ok) { toast('পাঠানো হয়েছে, ধন্যবাদ! একুশের ছবি পড়া এতে আরও ভালো হবে।'); document.getElementById('ocrSendBtn').hidden = true; }
    else toast(r.error === 'too_many' ? 'আজ অনেকবার পাঠানো হয়েছে, কাল আবার চেষ্টা করুন।' : 'পাঠানো যায়নি' + (r.error ? ' (' + r.error + ')' : '') + '। পরে আবার চেষ্টা করুন।');
  } catch (e) {
    document.getElementById('ocrSendDlg').close();
    toast('পাঠানো যায়নি: ইন্টারনেট সংযোগ দেখে আবার চেষ্টা করুন।');
  }
}
function ocrSave() {
  const blob = new Blob(['﻿' + document.getElementById('ocrText').value], { type: 'text/plain;charset=utf-8' });
  const a = document.createElement('a'); a.href = URL.createObjectURL(blob); a.download = 'ekushe-ocr.txt'; a.click();
  setTimeout(() => URL.revokeObjectURL(a.href), 2000);
}
(function ocrWire() {
  const drop = document.getElementById('ocrDrop');
  drop.addEventListener('dragover', e => { e.preventDefault(); drop.classList.add('over'); });
  drop.addEventListener('dragleave', () => drop.classList.remove('over'));
  drop.addEventListener('drop', e => { e.preventDefault(); drop.classList.remove('over'); ocrFiles(e.dataTransfer.files); });
  document.addEventListener('paste', e => {
    if (!document.getElementById('tab-ocr').classList.contains('active')) return;
    const files = Array.from(e.clipboardData && e.clipboardData.files || []);
    if (files.length) { e.preventDefault(); ocrFiles(files); }
  });
})();

// ── একুশেকে কণ্ঠ দিন: read the training sentences aloud and send them (only after agreeing) ──
// Recordings are used only to train Ekushe's own voice models and are never published.
const VOICE_SEND_URL = 'https://ekushenext.com/api/voice-sample';
const DONATE_BATCHES = [{"t":"রোজকার জীবন","note":"","s":["আজ সকালে ঘুম থেকে উঠে প্রথমে এক কাপ চা খেলাম।","বাইরে বেশ ঠান্ডা পড়েছে, গরম কাপড় পরে বের হয়ো।","বাজার থেকে আলু, পেঁয়াজ আর কিছু সবজি আনতে হবে।","ছেলেমেয়েরা স্কুল থেকে ফিরে খেলতে গেছে।","রাতে খাওয়ার পর আমরা সবাই মিলে একটু হাঁটতে বের হই।","আজকে আকাশ মেঘলা, মনে হচ্ছে বিকেলে বৃষ্টি হবে।","ঘরটা পরিষ্কার করতে প্রায় দুই ঘণ্টা লেগে গেল।","ছুটির দিনে আমি সাধারণত বই পড়ে সময় কাটাই।","পাশের বাসার ভদ্রলোক খুব মিশুক মানুষ।","মাছ ভাজার গন্ধে পুরো বাড়ি ভরে গেছে।","আজকে কাজের চাপ কম, তাই তাড়াতাড়ি বাসায় ফিরব।","বাগানের গোলাপ গাছে নতুন ফুল ফুটেছে।","সন্ধ্যায় টিভিতে একটা ভালো নাটক দেখলাম।","মা বললেন আজ রাতে খিচুড়ি রান্না হবে।","বাসের জন্য প্রায় আধা ঘণ্টা দাঁড়িয়ে থাকতে হলো।","শীতের সকালে কুয়াশায় রাস্তা ঠিকমতো দেখা যায় না।","আমার ছোট ভাই এখন বিশ্ববিদ্যালয়ে পড়ে।","বিদ্যুৎ চলে যাওয়ায় মোমবাতি জ্বালিয়ে বসে ছিলাম।","কাল সকালে একটু আগে উঠতে হবে।","আজকের দিনটা বেশ ভালোই কাটল।"]},{"t":"গল্পের বর্ণনা","note":"","s":["সেদিন ভোরবেলা গ্রামের পথ ধরে একা হাঁটছিল রফিক, চারদিকে তখনো কুয়াশার চাদর।","দূরে নদীর ঘাটে কয়েকটা নৌকা বাঁধা, মাঝিরা তখনো ঘুম থেকে ওঠেনি।","বাড়িটা অনেক পুরনো, দেয়ালের গায়ে শ্যাওলা জমে সবুজ হয়ে গেছে।","জানালার পাশে বসে মেয়েটি অনেকক্ষণ বৃষ্টির দিকে তাকিয়ে রইল।","হঠাৎ দরজায় কড়া নাড়ার শব্দে সবাই চমকে উঠল।","বাবার পুরনো ডায়েরিটা খুলতেই একটা হলুদ হয়ে যাওয়া চিঠি বেরিয়ে এল।","সন্ধ্যা নামার আগেই পাখিরা দল বেঁধে নীড়ে ফিরতে শুরু করল।","দাদির মুখে শোনা সেই গল্পটা আজও আমার স্পষ্ট মনে আছে।","ট্রেনটা যখন স্টেশন ছাড়ল, প্ল্যাটফর্মে দাঁড়িয়ে সে শুধু হাত নাড়ছিল।","অনেক বছর পর নিজের স্কুলের সামনে দাঁড়িয়ে তার চোখ ভিজে এল।","চাঁদের আলোয় পুরো মাঠটা যেন রুপালি হয়ে উঠেছিল।","ছোট্ট ছেলেটি মায়ের আঁচল ধরে ভিড়ের মধ্যে হাঁটছিল।","শীতের রাতে আগুন পোহাতে পোহাতে সবাই পুরনো দিনের গল্পে মেতে উঠল।","চিঠিটা পড়ে শেষ করার পর সে অনেকক্ষণ চুপ করে বসে রইল।","বর্ষায় নদীটা ফুলে ফেঁপে উঠে দুই পাড় ভাসিয়ে দিত।","সেই রাতে কেউ ঘুমাতে পারেনি, সবাই ভোরের অপেক্ষায় জেগে ছিল।","বাজারের কোলাহল পেরিয়ে সে একটা নির্জন গলিতে ঢুকে পড়ল।","মাস্টারমশাই চশমাটা নাকের ডগায় নামিয়ে ক্লাসের দিকে তাকালেন।","পুকুরপাড়ের আমগাছটার নিচে ছিল আমাদের ছোটবেলার খেলার জায়গা।","অবশেষে বহু প্রতীক্ষার পর গ্রামে বিদ্যুতের আলো এসে পৌঁছাল।"]},{"t":"কাজ ও অফিস","note":"","s":["আগামীকাল সকাল দশটায় দলের সবার সঙ্গে বৈঠক আছে।","প্রতিবেদনটা আজকের মধ্যে জমা দিতে হবে।","নতুন প্রকল্পের বাজেট এখনও চূড়ান্ত হয়নি।","দয়া করে সংযুক্ত নথিটি দেখে মতামত জানাবেন।","গ্রাহকের অভিযোগটা দ্রুত সমাধান করা দরকার।","এই মাসে আমাদের বিক্রি আগের মাসের চেয়ে বেড়েছে।","ছুটির আবেদনটা ব্যবস্থাপকের কাছে পাঠিয়েছি।","সভার সিদ্ধান্তগুলো সবাইকে জানিয়ে দেওয়া হবে।","আমাদের দলে দুজন নতুন সদস্য যোগ দিয়েছেন।","কাজটা শেষ করতে আরও তিন দিন সময় লাগবে।","চুক্তির শর্তগুলো আরেকবার পড়ে দেখা উচিত।","অফিসের নতুন নিয়ম সোমবার থেকে চালু হবে।","হিসাব বিভাগ থেকে বেতনের তথ্য চেয়ে পাঠানো হয়েছে।","প্রশিক্ষণ কর্মশালাটি দুই দিন ধরে চলবে।","সবাইকে নির্ধারিত সময়ের মধ্যে উপস্থিত থাকতে অনুরোধ করা হলো।","সরবরাহকারীর সঙ্গে দাম নিয়ে আবার আলোচনা করতে হবে।","বছরের শেষে সবার কাজের মূল্যায়ন হবে।","আমি আজ বাসা থেকে কাজ করছি।","প্রস্তাবটা অনুমোদন পেলে আমরা কাজ শুরু করব।","আপনার সহযোগিতার জন্য আন্তরিক ধন্যবাদ।"]},{"t":"সংখ্যা, টাকা, সময় ও তারিখ","note":"সংখ্যাগুলো যেভাবে মুখে বলেন সেভাবে কথায় পড়ুন।","s":["আমার মোবাইল নম্বরের শেষ চারটি অঙ্ক হলো পাঁচ তিন দুই আট।","জিনিসটার দাম পড়েছে বারোশো পঞ্চাশ টাকা।","ট্রেন ছাড়বে বিকেল চারটা পনেরো মিনিটে।","আজ ষোলোই ডিসেম্বর, আমাদের বিজয় দিবস।","ক্লাসে মোট পঁয়তাল্লিশজন ছাত্রছাত্রী আছে।","বাড়ি ভাড়া মাসে আঠারো হাজার টাকা।","মিটিং শুরু হবে সাড়ে এগারোটায়।","তার জন্ম উনিশশো নব্বই সালের মার্চ মাসে।","এক কেজি চালের দাম এখন পঁয়ষট্টি টাকা।","অফিস থেকে বাসার দূরত্ব প্রায় সাত কিলোমিটার।","দুই হাজার পঁচিশ সালে আমরা নতুন বাসায় উঠেছি।","পরীক্ষার ফল বের হবে পয়লা জানুয়ারি।","তিনি একশো দশ নম্বর বাড়িতে থাকেন।","আমাদের গ্রামে প্রায় তিন হাজার মানুষের বাস।","বিলটা আগামী সাতাশ তারিখের মধ্যে দিতে হবে।","গাড়িটা ঘণ্টায় আশি কিলোমিটার বেগে চলছিল।","পাঁচ ভাগের দুই ভাগ কাজ ইতিমধ্যে শেষ হয়েছে।","ব্যাংকে দেড় লাখ টাকা জমা দিয়েছি।","সকাল ছয়টা থেকে রাত দশটা পর্যন্ত দোকান খোলা থাকে।","তাপমাত্রা আজ বত্রিশ ডিগ্রি সেলসিয়াস।"]},{"t":"গল্পের সংলাপ","note":"","s":["রহিম বলল, আমি আর এক মুহূর্তও এখানে থাকতে চাই না।","মা ডেকে বললেন, ভাত ঠান্ডা হয়ে যাচ্ছে, তাড়াতাড়ি খেতে আয়।","মেয়েটি অবাক হয়ে জিজ্ঞেস করল, তুমি এতদিন কোথায় ছিলে?","বুড়ো মাঝি হেসে বলল, নদী কাউকে ফেরায় না, বাবা।","শিক্ষক বললেন, যে প্রশ্ন করে না, সে কিছুই শেখে না।","সে ফিসফিস করে বলল, কাউকে কিছু বোলো না, এটা আমাদের গোপন কথা।","দাদু চোখ বন্ধ করে বললেন, সেই দিনগুলো আর কখনো ফিরে আসবে না।","ছেলেটি কাঁদতে কাঁদতে বলল, আমার বইটা কেউ নিয়ে গেছে।","ডাক্তার শান্ত গলায় বললেন, ভয় পাবেন না, সব ঠিক হয়ে যাবে।","বন্ধুটি কাঁধে হাত রেখে বলল, যা হওয়ার হয়ে গেছে, এবার সামনে তাকাও।","পুলিশ কর্মকর্তা জানতে চাইলেন, ঘটনাটা ঠিক কখন ঘটেছিল?","নানি বললেন, মানুষের আসল পরিচয় তার ব্যবহারে।","সে রেগে গিয়ে বলল, তোমার কথার কোনো দাম নেই আমার কাছে।","অপরিচিত লোকটি নরম গলায় বলল, আমি পথ হারিয়েছি, একটু সাহায্য করবেন?","বাবা বললেন, কষ্ট করে যা পাওয়া যায়, তার মূল্য সবচেয়ে বেশি।","মেয়েটি হাসতে হাসতে বলল, তুমি একদম বদলাওনি।","গ্রামের মোড়ল ঘোষণা দিলেন, কাল সবাই স্কুলমাঠে জড়ো হবে।","সে দীর্ঘশ্বাস ফেলে বলল, জীবনটা যদি আবার শুরু করা যেত।","ছোট বোন জেদ ধরে বলল, আমিও তোমার সঙ্গে মেলায় যাব।","বিদায়ের সময় সে বলল, ভালো থেকো, আবার দেখা হবে।"]},{"t":"জায়গা, নাম ও ভ্রমণ","note":"","s":["আমরা ঈদের ছুটিতে কক্সবাজারে বেড়াতে যাব।","সিলেটের চা বাগানগুলো দেখতে অপূর্ব।","রাজশাহী আমের জন্য বিখ্যাত।","চট্টগ্রাম থেকে বান্দরবান যেতে কয়েক ঘণ্টা লাগে।","সুন্দরবনে রয়েল বেঙ্গল টাইগার দেখা যায়।","ঢাকার গুলশান থেকে উত্তরা যেতে অনেক সময় লাগে।","করিম সাহেব খুলনার একটি স্কুলে শিক্ষকতা করেন।","ফাতেমা আর রহিম গত বছর বিয়ে করেছে।","পদ্মা সেতু পার হয়ে আমরা বরিশাল পৌঁছালাম।","কুমিল্লার রসমালাই সবার প্রিয়।","নাসরিন আপা ময়মনসিংহে থাকেন।","আমার নানাবাড়ি রংপুরের একটি গ্রামে।","কলকাতার বইমেলায় অনেক মানুষের ভিড় হয়।","টরন্টোতে শীতকালে প্রচুর তুষার পড়ে।","মাহমুদ ভাই আগামী মাসে লন্ডন যাচ্ছেন।","সাজেক ভ্যালিতে মেঘের ভেতর দিয়ে হাঁটা যায়।","বগুড়ার দই খুব নামকরা।","তানভীর আর সুমি একই অফিসে কাজ করে।","নারায়ণগঞ্জ থেকে লঞ্চে চাঁদপুর যাওয়া যায়।","শ্রীমঙ্গলে সাত রঙের চা পাওয়া যায়।"]},{"t":"বাংলা কথায় ইংরেজি শব্দ","note":"","s":["আমার ল্যাপটপের ব্যাটারি একদম শেষ।","ইমেইলটা পাঠানোর আগে একবার চেক করে নিও।","অফিসের ওয়াইফাই আজকে খুব স্লো।","প্রেজেন্টেশনের স্লাইডগুলো রেডি করতে হবে।","মোবাইলে নতুন অ্যাপটা ইনস্টল করেছি।","ডাক্তারের অ্যাপয়েন্টমেন্ট নিয়েছি বৃহস্পতিবার।","পাসওয়ার্ড ভুলে গেলে রিসেট করতে হয়।","অনলাইনে অর্ডার দিলে দুই দিনে ডেলিভারি দেয়।","ফেসবুকে একটা ভিডিও শেয়ার করেছি।","মিটিংয়ের লিংকটা হোয়াটসঅ্যাপে পাঠিয়ে দাও।","কম্পিউটারটা রিস্টার্ট দিলে হয়তো ঠিক হয়ে যাবে।","এই সফটওয়্যারের লাইসেন্স কিনতে হবে না, এটা ফ্রি।","ব্যাংকের কার্ড দিয়ে পেমেন্ট করা যাবে।","প্রিন্টারে কাগজ আটকে গেছে।","ডেটাগুলো এক্সেল শিটে রাখা আছে।","ইউটিউবে আমার নতুন চ্যানেলটা দেখো।","ফোনের চার্জার বাসায় ফেলে এসেছি।","নতুন আপডেট আসার পর ফোনটা দ্রুত চলছে।","ক্লাউডে ব্যাকআপ রাখলে ফাইল হারানোর ভয় থাকে না।","কিবোর্ডে বাংলা লিখতে একুশে ব্যবহার করি।"]},{"t":"চিহ্নসহ বলা লেখা","note":"“কমা”, “দাঁড়ি”, “প্রশ্নবোধক চিহ্ন”, “নতুন লাইন” — লেখা শব্দগুলোও বলুন।","s":["প্রিয় রাশেদ কমা নতুন লাইন কেমন আছ প্রশ্নবোধক চিহ্ন","আজ বিকেলে আমাদের সভা হবে দাঁড়ি","আমি বাজারে যাচ্ছি কমা তোমার কিছু লাগবে প্রশ্নবোধক চিহ্ন","সবাইকে অনেক ধন্যবাদ দাঁড়ি নতুন লাইন","মিটিং শুরু হবে দশটায় দাঁড়ি কেউ দেরি করবেন না দাঁড়ি","চাল কমা ডাল কমা তেল আর লবণ আনতে হবে দাঁড়ি","তুমি কি কাল আসতে পারবে প্রশ্নবোধক চিহ্ন","শুভেচ্ছা রইল দাঁড়ি নতুন লাইন ইতি কমা তোমার বন্ধু","কাজটা শেষ হয়েছে দাঁড়ি এখন বাসায় যাচ্ছি দাঁড়ি","কী সুন্দর একটা দিন বিস্ময়সূচক চিহ্ন","বাজারের তালিকা কোলন নতুন লাইন আলু কমা পেঁয়াজ কমা রসুন","আজ এই পর্যন্ত দাঁড়ি নতুন অনুচ্ছেদ","আপনার চিঠি পেয়েছি দাঁড়ি উত্তর দিতে দেরি হলো বলে দুঃখিত দাঁড়ি","সভার বিষয় কোলন নতুন বছরের পরিকল্পনা দাঁড়ি","অভিনন্দন বিস্ময়সূচক চিহ্ন তুমি পরীক্ষায় প্রথম হয়েছ দাঁড়ি","তুমি কখন ফিরবে কমা আমাকে জানিও দাঁড়ি","প্রথমত কমা কাজটা সময়মতো শেষ করতে হবে দাঁড়ি","ঠিক আছে কমা আমি রাজি দাঁড়ি","নতুন অনুচ্ছেদ আগামীকাল অফিস বন্ধ থাকবে দাঁড়ি","আর কিছু জানার থাকলে প্রশ্ন করুন দাঁড়ি নতুন লাইন ধন্যবাদ দাঁড়ি"]},{"t":"স্বাস্থ্য, খাবার ও ঘর","note":"","s":["প্রতিদিন অন্তত আধা ঘণ্টা হাঁটা স্বাস্থ্যের জন্য ভালো।","বেশি করে পানি খেলে শরীর ভালো থাকে।","ডাক্তার বলেছেন তিন বেলা খাওয়ার পর ওষুধ খেতে।","আজ দুপুরে ভাত, ডাল আর ইলিশ মাছ রান্না হয়েছে।","মাথা ব্যথা করছে, একটু বিশ্রাম নেব।","রান্নাঘরের কল দিয়ে পানি পড়ছে, মিস্ত্রি ডাকতে হবে।","বাচ্চাদের টিকা দেওয়ার তারিখ মনে রাখতে হবে।","সকালে নাশতায় রুটি আর ডিম ভাজি খাই।","ফ্রিজে দুধ শেষ হয়ে গেছে।","বেশি রাত জাগলে পরদিন শরীর খারাপ লাগে।","শীতে ঠোঁট ফাটলে নারকেল তেল দিতে পারো।","ঘরের বাতিগুলো বদলে নতুন এলইডি লাগিয়েছি।","মায়ের হাতের পিঠা আমার খুব প্রিয়।","জ্বর থাকলে বেশি করে তরল খাবার খেতে হয়।","বারান্দার গাছগুলোতে প্রতিদিন পানি দিই।","বাসায় মেহমান আসবে, পোলাও আর মাংস রান্না হবে।","চোখ ভালো রাখতে মোবাইল কম দেখা উচিত।","ধুলাবালিতে আমার হাঁচি শুরু হয়।","কাপড়গুলো ছাদে শুকাতে দিয়েছি।","রাতে হালকা খাবার খেলে ভালো ঘুম হয়।"]},{"t":"লম্বা বাক্য","note":"","s":["আমরা ঠিক করেছি আগামী মাসের শুরুতে পুরো পরিবার নিয়ে কয়েক দিনের জন্য গ্রামের বাড়িতে বেড়াতে যাব।","প্রযুক্তি যত এগিয়ে যাচ্ছে, মানুষের কাজের ধরনও তত দ্রুত বদলে যাচ্ছে।","বাংলা ভাষায় কম্পিউটারে লেখা এখন আগের চেয়ে অনেক সহজ হয়ে গেছে।","ছোটবেলায় বৃষ্টির দিনে আমরা উঠানে কাগজের নৌকা ভাসিয়ে খেলতাম।","পরীক্ষার আগের রাতে বেশি না পড়ে ভালো করে ঘুমানো বুদ্ধিমানের কাজ।","শহরের রাস্তায় যানজট কমাতে হলে গণপরিবহন ব্যবস্থা আরও উন্নত করতে হবে।","নতুন কিছু শিখতে চাইলে প্রতিদিন অল্প অল্প করে চর্চা করাই সবচেয়ে ভালো উপায়।","তিনি সারা জীবন গ্রামের গরিব ছেলেমেয়েদের লেখাপড়ার জন্য কাজ করে গেছেন।","বইমেলায় গিয়ে আমি পছন্দের লেখকদের কয়েকটি নতুন বই কিনে এনেছি।","বিদেশে থাকলেও দেশের মানুষ আর খাবারের কথা সবসময় মনে পড়ে।","সকালে বৃষ্টি হওয়ায় রাস্তায় পানি জমে গিয়েছিল, তাই অফিসে পৌঁছাতে দেরি হলো।","একটা ভালো অভ্যাস গড়ে তুলতে সময় লাগে, কিন্তু ভেঙে ফেলতে এক মুহূর্তও লাগে না।","আমাদের পাড়ার লাইব্রেরিতে প্রতি শুক্রবার শিশুদের জন্য গল্প পড়ার আসর বসে।","নদীর পাড়ে বসে সূর্যাস্ত দেখার মতো শান্তির মুহূর্ত খুব কমই আছে।","অনলাইনে কেনাকাটা করার সময় বিক্রেতার পরিচয় ভালো করে যাচাই করে নেওয়া উচিত।","কৃষকেরা সারা বছর কঠোর পরিশ্রম করেন বলেই আমরা প্রতিদিন খেতে পাই।","মোবাইল ফোন আমাদের জীবন সহজ করেছে, তবে এর অতিরিক্ত ব্যবহার ক্ষতিকর।","আগামী সপ্তাহে স্কুলে বার্ষিক ক্রীড়া প্রতিযোগিতা আর পুরস্কার বিতরণী অনুষ্ঠান হবে।","পুরনো ছবিগুলো দেখলে ফেলে আসা দিনের কত কথা মনে পড়ে যায়।","ভালো মানুষ হওয়ার জন্য অনেক টাকা লাগে না, দরকার একটা সুন্দর মন।"]},{"t":"খবর ও আনুষ্ঠানিক বাংলা","note":"","s":["দেশের উত্তরাঞ্চলে শৈত্যপ্রবাহ অব্যাহত রয়েছে।","আবহাওয়া অধিদপ্তর আগামী তিন দিন বৃষ্টির সম্ভাবনার কথা জানিয়েছে।","চলতি অর্থবছরে রপ্তানি আয় উল্লেখযোগ্য হারে বেড়েছে।","সড়ক দুর্ঘটনা রোধে নতুন আইন কার্যকর করা হয়েছে।","বিশ্ববিদ্যালয়ে ভর্তি পরীক্ষার ফল প্রকাশিত হয়েছে।","বন্যাকবলিত এলাকায় ত্রাণ বিতরণ শুরু হয়েছে।","জাতীয় ক্রিকেট দল আগামী সপ্তাহে সফরে যাচ্ছে।","নগরীর বিভিন্ন স্থানে মশক নিধন অভিযান চালানো হচ্ছে।","কৃষকদের জন্য নতুন ঋণ সুবিধা ঘোষণা করা হয়েছে।","অনুষ্ঠানে প্রধান অতিথি হিসেবে উপস্থিত ছিলেন জেলা প্রশাসক।","গবেষকেরা ধানের একটি নতুন জাত উদ্ভাবন করেছেন।","শিল্পকলা একাডেমিতে সপ্তাহব্যাপী নাট্যোৎসব শুরু হয়েছে।","বিদ্যুৎ সাশ্রয়ে সবাইকে সচেতন হওয়ার আহ্বান জানানো হয়েছে।","মহাসড়কে যান চলাচল স্বাভাবিক রয়েছে।","ডেঙ্গু প্রতিরোধে বাড়ির আশপাশ পরিষ্কার রাখার পরামর্শ দেওয়া হয়েছে।","আন্তর্জাতিক মাতৃভাষা দিবস উপলক্ষে শহীদ মিনারে শ্রদ্ধা জানানো হয়।","মেট্রোরেল রাজধানীর যাতায়াত অনেক সহজ করেছে।","পরিবেশ রক্ষায় বেশি করে গাছ লাগানোর উদ্যোগ নেওয়া হয়েছে।","বাজারে নিত্যপ্রয়োজনীয় পণ্যের দাম কিছুটা কমেছে।","সংবাদ সম্মেলনে প্রতিষ্ঠানটির ভবিষ্যৎ পরিকল্পনা তুলে ধরা হয়।"]},{"t":"প্রবন্ধ ও নিবন্ধ","note":"","s":["ভাষা শুধু ভাব প্রকাশের মাধ্যম নয়, এটি একটি জাতির ইতিহাস আর সংস্কৃতির ধারক।","বায়ান্নর ভাষা আন্দোলন আমাদের আত্মপরিচয়ের পথে প্রথম বড় পদক্ষেপ ছিল।","বই পড়ার অভ্যাস মানুষের কল্পনাশক্তি আর চিন্তার গভীরতা বাড়িয়ে দেয়।","গ্রামীণ জীবনের সরলতা আজও অনেক লেখকের লেখায় বারবার ফিরে আসে।","জলবায়ু পরিবর্তনের প্রভাবে উপকূলীয় এলাকার মানুষ নতুন নতুন সংকটের মুখোমুখি হচ্ছে।","একটি ভালো গল্পের শুরুতেই পাঠকের মনোযোগ ধরে রাখার মতো কিছু থাকা দরকার।","শিশুর মানসিক বিকাশে পরিবারের ভূমিকা সবচেয়ে গুরুত্বপূর্ণ।","নগরায়ণের ফলে আমাদের চারপাশ থেকে খোলা মাঠ আর জলাশয় দ্রুত হারিয়ে যাচ্ছে।","সাহিত্য আমাদের অন্যের জীবনকে নিজের চোখে দেখতে শেখায়।","প্রযুক্তির সঠিক ব্যবহার শিক্ষাকে আরও সহজ আর আনন্দময় করে তুলতে পারে।","লোকসংগীত আমাদের মাটি আর মানুষের সুখ দুঃখের কথা বলে।","উপন্যাসের চরিত্রগুলো যত বাস্তব হয়, পাঠক তত সহজে তাদের সঙ্গে একাত্ম হয়।","সুস্থ সমাজ গড়তে হলে প্রত্যেক নাগরিককে নিজের দায়িত্ব সম্পর্কে সচেতন হতে হবে।","নদীমাতৃক এই দেশের মানুষের জীবন আর জীবিকা নদীর সঙ্গে গভীরভাবে জড়িয়ে আছে।","লেখালেখির জন্য প্রতিভার পাশাপাশি নিয়মিত চর্চা আর ধৈর্য প্রয়োজন।","পহেলা বৈশাখ বাঙালির সবচেয়ে বড় সর্বজনীন উৎসব।","ইতিহাস থেকে শিক্ষা না নিলে একই ভুল বারবার ফিরে আসে।","প্রবন্ধের প্রতিটি অনুচ্ছেদে একটি মূল ভাবনা স্পষ্টভাবে তুলে ধরা উচিত।","মানুষের প্রতি ভালোবাসা আর সহমর্মিতাই একটি সমাজকে সত্যিকারের মানবিক করে তোলে।","লেখা শেষ করার পর একবার জোরে পড়ে দেখলে অনেক ভুল সহজেই চোখে পড়ে।"]}];
let donate = { batch: 0, i: 0, takes: {}, rec: null, stream: null, timer: null, audio: null, mic: '', ctx: null, analyser: null, peak: 0 };
const donateStore = {
  get() { try { return JSON.parse(localStorage.getItem('ekushe-voice-donor') || 'null'); } catch (e) { return null; } },
  set(v) { try { localStorage.setItem('ekushe-voice-donor', JSON.stringify(v)); } catch (e) {} },
};
function donateInit() {
  if (!(navigator.mediaDevices && navigator.mediaDevices.getUserMedia && window.MediaRecorder)) return;
  document.getElementById('donateBox').hidden = false;
  const sel = document.getElementById('donateBatch');
  sel.innerHTML = DONATE_BATCHES.map((b, k) => `<option value="${k}">${bnNum(k + 1)}. ${b.t}</option>`).join('');
  const d = donateStore.get();
  if (d && d.agreed) donateShow(d);
}
function donateAsk() { document.getElementById('donateDlg').showModal(); }
function donateAgree() {
  if (!document.getElementById('donateAdult').checked || !document.getElementById('donateAgree').checked) {
    toast('শুরু করতে দুটি ঘরেই টিক দিন'); return;
  }
  const r = new Uint8Array(12); crypto.getRandomValues(r);
  const d = Object.assign(donateStore.get() || {}, {
    agreed: new Date().toISOString(),
    code: (donateStore.get() || {}).code || [...r].map(x => x.toString(16).padStart(2, '0')).join(''),
    region: document.getElementById('donateRegion').value, age: document.getElementById('donateAge').value,
    mic: document.getElementById('donateMic').value, sent: (donateStore.get() || {}).sent || {},
  });
  donateStore.set(d);
  document.getElementById('donateDlg').close();
  donateShow(d);
}
async function donateMics() {
  try {
    const sel = document.getElementById('donateMicSel'), keep = sel.value;
    const list = (await navigator.mediaDevices.enumerateDevices()).filter(d => d.kind === 'audioinput' && d.deviceId && d.deviceId !== 'default');
    sel.innerHTML = '<option value="">কম্পিউটারের ঠিক করা মাইক</option>' + list.map((d, k) => `<option value="${d.deviceId}">${(d.label || 'মাইক ' + bnNum(k + 1)).replace(/</g, '&lt;')}</option>`).join('');
    sel.value = keep || (donateStore.get() || {}).micId || '';
  } catch (e) {}
}
function donateSetMic(id) {
  if (donate.stream) { donate.stream.getTracks().forEach(t => t.stop()); donate.stream = null; }
  const d = donateStore.get() || {}; d.micId = id; donateStore.set(d);
}
// The microphone, raw like voice typing hears it, with a level meter while recording
async function donateOpenMic() {
  if (donate.stream) return donate.stream;
  const id = document.getElementById('donateMicSel').value;
  const audio = { echoCancellation: false, noiseSuppression: false, autoGainControl: false, channelCount: 1 };
  if (id) audio.deviceId = { exact: id };
  donate.stream = await navigator.mediaDevices.getUserMedia({ audio });
  try {
    donate.ctx = donate.ctx || new (window.AudioContext || window.webkitAudioContext)();
    const src = donate.ctx.createMediaStreamSource(donate.stream);
    donate.analyser = donate.ctx.createAnalyser(); donate.analyser.fftSize = 1024;
    src.connect(donate.analyser);
  } catch (e) { donate.analyser = null; }
  donateMics();   // device names show once the microphone is allowed
  return donate.stream;
}
function donateLevel() {
  if (!donate.analyser) return 0;
  const a = new Float32Array(donate.analyser.fftSize); donate.analyser.getFloatTimeDomainData(a);
  let m = 0; for (const x of a) m = Math.max(m, Math.abs(x));
  return m;
}
function donateShow(d) {
  document.getElementById('donateStart').hidden = true;
  document.getElementById('donateMain').hidden = false;
  donateMics();
  document.getElementById('donateCode').textContent = d.code;
  // the first part not fully sent yet
  const first = DONATE_BATCHES.findIndex((b, k) => b.s.some((_, i) => !(d.sent || {})[`${k + 1}-${i + 1}`]));
  donateOpen(first < 0 ? 0 : first);
}
function donateOpen(k) {
  donate.batch = k; donate.i = 0;
  document.getElementById('donateBatch').value = String(k);
  document.getElementById('donateNote').textContent = DONATE_BATCHES[k].note || 'স্বাভাবিকভাবে, যেভাবে বলে লেখেন সেভাবে পড়ুন। ভুল হলে আবার রেকর্ড করুন।';
  donateDraw();
}
function donateKey(k, i) { return `${k + 1}-${i + 1}`; }
function donateDraw() {
  const b = DONATE_BATCHES[donate.batch], key = donateKey(donate.batch, donate.i);
  const sentAll = (donateStore.get() || {}).sent || {};
  const sent = sentAll[key];
  // overall and per-part progress
  const total = DONATE_BATCHES.reduce((n, x) => n + x.s.length, 0), done = Object.keys(sentAll).length;
  document.getElementById('donateTotal').textContent = `${bnNum(done)} / ${bnNum(total)} বাক্য পাঠানো` + (done >= total ? ' · সব শেষ, অনেক ধন্যবাদ!' : '');
  document.getElementById('donateTotalBar').style.width = (100 * done / total) + '%';
  const sel = document.getElementById('donateBatch');
  DONATE_BATCHES.forEach((x, k) => {
    const n = x.s.filter((_, i) => sentAll[donateKey(k, i)]).length;
    if (sel.options[k]) sel.options[k].textContent = `${bnNum(k + 1)}. ${x.t} — ${bnNum(n)}/${bnNum(x.s.length)}${n === x.s.length ? ' ✓' : ''}`;
  });
  document.getElementById('donateGrid').innerHTML = b.s.map((_, i) => {
    const k2 = donateKey(donate.batch, i);
    const cls = [sentAll[k2] ? 'sent' : donate.takes[k2] ? 'rec' : '', i === donate.i ? 'cur' : ''].join(' ');
    return `<button type="button" class="${cls}" onclick="donateGo(${i})" title="${sentAll[k2] ? 'পাঠানো হয়েছে' : donate.takes[k2] ? 'রেকর্ড হয়েছে, পাঠানো বাকি' : 'এখনও রেকর্ড হয়নি'}">${bnNum(i + 1)}</button>`;
  }).join('');
  document.getElementById('donateSentence').textContent = b.s[donate.i];
  document.getElementById('donatePos').textContent = `বাক্য ${bnNum(donate.i + 1)} / ${bnNum(b.s.length)}` + (sent ? ' · ✓ আগেই পাঠানো হয়েছে' : donate.takes[key] ? ' · ✓ রেকর্ড হয়েছে' : '');
  document.getElementById('donatePlay').disabled = !donate.takes[key];
  document.getElementById('donateRec').textContent = donate.rec ? '■ থামান' : donate.takes[key] ? '● আবার রেকর্ড' : '● রেকর্ড';
  const n = Object.keys(donate.takes).length;
  document.getElementById('donateCount').textContent = n ? `${bnNum(n)}টি রেকর্ডিং পাঠানোর জন্য তৈরি (আপনার কম্পিউটারেই আছে)` : '';
  document.getElementById('donateSendBtn').disabled = !n || !!donate.rec;
}
function donateGo(i) { if (donate.rec) return; donate.i = i; donateDraw(); }
function donateMove(step) {
  if (donate.rec) return;
  const len = DONATE_BATCHES[donate.batch].s.length;
  donate.i = Math.max(0, Math.min(len - 1, donate.i + step));
  donateDraw();
}
async function donateRecord() {
  if (donate.rec) { donate.rec.stop(); return; }
  try { await donateOpenMic(); }
  catch (e) { toast('মাইক চালু করা গেল না। মাইকের অনুমতি দিন, বা অন্য মাইক বাছুন।'); donate.stream = null; return; }
  if (donate.ctx && donate.ctx.state === 'suspended') donate.ctx.resume();
  donate.peak = 0;
  const type = ['audio/webm;codecs=opus', 'audio/ogg;codecs=opus', 'audio/webm'].find(t => MediaRecorder.isTypeSupported(t)) || '';
  const rec = new MediaRecorder(donate.stream, type ? { mimeType: type, audioBitsPerSecond: 48000 } : {});
  const chunks = [], key = donateKey(donate.batch, donate.i), started = performance.now();
  rec.ondataavailable = e => { if (e.data.size) chunks.push(e.data); };
  rec.onstop = () => {
    clearInterval(donate.timer); donate.rec = null;
    const secs = (performance.now() - started) / 1000;
    document.getElementById('donateMeter').style.width = '0';
    document.getElementById('donateLevel').textContent = 'Space = রেকর্ড শুরু/থামান · ← → আগের/পরের বাক্য · P = শুনুন';
    if (secs < 1) { toast('খুব ছোট রেকর্ডিং, আবার চেষ্টা করুন'); donateDraw(); return; }
    if (donate.analyser && donate.peak < 0.02) { toast('প্রায় কিছুই শোনা যায়নি। মাইক ঠিক আছে কি না দেখে আবার রেকর্ড করুন।'); donateDraw(); return; }
    if (donate.peak > 0.98) toast('আওয়াজ খুব জোরে, কেটে যেতে পারে। মাইক একটু দূরে রেখে আবার রেকর্ড করতে পারেন।');
    donate.takes[key] = { blob: new Blob(chunks, { type: rec.mimeType || 'audio/webm' }), secs };
    const len = DONATE_BATCHES[donate.batch].s.length;
    if (donate.i < len - 1) donate.i++;   // straight on to the next sentence
    else toast('এই অংশ শেষ! „পাঠান“ চাপলে রেকর্ডিংগুলো যাবে।');
    donateDraw();
  };
  donate.rec = rec; rec.start();
  donate.timer = setInterval(() => {
    const t = (performance.now() - started) / 1000, lv = donateLevel();
    donate.peak = Math.max(donate.peak, lv);
    const bar = document.getElementById('donateMeter');
    bar.style.width = Math.min(100, Math.round(Math.sqrt(lv) * 110)) + '%'; bar.classList.toggle('loud', lv > 0.95);
    document.getElementById('donateLevel').textContent = `🔴 রেকর্ড হচ্ছে… ${bnNum(Math.floor(t))} সেকেন্ড (পড়া শেষ হলে „থামান“)`;
    if (t > 55) rec.stop();
  }, 60);
  donateDraw();
}
function donatePlay() {
  const t = donate.takes[donateKey(donate.batch, donate.i)];
  if (!t) return;
  if (donate.audio) donate.audio.pause();
  donate.audio = new Audio(URL.createObjectURL(t.blob)); donate.audio.play();
}
function donateB64(blob) {
  return new Promise((ok, bad) => { const r = new FileReader(); r.onload = () => ok(String(r.result).split(',')[1]); r.onerror = bad; r.readAsDataURL(blob); });
}
async function donateSend() {
  const d = donateStore.get(); if (!d) return;
  const keys = Object.keys(donate.takes);
  const btn = document.getElementById('donateSendBtn'); btn.disabled = true;
  let ok = 0, err = '';
  for (const key of keys) {
    const [b, n] = key.split('-').map(Number), t = donate.takes[key];
    document.getElementById('donateCount').textContent = `পাঠানো হচ্ছে… ${bnNum(ok + 1)} / ${bnNum(keys.length)}`;
    try {
      const res = await fetch(VOICE_SEND_URL, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({
        contributor: d.code, batch: b, n, text: DONATE_BATCHES[b - 1].s[n - 1], audio: await donateB64(t.blob), seconds: Math.round(t.secs * 10) / 10,
        region: d.region, age: d.age, mic: d.mic, app: appInfo.app, os: appInfo.os }) });
      const r = await res.json().catch(() => ({}));
      if (!r.ok) { err = r.error || String(res.status); break; }
      ok++; delete donate.takes[key];
      d.sent = d.sent || {}; d.sent[key] = 1; donateStore.set(d);
    } catch (e) { err = 'net'; break; }
  }
  if (!err) toast(`${bnNum(ok)}টি রেকর্ডিং পাঠানো হয়েছে, ধন্যবাদ!`);
  else toast(err === 'too_many' ? 'আজ অনেক পাঠানো হয়েছে, কাল আবার চেষ্টা করুন।' : err === 'net' ? 'পাঠানো যায়নি: ইন্টারনেট সংযোগ দেখে আবার চেষ্টা করুন।' : `পাঠানো যায়নি (${err})। পরে আবার চেষ্টা করুন।`);
  donateDraw();
}
async function donateDelete() {
  const d = donateStore.get(); if (!d) return;
  if (!confirm('আপনার এই কোডে পাঠানো সব রেকর্ডিং মুছে ফেলবেন?')) return;
  try {
    const r = await (await fetch(VOICE_SEND_URL, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ delete: d.code }) })).json();
    if (r.ok) { d.sent = {}; donateStore.set(d); toast(`মুছে ফেলা হয়েছে (${bnNum(r.deleted || 0)}টি ফাইল)`); donateDraw(); }
    else toast('মোছা যায়নি, পরে আবার চেষ্টা করুন।');
  } catch (e) { toast('মোছা যায়নি: ইন্টারনেট সংযোগ দেখে আবার চেষ্টা করুন।'); }
}
function donateStop() {
  if (donate.rec) donate.rec.stop();
  if (donate.stream) { donate.stream.getTracks().forEach(t => t.stop()); donate.stream = null; }
  document.getElementById('donateMain').hidden = true;
  document.getElementById('donateStart').hidden = false;
  const d = donateStore.get(); if (d) { d.agreed = null; donateStore.set(d); }
}
// Space to record or stop, arrows to move, P to listen, while the card is on screen
document.addEventListener('keydown', e => {
  const main = document.getElementById('donateMain');
  if (!main || main.hidden || !document.getElementById('tab-settings').classList.contains('active')) return;
  if (e.target.closest && e.target.closest('input, textarea, select, button, dialog')) return;
  const r = main.getBoundingClientRect();
  if (r.bottom < 0 || r.top > innerHeight) return;
  if (e.code === 'Space') { e.preventDefault(); donateRecord(); }
  else if (e.key === 'ArrowRight') { e.preventDefault(); donateMove(1); }
  else if (e.key === 'ArrowLeft') { e.preventDefault(); donateMove(-1); }
  else if (e.key === 'p' || e.key === 'P') donatePlay();
});
// recordings not sent yet live only in this window
addEventListener('beforeunload', e => { if (Object.keys(donate.takes).length) { e.preventDefault(); e.returnValue = ''; } });
donateInit();

// ── First run: how do you type Bangla? ──
let welcomeShown = false;
function welcomeMaybeShow(st) {
  if (welcomeShown || st.welcome_done !== false) return;
  welcomeShown = true;
  document.getElementById('welcomeBox').hidden = false;
}
async function welcomeDone() {
  document.getElementById('welcomeBox').hidden = true;
  await saveSettings({ welcome_done: true });
}
async function welcomeChoose(how) {
  await layoutsReady;
  const sel = document.getElementById('cfgLayout');
  // a fixed layout: the person's own layout file if one is added already, otherwise National
  const own = Array.from(sel.options).find(o => o.value.startsWith('Custom:'));
  sel.value = how !== 'fixed' ? 'AvroPhonetic' : own ? own.value : 'NationalJatiya';
  document.getElementById('welcomeLayoutName').textContent = sel.options[sel.selectedIndex]?.textContent || 'জাতীয়';
  await saveSettings({ welcome_done: true });
  await refreshState();
  if (how !== 'fixed') {
    document.getElementById('welcomeBox').hidden = true;
    toast('তৈরি! amar sonar bangla লিখে দেখুন');
    document.getElementById('romanInput').focus();
    return;
  }
  document.getElementById('welcomeHotkey').textContent = document.getElementById('hotkeyBadge').textContent || 'দুবার Shift';
  document.getElementById('welcomeBijoyOut').hidden = document.getElementById('bijoyOutBox').hidden;
  document.getElementById('welcomeAsk').hidden = true;
  document.getElementById('welcomeFixed').hidden = false;
}
function welcomeGo(tab, inputId) {
  showTab(tab);
  const el = document.getElementById(inputId);
  if (el) { el.closest('.etched')?.scrollIntoView({ behavior: 'smooth', block: 'center' }); el.click(); }
}
async function welcomeBijoyOutput() {
  document.getElementById('cfgBijoyOut').checked = true;
  await saveSettings();
  toast('বিজয় আউটপুট চালু: তালিকার ডিজাইন অ্যাপগুলোতে SutonnyMJ লেখা যাবে');
  showTab('settings');
  document.getElementById('bijoyOutBox').scrollIntoView({ behavior: 'smooth', block: 'center' });
}
async function showKeyMap() {
  const value = document.getElementById('cfgLayout').value;
  const res = await fetch('/api/layouts/export?id=' + encodeURIComponent(value));
  if (!res.ok) { toast('ফোনেটিক টাইপিংয়ের কোনো কী-ম্যাপ নেই'); return; }
  const l = await res.json();
  const back = document.createElement('div');
  back.className = 'keymap-back';
  back.onclick = (e) => { if (e.target === back) back.remove(); };
  const box = document.createElement('div');
  box.className = 'keymap';
  box.setAttribute('role', 'dialog');
  box.setAttribute('aria-label', l.name + ' কী-ম্যাপ');
  const h = document.createElement('h3');
  h.textContent = l.name;
  box.appendChild(h);
  const kb = document.createElement('div');
  kb.className = 'kb';
  for (const row of KEY_ROWS) {
    const r = document.createElement('div');
    r.className = 'kb-row';
    for (const pair of row) {
      const [lo, up] = [pair[0], pair[1]];
      const k = document.createElement('div');
      k.className = 'key' + (l.link_key === lo ? ' link' : '');
      const part = (cls, text) => { if (!text) return; const d = document.createElement('span'); d.className = cls + (/^[A-Z]{3,}$/.test(text) ? ' small' : ''); d.textContent = text; k.appendChild(d); };
      part('n', l.link_key === lo ? 'লিংক' : keyText(l.keys[lo]));
      part('s', keyText(l.keys[up]));
      part('a', keyText(l.altgr && (l.altgr[lo] || l.altgr[up])));
      part('e', lo.toUpperCase());
      k.title = `${lo}: ${l.keys[lo] || '—'}   Shift+${lo}: ${l.keys[up] || '—'}`;
      r.appendChild(k);
    }
    kb.appendChild(r);
  }
  box.appendChild(kb);
  const legend = document.createElement('div');
  legend.className = 'legend';
  legend.innerHTML = '<span>বড় অক্ষর: শুধু কী</span><span>উপরে বাঁয়ে: Shift + কী</span><span style="color:var(--accent)">নিচে ডানে: AltGr (ডান Alt) + কী</span>' +
    (l.link_key ? `<span>সবুজ ঘর (${l.link_key.toUpperCase()}): লিংক কী — এরপর ব্যঞ্জন দিলে যুক্তবর্ণ, কার দিলে পূর্ণ স্বরবর্ণ, দুবার দিলে হসন্ত</span>` : '') +
    (l.reph_after ? '<span>রেফ: অক্ষরের পরে টাইপ করুন</span>' : '');
  box.appendChild(legend);
  const btns = document.createElement('div');
  btns.className = 'row no-print';
  btns.style.cssText = 'gap:8px;margin-top:14px';
  const pr = document.createElement('button'); pr.className = 'btn primary'; pr.textContent = 'প্রিন্ট করুন'; pr.onclick = () => window.print();
  const cl = document.createElement('button'); cl.className = 'btn'; cl.textContent = 'বন্ধ করুন'; cl.onclick = () => back.remove();
  btns.append(pr, cl);
  box.appendChild(btns);
  back.appendChild(box);
  document.body.appendChild(back);
  document.addEventListener('keydown', function esc(e) { if (e.key === 'Escape') { back.remove(); document.removeEventListener('keydown', esc); } });
}

// ── Backup and restore ──
async function downloadBackup() {
  const msg = document.getElementById('backupMsg');
  const res = await fetch('/api/backup');
  if (!res.ok) { msg.textContent = 'ব্যাকআপ নেওয়া যায়নি — একুশে চালু আছে কি না দেখুন'; return; }
  const text = await res.text();
  let day = '';
  try { day = JSON.parse(text).created || ''; } catch (e) {}
  const a = document.createElement('a');
  a.href = URL.createObjectURL(new Blob([text], { type: 'application/json' }));
  a.download = 'ekushe-next-backup' + (day ? '-' + day : '') + '.json';
  document.body.appendChild(a);
  a.click();
  setTimeout(() => { URL.revokeObjectURL(a.href); a.remove(); }, 1000);
  msg.textContent = 'ব্যাকআপ ফাইল ডাউনলোড হয়েছে। নিরাপদ জায়গায় রেখে দিন।';
}
async function restoreBackup(input) {
  const msg = document.getElementById('backupMsg');
  const file = input.files && input.files[0];
  input.value = '';
  if (!file) return;
  if (file.size > 900 * 1024) { msg.textContent = 'ফাইলটি অনেক বড়, এটি একুশের ব্যাকআপ নয়।'; return; }
  const text = await file.text();
  const res = await fetch('/api/restore', { method: 'POST', headers: {'Content-Type': 'application/json'}, body: text });
  const out = res.ok ? await res.json() : { ok: false };
  if (!out.ok) { msg.textContent = 'ফিরিয়ে আনা যায়নি: এটি একুশে নেক্সটের ব্যাকআপ ফাইল নয়, বা ফাইলটি নষ্ট।'; return; }
  const r = out.restored;
  const bn = (n) => String(n).replace(/[0-9]/g, d => '০১২৩৪৫৬৭৮৯'[d]);
  msg.textContent = `ফিরে এসেছে: ${r.settings ? 'সেটিংস, ' : ''}${bn(r.snippets)}টি শর্টকাট, ${bn(r.learned_words)}টি শেখা শব্দ${r.layouts ? `, ${bn(r.layouts)}টি কীবোর্ড লেআউট` : ''}।`;
  toast('ব্যাকআপ থেকে ফিরিয়ে আনা হয়েছে');
  await refreshState();
  refreshLayouts();
}

// ── Settings ──
async function saveSettings(extra) {
  const opEl = document.getElementById('cfgOpacity');
  const body = Object.assign({
    hotkey: document.getElementById('cfgHotkey').value,
    typing_mode: document.getElementById('cfgMode').value,
    active_layout: document.getElementById('cfgLayout').value,
    font_family: document.getElementById('cfgFont').value,
    popup_opacity: opEl ? parseInt(opEl.value, 10) : 92,
    compact_popup: !!document.getElementById('cfgCompact').checked,
    auto_english_in_terminals: !!document.getElementById('cfgTerminal').checked,
    check_for_updates: !!document.getElementById('cfgUpdates').checked,
    start_bangla: !!document.getElementById('cfgStartBn').checked,
    fixed_spelling_hint: !!document.getElementById('cfgSpellHint').checked,
    fixed_word_suggestions: !!document.getElementById('cfgWordSugg').checked,
  }, document.getElementById('voiceBox').hidden ? {} : {
    voice_spoken_punctuation: !!document.getElementById('cfgVoicePunct').checked,
    voice_numbers_as_digits: !!document.getElementById('cfgVoiceNum').checked,
    voice_quality: document.getElementById('cfgVoiceQuality').value,
    voice_auto_stop: !!document.getElementById('cfgVoiceAutoStop').checked,
    voice_sounds: !!document.getElementById('cfgVoiceSounds').checked,
    voice_use_gpu: !!document.getElementById('cfgVoiceGpu').checked,
  }, document.getElementById('bijoyOutBox').hidden ? {} : {
    bijoy_output: !!document.getElementById('cfgBijoyOut').checked,
    bijoy_output_apps: document.getElementById('cfgBijoyApps').value,
  }, extra || {});
  const res = await fetch('/api/settings', { method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify(body) });
  document.getElementById('settingsSavedMsg').textContent = res.ok ? 'সংরক্ষিত হয়েছে' : 'সংরক্ষণ করা যায়নি — একুশে চালু আছে কি না দেখুন';
  await refreshState();
}
// Voice typing (Windows): model download, options, recent dictations to correct
let voiceTimer = null;
function applyVoiceState(v) {
  const box = document.getElementById('voiceBox');
  if (!v || !v.helper) { box.hidden = true; return; }
  box.hidden = false;
  document.getElementById('voiceHotkey').textContent = v.hotkey || 'Ctrl+Alt+Space';
  document.getElementById('cfgVoicePunct').checked = v.spoken_punctuation !== false;
  document.getElementById('cfgVoiceNum').checked = v.numbers_as_digits !== false;
  document.getElementById('cfgVoiceQuality').value = v.quality === 'best' ? 'best' : 'fast';
  document.getElementById('cfgVoiceAutoStop').checked = !!v.auto_stop;
  document.getElementById('cfgVoiceSounds').checked = v.sounds !== false;
  document.getElementById('cfgVoiceGpu').checked = v.use_gpu !== false;
  loadVoice();
}
const VOICE_MODELS = {
  live: ['লাইভ মডেল', 'বলার সঙ্গে সঙ্গে লেখা হয়; যেকোনো কম্পিউটারে চলে, গ্রাফিক্স কার্ড লাগে না। কিছু কণ্ঠে বা মাইকে ভুল বেশি হতে পারে। মডেল: SayedShaun, CC BY-SA 4.0'],
  large: ['নির্ভুল মডেল', 'ভুল কম, কিন্তু শক্তিশালী গ্রাফিক্স কার্ড ছাড়া ধীর; একটু থামলে একসঙ্গে লেখা হয়। মডেল: Bengali.AI, Apache-2.0'],
  small: ['ছোট মডেল', 'প্রায় তিন গুণ দ্রুত, ভুল একটু বেশি; সাধারণ বা পুরনো কম্পিউটারের জন্য'],
};
async function loadVoice() {
  let v;
  try { v = await (await fetch('/api/voice')).json(); } catch (e) { return; }
  const msg = document.getElementById('voiceModelMsg');
  document.getElementById('voiceOptions').hidden = !v.model;
  const mb = n => bnNum(Math.round(n / 1048576));
  if (v.downloading) {
    const m = (v.models || []).find(x => x.id === v.downloading_model) || { bytes: 1 };
    msg.textContent = `মডেল নামানো হচ্ছে… ${bnNum(Math.floor(100 * (v.downloaded || 0) / m.bytes))}% (${mb(v.downloaded || 0)} / ${mb(m.bytes)} MB)`;
  } else if (v.failed) {
    msg.textContent = 'নামানো যায়নি। ইন্টারনেট সংযোগ দেখে আবার চেষ্টা করুন।';
  } else if (v.model) {
    msg.textContent = '✓ ভয়েস মডেল আছে। যেকোনো অ্যাপে শর্টকাট চেপে বলুন। নামানোর পর ইন্টারনেট লাগে না।';
  } else {
    msg.textContent = 'ভয়েস টাইপিংয়ের জন্য একটি বাংলা ভয়েস মডেল একবার নামাতে হবে (একুশে শুধু তখনই ইন্টারনেট ব্যবহার করে)। পরে এখান থেকেই মুছে ফেলা যায়।';
  }
  const list = document.getElementById('voiceModels'); list.innerHTML = '';
  (v.models || []).filter(m => m.available).forEach(m => {
    const [name, desc] = VOICE_MODELS[m.id] || [m.id, ''];
    const row = document.createElement('div'); row.className = 'row';
    row.style.cssText = 'gap:10px;align-items:center;flex-wrap:wrap;padding:8px 0;border-top:1px solid var(--line, rgba(127,127,127,.2))';
    const info = document.createElement('div'); info.style.cssText = 'flex:1;min-width:220px';
    info.innerHTML = `<b></b> <span class="hint"></span><div class="hint"></div>`;
    info.querySelector('b').textContent = name + (m.active ? ' ✓ ব্যবহার হচ্ছে' : '');
    info.querySelector('span').textContent = m.bytes ? `(${mb(m.bytes)} MB)` : '';
    info.querySelector('div').textContent = m.available ? desc : 'শিগগির আসছে';
    row.appendChild(info);
    const btn = (label, cls, fn) => { const b = document.createElement('button'); b.className = 'btn ' + cls; b.textContent = label; b.onclick = fn; row.appendChild(b); };
    const post = (url) => fetch(url, { method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify({ model: m.id }) });
    if (!m.available) { /* not yet */ }
    else if (!m.ready) { if (!v.downloading) btn('নামান', 'primary', async () => { await post('/api/voice/download'); setTimeout(loadVoice, 300); }); }
    else {
      if (!m.active) btn('এটি ব্যবহার করুন', 'primary', async () => { await post('/api/voice/use'); loadVoice(); });
      btn('মুছুন', '', async () => {
        if (!confirm(`${name} মুছে ফেলবেন? (${mb(m.bytes)} MB জায়গা খালি হবে। পরে আবার নামানো যাবে।)`)) return;
        const r = await (await post('/api/voice/remove')).json();
        toast(r.ok ? 'মুছে ফেলা হয়েছে' : 'মোছা যায়নি'); loadVoice();
      });
    }
    list.appendChild(row);
  });
  const sp = document.getElementById('voiceSpeed'); sp.textContent = '';
  if (v.speed && v.speed.secs > 0) {
    const r = v.speed.took / v.speed.secs;
    const name = (VOICE_MODELS[v.speed.model] || [''])[0];
    sp.textContent = `শেষবার (${name}): ${bnNum(v.speed.secs.toFixed(1))} সেকেন্ডের কথা ${bnNum(v.speed.took.toFixed(1))} সেকেন্ডে লেখা হয়েছে।` +
      (v.speed.model === 'large' && r > 0.9 ? ' এই কম্পিউটারে বড় মডেলটি ধীর; ছোট মডেলটি নিয়ে দেখুন।' : '');
  }
  clearTimeout(voiceTimer);
  if (v.downloading) voiceTimer = setTimeout(loadVoice, 1000);
  const rb = document.getElementById('voiceRecent'); rb.innerHTML = '';
  document.getElementById('voiceRecentBox').hidden = !(v.recent && v.recent.length);
  (v.recent || []).slice(0, 5).forEach(t => {
    const row = document.createElement('div'); row.className = 'row'; row.style.cssText = 'gap:8px;margin-top:6px';
    const inp = document.createElement('input'); inp.type = 'text'; inp.value = t; inp.style.cssText = 'flex:1;font-family:var(--font-bn)';
    const b = document.createElement('button'); b.className = 'btn'; b.textContent = 'শিখুন';
    b.onclick = async () => {
      if (inp.value.trim() === t.trim()) { toast('লেখাটি বদলানো হয়নি'); return; }
      await fetch('/api/voice/learn', { method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify({ heard: t, wanted: inp.value }) });
      toast('শিখে নিয়েছে'); loadVoice();
    };
    row.append(inp, b); rb.appendChild(row);
  });
  const lb = document.getElementById('voiceLearned'); lb.innerHTML = '';
  const rows = (v.learned || '').split('\n').map(l => l.split('\t')).filter(f => f.length >= 2 && f[0]);
  document.getElementById('voiceLearnedBox').hidden = !rows.length;
  rows.forEach(([h, w]) => {
    const c = document.createElement('button'); c.className = 'chip'; c.textContent = `${h} → ${w}`;
    c.onclick = async () => { await fetch('/api/voice/forget', { method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify({ heard: h }) }); loadVoice(); };
    lb.appendChild(c);
  });
}
async function addCustomSnippet() {
  const kEl = document.getElementById('newSnipKey');
  const vEl = document.getElementById('newSnipVal');
  let key = (kEl.value || '').trim();
  const val = (vEl.value || '').trim();
  if (!key || !val) { toast('শর্টকাট আর পুরো লেখা, দুটোই লিখুন'); return; }
  if (!key.startsWith(';') && !key.startsWith('/')) key = ';' + key;
  await saveSettings({ snippet_trigger: key, snippet_expansion: val });
  kEl.value = ''; vEl.value = '';
  toast('স্নিপেট যোগ হয়েছে');
}

const initialTab = new URLSearchParams(window.location.search).get('tab');
if (initialTab === 'keymap') {
  // Opened from the menu's "key map": the settings tab with the key map on top
  showTab('settings');
  layoutsReady.then(showKeyMap);
} else if (initialTab === 'layouts' || initialTab === 'voice') {
  // From the tray menu: the settings tab, scrolled to that card (the voice card shows once the state is in)
  showTab('settings');
  const id = initialTab === 'layouts' ? 'ownLayoutBox' : 'voiceBox';
  setTimeout(() => document.getElementById(id)?.scrollIntoView({ behavior: 'smooth', block: 'start' }), 700);
} else {
  showTab(initialTab && document.getElementById('tab-' + initialTab) ? initialTab : 'playground');
}
if (initialTab === 'fix') document.getElementById('navFix').style.display = '';
updateCount();
refreshState();
loadSigns('');
</script>
</body>
</html>
"##;
