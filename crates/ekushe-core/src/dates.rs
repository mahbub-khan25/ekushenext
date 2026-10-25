//! Date and time shortcuts: `;aj` (today), `;tarikh`, `;bar` (weekday), `;bongabdo` (Bangla calendar),
//! `;somoy` (time). They expand to the current local date or time written in Bangla.
//!
//! The Bangla calendar is Bangladesh's revised calendar (in use since 1429 / 2019): 1 Boishakh is always
//! 14 April, the first six months have 31 days, Falgun has 29 days (30 when that February has a 29th)
//! and the other months 30. So 21 February is always 8 Falgun and 16 December is 1 Poush.

/// A local date and time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalTime {
    pub year: i32,
    /// 1–12
    pub month: u32,
    /// 1–31
    pub day: u32,
    /// 0 = Sunday … 6 = Saturday
    pub weekday: u32,
    /// 0–23
    pub hour: u32,
    pub minute: u32,
}

/// The shortcuts, with a short English description (shown in Studio).
pub const DATE_SHORTCUTS: &[(&str, &str)] = &[
    (";aj", "today's date"),
    (";tarikh", "today's date"),
    (";bar", "today's weekday"),
    (";bongabdo", "today in the Bangla calendar"),
    (";somoy", "the time now"),
];

const MONTHS: [&str; 12] = [
    "জানুয়ারি",
    "ফেব্রুয়ারি",
    "মার্চ",
    "এপ্রিল",
    "মে",
    "জুন",
    "জুলাই",
    "আগস্ট",
    "সেপ্টেম্বর",
    "অক্টোবর",
    "নভেম্বর",
    "ডিসেম্বর",
];

const WEEKDAYS: [&str; 7] = [
    "রবিবার",
    "সোমবার",
    "মঙ্গলবার",
    "বুধবার",
    "বৃহস্পতিবার",
    "শুক্রবার",
    "শনিবার",
];

const BANGLA_MONTHS: [&str; 12] = [
    "বৈশাখ",
    "জ্যৈষ্ঠ",
    "আষাঢ়",
    "শ্রাবণ",
    "ভাদ্র",
    "আশ্বিন",
    "কার্তিক",
    "অগ্রহায়ণ",
    "পৌষ",
    "মাঘ",
    "ফাল্গুন",
    "চৈত্র",
];

/// Western digits to Bangla digits.
pub fn bn_digits(s: &str) -> String {
    s.chars()
        .map(|c| match c.to_digit(10) {
            Some(d) => char::from_u32(0x09E6 + d).unwrap_or(c),
            None => c,
        })
        .collect()
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// Days since 1970-01-01 (proleptic Gregorian).
fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y } as i64;
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let m = m as i64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// (day, month index 0–11, year) in the Bangla calendar for a Gregorian date.
pub fn bongabdo(year: i32, month: u32, day: u32) -> (u32, usize, i32) {
    let start_year = if (month, day) >= (4, 14) {
        year
    } else {
        year - 1
    };
    let mut left = days_from_civil(year, month, day) - days_from_civil(start_year, 4, 14);
    // Falgun falls in February of the next Gregorian year
    let falgun = if is_leap(start_year + 1) { 30 } else { 29 };
    let lengths = [31, 31, 31, 31, 31, 31, 30, 30, 30, 30, falgun, 30];
    let mut m = 0;
    while m < 11 && left >= lengths[m] {
        left -= lengths[m];
        m += 1;
    }
    (left as u32 + 1, m, start_year - 593)
}

/// "২৪ অক্টোবর ২০২৬"
pub fn date_bn(t: &LocalTime) -> String {
    bn_digits(&format!(
        "{} {} {}",
        t.day,
        MONTHS[(t.month as usize).clamp(1, 12) - 1],
        t.year
    ))
}

/// "শনিবার"
pub fn weekday_bn(t: &LocalTime) -> &'static str {
    WEEKDAYS[(t.weekday % 7) as usize]
}

/// "৮ কার্তিক ১৪৩৩"
pub fn bongabdo_bn(t: &LocalTime) -> String {
    let (d, m, y) = bongabdo(t.year, t.month, t.day);
    bn_digits(&format!("{d} {} {y}", BANGLA_MONTHS[m]))
}

/// "সন্ধ্যা ৭:৫০"
pub fn time_bn(t: &LocalTime) -> String {
    let part = match t.hour {
        4..=5 => "ভোর",
        6..=11 => "সকাল",
        12..=14 => "দুপুর",
        15..=17 => "বিকাল",
        18..=19 => "সন্ধ্যা",
        _ => "রাত",
    };
    let h12 = match t.hour % 12 {
        0 => 12,
        h => h,
    };
    bn_digits(&format!("{part} {h12}:{:02}", t.minute))
}

