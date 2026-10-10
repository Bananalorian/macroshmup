//! What survives between sessions: the tuning panel's values and the options.
//! Ported from Last Train to Shinjuku: the browser build keeps it in localStorage
//! through a tiny JS plugin in web/template.html; desktop builds keep a text file.

use crate::sim::player::Tuning;
use crate::tune::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rumble {
    Off,
    Low,
    Full,
}

impl Rumble {
    pub fn scale(self) -> f32 {
        match self {
            Rumble::Off => 0.0,
            Rumble::Low => RUMBLE_LOW,
            Rumble::Full => 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub tuning: Tuning,
    pub touch_sens: f32,
    pub rumble: Rumble,
    /// The left stick: false snaps to 8 directions (the default), true is analog.
    pub analog: bool,
    pub muted: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { tuning: Tuning::default(), touch_sens: TOUCH_SENS, rumble: Rumble::Full, analog: false, muted: false }
    }
}

impl Settings {
    /// Read `key=value` lines; anything missing or out of range keeps its default.
    pub fn parse(text: &str) -> Settings {
        let mut s = Settings::default();
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else { continue };
            let (k, v) = (k.trim(), v.trim());
            let f = v.parse::<f32>().ok().filter(|f| f.is_finite());
            match k {
                "speed" => {
                    if let Some(f) = f {
                        s.tuning.speed = f.clamp(SPEED_RANGE.0, SPEED_RANGE.1);
                    }
                }
                "focus_ratio" => {
                    if let Some(f) = f {
                        s.tuning.focus_ratio = f.clamp(0.2, 1.0);
                    }
                }
                "focus_blend" => {
                    if let Ok(n) = v.parse::<u32>() {
                        s.tuning.focus_blend = n.min(BLEND_MAX);
                    }
                }
                "touch_sens" => {
                    if let Some(f) = f {
                        s.touch_sens = f.clamp(TOUCH_SENS_RANGE.0, TOUCH_SENS_RANGE.1);
                    }
                }
                "rumble" => {
                    s.rumble = match v {
                        "off" => Rumble::Off,
                        "low" => Rumble::Low,
                        _ => Rumble::Full,
                    }
                }
                "stick" => s.analog = v == "analog",
                "muted" => s.muted = v == "1",
                _ => {}
            }
        }
        s
    }

    pub fn format(&self) -> String {
        let rumble = match self.rumble {
            Rumble::Off => "off",
            Rumble::Low => "low",
            Rumble::Full => "full",
        };
        format!(
            "speed={}\nfocus_ratio={}\nfocus_blend={}\ntouch_sens={}\nrumble={}\nstick={}\nmuted={}\n",
            self.tuning.speed,
            self.tuning.focus_ratio,
            self.tuning.focus_blend,
            self.touch_sens,
            rumble,
            if self.analog { "analog" } else { "8way" },
            self.muted as u8
        )
    }

    pub fn load() -> Settings {
        Settings::parse(&read())
    }

    pub fn save(&self) {
        write(&self.format());
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn path() -> Option<std::path::PathBuf> {
    let base = std::env::var("APPDATA").or_else(|_| std::env::var("HOME")).ok()?;
    Some(std::path::PathBuf::from(base).join(".pilgrim").join("save.txt"))
}

#[cfg(not(target_arch = "wasm32"))]
fn read() -> String {
    if std::env::var("PG_FRESH").is_ok() {
        return String::new(); // screenshots and bot runs use the defaults
    }
    path().and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn write(data: &str) {
    if std::env::var("PG_FRESH").is_ok() {
        return;
    }
    if let Some(p) = path() {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(p, data);
    }
}

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    fn pg_store_get(ptr: *mut u8, cap: u32) -> u32;
    fn pg_store_set(ptr: *const u8, len: u32);
}

#[cfg(target_arch = "wasm32")]
fn read() -> String {
    let mut buf = vec![0u8; 4096];
    let n = unsafe { pg_store_get(buf.as_mut_ptr(), buf.len() as u32) } as usize;
    buf.truncate(n.min(4096));
    String::from_utf8(buf).unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
fn write(data: &str) {
    unsafe { pg_store_set(data.as_ptr(), data.len() as u32) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip() {
        let s = Settings {
            tuning: Tuning { speed: 4.5, focus_ratio: 0.67, focus_blend: 8 },
            touch_sens: 1.3,
            rumble: Rumble::Low,
            analog: true,
            muted: true,
        };
        assert_eq!(Settings::parse(&s.format()), s);
        assert_eq!(Settings::parse(&Settings::default().format()), Settings::default());
    }

    #[test]
    fn junk_and_extremes_fall_back_safely() {
        let s = Settings::parse("speed=99\nfocus_ratio=nan\nfocus_blend=-3\nrumble=loud\nwho=me\nnot a line");
        assert_eq!(s.tuning.speed, SPEED_RANGE.1);
        assert_eq!(s.tuning.focus_ratio, FOCUS_RATIO);
        assert_eq!(s.tuning.focus_blend, FOCUS_BLEND);
        assert_eq!(s.rumble, Rumble::Full);
        assert_eq!(Settings::parse(""), Settings::default());
    }
}
