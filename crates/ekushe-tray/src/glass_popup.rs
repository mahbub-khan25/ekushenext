//! The floating candidate popup and the language badge, drawn as frosted glass.
//!
//! Everything is drawn into a premultiplied 32-bit buffer in software: shapes as anti-aliased
//! rounded rectangles (signed distance), text through GDI into a grey mask that is then blended in,
//! so text keeps proper alpha on a translucent surface. The result goes to the screen with
//! `UpdateLayeredWindow` (per-pixel alpha: smooth corners, a see-through card), and Windows' acrylic
//! blur is switched on behind it when Windows' own "Transparency effects" are on. At 100 % opacity
//! (Studio → Settings) the popup is solid and no blur is used.
//!
//! Light or dark follows the Windows app theme. Sizes follow the monitor's DPI.

use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr;

use windows_sys::Win32::Foundation::{HWND, POINT, RECT, SIZE};
use windows_sys::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, CreateFontW, CreateRoundRectRgn, DeleteDC, DeleteObject,
    GetDC, GetTextExtentPoint32W, GetTextFaceW, GetTextMetricsW, MonitorFromPoint, ReleaseDC,
    SelectObject, SetBkMode, SetTextColor, SetWindowRgn, TextOutW, AC_SRC_ALPHA, AC_SRC_OVER,
    ANTIALIASED_QUALITY, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS,
    HBITMAP, HDC, HFONT, HGDIOBJ, MONITOR_DEFAULTTONEAREST, TEXTMETRICW, TRANSPARENT,
};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW};
use windows_sys::Win32::System::Registry::{
    RegGetValueW, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RRF_RT_REG_DWORD, RRF_RT_REG_SZ,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{UpdateLayeredWindow, ULW_ALPHA};

// ───────────────────────────── Colour ─────────────────────────────

/// Straight (non-premultiplied) colour, 0..1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgba(pub f32, pub f32, pub f32, pub f32);

const fn rgb(r: u8, g: u8, b: u8, a: f32) -> Rgba {
    Rgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
}

pub struct Palette {
    pub card: Rgba,
    pub rim: Rgba,
    pub hairline: Rgba,
    pub ink: Rgba,
    pub muted: Rgba,
    pub accent: Rgba,
    pub accent_ink: Rgba,
    /// A suggestion chip (and the small buttons)
    pub chip: Rgba,
    pub chip_line: Rgba,
}

impl Palette {
    /// The "Ekushe Default" look of the phone keyboards: a soft grey bar, light chips, and the
    /// chosen word in blue. `glass_alpha`: how opaque the bar is (1.0 = solid).
    pub fn new(dark: bool, glass_alpha: f32) -> Self {
        if dark {
            Palette {
                card: rgb(32, 33, 36, glass_alpha),
                rim: rgb(255, 255, 255, 0.10),
                hairline: rgb(0, 0, 0, 0.50),
                ink: rgb(232, 234, 237, 1.0),
                muted: rgb(154, 160, 166, 1.0),
                accent: rgb(138, 180, 248, 1.0),
                accent_ink: rgb(32, 33, 36, 1.0),
                chip: rgb(255, 255, 255, 0.09),
                chip_line: rgb(255, 255, 255, 0.05),
            }
        } else {
            Palette {
                card: rgb(232, 234, 239, glass_alpha),
                rim: rgb(255, 255, 255, 0.80),
                hairline: rgb(0, 0, 0, 0.14),
                ink: rgb(31, 31, 31, 1.0),
                muted: rgb(95, 99, 104, 1.0),
                accent: rgb(60, 123, 240, 1.0),
                accent_ink: rgb(255, 255, 255, 1.0),
                chip: rgb(255, 255, 255, 0.78),
                chip_line: rgb(0, 0, 0, 0.04),
            }
        }
    }
}

// ───────────────────────────── Canvas ─────────────────────────────

/// A premultiplied BGRA bitmap (top-down DIB), plus a tiny one whose DC measures text.
pub struct Canvas {
    pub w: i32,
    pub h: i32,
    dc: HDC,
    bmp: HBITMAP,
    old: HGDIOBJ,
    px: *mut u32,
    mask_dc: HDC,
    mask_bmp: HBITMAP,
    mask_old: HGDIOBJ,
}

unsafe fn dib(w: i32, h: i32) -> Option<(HDC, HBITMAP, HGDIOBJ, *mut u32)> {
    let dc = CreateCompatibleDC(ptr::null_mut());
    if dc.is_null() {
        return None;
    }
    let mut bi: BITMAPINFO = zeroed();
    bi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    bi.bmiHeader.biWidth = w;
    bi.bmiHeader.biHeight = -h; // top-down
    bi.bmiHeader.biPlanes = 1;
    bi.bmiHeader.biBitCount = 32;
    bi.bmiHeader.biCompression = BI_RGB;
    let mut bits: *mut c_void = ptr::null_mut();
    let bmp = CreateDIBSection(dc, &bi, DIB_RGB_COLORS, &mut bits, ptr::null_mut(), 0);
    if bmp.is_null() || bits.is_null() {
        DeleteDC(dc);
        return None;
    }
    let old = SelectObject(dc, bmp as HGDIOBJ);
    ptr::write_bytes(bits as *mut u32, 0, (w * h) as usize);
    Some((dc, bmp, old, bits as *mut u32))
}

impl Canvas {
    pub unsafe fn new(w: i32, h: i32) -> Option<Self> {
        let w = w.max(1);
        let h = h.max(1);
        let (dc, bmp, old, px) = dib(w, h)?;
        let Some((mask_dc, mask_bmp, mask_old, _)) = dib(1, 1) else {
            SelectObject(dc, old);
            DeleteObject(bmp as HGDIOBJ);
            DeleteDC(dc);
            return None;
        };
        SetBkMode(mask_dc, TRANSPARENT as i32);
        SetTextColor(mask_dc, 0x00FF_FFFF);
        Some(Canvas {
            w,
            h,
            dc,
            bmp,
            old,
            px,
            mask_dc,
            mask_bmp,
            mask_old,
        })
    }

    fn pixels(&mut self) -> &mut [u32] {
        unsafe { std::slice::from_raw_parts_mut(self.px, (self.w * self.h) as usize) }
    }

    /// Source-over blend of `c` with coverage `cov` into pixel `i` (premultiplied BGRA).
    #[inline]
    fn blend(p: &mut u32, c: Rgba, cov: f32) {
        let sa = (c.3 * cov).clamp(0.0, 1.0);
        if sa <= 0.0 {
            return;
        }
        let d = *p;
        let (da, dr, dg, db) = (
            ((d >> 24) & 0xFF) as f32 / 255.0,
            ((d >> 16) & 0xFF) as f32 / 255.0,
            ((d >> 8) & 0xFF) as f32 / 255.0,
            (d & 0xFF) as f32 / 255.0,
        );
        let k = 1.0 - sa;
        let a = sa + da * k;
        let r = c.0 * sa + dr * k;
        let g = c.1 * sa + dg * k;
        let b = c.2 * sa + db * k;
        let q = |v: f32| (v * 255.0 + 0.5).clamp(0.0, 255.0) as u32;
        *p = (q(a) << 24) | (q(r) << 16) | (q(g) << 8) | q(b);
    }

    /// Signed distance from (px, py) to a rounded rectangle.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn rrect_sd(px: f32, py: f32, x0: f32, y0: f32, x1: f32, y1: f32, r: f32) -> f32 {
        let cx = (x0 + x1) * 0.5;
        let cy = (y0 + y1) * 0.5;
        let hx = (x1 - x0) * 0.5;
        let hy = (y1 - y0) * 0.5;
        let r = r.min(hx).min(hy);
        let qx = (px - cx).abs() - hx + r;
        let qy = (py - cy).abs() - hy + r;
        let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
        outside + qx.max(qy).min(0.0) - r
    }

    /// Anti-aliased rounded rectangle; with `stroke` > 0 only a border of that width (inside the edge).
    #[allow(clippy::too_many_arguments)]
    pub fn rrect(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, r: f32, c: Rgba, stroke: f32) {
        let (w, h) = (self.w, self.h);
        let ix0 = (x0.floor() as i32).clamp(0, w);
        let iy0 = (y0.floor() as i32).clamp(0, h);
        let ix1 = (x1.ceil() as i32).clamp(0, w);
        let iy1 = (y1.ceil() as i32).clamp(0, h);
        let px = self.pixels();
        for y in iy0..iy1 {
            for x in ix0..ix1 {
                let sd = Self::rrect_sd(x as f32 + 0.5, y as f32 + 0.5, x0, y0, x1, y1, r);
                let outer = (0.5 - sd).clamp(0.0, 1.0);
                let cov = if stroke > 0.0 {
                    outer - (0.5 - (sd + stroke)).clamp(0.0, 1.0)
                } else {
                    outer
                };
                if cov > 0.0 {
                    Self::blend(&mut px[(y * w + x) as usize], c, cov);
                }
            }
        }
    }

    pub unsafe fn measure(&self, font: &Font, s: &str) -> (i32, i32) {
        let w: Vec<u16> = s.encode_utf16().collect();
        let old = SelectObject(self.mask_dc, font.f as HGDIOBJ);
        let mut sz: SIZE = zeroed();
        if !w.is_empty() {
            GetTextExtentPoint32W(self.mask_dc, w.as_ptr(), w.len() as i32, &mut sz);
        }
        let mut tm: TEXTMETRICW = zeroed();
        GetTextMetricsW(self.mask_dc, &mut tm);
        SelectObject(self.mask_dc, old);
        (sz.cx, tm.tmHeight)
    }

    /// Draw `s` with its line box vertically centred on `cy`, starting at `x`. Returns its width.
    ///
    /// The text is drawn SS times larger and then scaled down, averaging each SS×SS block into
    /// one pixel's coverage. At its real size GDI follows the font's own hinting rules, and many
    /// Bangla fonts switch anti-aliasing off at text sizes, which gave jagged, pixel-looking
    /// letters; drawn large and scaled down they come out smooth, like the rest of Windows.
    pub unsafe fn text(&mut self, font: &Font, s: &str, x: i32, cy: i32, c: Rgba) -> i32 {
        if s.is_empty() {
            return 0;
        }
        let (tw, th) = self.measure(font, s);
        let y = cy - th / 2;
        let (ow, oh) = (tw + 12, th + 4);
        let (bw, bh) = (ow * SS, oh * SS);
        let Some((dc, bmp, old, bits)) = dib(bw, bh) else {
            return tw;
        };
        SetBkMode(dc, TRANSPARENT as i32);
        SetTextColor(dc, 0x00FF_FFFF);
        let w: Vec<u16> = s.encode_utf16().collect();
        let of = SelectObject(dc, font.big as HGDIOBJ);
        TextOutW(dc, 4 * SS, 0, w.as_ptr(), w.len() as i32);
        SelectObject(dc, of);
        let big = std::slice::from_raw_parts(bits, (bw * bh) as usize);
        let n = (self.w * self.h) as usize;
        let px = std::slice::from_raw_parts_mut(self.px, n);
        let full = (255 * SS * SS) as f32;
        for oy in 0..oh {
            let dy = y + oy;
            if dy < 0 || dy >= self.h {
                continue;
            }
            for ox in 0..ow {
                let dx = x - 4 + ox;
                if dx < 0 || dx >= self.w {
                    continue;
                }
                let mut sum = 0u32;
                for sy in 0..SS {
                    let row = ((oy * SS + sy) * bw + ox * SS) as usize;
                    for sx in 0..SS as usize {
                        let m = big[row + sx];
                        sum += ((m >> 16) & 0xFF).max((m >> 8) & 0xFF).max(m & 0xFF);
                    }
                }
                if sum > 0 {
                    // a touch of weight, so thin strokes don't fade on the glass
                    let cov = (sum as f32 / full).powf(0.85);
                    Self::blend(&mut px[(dy * self.w + dx) as usize], c, cov);
                }
            }
        }
        SelectObject(dc, old);
        DeleteObject(bmp as HGDIOBJ);
        DeleteDC(dc);
        tw
    }

    /// Put the bitmap on screen at (x, y) as the window's whole content.
    pub unsafe fn present(&self, hwnd: HWND, x: i32, y: i32) -> bool {
        let screen = GetDC(ptr::null_mut());
        let dst = POINT { x, y };
        let size = SIZE {
            cx: self.w,
            cy: self.h,
        };
        let src = POINT { x: 0, y: 0 };
        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };
        let ok = UpdateLayeredWindow(
            hwnd, screen, &dst, &size, self.dc, &src, 0, &blend, ULW_ALPHA,
        ) != 0;
        ReleaseDC(ptr::null_mut(), screen);
        ok
    }

    /// The bitmap composited over `backdrop(x, y)` as an RGB PNG (for the automated preview).
    #[allow(clippy::wrong_self_convention)]
    pub fn to_png_over(&mut self, backdrop: impl Fn(i32, i32) -> [f32; 3]) -> Vec<u8> {
        let (w, h) = (self.w, self.h);
        let px = self.pixels().to_vec();
        let mut raw = Vec::with_capacity(((w * 3 + 1) * h) as usize);
        for y in 0..h {
            raw.push(0u8);
            for x in 0..w {
                let p = px[(y * w + x) as usize];
                let a = ((p >> 24) & 0xFF) as f32 / 255.0;
                let s = [
                    ((p >> 16) & 0xFF) as f32 / 255.0,
                    ((p >> 8) & 0xFF) as f32 / 255.0,
                    (p & 0xFF) as f32 / 255.0,
                ];
                let b = backdrop(x, y);
                for k in 0..3 {
                    raw.push(((s[k] + b[k] * (1.0 - a)) * 255.0).clamp(0.0, 255.0) as u8);
                }
            }
        }
        encode_png(w as u32, h as u32, &raw)
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.old);
            DeleteObject(self.bmp as HGDIOBJ);
            DeleteDC(self.dc);
            SelectObject(self.mask_dc, self.mask_old);
            DeleteObject(self.mask_bmp as HGDIOBJ);
            DeleteDC(self.mask_dc);
        }
    }
}

