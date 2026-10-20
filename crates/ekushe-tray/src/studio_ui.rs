//! Studio WebUI HTML/CSS/JS served by the running `ekushe.exe` tray daemon.
//!
//! Defaults to a modern Frosted Glass Light Mode (`:root`) with an instant one-click
//! Light / Dark mode switcher (`[data-theme="dark"]`) and a live Popup & Glass Opacity
//! slider (`70%–100%`) in the header bar.
//!
//! Includes unique daily-driver Bengali writing features beyond Avro / Lekho / OpenBangla:
//! 1. Zero-Case-Memorization Live Phonetic Pad + Clickable Candidate & Conjunct Pills
//! 2. Interactive "Word X-Ray & 2-Click Visual Conjunct Builder" (`🔬 শব্দ এক্স-রে ও ২-ক্লিক যুক্তবর্ণ নির্মাতা`)
//! 3. One-Click Daily Bangla Writing Power-Toolbar (`🔢 123 ➔ ১২৩`, `৳ টাকা`, `📅 তারিখ`, `✨ স্মার্ট পলিশ`)
//! 4. Series-Organized Connected Letters & Real-World Word Helper (`K/Ch/T/N/P/Sh/D/L-Series`)

pub const STUDIO_HTML: &str = r##"<!DOCTYPE html>
<html lang="bn" data-theme="light">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>একুশে নেক্সট স্টুডিও — Ekushe Next Modern Bangla Writing Suite</title>
<link rel="icon" type="image/png" href="/logo.png">
<style>
  /* Bundled Bangladeshi & Bengali OpenType Fonts (100% Offline, Embedded in App) */
  @font-face {
    font-family: 'Kalpurush';
    src: url('/fonts/Kalpurush.ttf') format('truetype');
    font-weight: 400 700;
    font-style: normal;
    font-display: block;
  }
  @font-face {
    font-family: 'SolaimanLipi';
    src: url('/fonts/SolaimanLipi.ttf') format('truetype');
    font-weight: 400 700;
    font-style: normal;
    font-display: block;
  }
  @font-face {
    font-family: 'Siyam Rupali';
    src: url('/fonts/SiyamRupali.ttf') format('truetype');
    font-weight: 400 700;
    font-style: normal;
    font-display: block;
  }
  @font-face {
    font-family: 'Hind Siliguri';
    src: url('/fonts/HindSiliguri-Regular.ttf') format('truetype');
    font-weight: 400;
    font-style: normal;
    font-display: block;
  }
  @font-face {
    font-family: 'Hind Siliguri';
    src: url('/fonts/HindSiliguri-Bold.ttf') format('truetype');
    font-weight: 600 700;
    font-style: normal;
    font-display: block;
  }

  /* DEFAULT: Modern Translucent Frosted-Glass Light Mode */
  :root, [data-theme="light"] {
    --font-bn: 'Kalpurush', 'SolaimanLipi', 'Siyam Rupali', 'Hind Siliguri', sans-serif;
    --glass-alpha: 0.78;
    --bg-page:
      radial-gradient(circle at 12% 16%, rgba(16, 185, 129, 0.16) 0%, transparent 42%),
      radial-gradient(circle at 88% 12%, rgba(217, 30, 46, 0.10) 0%, transparent 36%),
      radial-gradient(circle at 50% 88%, rgba(13, 110, 78, 0.13) 0%, transparent 48%),
      linear-gradient(145deg, #eef6f2 0%, #e3efe9 100%);
    --bg-header: rgba(255, 255, 255, 0.76);
    --bg-card: rgba(255, 255, 255, var(--glass-alpha));
    --bg-card-sub: rgba(245, 251, 248, 0.74);
    --bg-card-hover: rgba(232, 246, 239, 0.88);
    --bg-input: rgba(255, 255, 255, 0.84);
    --border-subtle: rgba(13, 110, 78, 0.18);
    --border-specular: rgba(255, 255, 255, 0.85);
    --border-active: #0d6e4e;
    --brand-primary: #095239;
    --brand-accent: #0d6e4e;
    --crimson: #d91e2e;
    --text-main: #0c2219;
    --text-muted: #466357;
    --chip-bg: rgba(229, 244, 236, 0.82);
    --chip-text: #095239;
    --shadow-card: 0 12px 32px rgba(9, 54, 38, 0.07), inset 0 1px 0 rgba(255, 255, 255, 0.85);
  }

  /* Optional Frosted-Glass Dark Mode */
  [data-theme="dark"] {
    --glass-alpha: 0.74;
    --bg-page:
      radial-gradient(circle at 16% 18%, rgba(16, 185, 129, 0.22) 0%, transparent 45%),
      radial-gradient(circle at 85% 15%, rgba(225, 37, 53, 0.15) 0%, transparent 38%),
      radial-gradient(circle at 18% 12%, #0c3b2c 0%, #051711 62%, #030e0a 100%);
    --bg-header: rgba(6, 26, 19, 0.75);
    --bg-card: rgba(11, 45, 34, var(--glass-alpha));
    --bg-card-sub: rgba(7, 31, 23, 0.74);
    --bg-card-hover: rgba(19, 72, 54, 0.86);
    --bg-input: rgba(5, 22, 16, 0.82);
    --border-subtle: rgba(229, 193, 104, 0.26);
    --border-specular: rgba(255, 255, 255, 0.10);
    --border-active: #e5c168;
    --brand-primary: #f7df96;
    --brand-accent: #e5c168;
    --crimson: #ff4757;
    --text-main: #f3f8f5;
    --text-muted: #a6c7b9;
    --chip-bg: rgba(229, 193, 104, 0.15);
    --chip-text: #f7df96;
    --shadow-card: 0 16px 38px rgba(0, 0, 0, 0.38), inset 0 1px 0 rgba(255, 255, 255, 0.10);
  }

  * {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
    font-family: var(--font-bn);
    font-feature-settings: "kern" 1, "liga" 1, "clig" 1, "calt" 1;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
  }
  body {
    background: var(--bg-page);
    background-attachment: fixed;
    color: var(--text-main);
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    line-height: 1.52;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 11px 24px;
    background: var(--bg-header);
    backdrop-filter: blur(22px) saturate(185%);
    -webkit-backdrop-filter: blur(22px) saturate(185%);
    border-bottom: 1px solid var(--border-subtle);
    box-shadow: 0 4px 20px rgba(9, 54, 38, 0.05);
    position: sticky;
    top: 0;
    z-index: 30;
    gap: 12px;
    flex-wrap: wrap;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .brand img {
    width: 42px;
    height: 42px;
    border-radius: 11px;
    box-shadow: 0 3px 10px rgba(0,0,0,0.14);
  }
  .brand-title h1 {
    font-size: 1.22rem;
    font-weight: 700;
    color: var(--brand-primary);
  }
  .brand-title p {
    font-size: 0.80rem;
    color: var(--text-muted);
  }
  .status-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .opacity-pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: var(--bg-card-sub);
    border: 1px solid var(--border-subtle);
    padding: 5px 11px;
    border-radius: 999px;
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-main);
  }
  .opacity-pill input[type="range"] {
    width: 68px;
    accent-color: var(--brand-accent);
    cursor: pointer;
  }
  .font-QuickSelect, .theme-btn {
    background: var(--bg-card-sub);
    border: 1px solid var(--border-subtle);
    color: var(--text-main);
    padding: 6px 12px;
    border-radius: 999px;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    width: auto;
    backdrop-filter: blur(12px);
  }
  .theme-btn:hover, .font-QuickSelect:hover {
    border-color: var(--border-active);
    background: var(--bg-card-hover);
  }
  .mode-pill {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 6px 14px;
    border-radius: 999px;
    font-weight: 700;
    font-size: 0.86rem;
    cursor: pointer;
    border: 1px solid transparent;
    transition: all 0.18s ease;
    user-select: none;
  }
  .mode-pill.bangla {
    background: linear-gradient(135deg, #0d6e4e 0%, #095239 100%);
    color: #fff;
    box-shadow: 0 3px 12px rgba(13, 110, 78, 0.28);
  }
  .mode-pill.english {
    background: var(--bg-card-sub);
    border-color: var(--border-subtle);
    color: var(--text-main);
  }
  .hotkey-badge {
    padding: 5px 11px;
    border-radius: 999px;
    background: var(--chip-bg);
    border: 1px solid var(--border-subtle);
    color: var(--chip-text);
    font-size: 0.80rem;
    font-weight: 600;
  }
  .layout {
    display: grid;
    grid-template-columns: 244px 1fr;
    flex: 1;
    max-width: 1400px;
    width: 100%;
    margin: 0 auto;
    padding: 18px 22px;
    gap: 18px;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 7px;
  }
  .nav-btn {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 10px 14px;
    border-radius: 12px;
    background: var(--bg-card);
    backdrop-filter: blur(18px) saturate(170%);
    border: 1px solid var(--border-subtle);
    color: var(--text-main);
    font-size: 0.92rem;
    font-weight: 600;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s ease;
    box-shadow: var(--shadow-card);
  }
  .nav-btn:hover {
    background: var(--bg-card-hover);
    border-color: var(--border-active);
    transform: translateX(2px);
  }
  .nav-btn.active {
    background: linear-gradient(135deg, #0d6e4e 0%, #095239 100%);
    border-color: var(--brand-accent);
    color: #ffffff;
    box-shadow: 0 6px 16px rgba(13, 110, 78, 0.26);
  }
  [data-theme="dark"] .nav-btn.active {
    background: linear-gradient(135deg, #e5c168 0%, #cfa746 100%);
    color: #061c15;
  }
  .why-ekushe-card {
    margin-top: 10px;
    padding: 12px 13px;
    border-radius: 13px;
    background: var(--bg-card-sub);
    backdrop-filter: blur(16px);
    border: 1px solid var(--border-subtle);
    font-size: 0.80rem;
    color: var(--text-muted);
    line-height: 1.48;
  }
  .why-ekushe-card b {
    color: var(--brand-primary);
    display: block;
    margin-bottom: 4px;
    font-size: 0.84rem;
  }
  .panel {
    display: none;
    background: var(--bg-card);
    backdrop-filter: blur(24px) saturate(185%);
    -webkit-backdrop-filter: blur(24px) saturate(185%);
    border: 1px solid var(--border-subtle);
    border-radius: 18px;
    padding: 20px 22px;
    box-shadow: var(--shadow-card);
  }
  .panel.active { display: block; }
  h2 {
    font-size: 1.20rem;
    color: var(--brand-primary);
    margin-bottom: 4px;
  }
  .subtitle {
    color: var(--text-muted);
    font-size: 0.88rem;
    margin-bottom: 12px;
  }
  code {
    background: var(--chip-bg);
    color: var(--chip-text);
    padding: 2px 6px;
    border-radius: 6px;
    font-size: 0.88em;
    font-weight: 700;
  }
  textarea, input[type="text"], select {
    width: 100%;
    background: var(--bg-input);
    backdrop-filter: blur(10px);
    border: 1px solid var(--border-subtle);
    border-radius: 12px;
    color: var(--text-main);
    padding: 10px 13px;
    font-size: 1.05rem;
    font-family: var(--font-bn);
    line-height: 1.5;
    outline: none;
  }
  textarea:focus, input[type="text"]:focus, select:focus {
    border-color: var(--border-active);
    box-shadow: 0 0 0 3px rgba(13, 110, 78, 0.14);
  }
  .chip-bar {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin: 8px 0;
  }
  .chip {
    padding: 5px 12px;
    border-radius: 999px;
    background: var(--chip-bg);
    border: 1px solid var(--border-subtle);
    color: var(--chip-text);
    cursor: pointer;
    font-size: 0.88rem;
    font-weight: 600;
    transition: all 0.14s ease;
  }
  .chip:hover, .chip.primary {
    background: var(--brand-accent);
    border-color: var(--brand-accent);
    color: #fff;
  }
  [data-theme="dark"] .chip:hover, [data-theme="dark"] .chip.primary {
    color: #061c15;
  }
  .power-toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 7px;
    margin: 10px 0 8px 0;
    padding: 8px 11px;
    border-radius: 12px;
    background: var(--bg-card-sub);
    border: 1px solid var(--border-subtle);
  }
  .power-toolbar .pt-label {
    font-size: 0.80rem;
    font-weight: 700;
    color: var(--brand-primary);
    margin-right: 4px;
  }
  .pt-btn {
    padding: 4px 10px;
    border-radius: 8px;
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    color: var(--text-main);
    font-size: 0.81rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.12s ease;
  }
  .pt-btn:hover {
    border-color: var(--border-active);
    background: var(--bg-card-hover);
    color: var(--brand-primary);
    transform: translateY(-1px);
  }
  .quick-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(112px, 1fr));
    gap: 7px;
    margin-top: 10px;
  }
  .sign-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 8px 6px;
    border-radius: 11px;
    background: var(--bg-card-sub);
    border: 1px solid var(--border-subtle);
    color: var(--text-main);
    cursor: pointer;
    transition: transform 0.12s ease, border-color 0.12s ease, background 0.12s ease;
  }
  .sign-btn:hover {
    transform: translateY(-2px);
    border-color: var(--border-active);
    background: var(--bg-card-hover);
  }
  .sign-btn .bn {
    font-size: 1.28rem;
    font-weight: 700;
    color: var(--brand-primary);
    line-height: 1.25;
  }
  .sign-btn .rm {
    font-size: 0.76rem;
    color: var(--crimson);
    font-weight: 700;
    margin-top: 2px;
  }
  .sign-btn .dc {
    font-size: 0.73rem;
    color: var(--text-muted);
    text-align: center;
  }
  .atlas-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 11px;
    margin-top: 12px;
    max-height: 560px;
    overflow-y: auto;
    padding-right: 6px;
  }
  .atlas-card {
    background: var(--bg-card-sub);
    border: 1px solid var(--border-subtle);
    border-radius: 14px;
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 9px;
    transition: all 0.14s ease;
  }
  .atlas-card:hover {
    border-color: var(--border-active);
    background: var(--bg-card-hover);
    box-shadow: 0 4px 14px rgba(13, 110, 78, 0.08);
  }
  .atlas-card-top {
    display: flex;
    align-items: center;
    gap: 11px;
    cursor: pointer;
  }
  .atlas-glyph {
    font-size: 1.72rem;
    font-weight: 700;
    color: var(--brand-primary);
    min-width: 58px;
    height: 52px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: 11px;
    line-height: 1.2;
  }
  .atlas-meta {
    flex: 1;
    font-size: 0.85rem;
  }
  .atlas-meta .key {
    color: var(--crimson);
    font-weight: 700;
    background: rgba(217, 30, 46, 0.09);
    padding: 2px 7px;
    border-radius: 6px;
    font-family: 'Segoe UI', monospace;
  }
  .series-tag {
    font-size: 0.72rem;
    font-weight: 700;
    color: var(--brand-primary);
    background: rgba(13, 110, 78, 0.11);
    padding: 2px 8px;
    border-radius: 999px;
    margin-left: 6px;
  }
  .word-list {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    padding-top: 6px;
    border-top: 1px dashed var(--border-subtle);
  }
  .word-pill {
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    border-radius: 8px;
    padding: 3px 8px;
    font-size: 0.80rem;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 4px;
    transition: all 0.12s ease;
  }
  .word-pill:hover {
    border-color: var(--brand-accent);
    background: var(--bg-card-hover);
    transform: translateY(-1px);
  }
  .word-pill .wb {
    font-weight: 700;
    color: var(--text-main);
    font-size: 0.90rem;
  }
  .word-pill .wr {
    color: var(--crimson);
    font-weight: 700;
    font-size: 0.75rem;
  }
  .word-pill .wm {
    color: var(--text-muted);
    font-size: 0.72rem;
  }
  /* Word X-Ray & 2-Click Conjunct Builder */
  .xray-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    margin-top: 12px;
  }
  @media (max-width: 960px) {
    .xray-grid { grid-template-columns: 1fr; }
  }
  .glass-subcard {
    background: var(--bg-card-sub);
    border: 1px solid var(--border-subtle);
    border-radius: 14px;
    padding: 15px 16px;
  }
  .xray-tokens {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 7px;
    margin-top: 12px;
  }
  .xray-token {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 7px 11px;
    border-radius: 10px;
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    min-width: 56px;
  }
  .xray-token .xt-char {
    font-size: 1.35rem;
    font-weight: 700;
    color: var(--brand-primary);
  }
  .xray-token .xt-key {
    font-size: 0.75rem;
    font-weight: 700;
    color: var(--crimson);
  }
  .xray-token .xt-name {
    font-size: 0.70rem;
    color: var(--text-muted);
  }
  .consonant-matrix {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(40px, 1fr));
    gap: 5px;
    margin-top: 7px;
  }
  .c-btn {
    padding: 6px 4px;
    border-radius: 8px;
    background: var(--bg-input);
    border: 1px solid var(--border-subtle);
    color: var(--text-main);
    font-size: 1.02rem;
    font-weight: 700;
    cursor: pointer;
    text-align: center;
    transition: all 0.12s ease;
  }
  .c-btn:hover, .c-btn.selected {
    background: var(--brand-accent);
    border-color: var(--brand-accent);
    color: #fff;
  }
  .fusion-preview {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-top: 12px;
    padding: 11px 14px;
    border-radius: 12px;
    background: var(--bg-input);
    border: 1px solid var(--border-active);
  }
  .fusion-big {
    font-size: 2.0rem;
    font-weight: 700;
    color: var(--brand-primary);
  }
  .btn {
    padding: 8px 15px;
    border-radius: 10px;
    border: 1px solid var(--brand-accent);
    background: var(--brand-accent);
    color: #ffffff;
    font-weight: 700;
    cursor: pointer;
    font-size: 0.88rem;
  }
  [data-theme="dark"] .btn {
    color: #061c15;
  }
  .btn.secondary {
    background: var(--bg-card-sub);
    border-color: var(--border-subtle);
    color: var(--text-main);
  }
  .row {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
    margin-top: 9px;
  }
  .form-group {
    margin-bottom: 13px;
  }
  .form-group label {
    display: block;
    font-weight: 600;
    margin-bottom: 4px;
    color: var(--brand-primary);
    font-size: 0.92rem;
  }
  .issue-card {
    background: rgba(217, 30, 46, 0.08);
    border: 1px solid rgba(217, 30, 46, 0.35);
    border-radius: 10px;
    padding: 11px;
    margin-top: 9px;
  }
