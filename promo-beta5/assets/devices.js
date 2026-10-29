/* The devices of the film, drawn in HTML at fixed sizes (the scene scales the whole device):
 * a Windows laptop, a MacBook, an Android phone, a tablet on a keyboard cover. Each holds the same
 * chat with Ma. Returns the pieces a scene fills: the chat column, the text box, where the bar goes. */
(function () {
  const { el, BN, LAT } = FILM;

  /** A laptop, 1300 × 840: a screen in its bezel on a base. os: 'win' | 'mac' */
  function Laptop(parent, os) {
    const mac = os === 'mac';
    const dev = el('div', 'position:absolute;left:0;top:0;width:1300px;height:840px', parent);
    const lid = el('div', `position:absolute;left:0;top:0;width:1300px;height:800px;border-radius:30px;background:${mac ? '#1B1B1D' : '#0D0D0F'};box-shadow:0 30px 70px rgba(0,0,0,0.35);border:2px solid ${mac ? '#4A4A4E' : '#2A2A2E'}`, dev);
    el('div', `position:absolute;left:-60px;top:798px;width:1420px;height:34px;border-radius:0 0 26px 26px;background:${mac ? '#D5D7DB' : '#BFC3C9'};box-shadow:0 14px 30px rgba(0,0,0,0.3)`, dev);
    el('div', `position:absolute;left:560px;top:798px;width:180px;height:12px;border-radius:0 0 12px 12px;background:${mac ? '#B9BCC1' : '#A6AAB0'}`, dev);
    const scr = el('div', `position:absolute;left:24px;top:24px;width:1252px;height:752px;border-radius:10px;overflow:hidden;background:${mac ? '#D9E2EC' : '#1D3B5E'}`, lid);
    // the system's own furniture: the Mac's menu bar, Windows' taskbar with its clock
    if (mac) {
      el('div', `position:absolute;left:0;top:0;right:0;height:30px;display:flex;align-items:center;gap:22px;padding:0 18px;background:rgba(255,255,255,0.55);font-family:${LAT};font-weight:600;font-size:15px;color:#111`, scr,
        '<span style="font-size:17px"></span><span style="font-weight:700">Messages</span><span style="font-weight:500">File</span><span style="font-weight:500">Edit</span><div style="flex:1"></div><span class="clock" style="font-weight:500"></span>');
      el('div', 'position:absolute;left:50%;bottom:10px;transform:translateX(-50%);display:flex;gap:12px;padding:9px 12px;border-radius:20px;background:rgba(255,255,255,0.45);border:1px solid rgba(255,255,255,0.6)', scr,
        ['#3A8EF6', '#F5F5F5', '#34C759', '#FF9F0A', '#5E5CE6'].map((c) => `<div style="width:48px;height:48px;border-radius:12px;background:${c}"></div>`).join(''));
    } else {
      el('div', `position:absolute;left:0;right:0;bottom:0;height:48px;display:flex;align-items:center;padding:0 16px;background:rgba(28,28,30,0.92);font-family:${LAT};color:#fff`, scr,
        '<div style="flex:1"></div><div style="display:flex;gap:14px">' + ['#4CC2FF', '#FFD34E', '#4CC2FF', '#7EE787'].map((c, i) => `<div style="width:30px;height:30px;border-radius:${i ? 7 : 4}px;background:${c};opacity:${i ? 0.85 : 1}"></div>`).join('') +
        '</div><div style="flex:1;display:flex;justify-content:flex-end"><div class="clock" style="text-align:right;font-size:13px;line-height:1.3"></div></div>');
    }
    // the chat window
    const win = el('div', `position:absolute;left:170px;top:${mac ? 60 : 40}px;width:912px;height:${mac ? 500 : 516}px;border-radius:${mac ? 14 : 10}px;overflow:hidden;display:flex;flex-direction:column;background:${mac ? '#FFFFFF' : '#202020'};box-shadow:0 24px 60px rgba(0,0,0,0.35);border:1px solid ${mac ? 'rgba(0,0,0,0.1)' : 'rgba(255,255,255,0.08)'}`, scr);
    const bar = el('div', `height:54px;display:flex;align-items:center;gap:14px;padding:0 18px;border-bottom:1px solid ${mac ? 'rgba(0,0,0,0.08)' : 'rgba(255,255,255,0.07)'};background:${mac ? '#F4F4F6' : '#1B1B1B'}`, win);
    if (mac) el('div', 'display:flex;gap:9px;margin-right:10px', bar, ['#FF5F57', '#FEBC2E', '#28C840'].map((c) => `<div style="width:14px;height:14px;border-radius:99px;background:${c}"></div>`).join(''));
    el('div', `width:34px;height:34px;border-radius:99px;background:#E0303F;display:grid;place-items:center;font-family:${BN};font-weight:700;font-size:17px;color:#fff`, bar, 'মা');
    el('div', `flex:1;font-family:${BN};font-weight:600;font-size:21px;color:${mac ? '#1D1D1F' : '#EEE'}`, bar, 'মা');
    if (!mac) el('div', `display:flex;gap:28px;font-family:${LAT};font-size:18px;color:#AAA`, bar, '<span>—</span><span>▢</span><span>✕</span>');
    const chat = el('div', 'flex:1;display:flex;flex-direction:column;justify-content:flex-end;gap:12px;padding:20px 28px', win);
    const box = el('div', `margin:0 22px 20px;height:54px;border-radius:27px;display:flex;align-items:center;padding:0 22px;background:${mac ? '#F2F2F4' : '#2B2B2B'};border:1px solid ${mac ? 'rgba(0,0,0,0.1)' : 'rgba(255,255,255,0.1)'}`, win);
    const field = el('div', 'flex:1', box);
    const send = el('div', `width:38px;height:38px;border-radius:99px;display:grid;place-items:center;background:${mac ? '#3C7BF0' : '#8AB4F8'};color:${mac ? '#fff' : '#202124'};font-size:18px`, box, '➤');
    // the bar opens just under the text box (in device coordinates)
    const winTop = 24 + (mac ? 60 : 40), winH = mac ? 500 : 516;
    const barAt = { x: 24 + 170 + 40, y: winTop + winH - 20 + 8 };
    const setClock = (t) => scr.querySelectorAll('.clock').forEach((c) => (c.innerHTML = t));
    return { dev, scr, chat, field, send, barAt, setClock, w: 1300, h: 840, dark: !mac };
  }

  /** An Android phone, 520 × 1060, with the real Ekushe Default keys under a suggestion strip */
  function Phone(parent, { name = 'মা', avatar = '#E0303F', letter = 'মা', clock = '' } = {}) {
    const dev = el('div', 'position:absolute;left:0;top:0;width:520px;height:1060px;border-radius:66px;background:#0E0E10;box-shadow:0 30px 70px rgba(0,0,0,0.38);border:2px solid #34343A', parent);
    const scr = el('div', 'position:absolute;left:14px;top:14px;width:492px;height:1032px;border-radius:54px;overflow:hidden;display:flex;flex-direction:column;background:#F7F8FA', dev);
    el('div', `height:46px;display:flex;align-items:center;justify-content:space-between;padding:0 38px;font-family:${LAT};font-weight:700;font-size:17px;color:#111`, scr, `<span>${clock}</span><span style="letter-spacing:2px">▲ ◆ ▮</span>`);
    el('div', 'position:absolute;top:16px;left:50%;margin-left:-9px;width:18px;height:18px;border-radius:99px;background:#000', scr);
    el('div', `display:flex;align-items:center;gap:12px;padding:6px 22px 12px;border-bottom:1px solid rgba(0,0,0,0.07)`, scr,
      `<span style="font-family:${LAT};font-size:22px;color:#444">←</span><div style="width:40px;height:40px;border-radius:99px;background:${avatar};display:grid;place-items:center;font-family:${BN};font-weight:700;font-size:17px;color:#fff">${letter}</div><div style="font-family:${BN};font-weight:600;font-size:22px;color:#111">${name}</div>`);
    const chat = el('div', 'flex:1;display:flex;flex-direction:column;justify-content:flex-end;gap:9px;padding:14px 16px;overflow:hidden', scr);
    const box = el('div', 'margin:6px 12px 10px;height:50px;border-radius:25px;display:flex;align-items:center;gap:8px;padding:0 8px 0 18px;background:#fff;border:1px solid rgba(0,0,0,0.12)', scr);
    const field = el('div', 'flex:1', box);
    const send = el('div', `width:38px;height:38px;flex:none;border-radius:99px;display:grid;place-items:center;background:#3C7BF0;color:#fff;font-size:17px`, box, '➤');
    const stripHost = el('div', 'position:relative;height:52px;flex:none', scr);
    el('img', 'display:block;width:492px;height:273px;flex:none', scr).src = 'assets/img/android-keys.png';
    el('div', 'height:34px;flex:none;background:#E8EAEF;display:flex;align-items:center;justify-content:center', scr, '<div style="width:120px;height:5px;border-radius:9px;background:#1F1F1F;opacity:0.75"></div>');
    // the strip's place in device coordinates: under the box, above the keys
    const stripY = 14 + 1032 - 34 - 273 - 52;
    const sendAt = { x: 14 + 492 - 12 - 8 - 38, y: stripY - 10 - 50 + 6, w: 38, h: 38 };
    return { dev, scr, chat, field, send, stripHost, stripY, sendAt, w: 520, h: 1060 };
  }

  /** A tablet in desktop mode on a keyboard cover, 1400 × 1010 */
  function Tablet(parent, { clock = '' } = {}) {
    const dev = el('div', 'position:absolute;left:0;top:0;width:1400px;height:1010px', parent);
    const body = el('div', 'position:absolute;left:0;top:0;width:1400px;height:800px;border-radius:40px;background:#111113;box-shadow:0 30px 70px rgba(0,0,0,0.38);border:2px solid #36363C', dev);
    const scr = el('div', 'position:absolute;left:20px;top:20px;width:1360px;height:760px;border-radius:24px;overflow:hidden;background:#3E5F57', body);
    // keyboard cover: three rows of keys and a space bar
    const kb = el('div', 'position:absolute;left:30px;top:812px;width:1340px;height:190px;border-radius:14px 14px 30px 30px;background:#2B2B30;padding:16px 22px;box-sizing:border-box;display:flex;flex-direction:column;gap:9px', dev);
    ['qwertyuiop', 'asdfghjkl', 'zxcvbnm'].forEach((r, i) => {
      const row = el('div', `display:flex;gap:9px;padding-left:${i * 34}px`, kb);
      for (const c of r) el('div', `width:96px;height:40px;border-radius:8px;background:#3B3B41;display:grid;place-items:center;font-family:${LAT};font-size:17px;color:#D8D8DC`, row, c);
    });
    // the desktop: a chat window, the taskbar, and the strip just above the taskbar
    const win = el('div', 'position:absolute;left:280px;top:40px;width:800px;height:520px;border-radius:16px;overflow:hidden;display:flex;flex-direction:column;background:#FFFFFF;box-shadow:0 20px 50px rgba(0,0,0,0.35)', scr);
    el('div', `height:50px;display:flex;align-items:center;gap:12px;padding:0 18px;border-bottom:1px solid rgba(0,0,0,0.07);background:#F3F4F6`, win,
      `<div style="width:32px;height:32px;border-radius:99px;background:#E0303F;display:grid;place-items:center;font-family:${BN};font-weight:700;font-size:15px;color:#fff">মা</div><div style="flex:1;font-family:${BN};font-weight:600;font-size:20px;color:#111">মা</div><span style="font-family:${LAT};font-size:16px;color:#777;letter-spacing:14px">— ▢ ✕</span>`);
    const chat = el('div', 'flex:1;display:flex;flex-direction:column;justify-content:flex-end;gap:10px;padding:16px 24px', win);
    const box = el('div', 'margin:0 18px 18px;height:50px;border-radius:25px;display:flex;align-items:center;gap:8px;padding:0 8px 0 20px;background:#F2F3F5;border:1px solid rgba(0,0,0,0.1)', win);
    const field = el('div', 'flex:1', box);
    el('div', 'width:36px;height:36px;border-radius:99px;display:grid;place-items:center;background:#3C7BF0;color:#fff;font-size:16px', box, '➤');
    el('div', `position:absolute;left:0;right:0;bottom:0;height:58px;display:flex;align-items:center;gap:16px;padding:0 22px;background:rgba(242,243,245,0.94);font-family:${LAT};font-size:16px;color:#222`, scr,
      '<div style="display:flex;gap:12px">' + ['#E8EAED', '#4285F4', '#34A853', '#FBBC05', '#EA4335', '#3C7BF0'].map((c, i) => `<div style="width:36px;height:36px;border-radius:${i ? 10 : 8}px;background:${c}"></div>`).join('') + `</div><div style="flex:1"></div><span class="clock" style="font-weight:600"></span>`);
    scr.querySelector('.clock').textContent = clock;
    // the strip sits on top of the taskbar (as beta.5 does in desktop mode)
    const stripY = 20 + 760 - 58 - 52;
    return { dev, scr, chat, field, stripY, w: 1400, h: 1010 };
  }

  window.FILM.Laptop = Laptop;
  window.FILM.Phone = Phone;
  window.FILM.Tablet = Tablet;
})();
