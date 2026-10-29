/* The scenes' wiring. Each compositions/*.html holds its words (titles, time and place) and calls one
 * of these with them; everything moves from the scene's frame number. */
(function () {
  const F = window.FILM;
  const { el } = F;

  /** The film's frame size, from the root composition (1920×1080 or 1080×1920) */
  function frameSize() {
    const main = document.querySelector('[data-composition-id="main"]');
    return { W: +(main && main.dataset.width) || 1920, H: +(main && main.dataset.height) || 1080 };
  }

  /** A device typing the next line of the chat. o: { id, kind, ground, darkTitles, quiet, title, en,
   * clock, history: [earlier lines], trace, len (frames) } */
  function deviceScene(o) {
    const { W, H } = frameSize();
    const root = document.getElementById(`${o.id}-root`);
    el('div', `position:absolute;inset:0;background:${o.ground}`, root);
    const cam = el('div', 'position:absolute;left:0;top:0;width:100%;height:100%;transform-origin:0 0', root);
    const D = o.kind === 'win' || o.kind === 'mac' ? F.Laptop(cam, o.kind) : o.kind === 'phone' ? F.Phone(cam, { clock: o.clock }) : F.Tablet(cam, { clock: o.clock });
    const pl = F.place(W, H, D.w, D.h);
    D.dev.style.transformOrigin = '0 0';
    D.dev.style.left = `${pl.x}px`;
    D.dev.style.top = `${pl.y}px`;
    D.dev.style.transform = `scale(${pl.k})`;
    if (D.setClock) D.setClock(o.clock);

    const big = o.kind === 'phone' ? 21 : 23;
    (o.history || []).forEach((line) => F.Bubble(D.chat, line, { mine: true, dark: D.dark, size: big, maxw: o.kind === 'phone' ? 82 : 66 }));
    const field = F.Field(D.field, { size: o.kind === 'phone' ? 21 : 24, ink: D.dark ? '#EEE' : '#1D1D1F', accent: D.dark ? F.BLUE_DARK : F.BLUE });

    let bar;
    if (o.kind === 'win' || o.kind === 'mac') bar = F.Bar(D.dev, o.kind, D.barAt.x, D.barAt.y);
    else if (o.kind === 'phone') bar = F.Bar(D.dev, 'strip', 14, D.stripY, 492);
    else bar = F.Bar(D.dev, 'strip', 20, D.stripY, 1360);
    const c = bar.chip;
    const chip = { x: pl.x + pl.k * c.x, y: pl.y + pl.k * c.y, w: pl.k * c.w, h: pl.k * c.h };

    const P = F.plan(F.TRACE()[o.trace], -2);
    // while typing: close on the text box and the bar, so the letters read at about 38 px
    const fontDev = o.kind === 'phone' ? 21 : 24;
    const fp = o.focus;
    const work = { s: Math.max(1.15, 38 / (fontDev * pl.k)), p: { x: pl.x + pl.k * fp.x, y: pl.y + pl.k * fp.y }, q: { x: W / 2, y: H * (H > W ? 0.56 : 0.54) } };
    const wideAt = F.endOf(P) + 12, zoomAt = o.len - 21;
    const camera = F.Camera(cam, W, H, { chipIn: chip, chipOut: chip, work, wideAt, zoomAt });
    const titles = F.Titles(root, W, H, { quiet: o.quiet, title: o.title, en: o.en, dark: o.darkTitles, at: wideAt + 10, outAt: zoomAt - 2 });

    const tl = gsap.timeline({ paused: true });
    F.tick(tl, o.len, (f) => {
      const t = F.typing(f, P);
      field(t, f);
      bar.update(t.cands, t.roman);
      camera(f);
      titles(f);
    });
    return tl;
  }

  /** Ma in Dhaka: the four lines arrive, she answers, and the send button opens into the end card */
  function momScene(o) {
    const { W, H } = frameSize();
    const root = document.getElementById(`${o.id}-root`);
    el('div', `position:absolute;inset:0;background:${o.ground}`, root);
    const cam = el('div', 'position:absolute;left:0;top:0;width:100%;height:100%;transform-origin:0 0', root);
    const D = F.Phone(cam, { name: o.name, letter: o.letter, avatar: '#0B8A5C', clock: o.clock });
    const pl = F.place(W, H, D.w, D.h);
    Object.assign(D.dev.style, { transformOrigin: '0 0', left: `${pl.x}px`, top: `${pl.y}px`, transform: `scale(${pl.k})` });
    const got = o.lines.map((line) => F.Bubble(D.chat, line, { mine: false, size: 21, maxw: 84 }));
    const reply = F.Bubble(D.chat, '', { mine: true, size: 21, maxw: 84 });
    const field = F.Field(D.field, { size: 21, ink: '#1D1D1F', accent: F.BLUE });
    const bar = F.Bar(D.dev, 'strip', 14, D.stripY, 492);
    const s = D.sendAt;
    const send = { x: pl.x + pl.k * s.x, y: pl.y + pl.k * s.y, w: pl.k * s.w, h: pl.k * s.h };

    const P = F.plan(F.TRACE()[o.trace], o.typeAt);
    const sentAt = F.endOf(P) + 12;
    reply.textContent = F.lineOf(P);
    const fp = o.focus;
    const work = { s: Math.max(1.15, 38 / (21 * pl.k)), p: { x: pl.x + pl.k * fp.x, y: pl.y + pl.k * fp.y }, q: { x: W / 2, y: H * 0.5 } };
    const wideAt = sentAt + 20, zoomAt = o.len - 21;
    const camera = F.Camera(cam, W, H, { chipIn: send, chipOut: send, work, wideAt, zoomAt });
    const titles = F.Titles(root, W, H, { quiet: o.quiet, title: o.title, en: o.en, dark: o.darkTitles, at: wideAt + 10, outAt: zoomAt - 2 });

    const tl = gsap.timeline({ paused: true });
    F.tick(tl, o.len, (f) => {
      got.forEach((b, i) => {
        const p = F.expoOut(F.clamp01((f - (o.arriveAt + i * 12)) / 10));
        b.style.opacity = p;
        b.style.transform = `translateY(${(1 - p) * 18}px)`;
        b.style.display = f >= o.arriveAt + i * 12 ? 'block' : 'none';
      });
      const sent = f >= sentAt;
      const t = sent ? { done: [], live: '', cands: [], roman: '' } : f < o.typeAt ? { done: [], live: '', cands: [], roman: '' } : F.typing(f, P);
      if (!sent && t.end) { t.done = t.done.concat([t.live]); t.live = ''; }
      field(t, f);
      bar.update(sent ? [] : t.cands, t.roman);
      const p = F.expoOut(F.clamp01((f - sentAt) / 10));
      reply.style.display = sent ? 'block' : 'none';
      reply.style.opacity = p;
      reply.style.transform = `translateY(${(1 - p) * 18}px)`;
      camera(f);
      titles(f);
    });
    return tl;
  }

  F.frameSize = frameSize;
  F.deviceScene = deviceScene;
  F.momScene = momScene;
})();