</style>
</head>
<body>
<header>
  <div class="brand">
    <img src="/logo.png" alt="Ekushe Next Logo">
    <div class="brand-title">
      <h1>একুশে নেক্সট (Ekushe Next) — Next-Gen Bangla Writing Suite</h1>
      <p>Zero-Case Phonetic · ২-ক্লিক যুক্তবর্ণ ও শব্দ এক্স-রে · ডেইলি বাংলা পাওয়ার-টুলস · কালপুরুষ ফন্ট</p>
    </div>
  </div>
  <div class="status-bar">
    <div class="opacity-pill" title="অ্যাপ এবং ফ্লোটিং পপ-আপের স্বচ্ছতা (Frosted Glass & Popup Opacity)">
      <span>💧 স্বচ্ছতা: <b id="opacityLabel">92%</b></span>
      <input type="range" id="headerOpacitySlider" min="70" max="100" value="92" oninput="onOpacityChange(this.value)">
    </div>
    <button class="theme-btn" id="themeBtn" onclick="toggleTheme()" title="Switch Light / Dark Mode">☀️ Light Mode</button>
    <select class="font-QuickSelect" id="headerFontSelect" onchange="changeBanglaFont(this.value)" title="বাংলা ফন্ট নির্বাচন করুন (Select Bundled Bangladeshi Font)">
      <option value="Kalpurush">ফন্ট: কালপুরুষ (Kalpurush)</option>
      <option value="SolaimanLipi">ফন্ট: সোলাইমানলিপি (SolaimanLipi)</option>
      <option value="Siyam Rupali">ফন্ট: সিয়াম রূপালী (Siyam Rupali)</option>
      <option value="Hind Siliguri">ফন্ট: হিন্দ শিলিগুড়ি (Hind Siliguri)</option>
    </select>
    <span class="hotkey-badge" id="hotkeyBadge">Switch: Double-Tap Shift</span>
    <div class="mode-pill bangla" id="modePill" onclick="toggleMode()">
      <span id="modeText">বাং · বাংলা সক্রিয়</span>
    </div>
  </div>
