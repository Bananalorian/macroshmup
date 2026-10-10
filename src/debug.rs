//! Debug hooks. Environment variables work in native builds only (the browser has
//! none); the debug keys work in debug builds only.
//!
//!   PG_SCENE=title | stage:<seconds> | shots:<n>   jump straight there (shots: n player shots alive)
//!   PG_SHOTS=60,300                    save those frames to ./shots/ and quit
//!   PG_AUTO=1                          the autopilot flies (weaves, fires, beams)
//!   PG_SEED=<n>                        fixed gameplay seed
//!   PG_GRAY=1                          grayscale screenshots (the value check)
//!   PG_SIZE=1280x720                   window size, for screenshots
//!   PG_PERF=1                          start with the performance overlay on
//!   PG_SHEET=1                         save the ship's animation sheet and quit
//!   PG_WAV=1                           dump every synthesized sound to ./shots/sfx/
//!   PG_FRESH=1                         ignore and don't touch the save file
//!   PG_MENU=1                          open the pause menu (for screenshots)
//!
//! F1 hitboxes, F2 grayscale, F3 quarter speed, F4 frame step (Enter resumes),
//! F8 performance overlay, F9 export the ship sheet.

use crate::render::Renderer;
use crate::render::art::Atlas;
use crate::render::hud::text;
use crate::render::layout::Layout;
use crate::render::palette::{ASH, CREAM, VOID};
use crate::sim::World;
use crate::sim::player::Controls;
use crate::tune::PERF_WINDOW;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SceneReq {
    Title,
    /// The stage, fast-forwarded this many seconds in.
    Stage(f32),
    /// The stage with this many extra player shots kept alive (performance check).
    Shots(usize),
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
        "shots" => arg
            .unwrap_or("500")
            .parse::<usize>()
            .ok()
            .filter(|n| *n <= 5000)
            .map(SceneReq::Shots)
            .ok_or_else(|| "PG_SCENE=shots:<n> wants a count up to 5000".to_string()),
        n if LATER.contains(&n) => Err(format!("PG_SCENE={n} arrives in a later sprint")),
        _ => Err(format!("unknown PG_SCENE {s:?} (try title, stage:<seconds> or shots:<n>)")),
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
    pub auto: bool,
    pub sheet: bool,
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))] // the sound dump is native-only
    pub wav: bool,
    pub menu: bool,
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
        auto: env("PG_AUTO").is_some(),
        sheet: env("PG_SHEET").is_some(),
        wav: env("PG_WAV").is_some(),
        menu: env("PG_MENU").is_some(),
    }
}

/// The debug keys' state (debug builds only).
#[derive(Default)]
pub struct Keys {
    pub hitboxes: bool,
    pub quarter: bool,
    /// Frame-step mode: the game holds still and F4 advances one tick.
    pub stepping: bool,
    pub step: bool,
    pub perf: bool,
    pub sheet: bool,
    quarter_n: u32,
}

impl Keys {
    pub fn new(perf: bool) -> Keys {
        Keys { perf, ..Default::default() }
    }

    pub fn read(&mut self, gray: &mut bool) {
        if !cfg!(debug_assertions) {
            return;
        }
        self.hitboxes ^= is_key_pressed(KeyCode::F1);
        *gray ^= is_key_pressed(KeyCode::F2);
        self.quarter ^= is_key_pressed(KeyCode::F3);
        if is_key_pressed(KeyCode::F4) {
            self.step = self.stepping;
            self.stepping = true;
        }
        if self.stepping && is_key_pressed(KeyCode::Enter) {
            self.stepping = false;
        }
        self.perf ^= is_key_pressed(KeyCode::F8);
        self.sheet |= is_key_pressed(KeyCode::F9);
    }

    /// How many of the clock's ticks actually run.
    pub fn filter(&mut self, ticks: u32) -> u32 {
        if self.stepping {
            return std::mem::take(&mut self.step) as u32;
        }
        if self.quarter {
            let mut run = 0;
            for _ in 0..ticks {
                self.quarter_n += 1;
                if self.quarter_n % 4 == 0 {
                    run += 1;
                }
            }
            return run;
        }
        ticks
    }
}

/// The autopilot (PG_AUTO): weaves across the playfield firing, beams every
/// few seconds. Sprint 2 turns this into the dodging bot.
pub fn autopilot(w: &World) -> Controls {
    let t = w.frame as f32;
    let (tx, ty) = (135.0 + 95.0 * (t * 0.021).sin(), 255.0 + 45.0 * (t * 0.013).sin());
    let (dx, dy) = (tx - w.player.x, ty - w.player.y);
    let d = dx.hypot(dy);
    let k = if d > 4.0 { 1.0 / d } else { 0.25 };
    let beam = (w.frame / 240) % 3 == 2;
    Controls { mv: (dx * k, dy * k), fire: true, focus: beam, ..Default::default() }
}

