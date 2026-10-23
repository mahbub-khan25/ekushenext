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
    <symbol id="i-gear" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M4 6h10M18 6h2M4 12h4M12 12h8M4 18h12M20 18h0"/><circle cx="16" cy="6" r="2"/><circle cx="10" cy="12" r="2"/><circle cx="18" cy="18" r="2"/></symbol>
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
    <button class="nav-btn" data-tab="settings"><svg><use href="#i-gear"/></svg><span class="lbl"><span class="bn">সেটিংস</span><span class="en">Settings</span></span></button>
    <div class="rail-foot">বাংলা ⇄ English বদলাতে<br><b id="hotkeyBadge">Double-Tap Shift</b></div>
  </nav>

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

    <!-- Writing pad -->
    <section class="panel active" id="tab-playground" data-title="লেখার প্যাড" data-sub="রোমান হরফে লিখুন, একুশে বাংলা করে দেবে। Space চাপলে প্রথম শব্দটি বসবে।">
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
        <div class="marks-grid">
          <div class="mark-item"><b>হ্</b><div><kbd>,,</kbd> হসন্ত<span>allah,, → আল্লাহ্</span></div></div>
          <div class="mark-item"><b>চাঁ</b><div><kbd>^</kbd> চন্দ্রবিন্দু<span>he^se → হেঁসে</span></div></div>
          <div class="mark-item"><b>কৃ</b><div><kbd>rri</kbd> ঋ-কার<span>krrishi → কৃষি</span></div></div>
          <div class="mark-item"><b>র্ম</b><div><kbd>rr</kbd> রেফ<span>dhorrmo → ধর্ম</span></div></div>
          <div class="mark-item"><b>ৎ</b><div><kbd>t``</kbd> খণ্ড-ত<span>hothat`` → হঠাৎ</span></div></div>
          <div class="mark-item"><b>দুঃ</b><div><kbd>:</kbd> বিসর্গ<span>du:kho → দুঃখ</span></div></div>
          <div class="mark-item"><b>ক্য</b><div><kbd>Z</kbd> য-ফলা<span>bakZo → বাক্য</span></div></div>
          <div class="mark-item"><b>ক্ত</b><div><kbd>+</kbd> যুক্ত করুন<span>k+t → ক্ত</span></div></div>
          <div class="mark-item"><b>লল</b><div><kbd>`</kbd> আলাদা রাখুন<span>bol`le → বললে</span></div></div>
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
        <textarea id="convInput" rows="7" placeholder="এখানে লেখা পেস্ট করুন" oninput="runConvert()">Avwg evsjvq Mvb MvB| evsjv†`k Avgvi Ÿ†`k|</textarea>
        <textarea id="convOutput" rows="7" placeholder="ফলাফল" readonly></textarea>
      </div>
      <div class="row" style="margin-top:10px;justify-content:flex-end"><button class="btn primary" onclick="copyText(document.getElementById('convOutput').value)"><svg><use href="#i-copy"/></svg>ফলাফল কপি করুন</button></div>
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
              <option value="Probhat">প্রভাত</option>
              <option value="NationalJatiya">জাতীয়</option>
              <option value="Bornona">বর্ণনা</option>
              <option value="MunirOptima">মুনীর অপটিমা</option>
            </select>
          </div>
        </div>

        <div class="etched">
          <div class="section-title">আচরণ</div>
          <label class="toggle">
            <span><b>ছোট পপ-আপ</b><small>টাইপ করার সময়ের পপ-আপে যুক্তবর্ণের সারি লুকায়</small></span>
            <span class="switch"><input type="checkbox" id="cfgCompact" onchange="saveSettings()"><i></i></span>
          </label>
          <label class="toggle">
            <span><b>টার্মিনালে ইংরেজি</b><small>টার্মিনাল (Windows Terminal, CMD, PowerShell, ম্যাকের Terminal) ইংরেজিতে শুরু হয়; দুবার Shift চাপলে সেখানেও বাংলা</small></span>
            <span class="switch"><input type="checkbox" id="cfgTerminal" checked onchange="saveSettings()"><i></i></span>
          </label>
        </div>

        <div class="etched">
          <div class="section-title">স্নিপেট</div>
          <div class="hint">শর্টকাট লিখে Space চাপলে পুরো লেখাটা বসে যায়। ক্লিক করলে প্যাডে বসবে।</div>
          <div class="setting">
            <input type="text" id="newSnipKey" placeholder="শর্টকাট, যেমন ;addr">
            <input type="text" id="newSnipVal" placeholder="পুরো লেখা, যেমন ধানমন্ডি, ঢাকা-১২০৯" style="font-family:var(--font-bn)">
            <div><button class="btn primary" onclick="addCustomSnippet()">স্নিপেট যোগ করুন</button></div>
          </div>
          <div class="chip-bar" id="snippetsList" style="margin-top:12px"></div>
        </div>
      </div>
      <div id="settingsSavedMsg" role="status" class="hint" style="margin-top:12px"></div>
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
  if (navigator.clipboard) navigator.clipboard.writeText(text).then(() => toast('কপি হয়েছে'), () => toast('কপি করা যায়নি'));
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
}
document.querySelectorAll('.nav-btn').forEach(b => b.addEventListener('click', () => {
  showTab(b.dataset.tab);
  history.replaceState(null, '', '?tab=' + b.dataset.tab);
}));

async function refreshState() {
  const res = await fetch('/api/state');
  const st = await res.json();
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
  if (st.report_url && /^https:\/\/ekushenext\.com\//.test(st.report_url)) document.getElementById('reportLink').href = st.report_url;
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
  }, extra || {});
  const res = await fetch('/api/settings', { method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify(body) });
  document.getElementById('settingsSavedMsg').textContent = res.ok ? 'সংরক্ষিত হয়েছে' : 'সংরক্ষণ করা যায়নি — একুশে চালু আছে কি না দেখুন';
  await refreshState();
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
showTab(initialTab && document.getElementById('tab-' + initialTab) ? initialTab : 'playground');
updateCount();
refreshState();
loadSigns('');
</script>
</body>
</html>
"##;