fn encode_png(w: u32, h: u32, raw_rgb_rows: &[u8]) -> Vec<u8> {
    fn crc32(data: &[u8]) -> u32 {
        let mut c = 0xFFFF_FFFFu32;
        for &b in data {
            c ^= b as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 {
                    0xEDB8_8320 ^ (c >> 1)
                } else {
                    c >> 1
                };
            }
        }
        !c
    }
    fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
        out.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let mut body = kind.to_vec();
        body.extend_from_slice(data);
        out.extend_from_slice(&body);
        out.extend_from_slice(&crc32(&body).to_be_bytes());
    }
    let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]); // 8-bit RGB
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(
        &mut out,
        b"IDAT",
        &miniz_oxide::deflate::compress_to_vec_zlib(raw_rgb_rows, 6),
    );
    chunk(&mut out, b"IEND", &[]);
    out
}

// ───────────────────────────── Windows look ─────────────────────────────

fn wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

unsafe fn reg_dword(
    root: windows_sys::Win32::System::Registry::HKEY,
    key: &str,
    value: &str,
) -> Option<u32> {
    let (k, v) = (wide_null(key), wide_null(value));
    let mut data: u32 = 0;
    let mut len = size_of::<u32>() as u32;
    let rc = RegGetValueW(
        root,
        k.as_ptr(),
        v.as_ptr(),
        RRF_RT_REG_DWORD,
        ptr::null_mut(),
        &mut data as *mut u32 as *mut c_void,
        &mut len,
    );
    (rc == 0).then_some(data)
}