</header>

<div class="layout">
  <nav>
    <button class="nav-btn active" onclick="showTab('playground', this)">⌨️ স্মার্ট বাংলা রাইটিং প্যাড</button>
    <button class="nav-btn" onclick="showTab('xray', this)">🔬 শব্দ এক্স-রে ও যুক্তবর্ণ নির্মাতা</button>
    <button class="nav-btn" onclick="showTab('signs', this)">🔤 যুক্তবর্ণ ও শব্দ সহায়িকা (Helper)</button>
    <button class="nav-btn" onclick="showTab('spell', this)">✅ বাংলা বানান পরীক্ষক</button>
    <button class="nav-btn" onclick="showTab('converter', this)">🔄 বিজয় ⇄ ইউনিকোড কনভার্টার</button>
    <button class="nav-btn" onclick="showTab('settings', this)">⚙️ সেটিংস, ফন্ট ও স্বচ্ছতা</button>

    <div class="why-ekushe-card">
      <b>🚀 কেন একুশে নেক্সট অনন্য? (Why Ekushe Next?)</b>
      • <b>Zero-Case Phonetic:</b> ছোট হাতের অক্ষরেই <code>srishti</code> ➔ <b>সৃষ্টি</b>, <code>smriti</code> ➔ <b>স্মৃতি</b>, <code>koshto</code> ➔ <b>কষ্ট</b><br>
      • <b>Inline Mix-English:</b> বাংলা মোডে <code>/meeting</code> বা <code>Ctrl+Backspace</code> চেপেই ইংরেজি শব্দ!<br>
      • <b>Smart Numbers:</b> বাংলা মোডে <code>০-৯</code> এবং ইংরেজি মোডে <code>0-9</code> স্বয়ংক্রিয়ভাবে কাজ করে।<br>
      • <b>Quick Snippets:</b> <code>;salam</code> ➔ <b>আসসালামু আলাইকুম</b>, <code>;tk</code> ➔ <b>৳</b>
    </div>
  </nav>

  <main>
    <!-- TAB 1: SMART BANGLA WRITING PAD + POWER TOOLBAR + QUICK CONJUNCTS -->
    <section class="panel active" id="tab-playground">
      <h2>স্মার্ট বাংলা রাইটিং প্যাড ও যুক্তবর্ণ (Smart Daily Bangla Writing Studio)</h2>
      <p class="subtitle">ছোট হাতের অক্ষরেই সাবলীল বাংলা লিখুন (যেমন: <code>srishti</code> → <b>সৃষ্টি</b>, <code>smriti</code> → <b>স্মৃতি</b>)। <b>প্রো টিপস:</b> বাংলা লেখার মাঝখানে ইংরেজি শব্দ লিখতে আগে <code>/</code> দিন (যেমন: <code>/office</code> → <b>office</b>), শেষ শব্দটি বাংলা ⇄ ইংরেজি পাল্টাতে <b>Ctrl+Backspace</b> চাপুন, আর কুইক স্নিপেটের জন্য লিখুন <code>;salam</code> বা <code>;tk</code>!</p>

      <input type="text" id="romanInput" placeholder="এখানে লিখুন... যেমন: srishti / smriti / /office (ইংরেজি) / ;salam (স্নিপেট) / 2026 (সংখ্যা)" oninput="onRomanInput()">

      <div class="chip-bar" id="candidateBar"></div>
      <div class="chip-bar" id="hintBar"></div>

      <!-- Daily Bangla Writing Power-Toolbar (Unique to Ekushe) -->
      <div class="power-toolbar">
        <span class="pt-label">⚡ স্মার্ট টুলস:</span>
        <button class="pt-btn" onclick="convertPadDigitsToBangla()" title="ইংরেজি সংখ্যাকে বাংলা সংখ্যায় রূপান্তর করুন (2026 ➔ ২০২৬)">🔢 123 ➔ ১২৩</button>
        <button class="pt-btn" onclick="convertPadDigitsToEnglish()" title="বাংলা সংখ্যাকে ইংরেজি সংখ্যায় রূপান্তর করুন (২০২৬ ➔ 2026)">🔢 ১২৩ ➔ 123</button>
        <button class="pt-btn" onclick="insertTakaCurrency()" title="টাকার চিহ্ন ও কমা ফরম্যাট যুক্ত করুন (৳ ১,০০০/-)">৳ টাকা ফরম্যাট</button>
        <button class="pt-btn" onclick="insertTodayBanglaDate()" title="আজকের বাংলা ও ইংরেজি তারিখ যুক্ত করুন">📅 আজকের তারিখ</button>
        <button class="pt-btn" onclick="smartPolishBanglaPad()" title="ইংরেজি ফুলস্টপ (.) কে দাড়ি (।) এবং ভাঙ্গা ইউনিকোড ঠিক করুন">✨ স্মার্ট বিরামচিহ্ন ও স্পেসিং পলিশ</button>
        <button class="pt-btn" onclick="inspectPadWordInXray()" title="প্যাডের শেষ শব্দটি কীভাবে গঠিত হয়েছে তা এক্স-রে তে দেখুন">🔬 শব্দ এক্স-রে</button>
      </div>

      <textarea id="banglaPad" rows="4" placeholder="আপনার বাংলা লেখা এখানে তৈরি হবে...">আমি বাংলায় গান গাই। সৃষ্টি ও স্মৃতি আমাদের অনুপ্রেরণা।</textarea>
      <div class="row">
        <button class="btn" onclick="copyPad()">📋 কপি করুন (Copy Bangla)</button>
        <button class="btn secondary" onclick="document.getElementById('banglaPad').value=''">মুছুন (Clear)</button>
        <button class="btn secondary" onclick="showTab('xray', null)">🔬 ২-ক্লিক যুক্তবর্ণ নির্মাতা ও শব্দ এক্স-রে</button>
        <button class="btn secondary" onclick="showTab('signs', null)">📖 সিরিজ অনুযায়ী সব যুক্তবর্ণ ও শব্দ</button>
      </div>

      <div class="row" style="justify-content: space-between; margin-top: 16px;">
        <h2 style="font-size:1.06rem; margin-bottom:0;">এক ক্লিকে যুক্তবর্ণ, ফলা, কার ও চিহ্ন (One-Click Connected Letters & Signs)</h2>
        <div class="chip-bar" style="margin:0;">
          <span class="chip primary" onclick="filterQuickGrid('ALL', this)">সব (All)</span>
          <span class="chip" onclick="filterQuickGrid('Conjunct', this)">যুক্তবর্ণ (ক্ত, ক্ষ, জ্ঞ, ষ্ট, স্ব)</span>
          <span class="chip" onclick="filterQuickGrid('Fola', this)">ফলা (ক্র, ক্য, স্ব, র্ক)</span>
          <span class="chip" onclick="filterQuickGrid('Kar', this)">কার (কৃ, কা, কি, কী...)</span>
          <span class="chip" onclick="filterQuickGrid('Mark', this)">চিহ্ন (চাঁ, বাং, দুঃ...)</span>
        </div>
      </div>
      <div class="quick-grid" id="quickSignsGrid"></div>
    </section>

    <!-- TAB 2: WORD X-RAY & 2-CLICK VISUAL CONJUNCT BUILDER (UNIQUE TO EKUSHE) -->
    <section class="panel" id="tab-xray">
      <h2>🔬 শব্দ এক্স-রে ও ২-ক্লিক যুক্তবর্ণ নির্মাতা (Word X-Ray & Visual Conjunct Builder)</h2>
      <p class="subtitle">যেকোনো জটিল বাংলা শব্দ কীভাবে ভাঙতে ও টাইপ করতে হয় তা তাৎক্ষণিক দেখুন, অথবা যেকোনো ২টি ব্যঞ্জনবর্ণে ক্লিক করে শর্টকাট মুখস্থ ছাড়াই যুক্তবর্ণ তৈরি করুন!</p>

      <div class="xray-grid">
        <!-- Left Card: Word X-Ray Decomposer -->
        <div class="glass-subcard">
          <h2 style="font-size:1.05rem;">🔍 বাংলা শব্দ এক্স-রে (Word X-Ray Inspector)</h2>
          <p class="subtitle" style="margin-bottom:8px;">যেকোনো বাংলা শব্দ লিখুন বা পেস্ট করুন—প্রতিটি অক্ষর, কার, ফলা ও যুক্তবর্ণ কীভাবে গঠিত তা দেখুন:</p>
          <input type="text" id="xrayInput" value="সৃষ্টি" placeholder="বাংলা শব্দ পেস্ট করুন, যেমন: সৃষ্টি, স্মৃতি, বিজ্ঞান, আশ্চর্য..." oninput="runWordXray()">
          <div class="chip-bar">
            <span class="chip" onclick="setXrayWord('সৃষ্টি')">সৃষ্টি</span>
            <span class="chip" onclick="setXrayWord('স্মৃতি')">স্মৃতি</span>
            <span class="chip" onclick="setXrayWord('বিজ্ঞান')">বিজ্ঞান</span>
            <span class="chip" onclick="setXrayWord('শিক্ষা')">শিক্ষা</span>
            <span class="chip" onclick="setXrayWord('স্বচ্ছ')">স্বচ্ছ</span>
            <span class="chip" onclick="setXrayWord('শ্রেষ্ঠ')">শ্রেষ্ঠ</span>
            <span class="chip" onclick="setXrayWord('আশ্চর্য')">আশ্চর্য</span>
            <span class="chip" onclick="setXrayWord('উত্তেজনা')">উত্তেজনা</span>
          </div>
          <div id="xraySummary" style="margin-top:8px;font-weight:700;color:var(--brand-primary);font-size:0.94rem;"></div>
          <div class="xray-tokens" id="xrayTokens"></div>
        </div>

        <!-- Right Card: 2-Click Visual Conjunct Builder -->
        <div class="glass-subcard">
          <h2 style="font-size:1.05rem;">🧩 ২-ক্লিক ভিজ্যুয়াল যুক্তবর্ণ নির্মাতা (2-Click Conjunct Builder)</h2>
          <p class="subtitle" style="margin-bottom:8px;">১ম ব্যঞ্জনবর্ণ এবং ২য় ব্যঞ্জনবর্ণে ক্লিক করুন—একুশে নিজে থেকেই যুক্তবর্ণ তৈরি করে দেবে:</p>
          <div style="font-size:0.82rem;font-weight:700;color:var(--brand-primary);">১ম অক্ষর নির্বাচন করুন (1st Consonant):</div>
          <div class="consonant-matrix" id="cMatrix1"></div>
          <div style="font-size:0.82rem;font-weight:700;color:var(--brand-primary);margin-top:9px;">২য় অক্ষর নির্বাচন করুন (2nd Consonant):</div>
          <div class="consonant-matrix" id="cMatrix2"></div>

          <div class="fusion-preview">
            <div>
              <div style="font-size:0.78rem;color:var(--text-muted);" id="fusionFormula">ষ + ্ + ট =</div>
              <div class="fusion-big" id="fusionResult">ষ্ট</div>
              <div style="font-size:0.82rem;color:var(--crimson);font-weight:700;" id="fusionRoman">Type: sht / ShT</div>
            </div>
            <div class="row" style="margin:0;">
              <button class="btn" onclick="insertBuiltConjunct()">➕ প্যাডে যুক্ত ও কপি করুন</button>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- TAB 3: SIGNS, CONJUNCTS & WORD EXAMPLES HELPER PAGE -->
    <section class="panel" id="tab-signs">
      <h2>যুক্তবর্ণ, কার-ফলা ও শব্দ উদাহরণ সহায়িকা (Connected Letters & Word Helper)</h2>
      <p class="subtitle">সিরিজ অনুযায়ী সাজানো (<b>K-Series, Ch-Series, T-Series, N-Series, P-Series, Sh-Series, D-Series, L-Series, ফলা ও কার</b>)। যেকোনো <b>যুক্তবর্ণ</b> বা <b>উদাহরণ শব্দে</b> ক্লিক করলে সাথে সাথে ক্লিপবোর্ডে কপি হবে এবং টাইপিং প্যাডে যুক্ত হবে!</p>
      <div class="row" style="justify-content: space-between; margin-bottom: 10px;">
        <div class="chip-bar" id="atlasFilterBar" style="margin:0;">
          <span class="chip primary" onclick="filterAtlasGrid('ALL', this)">সব (All Series)</span>
          <span class="chip" onclick="filterAtlasGrid('K-Series', this)">K-Series (ক/ঙ: ক্ত, ক্ষ, ঙ্ক, ঙ্গ)</span>
          <span class="chip" onclick="filterAtlasGrid('Ch-Series', this)">Ch-Series (চ/জ/ঞ: চ্চ, চ্ছ, জ্ঞ, ঞ্চ)</span>
          <span class="chip" onclick="filterAtlasGrid('T-Series', this)">T-Series (ত/ট: ট্ট, ত্ত, ত্র, ত্ব)</span>
          <span class="chip" onclick="filterAtlasGrid('N-Series', this)">N-Series (ন/ণ: ণ্ট, ণ্ঠ, ন্দ, ন্ধ, ন্ত)</span>
          <span class="chip" onclick="filterAtlasGrid('P-Series', this)">P-Series (প/ম: প্ত, ম্প, ম্ব, ম্ভ)</span>
          <span class="chip" onclick="filterAtlasGrid('Sh-Series', this)">Sh-Series (শ/ষ/স: শ্চ, ষ্ট, ষ্ঠ, স্ক, স্ত, স্থ, স্প, স্ব, স্ম)</span>
          <span class="chip" onclick="filterAtlasGrid('D-Series', this)">D-Series (দ/ধ: দ্ধ, দ্ব, ধ্ব)</span>
          <span class="chip" onclick="filterAtlasGrid('L-Series', this)">L/H-Series (ল/হ: ল্প, ল্ল, হ্ম)</span>
          <span class="chip" onclick="filterAtlasGrid('Fola', this)">ফলা (্র, ্য, র্, ্ব, ্ম)</span>
          <span class="chip" onclick="filterAtlasGrid('Kar', this)">কার (ৃ, া, ি, ী, ু, ূ)</span>
          <span class="chip" onclick="filterAtlasGrid('Mark', this)">চিহ্ন (ঁ, ং, ঃ, ৎ)</span>
        </div>
        <div id="signsToast" style="color:var(--brand-primary);font-weight:700;font-size:0.90rem;"></div>
      </div>
      <input type="text" id="signSearch" placeholder="যেকোনো যুক্তবর্ণ, বাংলা শব্দ, ইংরেজি উচ্চারণ বা অর্থ খুঁজুন: যেমন রক্ত, rokto, blood, সৃষ্টি, srishti, কষ্ট, জ্ঞ, kkh..." oninput="loadSigns(this.value)">
      <div class="atlas-grid" id="atlasGrid"></div>
    </section>

    <!-- TAB 4: SPELL CHECKER -->
    <section class="panel" id="tab-spell">
      <h2>বাংলা বানান পরীক্ষক (Bangla Spell Checker)</h2>
      <p class="subtitle">হ্রস্ব-ই/দীর্ঘ-ঈ, হ্রস্ব-উ/দীর্ঘ-ঊ, শ/ষ/স, ন/ণ, জ/য, র/ড় জনিত ভুল বানান খুঁজে বের করে সঠিক পরামর্শ দেয়।</p>
      <textarea id="spellInput" rows="5">আমি বাংলাদেস এ বাস করি। মানুস মাত্রই জিবন ভালোবাসে।</textarea>
      <div class="row">
        <button class="btn" onclick="runSpellCheck()">🔍 বানান পরীক্ষা করুন (Check Spelling)</button>
      </div>
      <div id="spellResults" style="margin-top:14px;"></div>
    </section>

    <!-- TAB 5: BIJOY <-> UNICODE CONVERTER -->
    <section class="panel" id="tab-converter">
      <h2>বিজয় (ANSI / SutonnyMJ) ⇄ ইউনিকোড এবং বাংলিশ কনভার্টার</h2>
      <p class="subtitle">যেকোনো অ্যাপে লেখা সিলেক্ট করে <b>Ctrl+Alt+C</b> চাপলে সরাসরি ইউনিকোড বাংলায় রূপান্তরিত হবে।</p>
      <textarea id="convInput" rows="4" placeholder="এখানে বিজয় (SutonnyMJ), ইউনিকোড অথবা বাংলিশ টেক্সট পেস্ট করুন...">Avwg evsjvq Mvb MvB| evsjv†`k Avgvi Ÿ†`k|</textarea>
      <div class="row">
        <button class="btn" onclick="runConvert('bijoy2uni')">বিজয় ➔ ইউনিকোড (Bijoy to Unicode)</button>
        <button class="btn secondary" onclick="runConvert('uni2bijoy')">ইউনিকোড ➔ বিজয় (Unicode to Bijoy)</button>
        <button class="btn secondary" onclick="runConvert('banglish')">বাংলিশ ➔ বাংলা (Banglish to Bangla)</button>
        <button class="btn secondary" onclick="navigator.clipboard.writeText(document.getElementById('convOutput').value)">📋 ফলাফল কপি করুন (Copy)</button>
      </div>
      <textarea id="convOutput" rows="4" style="margin-top:14px;" placeholder="রূপান্তরিত ফলাফল..."></textarea>
    </section>

    <!-- TAB 6: SETTINGS -->
    <section class="panel" id="tab-settings">
      <h2>সেটিংস, ফন্ট ও স্বচ্ছতা (Settings, Fonts & Glass Transparency)</h2>
      <p class="subtitle">সহজ ব্যবহারকারী এবং অ্যাডভান্সড ব্যবহারকারী—সবার জন্য তাৎক্ষণিক কাস্টমাইজেশন।</p>
      <div class="form-group">
        <label>পপ-আপ ও অ্যাপের স্বচ্ছতা (Floating Popup & Studio Glass Transparency: <span id="cfgOpacityVal">92%</span>)</label>
        <input type="range" id="cfgOpacity" min="70" max="100" value="92" oninput="onOpacityChange(this.value)" style="width:100%;accent-color:var(--brand-accent);">
      </div>
      <div class="form-group">
        <label>ডিফল্ট বাংলাদেশী বাংলা ফন্ট (Default Bundled Bangladeshi Font)</label>
        <select id="cfgFont" onchange="changeBanglaFont(this.value)">
          <option value="Kalpurush">কালপুরুষ (Kalpurush — বাংলাদেশী ডিফল্ট / OmicronLab Standard)</option>
          <option value="SolaimanLipi">সোলাইমানলিপি (SolaimanLipi — একুশে ক্লাসিক স্ট্যান্ডার্ড)</option>
          <option value="Siyam Rupali">সিয়াম রূপালী (Siyam Rupali — স্পষ্ট স্ক্রিন ফন্ট)</option>
          <option value="Hind Siliguri">হিন্দ শিলিগুড়ি (Hind Siliguri — আধুনিক ইউআই ফন্ট)</option>
        </select>
      </div>
      <div class="form-group">
        <label>কীবোর্ড পরিবর্তনের হটকি (Switch Between Normal Keyboard & Ekushe Next)</label>
        <select id="cfgHotkey" onchange="saveSettings()">
          <option value="DoubleTapShift">Double-Tap Shift (দুইবার Shift চাপুন — Recommended)</option>
          <option value="CtrlSpace">Ctrl + Space</option>
          <option value="F12">F12 (Classic Avro)</option>
          <option value="CtrlAltB">Ctrl + Alt + B</option>
        </select>
      </div>
      <div class="form-group">
        <label>টাইপিং মোড (Typing Mode)</label>
        <select id="cfgMode" onchange="saveSettings()">
          <option value="Smart">Smart Mode (ডিকশনারি সেরা শব্দটি বেছে নেয় — Default)</option>
          <option value="PhoneticFirst">Phonetic-First (আপনার টাইপ করা বানান প্রথমে থাকবে)</option>
          <option value="PhoneticOnly">Phonetic-Only (কোনো পপআপ ছাড়াই সরাসরি টাইপ)</option>
        </select>
      </div>
      <div class="form-group">
        <label>কীবোর্ড লেআউট (Keyboard Layout)</label>
        <select id="cfgLayout" onchange="saveSettings()">
          <option value="AvroPhonetic">Ekushe Next Zero-Case Phonetic (Avro-compatible + Smart Lowercase)</option>
          <option value="Probhat">প্রভাত (Probhat)</option>
          <option value="NationalJatiya">জাতীয় (National / Jatiya)</option>
          <option value="Bornona">বর্ণনা (Bornona)</option>
          <option value="MunirOptima">মুনীর অপটিমা (Munir Optima)</option>
        </select>
      </div>
      <div class="form-group" style="display:grid;grid-template-columns:1fr 1fr;gap:12px;">
        <label style="display:flex;align-items:center;gap:8px;padding:10px 12px;border-radius:10px;background:var(--bg-card-sub);border:1px solid var(--border-subtle);cursor:pointer;">
          <input type="checkbox" id="cfgCompact" onchange="saveSettings()" style="width:18px;height:18px;accent-color:var(--brand-accent);">
          <span><b>সংক্ষিপ্ত ফ্লোটিং পপ-আপ (Compact Popup)</b><br><small style="color:var(--text-muted)">পপ-আপের ৩য় যুক্তবর্ণ লাইন লুকিয়ে রাখুন (পপ-আপে 🔽 বাটনেও ক্লিক করা যায়)</small></span>
        </label>
        <label style="display:flex;align-items:center;gap:8px;padding:10px 12px;border-radius:10px;background:var(--bg-card-sub);border:1px solid var(--border-subtle);cursor:pointer;">
          <input type="checkbox" id="cfgTerminal" checked onchange="saveSettings()" style="width:18px;height:18px;accent-color:var(--brand-accent);">
          <span><b>টার্মিনালে স্বয়ংক্রিয় ইংরেজি (Terminal Smart Guard)</b><br><small style="color:var(--text-muted)">Windows Terminal / CMD / PowerShell এ কমান্ড ভাঙা রোধে স্বয়ংক্রিয় ইংরেজি</small></span>
        </label>
      </div>

      <div class="glass-subcard" style="margin-top:14px;">
        <h2 style="font-size:1.02rem;margin-bottom:4px;">⚡ কুইক বাংলা স্নিপেট ও সংক্ষিপ্ত রূপ (Personal Bangla Text Snippets)</h2>
        <p class="subtitle" style="margin-bottom:8px;">যেকোনো অ্যাপে শর্টকাট লিখে Space চাপলেই পুরো বাংলা বাক্য বা চিহ্ন বসে যাবে! যেমন: <code>;salam</code> ➔ <b>আসসালামু আলাইকুম</b>, <code>;tk</code> ➔ <b>৳</b>, <code>;bd</code> ➔ <b>গণপ্রজাতন্ত্রী বাংলাদেশ</b>। নিচে নিজের কাস্টম স্নিপেট যোগ করুন:</p>
        <div class="row" style="margin-bottom:10px;">
          <input type="text" id="newSnipKey" placeholder="শর্টকাট (যেমন: ;addr বা ;mail)" style="max-width:210px;">
          <input type="text" id="newSnipVal" placeholder="পুরো বাংলা লেখা (যেমন: ধানমন্ডি, ঢাকা-১২০৯)" style="flex:1;">
          <button class="btn" onclick="addCustomSnippet()">➕ স্নিপেট যোগ করুন</button>
        </div>
        <div class="chip-bar" id="snippetsList"></div>
      </div>

      <div id="settingsSavedMsg" style="color:var(--brand-primary);font-weight:700;margin-top:10px;"></div>
    </section>
  </main>
