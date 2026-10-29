/* এক বার্তা, সব পর্দা — shared pieces for every scene.
 *
 * Every scene is drawn from its frame number (30 fps) in one `tick` tween, so any frame renders on
 * its own: the typing, the suggestion bars (the engine's real output, assets/trace.js), and the
 * camera that opens out of a blue chip and dives into the next one. Layout is fixed numbers, never
 * measured, so every worker draws the same frame.
 */
(function () {
  const FPS = 30;
  const BN = "'Hind Siliguri', sans-serif";
  const LAT = "'Inter Tight', 'Hind Siliguri', sans-serif";
  const BLUE = '#3C7BF0', BLUE_DARK = '#8AB4F8';

  const el = (tag, css, parent, html) => {
    const e = document.createElement(tag);
    if (css) e.style.cssText = css;
    if (html !== undefined) e.innerHTML = html;
    if (parent) parent.appendChild(e);
    return e;
  };
  const clamp01 = (t) => Math.max(0, Math.min(1, t));
  const lerp = (a, b, t) => a + (b - a) * t;
  const expoOut = (t) => (t >= 1 ? 1 : 1 - Math.pow(2, -10 * t));
  const p3in = (t) => t * t * t;
  const p2out = (t) => 1 - (1 - t) * (1 - t);
  const seg = new Intl.Segmenter('bn', { granularity: 'grapheme' });
  const graphemes = (s) => Array.from(seg.segment(s), (g) => g.segment);
  const isEn = (w) => /^[\x00-\x7F]+$/.test(w);

  /** One tween that calls fn(frame) every frame of the scene, and once now. */
  function tick(tl, frames, fn) {
    const s = { f: 0 };
    tl.to(s, { f: frames, duration: frames / FPS, ease: 'none', onUpdate: () => fn(s.f) }, 0);
    fn(0);
  }

  /* ── typing ── 3 frames a letter, 6 between words; the word is committed 3 frames after its last
   * letter. Returns what is on screen at frame f. */
  const EVERY = 3, GAP = 6;
  function plan(words, start) {
    let t = start;
    return words.map((w) => {
      const s = t, e = s + w.r.length * EVERY;
      t = e + GAP;
      return { ...w, s, e, c: e + 3 };
    });
  }
  function typing(f, P) {
    const done = [];
    for (let i = 0; i < P.length; i++) {
      const w = P[i];
      const last = i === P.length - 1;
      // the last word stays as "being typed" (its chip is what the camera dives into)
      if (f >= w.c && !last) { done.push(w.b); continue; }
      if (f < w.s) return { done, live: '', cands: [], roman: '', n: 0, end: false };
      const n = Math.min(w.r.length, Math.floor((f - w.s) / EVERY) + 1);
      const [live, cands] = w.steps[n - 1];
      return { done, live, cands, roman: w.r.slice(0, n), n, end: last && f >= w.e };
    }
    return { done, live: '', cands: [], roman: '', n: 0, end: true };
  }
  const endOf = (P) => P[P.length - 1].e;
  const lineOf = (P) => P.map((w) => w.b).join(' ');

  /* ── the suggestion bars, as each system's beta.5 app draws them ──
   * The first chip (the chosen word, blue) has a fixed width, so the camera knows where it is. */
  const BAR = {
    win: { chip: 'rgba(255,255,255,0.09)', sel: BLUE_DARK, onSel: '#202124', ink: '#E8EAED', muted: '#9AA0A6', card: 'rgba(32,33,36,0.86)', h: 46, font: 26, cw: 190, pad: 10, head: 26, num: true, radius: 16 },
    mac: { chip: 'rgba(255,255,255,0.85)', sel: BLUE, onSel: '#FFFFFF', ink: '#1F1F1F', muted: '#5F6368', card: 'rgba(238,240,244,0.88)', h: 46, font: 26, cw: 170, pad: 9, head: 0, num: false, radius: 16 },
    strip: { chip: '#F4F5F8', sel: BLUE, onSel: '#FFFFFF', ink: '#1F1F1F', muted: '#5F6368', card: '#E8EAEF', h: 38, font: 21, cw: 132, pad: 7, head: 0, num: false, radius: 0 },
  };
  /** A bar at (x, y) in its device; returns { update(cands, roman), chip: {x, y, w, h} } */
  function Bar(parent, kind, x, y, width) {
    const B = BAR[kind];
    const wrap = el('div', `position:absolute;left:${x}px;top:${y}px;${width ? `width:${width}px;` : ''}padding:${B.pad}px;border-radius:${B.radius}px;background:${B.card};${kind === 'strip' ? '' : `border:1px solid ${kind === 'win' ? 'rgba(255,255,255,0.10)' : 'rgba(255,255,255,0.9)'};box-shadow:0 16px 40px rgba(0,0,0,0.28);`}display:flex;flex-direction:column;gap:6px;overflow:hidden`, parent);
    let head = null;
    if (B.head) {
      head = el('div', `height:${B.head}px;display:flex;align-items:center;gap:8px;padding:0 4px`, wrap);
      head.innerHTML = `<span class="r" style="flex:1;font-family:${LAT};font-size:17px;color:${B.muted}"></span>` +
        ['যুক্তবর্ণ লুকান', 'সহায়িকা'].map((t) => `<span style="padding:1px 9px;border-radius:8px;background:${B.chip};font-family:${BN};font-weight:600;font-size:14px;color:${B.muted}">${t}</span>`).join('');
    }
    const row = el('div', `display:flex;gap:${kind === 'strip' ? 6 : 7}px;height:${B.h}px`, wrap);
    let key = null;
    const update = (cands, roman) => {
      const show = cands && cands.length;
      wrap.style.opacity = show ? 1 : 0;
      if (head) head.querySelector('.r').textContent = roman || '';
      const k = (cands || []).join('|');
      if (k === key) return;
      key = k;
      row.innerHTML = (cands || []).map((w, i) => {
        const sel = i === 0, en = isEn(w);
        const soft = sel ? (kind === 'win' ? 'rgba(32,33,36,0.66)' : 'rgba(255,255,255,0.78)') : B.muted;
        const num = B.num ? `<span style="font-family:${LAT};font-size:15px;color:${soft}">${i + 1}</span>` : '';
        const tag = en ? `<span style="font-family:${LAT};font-weight:700;font-size:11px;color:${soft}">EN</span>` : '';
        const width = sel ? `width:${B.cw}px;justify-content:center;` : '';
        return `<div style="flex:none;display:flex;align-items:center;gap:6px;${width}height:${B.h}px;padding:0 ${kind === 'strip' ? 11 : 14}px;border-radius:${kind === 'strip' ? 9 : 11}px;background:${sel ? B.sel : B.chip};box-sizing:border-box">${num}<span style="font-family:${en ? LAT : BN};font-weight:${en ? 500 : 600};font-size:${en ? B.font - 4 : B.font}px;color:${sel ? B.onSel : B.ink};white-space:nowrap">${w}</span>${tag}</div>`;
      }).join('');
    };
    const chip = { x: x + B.pad, y: y + B.pad + (B.head ? B.head + 6 : 0), w: B.cw, h: B.h };
    return { update, chip, el: wrap };
  }

  /* ── the chat with Ma: earlier lines as sent bubbles, the line being typed in the box ── */
  function Bubble(parent, text, { mine = true, dark = false, size = 24, maxw = 70 } = {}) {
    const bg = mine ? (dark ? '#2B5FBF' : BLUE) : dark ? '#2F3236' : '#E9EBEE';
    const ink = mine ? '#fff' : dark ? '#E8EAED' : '#1D1D1F';
    return el('div', `align-self:${mine ? 'flex-end' : 'flex-start'};max-width:${maxw}%;padding:${size * 0.42}px ${size * 0.7}px;border-radius:${size * 0.9}px;background:${bg};color:${ink};font-family:${BN};font-size:${size}px;line-height:1.35`, parent, text);
  }
  /** The text box: committed words, the word being typed (underlined), a caret */
  function Field(parent, { size = 26, ink = '#1D1D1F', accent = BLUE }) {
    const done = el('span', '', parent);
    const live = el('span', `color:${accent};border-bottom:2px solid ${accent}`, parent);
    const caret = el('span', `display:inline-block;width:2px;height:${size}px;margin-left:2px;vertical-align:-0.18em;background:${accent}`, parent);
    parent.style.cssText += `;font-family:${BN};font-size:${size}px;color:${ink};white-space:nowrap;overflow:hidden`;
    return (t, f) => {
      const d = t.done.join(' ') + (t.done.length && t.live ? ' ' : '');
      if (done.textContent !== d) done.textContent = d;
      if (live.textContent !== t.live) live.textContent = t.live;
      caret.style.opacity = Math.floor(f / 15) % 2 ? 0.25 : 1;
    };
  }

  /* ── the camera ── four framings, in cam coordinates: out of a blue chip (it covers the frame),
   * a close framing on the text box and bar while the line is typed (text about 38 px), the whole
   * device while the title is read, and into the next blue chip. A framing is { s, p: the cam point,
   * q: where on screen it sits }; moves interpolate the scale in log space. */
  function Camera(cam, W, H, { chipIn, chipOut, work, pull = 36, wideAt, wide = 24, zoomAt, zoom = 21 }) {
    const center = (c) => ({ x: c.x + c.w / 2, y: c.y + c.h / 2 });
    const coverOf = (c) => ({ s: 1.6 * Math.max(W / c.w, H / c.h), p: center(c), q: { x: W / 2, y: H / 2 } });
    const FULL = { s: 1, p: { x: W / 2, y: H / 2 }, q: { x: W / 2, y: H / 2 } };
    const A = chipIn ? coverOf(chipIn) : FULL, B = coverOf(chipOut);
    // Move between two framings about one anchor point (the thing zoomed into or out of), so it
    // travels in a straight line on screen while the scale changes evenly
    const at = (c, p) => ({ s: c.s, p, q: { x: c.q.x + c.s * (p.x - c.p.x), y: c.q.y + c.s * (p.y - c.p.y) } });
    const mixF = (a, b, t, anchor) => {
      const A2 = at(a, anchor), B2 = at(b, anchor);
      return { s: Math.exp(lerp(Math.log(A2.s), Math.log(B2.s), t)), p: anchor, q: { x: lerp(A2.q.x, B2.q.x, t), y: lerp(A2.q.y, B2.q.y, t) } };
    };
    const io = (t) => (t < 0.5 ? 2 * t * t : 1 - Math.pow(-2 * t + 2, 2) / 2);
    return (f) => {
      let c;
      if (f < pull) c = mixF(A, work, expoOut(f / pull), A.p);
      else if (f < wideAt) c = work;
      else if (f < wideAt + wide) c = mixF(work, FULL, io((f - wideAt) / wide), work.p);
      else if (f < zoomAt) c = FULL;
      else c = mixF(FULL, B, p3in(clamp01((f - zoomAt) / zoom)), B.p);
      cam.style.transform = `translate(${c.q.x - c.s * c.p.x}px, ${c.q.y - c.s * c.p.y}px) scale(${c.s})`;
    };
  }

  /* ── titles: the time and place (quiet), the feature (large), its English line ──
   * Each word blurs in (6 px → 0, 0.13 s, 0.07 s apart) and then holds still. */
  function Titles(root, W, H, { quiet, title, en, dark, at = 40, quietAt, outAt = 1e9 }) {
    const vertical = H > W;
    const ink = dark ? '#FFFFFF' : '#1D1D1F';
    const soft = dark ? 'rgba(255,255,255,0.72)' : 'rgba(29,29,31,0.66)';
    const box = el('div', `position:absolute;left:${vertical ? 60 : 0}px;right:${vertical ? 60 : 0}px;top:${vertical ? 170 : 46}px;text-align:center`, root);
    const q = el('div', `font-family:${BN};font-weight:600;font-size:${vertical ? 38 : 30}px;color:${soft};letter-spacing:0.01em`, box, quiet);
    const t = el('div', `margin-top:${vertical ? 14 : 4}px;font-family:${BN};font-weight:700;font-size:${vertical ? 76 : 64}px;line-height:1.25;color:${ink}`, box);
    const words = title.split(' ').map((w) => el('span', 'display:inline-block;margin:0 0.13em', t, w));
    const e = el('div', `margin-top:${vertical ? 12 : 2}px;font-family:${LAT};font-weight:500;font-size:${vertical ? 34 : 30}px;color:${soft}`, box, en);
    return (f) => {
      const show = (el2, a) => {
        const p = clamp01((f - a) / 4);
        el2.style.opacity = p;
        el2.style.filter = p < 1 ? `blur(${(1 - p) * 6}px)` : 'none';
      };
      show(q, quietAt === undefined ? at - 6 : quietAt);
      words.forEach((w, i) => show(w, at + i * 2));
      show(e, at + words.length * 2 + 4);
      // leave as the camera dives: blur out and fade in 0.2 s
      const o = clamp01((f - outAt) / 6);
      box.style.opacity = 1 - o;
      box.style.filter = o > 0 ? `blur(${o * 8}px)` : 'none';
    };
  }

  /** Where the device sits and how large: below the titles, as big as the frame allows */
  function place(W, H, dw, dh) {
    const vertical = H > W;
    const top = vertical ? 560 : 236, bottom = vertical ? 1840 : 1056;
    const k = Math.min((bottom - top) / dh, (W - (vertical ? 70 : 220)) / dw);
    return { k, x: (W - dw * k) / 2, y: top + (bottom - top - dh * k) / 2 };
  }

  const TRACE = () => window.FILM_TRACE || {};
  window.FILM = { FPS, BN, LAT, BLUE, BLUE_DARK, el, clamp01, lerp, expoOut, p2out, graphemes, tick, plan, typing, endOf, lineOf, Bar, Bubble, Field, Camera, Titles, place, TRACE };
})();