/// The Windows app theme and whether "Transparency effects" are on.
#[derive(Clone, Copy, Debug)]
pub struct Look {
    pub dark: bool,
    pub transparency: bool,
}

pub fn windows_look() -> Look {
    const KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize";
    unsafe {
        Look {
            dark: reg_dword(HKEY_CURRENT_USER, KEY, "AppsUseLightTheme") == Some(0),
            transparency: reg_dword(HKEY_CURRENT_USER, KEY, "EnableTransparency") != Some(0),
        }
    }
}

fn windows_build() -> u32 {
    static BUILD: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *BUILD.get_or_init(read_windows_build)
}

fn read_windows_build() -> u32 {
    unsafe {
        let k = wide_null("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion");
        let v = wide_null("CurrentBuildNumber");
        let mut buf = [0u16; 16];
        let mut len = (buf.len() * 2) as u32;
        let rc = RegGetValueW(
            HKEY_LOCAL_MACHINE,
            k.as_ptr(),
            v.as_ptr(),
            RRF_RT_REG_SZ,
            ptr::null_mut(),
            buf.as_mut_ptr() as *mut c_void,
            &mut len,
        );
        if rc != 0 {
            return 0;
        }
        String::from_utf16_lossy(&buf[..(len as usize / 2).saturating_sub(1)])
            .trim()
            .parse()
            .unwrap_or(0)
    }
}