</div>

<script>
// Every Studio API call carries this page's token; other websites cannot read or change settings.
const EKUSHE_TOKEN = "__EKUSHE_TOKEN__";
const nativeFetch = window.fetch.bind(window);
window.fetch = (url, opts = {}) => {
  if (typeof url === 'string' && url.startsWith('/api/')) {
    opts = Object.assign({}, opts);
    opts.headers = Object.assign({}, opts.headers || {}, { 'X-Ekushe-Token': EKUSHE_TOKEN });
  }
  return nativeFetch(url, opts);
};
let allSignItems = [];
let currentSearchItems = [];
let activeQuickKind = 'ALL';
let activeAtlasKind = 'ALL';
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
  'া':'a (আ-কার)','ি':'i (ই-কার)','ী':'I/ii (ঈ-কার)','ু':'u (উ-কার)','ূ':'U/uu (ঊ-কার)',
  'ৃ':'ri/rri (ঋ-কার)','ে':'e (এ-কার)','ৈ':'OI (ঐ-কার)','ো':'o (ও-কার)','ৌ':'OU (ঔ-কার)','্':'+ (হসন্ত/যুক্ত)'
};

function toggleTheme() {
  const html = document.documentElement;
  const current = html.getAttribute('data-theme') || 'light';
  const next = current === 'light' ? 'dark' : 'light';
  html.setAttribute('data-theme', next);
  localStorage.setItem('ekushe_theme', next);
  document.getElementById('themeBtn').textContent = next === 'light' ? '☀️ Light Mode' : '🌙 Dark Mode';
}

