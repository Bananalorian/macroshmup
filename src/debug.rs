//! Debug hooks. Environment variables work in native builds only (the browser has
//! none); the debug keys work in debug builds only.
//!
//!   PG_SCENE=title | stage:<seconds>   jump straight there
//!   PG_SHOTS=60,300                    save those frames to ./shots/ and quit
//!   PG_SEED=<n>                        fixed gameplay seed
//!   PG_GRAY=1                          grayscale screenshots (the value check)
//!   PG_SIZE=1280x720                   window size, for screenshots
//!   PG_PERF=1                          start with the performance overlay on
//!
//! F8 toggles the performance overlay.

use crate::render::art::Atlas;
use crate::render::hud::text;
use crate::render::layout::Layout;
use crate::render::palette::{ASH, CREAM, VOID};
use crate::tune::PERF_WINDOW;
use macroquad::color::Color;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SceneReq {
    Title,
    /// The stage, fast-forwarded this many seconds in.
    Stage(f32),
}

/// Scenes from the plan that later sprints will build.
const LATER: [&str; 4] = ["midboss", "boss", "sandbox", "stress"];

pub fn parse_scene(s: &str) -> Result<SceneReq, String> {
    let s = s.trim().to_ascii_lowercase();
    let (name, arg) = s.split_once(':').map_or((s.as_str(), None), |(n, a)| (n, Some(a)));
    match name {
        "title" => Ok(SceneReq::Title),
        "stage" => match arg {
            None => Ok(SceneReq::Stage(0.0)),
            Some(a) => a
                .parse::<f32>()
                .ok()
                .filter(|v| v.is_finite() && *v >= 0.0)
                .map(SceneReq::Stage)
                .ok_or_else(|| format!("PG_SCENE=stage:<seconds> wants a number of seconds, got {a:?}")),
        },
        n if LATER.contains(&n) => Err(format!("PG_SCENE={n} arrives in a later sprint")),
        _ => Err(format!("unknown PG_SCENE {s:?} (try title or stage:<seconds>)")),
    }
}

/// `60,300` -> [60, 300]; junk entries are skipped.
pub fn parse_frames(s: &str) -> Vec<u64> {
    s.split(',').filter_map(|n| n.trim().parse().ok()).filter(|&n| n > 0).collect()
}

/// `1280x720` -> (1280, 720).
pub fn parse_size(s: &str) -> Option<(i32, i32)> {
    let (w, h) = s.trim().to_ascii_lowercase().split_once('x').map(|(w, h)| (w.parse().ok(), h.parse().ok()))?;
    Some((w.filter(|&w| w > 0)?, h.filter(|&h| h > 0)?))
}

#[derive(Default)]
pub struct Hooks {
    pub scene: Option<SceneReq>,
    pub shots: Vec<u64>,
    pub seed: Option<u64>,
    pub gray: bool,
    pub perf: bool,
}

fn env(name: &str) -> Option<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var(name).ok()
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = name;
        None
    }
}

pub fn window_size() -> Option<(i32, i32)> {
    env("PG_SIZE").and_then(|s| parse_size(&s))
}

pub fn hooks() -> Hooks {
    let scene = env("PG_SCENE").and_then(|s| match parse_scene(&s) {
        Ok(r) => Some(r),
        Err(e) => {
            eprintln!("{e}; starting at the title");
            None
        }
    });
    Hooks {
        scene,
        shots: env("PG_SHOTS").map(|s| parse_frames(&s)).unwrap_or_default(),
        seed: env("PG_SEED").and_then(|s| s.trim().parse().ok()),
        gray: env("PG_GRAY").is_some(),
        perf: env("PG_PERF").is_some(),
    }
}

/// Save the screen to `./shots/<frame>_<w>x<h>.png` (native only).
pub fn save_shot(frame: u64, gray: bool) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut img = macroquad::texture::get_screen_data();
        if gray {
            grayscale(&mut img.bytes);
        }
        let _ = std::fs::create_dir_all("shots");
        let tag = if gray { "_gray" } else { "" };
        let path = format!("shots/{frame:05}_{}x{}{tag}.png", img.width, img.height);
        img.export_png(&path);
        println!("saved {path}");
    }
    #[cfg(target_arch = "wasm32")]
    let _ = (frame, gray);
}