#[repr(C)]
struct AccentPolicy {
    state: u32,
    flags: u32,
    gradient: u32,
    animation: u32,
}
#[repr(C)]
struct CompositionData {
    attribute: u32,
    data: *mut c_void,
    size: usize,
}
type SetWindowCompositionAttribute = unsafe extern "system" fn(HWND, *mut CompositionData) -> i32;

/// Blur what is behind the window (acrylic on Windows 10 1803+ and 11, plain blur before that), or
/// switch it off. The blur covers the window region, so a rounded region is set to match the card.
pub unsafe fn set_backdrop(hwnd: HWND, on: bool, dark: bool, w: i32, h: i32, radius: i32) {
    let user32 = GetModuleHandleW(wide_null("user32.dll").as_ptr());
    let Some(f) = GetProcAddress(
        user32,
        c"SetWindowCompositionAttribute".as_ptr() as *const u8,
    ) else {
        return;
    };
    let set: SetWindowCompositionAttribute = std::mem::transmute(f);
    // ACCENT_DISABLED = 0, ACCENT_ENABLE_BLURBEHIND = 3, ACCENT_ENABLE_ACRYLICBLURBEHIND = 4
    let state = if !on {
        0
    } else if windows_build() >= 17134 {
        4
    } else {
        3
    };
    // A faint tint (AABBGGRR); the card itself carries the real colour
    let gradient = if dark { 0x2E1B_1E16 } else { 0x18F6_F8F5 };
    let mut accent = AccentPolicy {
        state,
        flags: 0,
        gradient,
        animation: 0,
    };
    let mut data = CompositionData {
        attribute: 19, // WCA_ACCENT_POLICY
        data: &mut accent as *mut AccentPolicy as *mut c_void,
        size: size_of::<AccentPolicy>(),
    };
    set(hwnd, &mut data);
    if on {
        let rgn = CreateRoundRectRgn(0, 0, w + 1, h + 1, radius * 2, radius * 2);
        SetWindowRgn(hwnd, rgn, 0);
    } else {
        SetWindowRgn(hwnd, ptr::null_mut(), 0);
    }
}

type SetThreadDpiAwarenessContext = unsafe extern "system" fn(isize) -> isize;
type GetDpiForMonitor = unsafe extern "system" fn(*mut c_void, i32, *mut u32, *mut u32) -> i32;