/// Save the screen to `./shots/<frame>_<w>x<h>.png` (native only).
pub fn save_shot(frame: u64, gray: bool) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut img = get_screen_data();
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

/// Save the ship's animation sheet as `./shots/pilgrim_sheet.png` (native only).
pub fn export_sheet(r: &Renderer) {
    #[cfg(not(target_arch = "wasm32"))]
    crate::sheet::export(&r.atlas);
    #[cfg(target_arch = "wasm32")]
    let _ = r;
}

/// The performance overlay: frame rate, frame and CPU times, ticks, counts, sizes.
pub struct Perf {
    dts: [f32; PERF_WINDOW],
    cpu: [f32; PERF_WINDOW],
    i: usize,
    ticks: u32,
    frames: u64,
    worst: f32,
    sum: f64,
    cpu_sum: f64,
}

impl Perf {
    pub fn new() -> Perf {
        Perf { dts: [1.0 / 60.0; PERF_WINDOW], cpu: [0.0; PERF_WINDOW], i: 0, ticks: 0, frames: 0, worst: 0.0, sum: 0.0, cpu_sum: 0.0 }
    }

    /// `dt` is the whole frame; `cpu` is the time spent updating and drawing it.
    pub fn record(&mut self, dt: f32, cpu: f32, ticks: u32) {
        self.dts[self.i] = dt;
        self.cpu[self.i] = cpu;
        self.i = (self.i + 1) % PERF_WINDOW;
        self.ticks = ticks;
        self.frames += 1;
        if self.frames > 30 {
            // skip startup
            self.worst = self.worst.max(dt);
            self.sum += dt as f64;
            self.cpu_sum += cpu as f64;
        }
    }

    /// One line for the console when a scripted run ends.
    pub fn summary(&self) -> String {
        let n = self.frames.saturating_sub(30).max(1) as f64;
        format!(
            "frames {}  avg {:.2} ms ({:.1} fps)  worst {:.2} ms  cpu avg {:.2} ms",
            self.frames,
            self.sum / n * 1000.0,
            n / self.sum.max(1e-9),
            self.worst * 1000.0,
            self.cpu_sum / n * 1000.0
        )
    }

    pub fn draw(&self, a: &Atlas, lay: &Layout, sim_frame: u64, seed: u64, shots: usize, particles: usize) {
        let avg = self.dts.iter().sum::<f32>() / PERF_WINDOW as f32;
        let worst = self.dts.iter().cloned().fold(0.0, f32::max);
        let cpu = self.cpu.iter().sum::<f32>() / PERF_WINDOW as f32;
        let lines = [
            format!("FPS {:.1}  AVG {:.1}MS  MAX {:.1}MS", 1.0 / avg.max(1e-4), avg * 1000.0, worst * 1000.0),
            format!("CPU {:.2}MS  TICKS {}  SIM {}", cpu * 1000.0, self.ticks, sim_frame),
            format!("SHOTS {}  PARTICLES {}", shots, particles),
            format!("SEED {}  CANVAS {}X{} X{}", seed, lay.cw, lay.ch, lay.scale),
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
        assert_eq!(parse_scene("shots:500"), Ok(SceneReq::Shots(500)));
        assert_eq!(parse_scene("shots"), Ok(SceneReq::Shots(500)));
        assert!(parse_scene("shots:many").is_err());
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

    #[test]
    fn quarter_speed_and_frame_step_hold_the_game() {
        let mut k = Keys { quarter: true, ..Default::default() };
        let run: u32 = (0..40).map(|_| k.filter(1)).sum();
        assert_eq!(run, 10);
        let mut k = Keys { stepping: true, ..Default::default() };
        assert_eq!(k.filter(1), 0);
        k.step = true;
        assert_eq!(k.filter(3), 1);
        assert_eq!(k.filter(1), 0);
    }

    #[test]
    fn the_autopilot_stays_on_the_playfield_and_shoots() {
        let mut w = World::new(5, Default::default());
        let mut beamed = false;
        for _ in 0..1200 {
            let c = autopilot(&w);
            assert!(c.fire);
            beamed |= c.focus;
            w.tick(c);
            w.events.clear();
        }
        assert!(beamed);
    }
}