/// The expansions for a date shortcut (best first), or `None` if `trigger` is not one.
pub fn expand(trigger: &str, t: &LocalTime) -> Option<Vec<String>> {
    let date = date_bn(t);
    let full = format!("{}, {date}", weekday_bn(t));
    let numeric = bn_digits(&format!("{:02}/{:02}/{}", t.day, t.month, t.year));
    let bangla = bongabdo_bn(t);
    Some(match trigger {
        ";aj" | ";tarikh" => vec![date, full, numeric, format!("{bangla} বঙ্গাব্দ")],
        ";bar" => vec![weekday_bn(t).to_string(), full],
        ";bongabdo" => vec![
            format!("{bangla} বঙ্গাব্দ"),
            format!("{}, {bangla} বঙ্গাব্দ", weekday_bn(t)),
            bangla,
        ],
        ";somoy" => vec![
            time_bn(t),
            bn_digits(&format!("{}:{:02}", t.hour, t.minute)),
        ],
        _ => return None,
    })
}

/// The local date and time now, or `None` where it cannot be read.
pub fn now() -> Option<LocalTime> {
    imp::now()
}

#[cfg(windows)]
mod imp {
    use super::LocalTime;
    pub fn now() -> Option<LocalTime> {
        // SAFETY: GetLocalTime only fills the struct it is given.
        let st = unsafe {
            let mut st = std::mem::zeroed::<windows_sys::Win32::Foundation::SYSTEMTIME>();
            windows_sys::Win32::System::SystemInformation::GetLocalTime(&mut st);
            st
        };
        Some(LocalTime {
            year: st.wYear as i32,
            month: st.wMonth as u32,
            day: st.wDay as u32,
            weekday: st.wDayOfWeek as u32,
            hour: st.wHour as u32,
            minute: st.wMinute as u32,
        })
    }
}

#[cfg(unix)]
mod imp {
    use super::LocalTime;
    pub fn now() -> Option<LocalTime> {
        // SAFETY: time() and localtime_r() only write to the values they are given.
        unsafe {
            let t = libc::time(std::ptr::null_mut());
            let mut tm = std::mem::zeroed::<libc::tm>();
            if libc::localtime_r(&t, &mut tm).is_null() {
                return None;
            }
            Some(LocalTime {
                year: tm.tm_year + 1900,
                month: (tm.tm_mon + 1) as u32,
                day: tm.tm_mday as u32,
                weekday: tm.tm_wday as u32,
                hour: tm.tm_hour as u32,
                minute: tm.tm_min as u32,
            })
        }
    }
}

#[cfg(not(any(windows, unix)))]
mod imp {
    pub fn now() -> Option<super::LocalTime> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> LocalTime {
        // 1970-01-01 was a Thursday
        let weekday = (days_from_civil(year, month, day) + 4).rem_euclid(7) as u32;
        LocalTime {
            year,
            month,
            day,
            weekday,
            hour,
            minute,
        }
    }

    #[test]
    fn the_bangla_calendar_matches_bangladesh() {
        assert_eq!(bongabdo(2026, 4, 14), (1, 0, 1433)); // পহেলা বৈশাখ
        assert_eq!(bongabdo(2026, 4, 13), (30, 11, 1432)); // last day of চৈত্র
        assert_eq!(bongabdo(2026, 2, 21), (8, 10, 1432)); // ৮ ফাল্গুন
        assert_eq!(bongabdo(2028, 2, 21), (8, 10, 1434));
        assert_eq!(bongabdo(2026, 12, 16), (1, 8, 1433)); // ১ পৌষ
        assert_eq!(bongabdo(2026, 3, 26), (12, 11, 1432)); // ১২ চৈত্র
        assert_eq!(bongabdo(2028, 3, 26), (12, 11, 1434)); // leap year: ফাল্গুন has 30 days
        assert_eq!(bongabdo(2028, 3, 15), (1, 11, 1434));
        assert_eq!(bongabdo(2028, 3, 14), (30, 10, 1434));
        assert_eq!(bongabdo(2026, 10, 24), (8, 6, 1433));
    }

    #[test]
    fn shortcuts_write_the_date_in_bangla() {
        let t = at(2026, 10, 24, 19, 50);
        assert_eq!(weekday_bn(&t), "শনিবার");
        assert_eq!(expand(";aj", &t).unwrap()[0], "২৪ অক্টোবর ২০২৬");
        assert_eq!(expand(";aj", &t).unwrap()[1], "শনিবার, ২৪ অক্টোবর ২০২৬");
        assert_eq!(expand(";aj", &t).unwrap()[2], "২৪/১০/২০২৬");
        assert_eq!(expand(";bongabdo", &t).unwrap()[0], "৮ কার্তিক ১৪৩৩ বঙ্গাব্দ");
        assert_eq!(expand(";somoy", &t).unwrap()[0], "সন্ধ্যা ৭:৫০");
        assert_eq!(time_bn(&at(2026, 1, 1, 0, 5)), "রাত ১২:০৫");
        assert_eq!(time_bn(&at(2026, 1, 1, 12, 0)), "দুপুর ১২:০০");
        assert_eq!(time_bn(&at(2026, 1, 1, 9, 7)), "সকাল ৯:০৭");
        assert!(expand(";salam", &t).is_none());
        assert_eq!(weekday_bn(&at(1971, 3, 26, 0, 0)), "শুক্রবার");
        assert_eq!(weekday_bn(&at(1952, 2, 21, 0, 0)), "বৃহস্পতিবার");
    }

    #[test]
    fn the_clock_can_be_read() {
        #[cfg(any(windows, unix))]
        {
            let t = now().expect("local time");
            assert!((2024..2200).contains(&t.year) && (1..=12).contains(&t.month));
        }
    }
}