/// Run `f` with this thread per-monitor DPI aware (Windows 10 1607+), so positions and sizes are
/// real pixels and the popup is sharp on scaled displays. Older Windows: `f` runs unchanged.
pub unsafe fn with_dpi_awareness<T>(f: impl FnOnce() -> T) -> T {
    let user32 = GetModuleHandleW(wide_null("user32.dll").as_ptr());
    let set = GetProcAddress(
        user32,
        c"SetThreadDpiAwarenessContext".as_ptr() as *const u8,
    )
    .map(|p| std::mem::transmute::<_, SetThreadDpiAwarenessContext>(p));
    let prev = set.map(|s| s(-4)); // DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2
    let r = f();
    if let (Some(s), Some(p)) = (set, prev) {
        if p != 0 {
            s(p);
        }
    }
    r
}

/// Scale factor (1.0 = 96 dpi) of the monitor containing `pt`.
pub unsafe fn scale_at(pt: POINT) -> f32 {
    static GET: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
    let addr = *GET.get_or_init(|| {
        let shcore = LoadLibraryW(wide_null("shcore.dll").as_ptr());
        if shcore.is_null() {
            return None;
        }
        GetProcAddress(shcore, c"GetDpiForMonitor".as_ptr() as *const u8).map(|p| p as usize)
    });
    let Some(addr) = addr else {
        return 1.0;
    };
    let get: GetDpiForMonitor = std::mem::transmute(addr);
    let mon = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
    let (mut dx, mut dy) = (96u32, 96u32);
    if get(mon, 0, &mut dx, &mut dy) == 0 && dx >= 96 {
        dx as f32 / 96.0
    } else {
        1.0
    }
}

// ───────────────────────────── Popup layout ─────────────────────────────

pub enum Content {
    Candidates {
        roman: String,
        candidates: Vec<String>,
        selected: usize,
        hints: Vec<(String, String)>,
        /// How many of `hints` (at the front) are tips for a word on offer (হেঁসে ← he^se):
        /// highlighted, and shown even in the compact popup
        tips: usize,
        compact: bool,
    },
    Badge(String),
}

/// Clickable areas, in window pixels.
pub struct HitMap {
    pub candidates: Vec<(RECT, usize)>,
    pub hints: Vec<(RECT, String)>,
    pub helper: RECT,
    pub compact: RECT,
}

/// How much larger text is drawn before it is scaled down (see `Canvas::text`)
const SS: i32 = 4;

/// A font at its real size (for measuring) and SS times larger (for drawing).
pub struct Font {
    f: HFONT,
    big: HFONT,
}

struct Fonts {
    bn: Font,
    /// English words on offer: Windows' own UI font, not the Bangla font's Latin letters
    en: Font,
    bn_small: Font,
    ui: Font,
    ui_small: Font,
}

/// Whether Windows has a font family by this name (installed, or one of Ekushe's own).
unsafe fn has_face(face: &str) -> bool {
    let f = wide_null(face);
    let font = CreateFontW(-20, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 0, 0, f.as_ptr());
    if font.is_null() {
        return false;
    }
    let dc = CreateCompatibleDC(ptr::null_mut());
    let old = SelectObject(dc, font as HGDIOBJ);
    let mut buf = [0u16; 64];
    let n = GetTextFaceW(dc, buf.len() as i32, buf.as_mut_ptr());
    SelectObject(dc, old);
    DeleteDC(dc);
    DeleteObject(font as HGDIOBJ);
    n > 0
        && String::from_utf16_lossy(&buf[..(n as usize).saturating_sub(1)])
            .eq_ignore_ascii_case(face)
}

impl Fonts {
    unsafe fn new(bangla: &str, s: f32) -> Self {
        // A family Windows doesn't have would fall back to whatever it substitutes; Nirmala UI is
        // Windows' own Bangla font
        let bangla = if has_face(bangla) {
            bangla
        } else {
            "Nirmala UI"
        };
        let mk = |px: f32, weight: i32, face: &str| {
            let f = wide_null(face);
            let one = |scale: f32| {
                CreateFontW(
                    -((px * s * scale).round() as i32),
                    0,
                    0,
                    0,
                    weight,
                    0,
                    0,
                    0,
                    1,
                    0,
                    0,
                    ANTIALIASED_QUALITY as u32,
                    0,
                    f.as_ptr(),
                )
            };
            Font {
                f: one(1.0),
                big: one(SS as f32),
            }
        };
        Fonts {
            bn: mk(19.0, 400, bangla),
            en: mk(16.0, 400, "Segoe UI"),
            bn_small: mk(15.0, 400, bangla),
            ui: mk(13.0, 400, "Segoe UI"),
            ui_small: mk(11.0, 600, "Segoe UI"),
        }
    }
}

impl Drop for Fonts {
    fn drop(&mut self) {
        unsafe {
            for f in [&self.bn, &self.en, &self.bn_small, &self.ui, &self.ui_small] {
                DeleteObject(f.f as HGDIOBJ);
                DeleteObject(f.big as HGDIOBJ);
            }
        }
    }
}