(function initTheme() {
  const saved = localStorage.getItem('ekushe_theme') || 'light';
  document.documentElement.setAttribute('data-theme', saved);
  const btn = document.getElementById('themeBtn');
  if (btn) btn.textContent = saved === 'light' ? '☀️ Light Mode' : '🌙 Dark Mode';
})();

function applyOpacityVisuals(pct) {
  const clamped = Math.max(70, Math.min(100, parseInt(pct || 92, 10)));
  const glassAlpha = (clamped / 100) * 0.85;
  document.documentElement.style.setProperty('--glass-alpha', glassAlpha.toFixed(2));
  const lbl = document.getElementById('opacityLabel');
  const cVal = document.getElementById('cfgOpacityVal');
  const hSl = document.getElementById('headerOpacitySlider');
  const cSl = document.getElementById('cfgOpacity');
  if (lbl) lbl.textContent = clamped + '%';
  if (cVal) cVal.textContent = clamped + '%';
  if (hSl) hSl.value = clamped;
  if (cSl) cSl.value = clamped;
}

async function onOpacityChange(pct) {
  applyOpacityVisuals(pct);
  await saveSettings();
}

function applyCssFont(fontName) {
  document.documentElement.style.setProperty(
    '--font-bn',
    `'${fontName}', 'Kalpurush', 'SolaimanLipi', 'Siyam Rupali', 'Hind Siliguri', sans-serif`
  );
  const hSel = document.getElementById('headerFontSelect');
  const cSel = document.getElementById('cfgFont');
  if (hSel) hSel.value = fontName;
  if (cSel) cSel.value = fontName;
}