/// RGBA bytes -> their Rec. 709 luma, the same formula as `palette::luma`.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))] // screenshots are native-only
pub fn grayscale(rgba: &mut [u8]) {
    for p in rgba.chunks_exact_mut(4) {
        let l = 0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32;
        let l = l.round().clamp(0.0, 255.0) as u8;
        p[0] = l;
        p[1] = l;
        p[2] = l;
    }
}

/// The performance overlay: frame rate, frame times, ticks, sizes.
pub struct Perf {
    dts: [f32; PERF_WINDOW],
    i: usize,
    ticks: u32,
}

impl Perf {
    pub fn new() -> Perf {
        Perf { dts: [1.0 / 60.0; PERF_WINDOW], i: 0, ticks: 0 }
    }

    pub fn record(&mut self, dt: f32, ticks: u32) {
        self.dts[self.i] = dt;
        self.i = (self.i + 1) % PERF_WINDOW;
        self.ticks = ticks;
    }

    pub fn draw(&self, a: &Atlas, lay: &Layout, sim_frame: u64, seed: u64) {
        let avg = self.dts.iter().sum::<f32>() / PERF_WINDOW as f32;
        let worst = self.dts.iter().cloned().fold(0.0, f32::max);
        let lines = [
            format!("FPS {:.1}  AVG {:.1}MS  MAX {:.1}MS", 1.0 / avg.max(1e-4), avg * 1000.0, worst * 1000.0),
            format!("TICKS {}  SIM {}  SEED {}", self.ticks, sim_frame, seed),
            format!("CANVAS {}X{} X{}  SCREEN {}X{}", lay.cw, lay.ch, lay.scale, lay.cw * lay.scale + 2 * lay.ox, lay.ch * lay.scale + 2 * lay.oy),
        ];
        let (x, y) = (lay.field.x + 2, lay.field.y + 2);
        let w = lines.iter().map(|l| crate::render::font::text_width(l, 1, 1)).max().unwrap_or(0);
        a.fill(x, y, w + 6, lines.len() as i32 * 10 + 4, Color { a: 0.85, ..VOID });
        for (i, l) in lines.iter().enumerate() {
            text(a, l, x + 3, y + 3 + i as i32 * 10, 1, 1, if i == 0 { CREAM } else { ASH });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenes_parse() {
        assert_eq!(parse_scene("title"), Ok(SceneReq::Title));
        assert_eq!(parse_scene("stage"), Ok(SceneReq::Stage(0.0)));
        assert_eq!(parse_scene("stage:42"), Ok(SceneReq::Stage(42.0)));
        assert_eq!(parse_scene(" STAGE:1.5 "), Ok(SceneReq::Stage(1.5)));
        assert!(parse_scene("stage:-3").is_err());
        assert!(parse_scene("stage:soon").is_err());
        assert!(parse_scene("boss:2").unwrap_err().contains("later sprint"));
        assert!(parse_scene("pachinko").is_err());
    }

    #[test]
    fn shot_frames_and_sizes_parse() {
        assert_eq!(parse_frames("60,300"), vec![60, 300]);
        assert_eq!(parse_frames(" 1, x, 0, 9 "), vec![1, 9]);
        assert_eq!(parse_size("1280x720"), Some((1280, 720)));
        assert_eq!(parse_size("390X844"), Some((390, 844)));
        assert_eq!(parse_size("0x720"), None);
        assert_eq!(parse_size("wide"), None);
    }

    #[test]
    fn grayscale_uses_the_palette_luma() {
        use crate::render::palette::{SIGNAL, TEAL, luma};
        for c in [SIGNAL, TEAL, CREAM, VOID] {
            let mut px = [(c.r * 255.0).round() as u8, (c.g * 255.0).round() as u8, (c.b * 255.0).round() as u8, 255];
            grayscale(&mut px);
            assert!((px[0] as f32 / 255.0 - luma(c)).abs() < 0.01);
            assert_eq!((px[0], px[1], px[3]), (px[2], px[2], 255));
        }
    }
}