fn r(x0: f32, y0: f32, x1: f32, y1: f32) -> RECT {
    RECT {
        left: x0 as i32,
        top: y0 as i32,
        right: x1 as i32,
        bottom: y1 as i32,
    }
}

/// Card corner radius in pixels at scale `s` (the window region uses the same value).
pub fn radius(s: f32) -> i32 {
    (14.0 * s).round() as i32
}

/// Draw the popup. `glass_alpha` is the card's opacity (1.0 = solid).
pub unsafe fn render(
    content: &Content,
    bangla_font: &str,
    s: f32,
    dark: bool,
    glass_alpha: f32,
) -> Option<(Canvas, HitMap)> {
    let pal = Palette::new(dark, glass_alpha);
    let fonts = Fonts::new(
        if bangla_font.is_empty() {
            "Kalpurush"
        } else {
            bangla_font
        },
        s,
    );
    let pad = 10.0 * s;
    let gap = 6.0 * s;
    const NONE: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut hits = HitMap {
        candidates: Vec::new(),
        hints: Vec::new(),
        helper: NONE,
        compact: NONE,
    };

    // Measure with a throwaway canvas (fonts need a DC)
    let probe = Canvas::new(1, 1)?;
    match content {
        Content::Badge(text) => {
            let (label, body, english) = match text.split_once(" · ") {
                Some((l, b)) if l == "বাং" || l == "EN" => (Some(l), b, l == "EN"),
                _ => (None, text.as_str(), false),
            };
            let pill_w = label
                .map(|l| probe.measure(&fonts.bn_small, l).0 as f32 + 20.0 * s)
                .unwrap_or(0.0);
            let (bw, _) = probe.measure(&fonts.bn_small, body);
            let h = 44.0 * s;
            let w = pad
                + pill_w
                + if label.is_some() { 10.0 * s } else { 4.0 * s }
                + bw as f32
                + 16.0 * s;
            drop(probe);
            let mut c = Canvas::new(w.ceil() as i32, h.ceil() as i32)?;
            card(&mut c, &pal, s);
            let cy = (h / 2.0) as i32;
            let mut x = pad;
            if let Some(l) = label {
                let (fill, ink) = if english {
                    (pal.muted, pal.card_solid_ink(dark))
                } else {
                    (pal.accent, pal.accent_ink)
                };
                c.rrect(x, 8.0 * s, x + pill_w, h - 8.0 * s, 10.0 * s, fill, 0.0);
                let (lw, _) = c.measure(&fonts.bn_small, l);
                c.text(
                    &fonts.bn_small,
                    l,
                    (x + (pill_w - lw as f32) / 2.0) as i32,
                    cy,
                    ink,
                );
                x += pill_w + 10.0 * s;
            } else {
                x += 4.0 * s;
            }
            c.text(&fonts.bn_small, body, x as i32, cy, pal.ink);
            Some((c, hits))
        }
        Content::Candidates {
            roman,
            candidates,
            selected,
            hints,
            tips,
            compact,
        } => {
            let max_w = 760.0 * s;
            let header_h = 20.0 * s;
            let cand_h = 40.0 * s;
            let hint_h = 28.0 * s;

            // Header buttons
            let toggle_lbl = if *compact {
                "যুক্তবর্ণ দেখান"
            } else {
                "যুক্তবর্ণ লুকান"
            };
            let helper_lbl = "সহায়িকা";
            let tw = probe.measure(&fonts.ui_small, toggle_lbl).0 as f32 + 14.0 * s;
            let hw = probe.measure(&fonts.ui_small, helper_lbl).0 as f32 + 14.0 * s;
            let header_w = probe.measure(&fonts.ui, roman).0 as f32 + 24.0 * s + tw + hw + 6.0 * s;

            // Candidate pills (number + word)
            let mut pills = Vec::new();
            let mut x = pad;
            for (i, text) in candidates.iter().take(6).enumerate() {
                let n = (i + 1).to_string();
                let nw = probe.measure(&fonts.ui_small, &n).0 as f32;
                let word_font = if is_english(text) {
                    &fonts.en
                } else {
                    &fonts.bn
                };
                let ww = probe.measure(word_font, text).0 as f32;
                // an English word on offer carries a small EN, as on the phone keyboards
                let en = if is_english(text) {
                    probe.measure(&fonts.ui_small, "EN").0 as f32 + 7.0 * s
                } else {
                    0.0
                };
                let pw = (12.0 * s + nw + 7.0 * s + ww + en + 14.0 * s).max(46.0 * s);
                if x + pw > max_w - pad && !pills.is_empty() {
                    break;
                }
                pills.push((i, n, text.clone(), x, pw, nw));
                x += pw + gap;
            }
            let cands_w = x - gap + pad;

            let mut chips = Vec::new();
            let mut hx = pad;
            {
                let shown = if *compact {
                    (*tips).min(hints.len())
                } else {
                    hints.len()
                };
                for (bn, key) in hints.iter().take(shown.min(10)) {
                    let bw = probe.measure(&fonts.bn_small, bn).0 as f32;
                    let kw = probe.measure(&fonts.ui_small, key).0 as f32;
                    let cw = 10.0 * s + bw + 5.0 * s + kw + 10.0 * s;
                    if hx + cw > max_w - pad {
                        break;
                    }
                    chips.push((bn.clone(), key.clone(), hx, cw, bw));
                    hx += cw + 5.0 * s;
                }
            }
            let hints_w = hx - 5.0 * s + pad;
            drop(probe);

            let w = cands_w
                .max(header_w + 2.0 * pad)
                .max(if chips.is_empty() { 0.0 } else { hints_w })
                .max(280.0 * s)
                .min(max_w);
            let h = pad
                + header_h
                + 6.0 * s
                + cand_h
                + if chips.is_empty() {
                    0.0
                } else {
                    6.0 * s + hint_h
                }
                + pad;
            let mut c = Canvas::new(w.ceil() as i32, h.ceil() as i32)?;
            card(&mut c, &pal, s);

            // Header: what was typed, then the two small buttons on the right
            let hy = pad + header_h / 2.0;
            c.text(
                &fonts.ui,
                roman,
                (pad + 4.0 * s) as i32,
                hy as i32,
                pal.muted,
            );
            // Right-aligned, helper outermost
            let mut right = w - pad;
            for (lbl, bw, slot) in [(helper_lbl, hw, 0), (toggle_lbl, tw, 1)] {
                let bx = right - bw;
                let rect = r(bx, pad, bx + bw, pad + header_h);
                c.rrect(bx, pad, bx + bw, pad + header_h, 7.0 * s, pal.chip, 0.0);
                c.text(
                    &fonts.ui_small,
                    lbl,
                    (bx + 7.0 * s) as i32,
                    hy as i32,
                    pal.muted,
                );
                if slot == 0 {
                    hits.helper = rect;
                } else {
                    hits.compact = rect;
                }
                right = bx - 6.0 * s;
            }

            // Candidates
            let y0 = pad + header_h + 6.0 * s;
            let cy = (y0 + cand_h / 2.0) as i32;
            for (i, n, text, px, pw, nw) in pills {
                let sel = i == *selected;
                let rad = 10.0 * s;
                if sel {
                    c.rrect(px, y0, px + pw, y0 + cand_h, rad, pal.accent, 0.0);
                } else {
                    c.rrect(px, y0, px + pw, y0 + cand_h, rad, pal.chip, 0.0);
                    c.rrect(px, y0, px + pw, y0 + cand_h, rad, pal.chip_line, 1.0);
                }
                let (ink, soft) = if sel {
                    let k = pal.accent_ink;
                    (k, Rgba(k.0, k.1, k.2, 0.72))
                } else {
                    let m = pal.muted;
                    (pal.ink, Rgba(m.0, m.1, m.2, 0.85))
                };
                c.text(
                    &fonts.ui_small,
                    &n,
                    (px + 12.0 * s) as i32,
                    cy + (1.0 * s) as i32,
                    soft,
                );
                let wx = px + 12.0 * s + nw + 7.0 * s;
                let word_font = if is_english(&text) {
                    &fonts.en
                } else {
                    &fonts.bn
                };
                let ww = c.text(word_font, &text, wx as i32, cy, ink) as f32;
                if is_english(&text) {
                    c.text(
                        &fonts.ui_small,
                        "EN",
                        (wx + ww + 7.0 * s) as i32,
                        cy + (1.0 * s) as i32,
                        soft,
                    );
                }
                hits.candidates.push((r(px, y0, px + pw, y0 + cand_h), i));
            }

            // Conjunct hints
            if !chips.is_empty() {
                let y1 = y0 + cand_h + 6.0 * s;
                let hcy = (y1 + hint_h / 2.0) as i32;
                for (i, (bn, key, hx, cw, bw)) in chips.into_iter().enumerate() {
                    if i < *tips {
                        // A tip: how to type a word in the list directly. Accent-tinted so it is seen.
                        let a = pal.accent;
                        c.rrect(
                            hx,
                            y1,
                            hx + cw,
                            y1 + hint_h,
                            9.0 * s,
                            Rgba(a.0, a.1, a.2, 0.14),
                            0.0,
                        );
                        c.rrect(
                            hx,
                            y1,
                            hx + cw,
                            y1 + hint_h,
                            9.0 * s,
                            Rgba(a.0, a.1, a.2, 0.55),
                            1.2 * s,
                        );
                    } else {
                        c.rrect(hx, y1, hx + cw, y1 + hint_h, 9.0 * s, pal.chip, 0.0);
                    }
                    c.text(&fonts.bn_small, &bn, (hx + 10.0 * s) as i32, hcy, pal.ink);
                    c.text(
                        &fonts.ui_small,
                        &key,
                        (hx + 10.0 * s + bw + 5.0 * s) as i32,
                        hcy,
                        pal.accent,
                    );
                    hits.hints.push((r(hx, y1, hx + cw, y1 + hint_h), key));
                }
            }
            Some((c, hits))
        }
    }
}

