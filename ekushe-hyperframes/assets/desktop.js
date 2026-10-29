/* The welcome typed on a computer: a Windows, macOS or Linux desktop drawn in HTML, the text
 * typing word by word with Ekushe's suggestion bar, the system bar's EN → এ badge, and the key caps
 * underneath (LaunchReveal.tsx Desktop + ComputerScene). Used by compositions/win.html, mac.html, linux.html. */
(function () {
  const { C, FONT_BN, FONT_LATIN, el } = EK;
  const CLOCK = { win: ['10:30 AM', '30-10-2026'], mac: 'Fri 30 Oct 10:30', linux: 'Oct 30  10:30' };
  const icon = (k, n) => `<span data-os="${k}" data-size="${n}"></span>`;

  function badge(parent, dark, size) {
    const b = el('div', `display:grid;place-items:center;min-width:${size * 1.25}px;height:${size}px;padding:0 8px;border-radius:8px;font-weight:700;font-size:${size * 0.62}px`, parent);
    return (on) => {
      b.textContent = on ? 'এ' : 'EN';
      b.style.background = on ? C.greenBright : 'transparent';
      b.style.border = on ? 'none' : `2px solid ${dark ? 'rgba(255,255,255,0.7)' : 'rgba(0,0,0,0.6)'}`;
      b.style.color = on ? '#03170F' : dark ? '#fff' : '#111';
      b.style.fontFamily = on ? FONT_BN : FONT_LATIN;
      b.style.boxShadow = on ? '0 0 24px rgba(47,209,142,0.7)' : 'none';
    };
  }

  /** Build the desktop into `host`; returns update(f, typingState, on, glow, note) */
  function Desktop(host, os) {
    const W = 1280, H = 1240;
    const dark = os !== 'mac';
    const ink = dark ? '#F2F5F3' : '#1B1F1D';
    const accent = dark ? C.greenBright : '#0B8A5C';
    const wall = os === 'win'
      ? 'radial-gradient(90% 70% at 30% 30%, #2c6bd6 0%, #0d3a8f 45%, #06173d 100%)'
      : os === 'mac'
        ? 'linear-gradient(160deg, #f6a35b 0%, #e2546e 38%, #6b3fa0 75%, #2a2160 100%)'
        : 'radial-gradient(90% 70% at 70% 20%, #4a2a6e 0%, #241a44 45%, #0e0c1f 100%)';
    const win = { win: { x: 70, y: 90, w: W - 140, h: H - 270 }, mac: { x: 70, y: 130, w: W - 140, h: H - 330 }, linux: { x: 70, y: 120, w: W - 140, h: H - 210 } }[os];
    const title = os === 'win' ? 'শুভেচ্ছা.txt – Notepad' : os === 'mac' ? 'শুভেচ্ছা' : 'শুভেচ্ছা — Text Editor';
    const scr = el('div', `position:relative;width:${W}px;height:${H}px;border-radius:44px;overflow:hidden;background:${wall}`, host);
    const badges = [];

    if (os === 'mac') {
      const bar = el('div', `position:absolute;top:0;left:0;right:0;height:60px;display:flex;align-items:center;gap:34px;padding:0 30px;background:rgba(255,255,255,0.32);backdrop-filter:blur(20px);color:#111;font-family:${FONT_LATIN};font-weight:600;font-size:28px`, scr,
        `${icon('mac', 32)}<span style="font-weight:800">TextEdit</span><span style="font-weight:500">File</span><span style="font-weight:500">Edit</span><span style="font-weight:500">Format</span><div style="flex:1"></div>`);
      badges.push(badge(bar, false, 40));
      el('span', 'font-weight:500', bar, CLOCK.mac);
    }
    if (os === 'linux') {
      const bar = el('div', `position:absolute;top:0;left:0;right:0;height:60px;display:flex;align-items:center;padding:0 26px;background:#0b0b0d;color:#fff;font-family:${FONT_LATIN};font-weight:600;font-size:28px`, scr,
        `<div style="display:flex;gap:8px">${[1, 0, 0].map((a) => `<div style="width:${a ? 44 : 12}px;height:12px;border-radius:9px;background:${a ? '#fff' : 'rgba(255,255,255,0.45)'}"></div>`).join('')}</div><div style="flex:1;text-align:center;white-space:pre">${CLOCK.linux}</div>`);
      const right = el('div', 'display:flex;align-items:center;gap:20px', bar);
      badges.push(badge(right, true, 40));
      right.insertAdjacentHTML('beforeend', [0, 1, 2].map((i) => `<div style="width:24px;height:24px;border-radius:${i === 2 ? 6 : 99}px;border:3px solid #fff"></div>`).join(''));
    }

    const w = el('div', `position:absolute;left:${win.x}px;top:${win.y}px;width:${win.w}px;height:${win.h}px;border-radius:${os === 'win' ? 18 : 26}px;overflow:hidden;background:${dark ? (os === 'win' ? '#202020' : '#242424') : '#FFFFFF'};box-shadow:0 30px 80px rgba(0,0,0,0.55);border:1.5px solid ${dark ? 'rgba(255,255,255,0.1)' : 'rgba(0,0,0,0.12)'}`, scr);
    let head = '';
    if (os === 'mac') head += `<div style="display:flex;gap:14px">${['#FF5F57', '#FEBC2E', '#28C840'].map((c) => `<div style="width:24px;height:24px;border-radius:99px;background:${c}"></div>`).join('')}</div>`;
    if (os === 'win') head += '<div style="width:30px;height:34px;border-radius:4px;background:#4CC2FF"></div>';
    head += `<div style="flex:1;text-align:${os === 'win' ? 'left' : 'center'};margin-right:${os === 'mac' ? 100 : 0}px">${title}</div>`;
    if (os === 'win') head += `<div style="display:flex;gap:40px;font-family:${FONT_LATIN};font-size:30px;color:#ccc"><span>—</span><span>▢</span><span>✕</span></div>`;
    if (os === 'linux') head += `<div style="width:40px;height:40px;border-radius:99px;background:rgba(255,255,255,0.1);display:grid;place-items:center;color:#ddd;font-family:${FONT_LATIN};font-size:24px">✕</div>`;
    el('div', `height:76px;display:flex;align-items:center;padding:0 26px;gap:18px;background:${os === 'win' ? '#1b1b1b' : os === 'mac' ? '#ECECEC' : '#303030'};color:${dark ? '#ddd' : '#333'};font-family:${FONT_BN};font-weight:500;font-size:30px;border-bottom:1.5px solid ${dark ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.08)'}`, w, head);
    if (os === 'win') el('div', `display:flex;gap:36px;padding:10px 30px;font-family:${FONT_LATIN};font-size:26px;color:#bbb;background:#202020;border-bottom:1.5px solid rgba(255,255,255,0.06)`, w, '<span>File</span><span>Edit</span><span>View</span>');
    const body = el('div', 'padding:50px 56px', w);
    const written = EK.Written(el('div', '', body), { ink, accent, size: 74 });
    const cands = EK.Cands(el('div', 'margin-top:24px', body), { style: os === 'win' ? 'win' : os === 'mac' ? 'mac' : 'ibus' });
    let note = null;
    if (os === 'linux') note = el('div', `position:absolute;left:60px;top:150px;padding:10px 22px;border-radius:14px;background:rgba(20,20,22,0.96);border:1.5px solid rgba(255,255,255,0.15);font-family:${FONT_BN};font-weight:600;font-size:34px;color:#fff`, w, 'এ · বাংলা');

    if (os === 'win') {
      const tb = el('div', `position:absolute;left:0;right:0;bottom:0;height:96px;display:flex;align-items:center;padding:0 26px;background:rgba(24,24,28,0.86);backdrop-filter:blur(20px);color:#fff;font-family:${FONT_LATIN}`, scr,
        `<div style="flex:1"></div><div style="display:flex;gap:22px;align-items:center"><div style="color:#4CC2FF">${icon('win', 46)}</div>${['#ffd34e', '#4CC2FF', '#7ee787'].map((c) => `<div style="width:46px;height:46px;border-radius:10px;background:${c};opacity:0.85"></div>`).join('')}</div>`);
      const tray = el('div', 'flex:1;display:flex;justify-content:flex-end;align-items:center;gap:20px', tb, '<img src="assets/img/logo_64.png" style="width:40px;height:40px;border-radius:8px" />');
      badges.push(badge(tray, true, 42));
      el('div', 'text-align:right;font-size:22px;line-height:1.25', tray, `${CLOCK.win[0]}<br />${CLOCK.win[1]}`);
    }
    if (os === 'mac') {
      el('div', 'position:absolute;left:50%;bottom:26px;transform:translateX(-50%);display:flex;gap:18px;padding:14px 20px;border-radius:30px;background:rgba(255,255,255,0.3);backdrop-filter:blur(20px);border:1.5px solid rgba(255,255,255,0.4)', scr,
        ['#3a8ef6', '#f5f5f5', '#34c759', '#ff9f0a'].map((c) => `<div style="width:92px;height:92px;border-radius:22px;background:${c}"></div>`).join('') +
        '<img src="assets/img/logo_128.png" style="width:92px;height:92px;border-radius:22px" />');
    }
    EK.icons(scr);

    return (f, t, on, glow, noteV) => {
      scr.style.boxShadow = `0 50px 140px -30px rgba(0,0,0,0.9), 0 0 ${120 * glow}px rgba(47,209,142,${0.45 * glow})`;
      scr.style.border = `3px solid rgba(255,255,255,${0.12 + glow * 0.3})`;
      badges.forEach((b) => b(on));
      written(t.done, t.live, f);
      cands(t.cands, t.picked, t.roman.slice(0, t.n));
      if (note) {
        note.style.display = noteV > 0 ? 'block' : 'none';
        note.style.opacity = noteV;
        note.style.transform = `translateY(${(1 - noteV) * 10}px)`;
      }
    };
  }

  /** Wire up a computer scene: `ids` prefix (win/mac/linux), its timeline, length in frames */
  function computerScene(os, tl, len) {
    const $ = (id) => document.getElementById(`${os}-${id}`);
    const cfg = EK.L[os];
    const P = EK.plan(cfg.tokens, cfg.start);
    const fin = EK.endOf(P);
    const s = (fr) => fr / 60;
    EK.icons($('head'));
    // the heading, the screen, the "new" tag, and leaving
    tl.fromTo($('head'), { opacity: 0, y: 60 }, { opacity: 1, y: 0, duration: 2, ease: EK.sp({ damping: 16, stiffness: 90 }) }, s(4));
    tl.fromTo($('screen'), { opacity: 0, y: 90, scale: 0.94 }, { opacity: 1, y: 0, scale: 1, duration: 2, ease: EK.sp({ damping: 17, stiffness: 85 }) }, s(22));
    if ($('tag')) tl.fromTo($('tag'), { scale: 0, rotation: 24 }, { scale: 1, rotation: -6, duration: 2, ease: EK.sp({ damping: 9, stiffness: 180 }) }, s(40));
    tl.fromTo($('line'), { opacity: 0 }, { opacity: 1, duration: s(25), ease: 'power2.out' }, s(fin));
    tl.to($('out'), { opacity: 0, y: -50, filter: 'blur(10px)', duration: s(24), ease: 'power2.out' }, s(len - 24));

    const update = Desktop($('screen'), os);
    const keys = EK.KeyStrip($('keys'));
    const keysEl = $('keys');
    const fx = $('fx');
    const burst = EK.Burst(fx, { at: fin, y: 0.36, count: 34, seed: `c${os}`, spread: 0.5, colors: [C.greenBright, '#ffffff', C.mustard, C.red] });
    const sweep = EK.Sweep(fx, { at: fin + 4, dur: 46, strength: 0.12 });
    const hint = $('hint');
    EK.tick(tl, len, (f) => {
      const t = EK.typing(f, P, (window.EK_TRACE || {})[os]);
      const glow = EK.interp(f, [fin, fin + 10, fin + 60], [0, 1, 0.45]);
      const on = f >= cfg.start - 40;
      const note = Math.min(EK.ease(f, cfg.start - 40, cfg.start - 28), 1 - EK.ease(f, cfg.start + 4, cfg.start + 16));
      update(f, t, on, glow, os === 'linux' ? note : 0);
      keys(t.roman, t.n);
      keysEl.style.opacity = t.roman ? 1 : 0;
      if (hint) hint.style.opacity = note;
      burst(f); sweep(f);
    });
  }

  EK.computerScene = computerScene;
})();
