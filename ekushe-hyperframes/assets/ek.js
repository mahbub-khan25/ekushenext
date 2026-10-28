/* Shared helpers for the Ekushe Next launch reveal (HyperFrames port of ekushe-promo's LaunchReveal.tsx).
 *
 * Everything here is a pure function of the frame number (60 fps), so every render is identical.
 * - EK.random / EK.spring are exact copies of Remotion's, so the port moves like the original.
 * - EK.sp(config) turns a Remotion spring into a GSAP ease, for ordinary GSAP tweens.
 * - The builders (Burst, Sweep, Petals, TypeOn, AddressPill, typing pieces) draw the procedural parts
 *   from a scene's frame number; each scene calls them from one `tick` tween on its own timeline. */
(function () {
  const FPS = 60;
  const C = {
    night: '#03120C', green: '#0B6B4A', greenBright: '#2FD18E', red: '#E0303F', coral: '#F26B4F',
    mustard: '#E9B949', white: '#FFFFFF',
  };
  const FONT_BN = "'Hind Siliguri', 'Kalpurush', sans-serif";
  const FONT_LATIN = "'Inter Tight', 'Hind Siliguri', sans-serif";
  const MONO = "'Ekushe Mono', monospace";

  /* What is typed, word by word: [roman, what Ekushe commits]. Checked against the real engine
   * (crates/ekushe-core/examples/sent_check.rs); music/launch_reveal.py places a click on each letter. */
  const L = {
    every: 3, gap: 14,
    R: { dark: 0, sun: 480, date: 960, name: 1320, win: 1620, mac: 2220, linux: 2820, phones: 3360, party: 3960, end: 4560, total: 5040 },
    win: { start: 110, tokens: [['shobaike', 'সবাইকে'], ['shagotom!', 'স্বাগতম!'], ['aj', 'আজ'], ['theke', 'থেকে'], ['bangla', 'বাংলা'], ['lekha', 'লেখা'], ['hok', 'হোক'], ['aro', 'আরও'], ['shohoj,', 'সহজ,'], ['aro', 'আরও'], ['shundor.', 'সুন্দর।']] },
    mac: { start: 130, tokens: [['priyo', 'প্রিয়'], ['bangla', 'বাংলা'], ['bhashi', 'ভাষী'], ['bondhura,', 'বন্ধুরা,'], ['shubhechha', 'শুভেচ্ছা'], ['roilo.', 'রইল।']] },
    linux: { start: 130, tokens: [['computer', 'কম্পিউটার'], ['jetai', 'যেটাই'], ['hok,', 'হোক,'], ['bangla', 'বাংলা'], ['likhun', 'লিখুন'], ['moner', 'মনের'], ['moto.', 'মতো।']] },
    phone: { android: 110, ios: 190, tokens: [['ashchi', 'আসছি'], ['tomar', 'তোমার'], ['phoneo,', 'ফোনেও,'], ['ektu', 'একটু'], ['opekkha', 'অপেক্ষা'], ['koro', 'করো']] },
  };

  /* ── Remotion's random() ── */
  function mulberry32(a) {
    let t = a + 0x6d2b79f5;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  }
  function hashCode(str) {
    let hash = 0;
    for (let i = 0; i < str.length; i++) { hash = (hash << 5) - hash + str.charCodeAt(i); hash |= 0; }
    return hash;
  }
  const random = (seed) => mulberry32(hashCode(String(seed)));

  /* ── Remotion's spring(), stepped exactly as Remotion steps it; cached per config ── */
  const springCache = {};
  function springTable(cfg) {
    const c = cfg.damping ?? 10, m = cfg.mass ?? 1, k = cfg.stiffness ?? 100;
    const key = c + '|' + m + '|' + k;
    if (springCache[key]) return springCache[key];
    const out = [0];
    let cur = 0, vel = 0, last = 0;
    const zeta = c / (2 * Math.sqrt(k * m)), w0 = Math.sqrt(k / m), w1 = w0 * Math.sqrt(Math.max(0, 1 - zeta * zeta));
    for (let f = 1; f <= 2400; f++) {
      const now = (f / FPS) * 1000;
      const t = Math.min(now - last, 64) / 1000;
      const v0 = -vel, x0 = 1 - cur;
      if (zeta < 1) {
        const env = Math.exp(-zeta * w0 * t), s1 = Math.sin(w1 * t), c1 = Math.cos(w1 * t);
        const frag = env * (s1 * ((v0 + zeta * w0 * x0) / w1) + x0 * c1);
        cur = 1 - frag;
        vel = zeta * w0 * frag - env * (c1 * (v0 + zeta * w0 * x0) - w1 * x0 * s1);
      } else {
        const env = Math.exp(-w0 * t);
        cur = 1 - env * (x0 + (v0 + w0 * x0) * t);
        vel = env * (v0 * (t * w0 - 1) + t * x0 * w0 * w0);
      }
      last = now;
      out.push(cur);
    }
    return (springCache[key] = out);
  }
  /** spring value at a (scene) frame; 0 before it starts */
  function spring(frame, cfg = {}) {
    if (frame <= 0) return 0;
    const tab = springTable(cfg);
    const i = Math.floor(frame);
    if (i >= tab.length - 1) return 1;
    const r = frame - i;
    return tab[i] + (tab[i + 1] - tab[i]) * r;
  }
  /** a GSAP ease that follows a Remotion spring over `D` frames (the tween's duration must be D/60 s).
   * `gain` reproduces Math.min(1, spring * gain), used for "opacity: min(1, p*1.4)". */
  function sp(cfg, D = 120, gain = 1) {
    return (p) => {
      const v = spring(p * D, cfg);
      return gain === 1 ? v : Math.min(1, v * gain);
    };
  }

  const clamp01 = (x) => Math.max(0, Math.min(1, x));
  /** Remotion interpolate() with clamping */
  function interp(f, inp, outp) {
    if (f <= inp[0]) return outp[0];
    for (let i = 1; i < inp.length; i++) {
      if (f <= inp[i]) return outp[i - 1] + ((outp[i] - outp[i - 1]) * (f - inp[i - 1])) / (inp[i] - inp[i - 1]);
    }
    return outp[outp.length - 1];
  }
  /** 0→1 over [a, b], cubic ease-out (the original's ease()) = GSAP power2.out */
  const ease = (f, a, b) => { const t = clamp01((f - a) / (b - a)); return 1 - Math.pow(1 - t, 3); };
  const seg = new Intl.Segmenter('bn', { granularity: 'grapheme' });
  const graphemes = (s) => Array.from(seg.segment(s), (g) => g.segment);
  const mix = (a, b, t) => `rgb(${a.map((v, i) => Math.round(v + (b[i] - v) * t)).join(',')})`;
  const el = (tag, style, parent, html) => {
    const e = document.createElement(tag);
    if (style) e.style.cssText = style;
    if (html !== undefined) e.innerHTML = html;
    if (parent) parent.appendChild(e);
    return e;
  };

  /** One tween that calls fn(frame) for every frame of a scene, plus once now (frame 0). */
  function tick(tl, frames, fn) {
    const s = { f: 0 };
    tl.to(s, { f: frames, duration: frames / FPS, ease: 'none', onUpdate: () => fn(Math.round(s.f * 1000) / 1000) }, 0);
    fn(0);
  }

  /* ── procedural pieces ── */

  /** A burst of petals and sparks from a point (common.tsx Burst) */
  function Burst(layer, { at, x = 0.5, y = 0.5, count = 36, spread = 0.55, seed = 'b', colors = [C.red, C.coral, C.greenBright, '#ffffff', C.mustard] }) {
    const W = 1440, H = 2560;
    const box = el('div', 'position:absolute;inset:0;pointer-events:none;display:none', layer);
    const parts = [];
    for (let i = 0; i < count; i++) {
      const a = random(`${seed}a${i}`) * Math.PI * 2;
      const dd = 0.25 + random(`${seed}d${i}`) * 0.75;
      const s = 8 + random(`${seed}s${i}`) * 22;
      const petal = random(`${seed}p${i}`) > 0.45;
      const col = colors[i % colors.length];
      const d = el('div', `position:absolute;left:0;top:0;width:${s}px;height:${petal ? s * 0.62 : s * 0.4}px;border-radius:${petal ? '70% 10% 70% 10%' : '99px'};background:${col};box-shadow:${petal ? 'none' : `0 0 10px ${col}`}`, box);
      parts.push({ d, a, dd });
    }
    return (fAbs) => {
      const f = fAbs - at;
      if (f < 0 || f > 80) { box.style.display = 'none'; return; }
      box.style.display = 'block';
      const t = ease(f, 0, 70);
      for (const p of parts) {
        const dist = p.dd * Math.min(W, H) * spread * t;
        p.d.style.transform = `translate(${W * x + Math.cos(p.a) * dist}px, ${H * y + Math.sin(p.a) * dist + t * t * 120}px) rotate(${p.a * 57 + f * 9}deg)`;
        p.d.style.opacity = 1 - t;
      }
    };
  }

  /** A diagonal sheen across the frame (common.tsx Sweep) */
  function Sweep(layer, { at, dur = 50, strength = 0.18 }) {
    const d = el('div', 'position:absolute;inset:0;pointer-events:none;mix-blend-mode:screen;display:none', layer);
    return (f) => {
      if (f < at || f > at + dur) { d.style.display = 'none'; return; }
      const p = interp(f, [at, at + dur], [-30, 130]);
      d.style.display = 'block';
      d.style.background = `linear-gradient(105deg, transparent ${p - 12}%, rgba(255,255,255,${strength}) ${p}%, transparent ${p + 12}%)`;
    };
  }

  /** Krishnachura petals drifting down (v2/parts.tsx Petals) */
  function Petals(layer, { count = 28, from = 0, opacity = 1 }) {
    const W = 1440, H = 2560;
    const box = el('div', `position:absolute;inset:0;pointer-events:none;opacity:${opacity}`, layer);
    const ps = [];
    for (let i = 0; i < count; i++) {
      const size = 14 + random(`pz${i}`) * 26;
      const tint = random(`pc${i}`) > 0.3 ? C.red : C.coral;
      const d = el('div', `position:absolute;left:0;top:0;width:${size}px;height:${size * 0.62}px;background:${tint};border-radius:70% 10% 70% 10%;opacity:${0.55 + random(`po${i}`) * 0.4};filter:blur(${random(`pb${i}`) > 0.7 ? 2 : 0}px)`, box);
      ps.push({ d, x0: random(`px${i}`) * W, speed: 1.2 + random(`ps${i}`) * 2.2, py: random(`py${i}`), pr: random(`pr${i}`), i });
    }
    return (fAbs) => {
      const f = fAbs - from;
      for (const p of ps) {
        let y = (f * p.speed + p.py * (H + 200)) % (H + 200);
        if (y < 0) y += H + 200;
        y -= 100;
        const x = p.x0 + Math.sin((f + p.i * 40) / 45) * 40;
        p.d.style.transform = `translate(${x}px, ${y}px) rotate(${f * (1 + p.pr * 2) + p.i * 37}deg)`;
      }
    };
  }

  /** Text that types itself in, grapheme by grapheme, with a caret (common.tsx TypeOn) */
  function TypeOn(span, { text, start, every = 2, caret = C.greenBright }) {
    const g = graphemes(text);
    span.style.whiteSpace = 'nowrap';
    const t = el('span', '', span);
    const c = el('span', `display:inline-block;width:0.08em;height:0.95em;margin-left:0.06em;vertical-align:-0.1em;background:${caret};border-radius:3px`, span);
    let last = -1;
    return (f) => {
      const n = Math.max(0, Math.min(g.length, Math.floor((f - start) / every)));
      if (n !== last) { t.textContent = g.slice(0, n).join(''); last = n; }
      const done = n >= g.length;
      c.style.opacity = f < start ? 0 : done ? (Math.floor(f / 20) % 2 ? 0 : 1) : 1;
    };
  }

  const ICON = {
    web: '<circle cx="12" cy="12" r="9" /><path d="M3 12h18M12 3c2.6 2.8 2.6 15.2 0 18M12 3c-2.6 2.8-2.6 15.2 0 18" />',
  };
  /** The website pill: pops in, the address types itself, a glow and a sheen (common.tsx AddressPill) */
  function AddressPill(host, { text, at }) {
    const pill = el('div', 'position:relative;overflow:hidden;display:flex;align-items:center;gap:22px;padding:18px 36px 18px 26px;border-radius:999px;background:rgba(8,28,20,0.6);border:2px solid rgba(47,209,142,0.35);backdrop-filter:blur(18px)', host);
    el('div', `display:grid;place-items:center;width:64px;height:64px;border-radius:999px;background:${C.greenBright};color:#03170F`, pill,
      `<svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">${ICON.web}</svg>`);
    const span = el('span', `font-family:${FONT_LATIN};font-weight:600;font-size:56px;color:#fff;letter-spacing:0.005em`, pill);
    const type = TypeOn(span, { text, start: at + 10, every: 2 });
    const done = at + 10 + text.length * 2;
    const sweep = Sweep(pill, { at: done, dur: 40, strength: 0.25 });
    return (f) => {
      const pop = spring(f - at, { damping: 13, stiffness: 150 });
      const glow = interp(f, [done, done + 8, done + 40], [0, 1, 0.35]);
      pill.style.opacity = pop;
      pill.style.transform = `translateY(${(1 - pop) * 50}px) scale(${0.85 + pop * 0.15})`;
      pill.style.borderColor = `rgba(47,209,142,${0.35 + glow * 0.5})`;
      pill.style.boxShadow = `0 0 ${60 * glow}px rgba(47,209,142,${0.55 * glow}), inset 0 1px 0 rgba(255,255,255,0.2)`;
      type(f); sweep(f);
    };
  }

  /* ── typing, word by word (LaunchReveal.tsx plan/typing) ── */
  function plan(tokens, start) {
    let t = start;
    return tokens.map(([r, b]) => {
      const s = t, e = s + r.length * L.every;
      t = e + L.gap;
      return { r, b, s, e, c: e + L.gap - 4 };
    });
  }
  function typing(f, P) {
    let done = '';
    for (const w of P) {
      if (f >= w.c) { done += (done ? ' ' : '') + w.b; continue; }
      if (f < w.s) break;
      const n = Math.min(w.r.length, Math.floor((f - w.s) / L.every) + 1);
      const g = graphemes(w.b);
      const k = f >= w.e ? g.length : Math.max(1, Math.round((n / w.r.length) * g.length));
      return { done, live: g.slice(0, k).join(''), roman: w.r, n, picked: f >= w.e };
    }
    return { done, live: '', roman: '', n: 0, picked: false };
  }
  const endOf = (P) => P[P.length - 1].c;

  /** The Roman letters of the word being typed, in small key caps */
  function KeyStrip(box) {
    box.style.cssText += ';display:flex;gap:10px;justify-content:center;min-height:84px';
    let key = '';
    return (roman, n) => {
      const k = roman + '|' + n;
      if (k === key) return;
      key = k;
      box.innerHTML = Array.from(roman).slice(0, n).map((c, i) => {
        const last = i === n - 1;
        return `<div style="min-width:66px;height:80px;padding:0 12px;display:grid;place-items:center;border-radius:18px;background:${last ? 'rgba(47,209,142,0.32)' : 'rgba(255,255,255,0.09)'};border:2px solid rgba(255,255,255,0.2);box-shadow:0 ${last ? 2 : 6}px 0 rgba(0,0,0,0.45);transform:translateY(${last ? 4 : 0}px);font-family:${MONO};font-weight:700;font-size:42px;color:#fff">${c}</div>`;
      }).join('');
    };
  }

  /** Text being written: committed words, the word forming (underlined), a blinking caret */
  function Written(box, { ink, accent, size, align = 'left' }) {
    box.style.cssText += `;font-family:${FONT_BN};font-weight:500;font-size:${size}px;line-height:1.55;color:${ink};text-align:${align};word-break:keep-all`;
    const done = el('span', '', box);
    const live = el('span', `color:${accent};border-bottom:4px solid ${accent}`, box);
    const caret = el('span', `display:inline-block;width:4px;height:${size}px;margin-left:4px;vertical-align:-0.15em;background:${accent}`, box);
    return (d, l, f) => {
      const dt = d + (d && l ? ' ' : '');
      if (done.textContent !== dt) done.textContent = dt;
      if (live.textContent !== l) live.textContent = l;
      live.style.display = l ? 'inline' : 'none';
      caret.style.opacity = Math.floor(f / 18) % 2 ? 0.2 : 1;
    };
  }

  /** Ekushe's suggestion bar under the text */
  function Cands(box, { dark, logo }) {
    const bar = el('div', `display:inline-flex;align-items:center;gap:14px;padding:10px 12px;border-radius:24px;background:${dark ? 'rgba(30,36,34,0.96)' : 'rgba(255,255,255,0.98)'};border:2px solid ${dark ? 'rgba(255,255,255,0.12)' : 'rgba(0,0,0,0.1)'};box-shadow:0 18px 50px rgba(0,0,0,0.35)`, box);
    el('img', 'width:44px;height:44px;border-radius:10px;margin-left:6px', bar).src = logo;
    const pick = el('div', `display:flex;align-items:baseline;gap:10px;padding:8px 24px;border-radius:16px;background:${C.greenBright};color:#03170F`, bar);
    el('span', `font-family:${MONO};font-size:24px;opacity:0.6`, pick, '1');
    const word = el('span', `font-family:${FONT_BN};font-weight:600;font-size:50px`, pick);
    const spacer = el('div', 'height:96px;display:none', box);
    return (w, picked) => {
      bar.style.display = w ? 'inline-flex' : 'none';
      spacer.style.display = w ? 'none' : 'block';
      if (word.textContent !== w) word.textContent = w;
      pick.style.transform = `scale(${picked ? 1.06 : 1})`;
      pick.style.boxShadow = picked ? '0 0 36px rgba(47,209,142,0.6)' : 'none';
    };
  }

  const OS_ICON = {
    win: '<path fill="currentColor" d="M3 5.6l7.4-1v7H3zm8.4-1.2L21 3v8.6h-9.6zM3 12.6h7.4v7l-7.4-1zm8.4 0H21V21l-9.6-1.4z" />',
    mac: '<path fill="currentColor" d="M16.4 12.6c0-2.4 2-3.6 2.1-3.7-1.2-1.7-3-1.9-3.6-2-1.5-.2-3 .9-3.8.9-.8 0-2-.9-3.3-.9-1.7 0-3.3 1-4.2 2.5-1.8 3.1-.5 7.7 1.3 10.2.9 1.2 1.9 2.6 3.2 2.6 1.3-.1 1.8-.8 3.3-.8s2 .8 3.3.8c1.4 0 2.3-1.2 3.1-2.5 1-1.4 1.4-2.8 1.4-2.9 0 0-2.8-1.1-2.8-4.2zM14 5.3c.7-.9 1.2-2 1-3.3-1 0-2.3.7-3 1.6-.6.7-1.2 1.9-1 3.1 1.1.1 2.3-.6 3-1.4z" />',
    ios: '<rect x="6.5" y="2.5" width="11" height="19" rx="2.6" fill="none" stroke="currentColor" stroke-width="1.7" /><rect x="10" y="4.3" width="4" height="1.2" rx="0.6" fill="currentColor" />',
    linux: '<path fill="currentColor" d="M12 2c-2.2 0-3.4 1.9-3.4 4.3 0 1.4.3 2.3-.6 3.7-1 1.6-2.5 3.6-2.5 6.1 0 .6.1 1.1.2 1.6-.9.4-1.7.9-1.7 1.7 0 1.1 1.7 1.3 3.2 1.7 1.2.3 2 .9 3 .9.8 0 1.4-.4 1.8-.9.4.5 1 .9 1.8.9 1 0 1.8-.6 3-.9 1.5-.4 3.2-.6 3.2-1.7 0-.8-.8-1.3-1.7-1.7.1-.5.2-1 .2-1.6 0-2.5-1.5-4.5-2.5-6.1-.9-1.4-.6-2.3-.6-3.7C15.4 3.9 14.2 2 12 2z" />',
    android: '<path fill="currentColor" d="M7.2 8.6h9.6c.5 0 .9.4.9.9v7.2c0 .5-.4.9-.9.9h-.9v2.5a1.1 1.1 0 01-2.2 0v-2.5h-3.4v2.5a1.1 1.1 0 01-2.2 0v-2.5h-.9a.9.9 0 01-.9-.9V9.5c0-.5.4-.9.9-.9zM4.4 9a1.1 1.1 0 011.1 1.1v5a1.1 1.1 0 01-2.2 0v-5C3.3 9.5 3.8 9 4.4 9zm15.2 0a1.1 1.1 0 011.1 1.1v5a1.1 1.1 0 01-2.2 0v-5c0-.6.5-1.1 1.1-1.1zM15.6 3.1l-.9 1.5A5.3 5.3 0 0117.6 7.8H6.4a5.3 5.3 0 012.9-3.2l-.9-1.5a.3.3 0 01.5-.3l.9 1.6a5.8 5.8 0 014.4 0l.9-1.6a.3.3 0 01.5.3zM9.6 5.7a.6.6 0 100 1.2.6.6 0 000-1.2zm4.8 0a.6.6 0 100 1.2.6.6 0 000-1.2z" />',
  };
  /** Fill every <span data-os="kind" data-size="N"> in a scene with its icon */
  function icons(root) {
    root.querySelectorAll('[data-os]').forEach((s) => {
      const n = s.getAttribute('data-size') || 40;
      s.innerHTML = `<svg width="${n}" height="${n}" viewBox="0 0 24 24" style="display:block">${OS_ICON[s.getAttribute('data-os')]}</svg>`;
    });
  }

  window.EK = { FPS, C, FONT_BN, FONT_LATIN, MONO, L, random, spring, sp, interp, ease, graphemes, mix, el, tick, Burst, Sweep, Petals, TypeOn, AddressPill, plan, typing, endOf, KeyStrip, Written, Cands, icons };
})();