impl Palette {
    fn card_solid_ink(&self, dark: bool) -> Rgba {
        if dark {
            rgb(14, 20, 18, 1.0)
        } else {
            rgb(255, 255, 255, 1.0)
        }
    }
}

/// A word in Latin letters: an English suggestion
fn is_english(s: &str) -> bool {
    !s.is_empty() && s.is_ascii()
}

/// The glass card: tinted fill, a bright inner rim and a hairline outside it.
fn card(c: &mut Canvas, pal: &Palette, s: f32) {
    let (w, h) = (c.w as f32, c.h as f32);
    let rad = radius(s) as f32;
    c.rrect(0.0, 0.0, w, h, rad, pal.card, 0.0);
    c.rrect(0.0, 0.0, w, h, rad, pal.hairline, 1.0);
    c.rrect(1.0, 1.0, w - 1.0, h - 1.0, rad - 1.0, pal.rim, 1.0);
}

/// Card opacity for the "popup transparency" setting (70–100 %). With blur available the card is
/// see-through; without it a little translucency would only hurt legibility, so it stays solid.
pub fn glass_alpha(opacity_pct: u8, blur: bool) -> f32 {
    if !blur || opacity_pct >= 100 {
        1.0
    } else {
        0.50 + (opacity_pct.clamp(70, 100) as f32 - 70.0) / 30.0 * 0.42
    }
}