async function changeBanglaFont(fontName) {
  applyCssFont(fontName);
  await saveSettings();
}

function showTab(id, btn) {
  const panel = document.getElementById('tab-' + id);
  if (!panel) return;
  document.querySelectorAll('.panel').forEach(p => p.classList.remove('active'));
  document.querySelectorAll('.nav-btn').forEach(b => {
    b.classList.remove('active');
    if (!btn && (b.getAttribute('onclick') || '').includes("'" + id + "'")) {
      b.classList.add('active');
    }
  });
  panel.classList.add('active');
  if (btn) btn.classList.add('active');
  if (id === 'spell') runSpellCheck();
  if (id === 'converter') runConvert('bijoy2uni');
  if (id === 'xray') runWordXray();
}

async function refreshState() {
  const res = await fetch('/api/state');
  const st = await res.json();
  const pill = document.getElementById('modePill');
  const txt = document.getElementById('modeText');
  if (st.language === 'Bangla') {
    pill.className = 'mode-pill bangla';
    txt.textContent = 'বাং · বাংলা সক্রিয় (০-৯ · Tap Shift×2)';
  } else {
    pill.className = 'mode-pill english';
    txt.textContent = 'EN · Default English (0-9)';
  }
  document.getElementById('hotkeyBadge').textContent = 'Switch: ' + st.hotkey_display;
  document.getElementById('cfgHotkey').value = st.hotkey_id;
  document.getElementById('cfgMode').value = st.typing_mode;
  document.getElementById('cfgLayout').value = st.active_layout;
  const cmpEl = document.getElementById('cfgCompact');
  if (cmpEl && typeof st.compact_popup === 'boolean') cmpEl.checked = st.compact_popup;
  const trmEl = document.getElementById('cfgTerminal');
  if (trmEl && typeof st.auto_english_in_terminals === 'boolean') trmEl.checked = st.auto_english_in_terminals;
  if (Array.isArray(st.snippets)) {
    const sBox = document.getElementById('snippetsList');
    if (sBox) {
      sBox.innerHTML = st.snippets.map(([k, v]) =>
        `<span class="chip" onclick="insertIntoPad('${v} ')" title="ক্লিক করলে রাইটিং প্যাডে যুক্ত হবে"><code>${k}</code> ➔ <b>${v}</b></span>`
      ).join('');
    }
  }
  if (st.font_family) {
    applyCssFont(st.font_family);
  }
  if (st.popup_opacity) {
    applyOpacityVisuals(st.popup_opacity);
  }
}

async function toggleMode() {
  await fetch('/api/toggle', { method: 'POST' });
  await refreshState();
}

async function onRomanInput() {
  const val = document.getElementById('romanInput').value;
  const res = await fetch('/api/suggest?q=' + encodeURIComponent(val));
  const data = await res.json();
  const cBar = document.getElementById('candidateBar');
  cBar.innerHTML = '';
  data.candidates.forEach((c, idx) => {
    const el = document.createElement('span');
    el.className = 'chip' + (idx === 0 ? ' primary' : '');
    el.textContent = (idx + 1) + '. ' + c.text + (c.annotation ? ' (' + c.annotation + ')' : '');
    el.onclick = () => insertIntoPad(c.text + ' ');
    cBar.appendChild(el);
  });

  const hBar = document.getElementById('hintBar');
  hBar.innerHTML = '';
  data.sign_hints.slice(0, 12).forEach(h => {
    const el = document.createElement('span');
    el.className = 'chip';
    el.textContent = h.result_bangla + ' [' + h.key_hint + ' · ' + h.label + ']';
    el.onclick = () => {
      document.getElementById('romanInput').value = h.key_hint;
      onRomanInput();
    };
    hBar.appendChild(el);
  });
}

document.getElementById('romanInput').addEventListener('keydown', (e) => {
  if (e.key === ' ' || e.key === 'Enter') {
    const first = document.querySelector('#candidateBar .chip');
    if (first) {
      e.preventDefault();
      const text = first.textContent.replace(/^\d+\.\s*/, '').replace(/\s*\(.*\)$/, '');
      insertIntoPad(text + ' ');
      document.getElementById('romanInput').value = '';
      onRomanInput();
    }
  }
});

// ════════════════════════════════════════════════════════════════════════
// Daily Bangla Writing Power-Toolbar Functions
// ════════════════════════════════════════════════════════════════════════
function convertPadDigitsToBangla() {
  const pad = document.getElementById('banglaPad');
  const map = {'0':'০','1':'১','2':'২','3':'৩','4':'৪','5':'৫','6':'৬','7':'৭','8':'৮','9':'৯'};
  pad.value = pad.value.replace(/[0-9]/g, d => map[d]);
}

function convertPadDigitsToEnglish() {
  const pad = document.getElementById('banglaPad');
  const map = {'০':'0','১':'1','২':'2','৩':'3','৪':'4','৫':'5','৬':'6','৭':'7','৮':'8','৯':'9'};
  pad.value = pad.value.replace(/[০-৯]/g, d => map[d]);
}

function insertTakaCurrency() {
  const pad = document.getElementById('banglaPad');
  // Convert any trailing number or insert sample formatted Taka
  const replaced = pad.value.replace(/([0-9০-৯]{4,})/g, (m) => {
    return '৳ ' + m + '/-';
  });
  if (replaced !== pad.value) {
    pad.value = replaced;
  } else {
    insertIntoPad('৳ ১,০০০/- ');
  }
}

function insertTodayBanglaDate() {
  const now = new Date();
  const bnDigits = ['০','১','২','৩','৪','৫','৬','৭','৮','৯'];
  const toBn = (n) => String(n).replace(/[0-9]/g, d => bnDigits[+d]);
  const monthsBn = ['জানুয়ারি','ফেব্রুয়ারি','মার্চ','এপ্রিল','মে','জুন','জুলাই','আগস্ট','সেপ্টেম্বর','অক্টোবর','নভেম্বর','ডিসেম্বর'];
  const stamp = `${toBn(now.getDate())} ${monthsBn[now.getMonth()]} ${toBn(now.getFullYear())} `;
  insertIntoPad(stamp);
}

function smartPolishBanglaPad() {
  const pad = document.getElementById('banglaPad');
  let t = pad.value;
  // Normalize decomposed nukta characters to canonical Bangla Unicode
  t = t.replace(/\u09AF\u09BC/g, 'য়').replace(/\u09A1\u09BC/g, 'ড়').replace(/\u09A2\u09BC/g, 'ঢ়');
  // Replace English period after Bangla characters with Bangla Danda (।)
  t = t.replace(/([\u0980-\u09FF])\./g, '$1।');
  // Clean multiple spaces
  t = t.replace(/[ \t]{2,}/g, ' ');
  pad.value = t;
}

function inspectPadWordInXray() {
  const pad = document.getElementById('banglaPad').value.trim();
  const words = pad.split(/\s+/);
  const last = (words[words.length - 1] || 'সৃষ্টি').replace(/[।,.!?]/g, '');
  document.getElementById('xrayInput').value = last || 'সৃষ্টি';
  showTab('xray', null);
}

// ════════════════════════════════════════════════════════════════════════
// Word X-Ray & 2-Click Visual Conjunct Builder
// ════════════════════════════════════════════════════════════════════════
function setXrayWord(w) {
  document.getElementById('xrayInput').value = w;
  runWordXray();
}

function runWordXray() {
  const word = (document.getElementById('xrayInput').value || '').trim();
  const box = document.getElementById('xrayTokens');
  const sum = document.getElementById('xraySummary');
  box.innerHTML = '';
  if (!word) {
    sum.textContent = '';
    return;
  }

  // Check if this word or conjunct is in our curated helper database
  let matchedRoman = '';
  for (const it of allSignItems) {
    if (it.words) {
      const found = it.words.find(w => w.bangla === word);
      if (found) {
        matchedRoman = `${found.roman} (${found.meaning})`;
        break;
      }
    }
  }

  const chars = Array.from(word);
  let i = 0;
  const parts = [];
  while (i < chars.length) {
    // Detect C1 + Hasanta + C2 (+ Hasanta + C3) conjunct cluster
    if (i + 2 < chars.length && chars[i + 1] === '্') {
      if (i + 4 < chars.length && chars[i + 3] === '্') {
        const cluster = chars.slice(i, i + 5).join('');
        parts.push({ glyph: cluster, formula: `${chars[i]} + ্ + ${chars[i+2]} + ্ + ${chars[i+4]}`, type: 'যুক্তবর্ণ' });
        i += 5;
        continue;
      }
      const cluster = chars.slice(i, i + 3).join('');
      const known = allSignItems.find(it => it.bangla === cluster);
      const keyHint = known ? known.primary_key : ((CHAR_ROMAN_MAP[chars[i]] || '') + (CHAR_ROMAN_MAP[chars[i+2]] || ''));
      parts.push({
        glyph: cluster,
        formula: `${chars[i]} + ${chars[i+2]} (${keyHint})`,
        type: 'যুক্তবর্ণ'
      });
      i += 3;
      continue;
    }
    const ch = chars[i];
    const key = CHAR_ROMAN_MAP[ch] || ch;
    const isKar = ch >= '\u09BE' && ch <= '\u09CC';
    parts.push({
      glyph: ch,
      formula: key,
      type: isKar ? 'কার (Vowel Sign)' : 'বর্ণ (Letter)'
    });
    i++;
  }

  sum.textContent = matchedRoman
    ? `✨ "${word}" সহজে টাইপ করুন: ${matchedRoman}`
    : `✨ "${word}" শব্দের অক্ষর ও যুক্তবর্ণ বিশ্লেষণ (${parts.length}টি অংশ):`;

  parts.forEach(p => {
    const d = document.createElement('div');
    d.className = 'xray-token';
    d.innerHTML = `<span class="xt-char">${p.glyph}</span><span class="xt-key">${p.formula}</span><span class="xt-name">${p.type}</span>`;
    box.appendChild(d);
  });
}

function initConjunctBuilder() {
  const m1 = document.getElementById('cMatrix1');
  const m2 = document.getElementById('cMatrix2');
  if (!m1 || !m2) return;
  m1.innerHTML = '';
  m2.innerHTML = '';

  BANGLA_CONSONANTS.forEach(item => {
    const b1 = document.createElement('div');
    b1.className = 'c-btn' + (item.c === selectedC1 ? ' selected' : '');
    b1.textContent = item.c;
    b1.title = `${item.c} (${item.r})`;
    b1.onclick = () => {
      selectedC1 = item.c;
      initConjunctBuilder();
      updateFusionPreview();
    };
    m1.appendChild(b1);

    const b2 = document.createElement('div');
    b2.className = 'c-btn' + (item.c === selectedC2 ? ' selected' : '');
    b2.textContent = item.c;
    b2.title = `${item.c} (${item.r})`;
    b2.onclick = () => {
      selectedC2 = item.c;
      initConjunctBuilder();
      updateFusionPreview();
    };
    m2.appendChild(b2);
  });
  updateFusionPreview();
}

function updateFusionPreview() {
  const fused = selectedC1 + '্' + selectedC2;
  document.getElementById('fusionFormula').textContent = `${selectedC1} + ্ + ${selectedC2} =`;
  document.getElementById('fusionResult').textContent = fused;
  const known = allSignItems.find(it => it.bangla === fused);
  const r1 = (BANGLA_CONSONANTS.find(x => x.c === selectedC1) || {r:''}).r;
  const r2 = (BANGLA_CONSONANTS.find(x => x.c === selectedC2) || {r:''}).r;
  const romanHint = known
    ? `${known.primary_key}${known.alt_keys && known.alt_keys.length ? ' / ' + known.alt_keys.join(', ') : ''}`
    : `${r1}${r2}`;
  document.getElementById('fusionRoman').textContent = `টাইপ শর্টকাট (Type): ${romanHint}`;
}

function insertBuiltConjunct() {
  const fused = selectedC1 + '্' + selectedC2;
  insertIntoPad(fused);
  if (navigator.clipboard) {
    navigator.clipboard.writeText(fused);
  }
}