/// Render the popup previews to PNGs over a sample backdrop (used by CI: `ekushe --render-popup DIR`).
pub fn render_previews(dir: &std::path::Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    std::fs::create_dir_all(dir)?;
    // A stand-in for blurred desktop content: green field, red sun, white page
    let backdrop = |x: i32, y: i32, w: i32, h: i32| -> [f32; 3] {
        let (fx, fy) = (x as f32 / w.max(1) as f32, y as f32 / h.max(1) as f32);
        let sun = (-(((fx - 0.75).powi(2) + (fy - 0.3).powi(2)) / 0.05)).exp();
        let field = fy;
        [
            0.96 * (1.0 - field) + 0.05 * field + 0.85 * sun,
            0.97 * (1.0 - field) + 0.42 * field - 0.4 * sun,
            0.96 * (1.0 - field) + 0.29 * field - 0.4 * sun,
        ]
        .map(|v: f32| v.clamp(0.0, 1.0))
    };
    let samples = [
        (
            "candidates",
            Content::Candidates {
                roman: "amar".into(),
                candidates: ["আমার", "আমরা", "আমির", "আমারও", "amar", "আমের"]
                    .map(String::from)
                    .to_vec(),
                selected: 0,
                hints: [
                    ("র্ব", "amarw"),
                    ("র্র", "amarr"),
                    ("র্য", "amary"),
                    ("রা", "amara"),
                    ("রি", "amari"),
                ]
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .to_vec(),
                tips: 0,
                compact: false,
            },
        ),
        (
            "tip",
            Content::Candidates {
                roman: "hese".into(),
                candidates: ["হেসে", "হয়েছে", "হেঁসে", "হিসেবে"]
                    .map(String::from)
                    .to_vec(),
                selected: 0,
                hints: [
                    ("হেঁসে", "he^se"),
                    ("স্ব", "hesw"),
                    ("স্ম", "hesm"),
                    ("স্", "hese,,"),
                ]
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .to_vec(),
                tips: 1,
                compact: false,
            },
        ),
        (
            "compact",
            Content::Candidates {
                roman: "shikkha".into(),
                candidates: ["শিক্ষা", "শিক্ষার", "শিক্ষক"].map(String::from).to_vec(),
                selected: 1,
                hints: vec![("শিক্ষা".to_string(), "shikkha".to_string())],
                tips: 0,
                compact: true,
            },
        ),
        ("badge-bangla", Content::Badge("বাং · বাংলায় লিখছেন".into())),
        (
            "badge-english",
            Content::Badge("EN · English typing".into()),
        ),
    ];
    let mut out = Vec::new();
    for (name, content) in &samples {
        for (dark, theme) in [(false, "light"), (true, "dark")] {
            for (scale, tag) in [(1.0f32, "100"), (1.5, "150")] {
                let Some((mut canvas, _)) =
                    (unsafe { render(content, "Kalpurush", scale, dark, glass_alpha(85, true)) })
                else {
                    continue;
                };
                // Margin around the card so the backdrop shows
                let (w, h) = (canvas.w, canvas.h);
                let png = canvas.to_png_over(|x, y| {
                    let b = backdrop(x, y, w, h);
                    if dark {
                        b.map(|v| v * 0.25)
                    } else {
                        b
                    }
                });
                let path = dir.join(format!("popup-{name}-{theme}-{tag}.png"));
                std::fs::write(&path, png)?;
                out.push(path);
            }
        }
    }
    Ok(out)
}