function isBanglaConsonant(ch) {
  if (!ch) return false;
  const code = ch.charCodeAt(0);
  return (code >= 0x0995 && code <= 0x09B9) || code === 0x09DC || code === 0x09DD || code === 0x09DF;
}

function insertSignItemIntoPad(item) {
  const pad = document.getElementById('banglaPad');
  const current = pad.value;
  const lastChar = current.length > 0 ? current[current.length - 1] : '';
  let inserted = item.bangla;
  if ((item.kind === 'Kar' || item.kind === 'Fola') && !isBanglaConsonant(lastChar)) {
    const cleanSyllable = item.display_bangla.split('/')[0].trim();
    pad.value += cleanSyllable;
    inserted = cleanSyllable;
  } else {
    pad.value += item.bangla;
  }
  if (navigator.clipboard) {
    navigator.clipboard.writeText(item.bangla);
  }
  const toast = document.getElementById('signsToast');
  if (toast) {
    toast.textContent = `✓ '${inserted}' কপি হয়েছে এবং প্যাডে যুক্ত হয়েছে! (Type: ${item.primary_key})`;
  }
}

function insertWordExample(banglaWord, romanKey, meaning, ev) {
  if (ev) ev.stopPropagation();
  const pad = document.getElementById('banglaPad');
  pad.value += (pad.value && !pad.value.endsWith(' ') ? ' ' : '') + banglaWord + ' ';
  if (navigator.clipboard) {
    navigator.clipboard.writeText(banglaWord);
  }
  const toast = document.getElementById('signsToast');
  if (toast) {
    toast.textContent = `✓ '${banglaWord}' কপি হয়েছে! (Type: ${romanKey} — ${meaning})`;
  }
}

function insertIntoPad(str) {
  const pad = document.getElementById('banglaPad');
  pad.value += str;
  pad.focus();
}

function copyPad() {
  const pad = document.getElementById('banglaPad');
  navigator.clipboard.writeText(pad.value);
}

function filterQuickGrid(kind, chipEl) {
  activeQuickKind = kind;
  if (chipEl && chipEl.parentElement) {
    chipEl.parentElement.querySelectorAll('.chip').forEach(c => c.classList.remove('primary'));
    chipEl.classList.add('primary');
  }
  renderQuickGrid();
}

function filterAtlasGrid(kindOrSeries, chipEl) {
  activeAtlasKind = kindOrSeries;
  if (chipEl && chipEl.parentElement) {
    chipEl.parentElement.querySelectorAll('.chip').forEach(c => c.classList.remove('primary'));
    chipEl.classList.add('primary');
  }
  renderAtlasGrid();
}

function renderQuickGrid() {
  const qGrid = document.getElementById('quickSignsGrid');
  qGrid.innerHTML = '';
  const filtered = activeQuickKind === 'ALL'
    ? allSignItems
    : allSignItems.filter(it => it.kind === activeQuickKind || it.series === activeQuickKind);
  filtered.slice(0, 36).forEach(item => {
    const b = document.createElement('div');
    b.className = 'sign-btn';
    const alts = item.alt_keys && item.alt_keys.length ? ' / ' + item.alt_keys[0] : '';
    b.innerHTML = `<span class="bn">${item.display_bangla}</span><span class="rm">${item.primary_key}${alts}</span><span class="dc">${item.decomposition}</span>`;
    b.onclick = () => insertSignItemIntoPad(item);
    qGrid.appendChild(b);
  });
}

function renderAtlasGrid() {
  const atlas = document.getElementById('atlasGrid');
  atlas.innerHTML = '';
  const filtered = activeAtlasKind === 'ALL'
    ? currentSearchItems
    : currentSearchItems.filter(it => it.series === activeAtlasKind || it.kind === activeAtlasKind);
  filtered.forEach(item => {
    const card = document.createElement('div');
    card.className = 'atlas-card';
    const alts = item.alt_keys && item.alt_keys.length ? ' / ' + item.alt_keys.join(', ') : '';
    const wordsHtml = (item.words || []).map(w =>
      `<span class="word-pill" title="ক্লিক করে '${w.bangla}' কপি করুন (Type: ${w.roman})" onclick="insertWordExample('${w.bangla}', '${w.roman}', '${w.meaning.replace(/'/g, "\\'")}', event)">
        <span class="wb">${w.bangla}</span>
        <span class="wr">${w.roman}</span>
        <span class="wm">(${w.meaning})</span>
      </span>`
    ).join('');

    card.innerHTML = `
      <div class="atlas-card-top" title="ক্লিক করে '${item.bangla}' কপি করুন">
        <div class="atlas-glyph">${item.display_bangla}</div>
        <div class="atlas-meta">
          <div>
            <span class="key">${item.primary_key}${alts}</span>
            <span class="series-tag">${item.series || item.kind}</span>
          </div>
          <div style="font-weight:700;margin-top:4px;color:var(--text-main);">${item.decomposition}</div>
        </div>
      </div>
      <div class="word-list">${wordsHtml}</div>
    `;
    card.querySelector('.atlas-card-top').onclick = () => insertSignItemIntoPad(item);
    atlas.appendChild(card);
  });
}

async function loadSigns(q) {
  const res = await fetch('/api/signs?q=' + encodeURIComponent(q || ''));
  const items = await res.json();
  currentSearchItems = items;
  renderAtlasGrid();

  if (!q) {
    allSignItems = items;
    renderQuickGrid();
    initConjunctBuilder();
    runWordXray();
  }
}

async function runSpellCheck() {
  const text = document.getElementById('spellInput').value;
  const res = await fetch('/api/spell', {
    method: 'POST',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify({ text })
  });
  const issues = await res.json();
  const box = document.getElementById('spellResults');
  if (!issues.length) {
    box.innerHTML = '<div style="color:#0d6e4e;font-weight:700;">✓ কোনো বানান ভুল পাওয়া যায়নি! (All words look good!)</div>';
    return;
  }
  box.innerHTML = '';
  issues.forEach(iss => {
    const div = document.createElement('div');
    div.className = 'issue-card';
    const chips = iss.suggestions.map(s => `<span class="chip" onclick="replaceSpell('${iss.word}', '${s}')">${s}</span>`).join(' ');
    div.innerHTML = `<b>সম্ভাব্য ভুল: <span style="color:var(--crimson)">${iss.word}</span></b> — সঠিক পরামর্শে ক্লিক করুন:<div class="chip-bar">${chips}</div>`;
    box.appendChild(div);
  });
}

function replaceSpell(wrong, right) {
  const el = document.getElementById('spellInput');
  el.value = el.value.replace(wrong, right);
  runSpellCheck();
}

async function runConvert(mode) {
  const text = document.getElementById('convInput').value;
  const res = await fetch('/api/convert', {
    method: 'POST',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify({ mode, text })
  });
  const out = await res.json();
  document.getElementById('convOutput').value = out.result;
}

async function saveSettings(extra) {
  const opEl = document.getElementById('cfgOpacity');
  const cmpEl = document.getElementById('cfgCompact');
  const trmEl = document.getElementById('cfgTerminal');
  const body = Object.assign({
    hotkey: document.getElementById('cfgHotkey').value,
    typing_mode: document.getElementById('cfgMode').value,
    active_layout: document.getElementById('cfgLayout').value,
    font_family: document.getElementById('cfgFont').value,
    popup_opacity: opEl ? parseInt(opEl.value, 10) : 92,
    compact_popup: cmpEl ? !!cmpEl.checked : false,
    auto_english_in_terminals: trmEl ? !!trmEl.checked : true,
  }, extra || {});
  await fetch('/api/settings', {
    method: 'POST',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify(body)
  });
  document.getElementById('settingsSavedMsg').textContent = '✓ সেটিংস, ফন্ট, স্নিপেট ও পপ-আপ স্বচ্ছতা সংরক্ষিত এবং সক্রিয় হয়েছে!';
  await refreshState();
}

async function addCustomSnippet() {
  const kEl = document.getElementById('newSnipKey');
  const vEl = document.getElementById('newSnipVal');
  let key = (kEl.value || '').trim();
  const val = (vEl.value || '').trim();
  if (!key || !val) return;
  if (!key.startsWith(';') && !key.startsWith('/')) {
    key = ';' + key;
  }
  await saveSettings({ snippet_trigger: key, snippet_expansion: val });
  kEl.value = '';
  vEl.value = '';
}

const initialTab = new URLSearchParams(window.location.search).get('tab');
if (initialTab) {
  showTab(initialTab, null);
}
refreshState();
loadSigns('');
runSpellCheck();
runConvert('bijoy2uni');
</script>
</body>
</html>
"##;
