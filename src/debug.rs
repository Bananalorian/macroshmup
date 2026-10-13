//! Debug hooks. Environment variables work in native builds only (the browser has
//! none); the debug keys work in debug builds only.
//!
//!   PG_SCENE=title | options | stage:<seconds> | midboss | boss[:<phase>] | shots:<n> | sandbox:<pattern> | stress   jump straight there
//!   PG_SHOTS=60,300                    save those frames to ./shots/ and quit
//!   PG_AUTO=1                          the dodging bot plays
//!   PG_SIM=<n>                         n seeded bot runs of Stage 1, headless, with a report
//!   PG_SIM=patterns:<n>                n seeded bot runs of every sandbox pattern
//!   PG_SIM=boss:<n>                    n seeded bot runs of each of the boss's phases, no bombs
//!   PG_SEED=<n>                        fixed gameplay seed
//!   PG_GRAY=1                          grayscale screenshots (the value check)
//!   PG_SIZE=1280x720                   window size, for screenshots
//!   PG_PERF=1                          start with the performance overlay on
//!   PG_SHEET=1                         save the ship's animation sheet and quit
//!   PG_FXSHEET=1                       save the explosions and black hole sheet and quit
//!   PG_MOCKUP=1                        save the boss's review sheet and quit
//!   PG_WAV=1                           dump every synthesized sound to ./shots/sfx/
//!   PG_FRESH=1                         ignore and don't touch the save file
//!   PG_MENU=1                          open the pause menu (for screenshots)
//!   PG_BOOM=<0-3>                      set off a small/medium/large/boss blast 30 frames in
//!   PG_INVINCIBLE=1                    bullets can't hurt you (as in the pause menu)
//!   PG_BOMB=<frame>                    press bomb on that frame
//!   PG_RUSH=<0-3>                      keep the rush gauge at least this full (slam chains)
//!   PG_KIT=<1-3>                       keep the kit at least this far: 1 the left augment, 2 both, 3 the shield too
//!   PG_DIE=<frame>                     put a bullet on the ship on that frame
//!   PG_KILL=<frame>                    break the boss's phase on that screen frame (F6 does it too)
//!
//! F1 hitboxes, F2 grayscale, F3 quarter speed, F4 frame step (Enter resumes),
//! F5 invincible, F6 next stage section, F7 next pattern, F8 performance overlay,
//! F9 export the sprite sheets.
//! (F1-F3, F5, F7 and F8 are also in the pause menu, in every build.)

use crate::render::Renderer;
use crate::render::art::Atlas;
use crate::render::hud::text;
use crate::render::layout::Layout;
use crate::render::palette::{ASH, CREAM, VOID};
use crate::storage::View;
use crate::tune::SANDBOX;
use crate::tune::PERF_WINDOW;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SceneReq {
    Title,
    /// The title's options screen.
    Options,
    /// Stage 1, starting this many seconds in.
    Stage(f32),
    /// Boss practice: JAWSTHIC from its intro (0) or straight into phase 1-4.
    Boss(u8),
    /// The sandbox with this many extra player shots kept alive (performance check).
    Shots(usize),
    /// The sandbox with pattern `i` running.
    Sandbox(usize),
}


/// A sandbox pattern by number or by name (`3`, `spiral`, `slow-pause-aim`).
fn find_pattern(arg: &str) -> Option<usize> {
    if let Ok(n) = arg.parse::<usize>() {
        return (n < SANDBOX.len()).then_some(n);
    }
    let squash = |s: &str| s.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_ascii_lowercase();
    SANDBOX.iter().position(|p| squash(p.name) == squash(arg))
}

pub fn parse_scene(s: &str) -> Result<SceneReq, String> {
    let s = s.trim().to_ascii_lowercase();
    let (name, arg) = s.split_once(':').map_or((s.as_str(), None), |(n, a)| (n, Some(a)));
    match name {
        "title" => Ok(SceneReq::Title),
        "options" => Ok(SceneReq::Options),
        "midboss" => Ok(SceneReq::Stage(crate::tune::SECTIONS[4].0)),
        "boss" => match arg {
            None => Ok(SceneReq::Boss(0)),
            Some(a) => a.parse::<u8>().ok().filter(|p| *p <= 4).map(SceneReq::Boss).ok_or_else(|| format!("PG_SCENE=boss:<phase> wants a phase 0-4 (0 is its intro), got {a:?}")),
        },
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
        "sandbox" => find_pattern(arg.unwrap_or("1")).map(SceneReq::Sandbox).ok_or_else(|| format!("no sandbox pattern {arg:?}")),
        "stress" => Ok(SceneReq::Sandbox(SANDBOX.iter().position(|p| p.fill >= 4000).unwrap_or(0))),
        _ => Err(format!("unknown PG_SCENE {s:?} (try title, options, stage:<seconds>, midboss, boss:<phase>, shots:<n>, sandbox:<pattern> or stress)")),
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
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))] // the sheets are native-only
    pub fx_sheet: bool,
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub mockup: bool,
    pub boom: Option<usize>,
    pub invincible: bool,
    pub bomb_at: Option<u64>,
    pub rush: Option<f32>,
    pub kit: Option<u8>,
    pub die_at: Option<u64>,
    pub kill_at: Option<u64>,
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

/// A debug hook is driving this run (screenshots, bot runs, sheets): it keeps to a
/// window of a known size instead of going full screen.
pub fn scripted() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::vars().any(|(k, _)| k.starts_with("PG_"))
    }
    #[cfg(target_arch = "wasm32")]
    {
        false
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
        fx_sheet: env("PG_FXSHEET").is_some(),
        mockup: env("PG_MOCKUP").is_some(),
        invincible: env("PG_INVINCIBLE").is_some(),
        bomb_at: env("PG_BOMB").and_then(|s| s.trim().parse().ok()),
        rush: env("PG_RUSH").and_then(|s| s.trim().parse().ok()),
        kit: env("PG_KIT").and_then(|s| s.trim().parse().ok()).filter(|&n: &u8| (1..=3).contains(&n)),
        die_at: env("PG_DIE").and_then(|s| s.trim().parse().ok()),
        kill_at: env("PG_KILL").and_then(|s| s.trim().parse().ok()),
        boom: env("PG_BOOM").and_then(|s| s.trim().parse().ok()).filter(|&n: &usize| n < 4),
        wav: env("PG_WAV").is_some(),
        menu: env("PG_MENU").is_some(),
    }
}

/// Run `job(k)` for k in 0..n on every core, collecting the results (in any order).
#[cfg(not(target_arch = "wasm32"))]
fn parallel<T: Send>(n: u32, job: impl Fn(u32) -> T + Sync) -> Vec<T> {
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let next = std::sync::atomic::AtomicU32::new(0);
    let results = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                loop {
                    let k = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if k >= n {
                        break;
                    }
                    let r = job(k);
                    results.lock().unwrap().push(r);
                }
            });
        }
    });
    results.into_inner().unwrap()
}

/// PG_SIM=<n> plays Stage 1 with the bot on `n` seeds; PG_SIM=patterns:<n> runs every
/// sandbox pattern. Headless, in parallel, with a balance report; runs before any
/// window opens.
#[cfg(not(target_arch = "wasm32"))]
pub fn sim_report(arg: &str) {
    let (what, n) = match arg.trim().split_once(':') {
        Some((what, n)) => (what.to_ascii_lowercase(), n),
        None => (String::new(), arg),
    };
    let seeds = n.trim().parse().unwrap_or(200);
    match what.as_str() {
        "patterns" => pattern_report(seeds),
        "boss" => boss_report(seeds),
        _ => stage_report(seeds),
    }
}

/// Each of the boss's phases from its start, played by the bot without bombs and
/// with endless lives: how often it gets through without a death, and how long the
/// phase takes to break (the sprint's bar: 80% survive, 20-35 s).
#[cfg(not(target_arch = "wasm32"))]
fn boss_report(seeds: u32) {
    use crate::sim::bot;
    use crate::tune::{BOSS_NAME, JAWS_PHASES, JAWS_PHASE_NAMES};
    let start = std::time::Instant::now();
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    println!("PG_SIM: {seeds} seeds of each {BOSS_NAME} phase, the bot without bombs, {threads} threads");
    println!("{:<16} {:>9} {:>11} {:>9} {:>17} {:>9} {:>7} {:>6}", "PHASE", "SURVIVED", "DEATHS/RUN", "BROKEN", "KILL (MED, RANGE)", "TIMEOUTS", "GRAZES", "PEAK");
    for p in 1..=4u8 {
        let runs = parallel(seeds, |k| bot::phase_trial(1000 + k as u64, p, false));
        let n = runs.len().max(1) as f32;
        let mut kills: Vec<u32> = runs.iter().filter_map(|r| r.kill).collect();
        kills.sort();
        let secs = |f: u32| f as f32 / 60.0;
        let kill = match (kills.first(), kills.get(kills.len() / 2), kills.last()) {
            (Some(a), Some(m), Some(z)) => format!("{:.1} s ({:.1}-{:.1})", secs(*m), secs(*a), secs(*z)),
            _ => "never".into(),
        };
        println!(
            "{:<16} {:>8.0}% {:>11.2} {:>8.0}% {:>17} {:>8.0}% {:>7.0} {:>6}",
            format!("{} {}", p, JAWS_PHASE_NAMES[p as usize - 1]),
            runs.iter().filter(|r| r.deaths == 0).count() as f32 * 100.0 / n,
            runs.iter().map(|r| r.deaths).sum::<u32>() as f32 / n,
            kills.len() as f32 * 100.0 / n,
            kill,
            (runs.len() - kills.len()) as f32 * 100.0 / n,
            runs.iter().map(|r| r.grazes).sum::<u32>() as f32 / n,
            runs.iter().map(|r| r.peak).max().unwrap_or(0),
        );
        let _ = JAWS_PHASES;
    }
    println!("done in {:.1} s", start.elapsed().as_secs_f32());
}

/// Stage 1, played by the bot with 3 lives and its bombs: clears, deaths per section,
/// when each section began, and how fast the derelict went down.
#[cfg(not(target_arch = "wasm32"))]
fn stage_report(seeds: u32) {
    use crate::sim::bot;
    use crate::sim::stage1::frames;
    use crate::tune::{SECTIONS, STAGE_NAME};
    let start = std::time::Instant::now();
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    println!("PG_SIM: {seeds} seeds of STAGE 1 // {STAGE_NAME}, the bot with 3 lives and bombs, {threads} threads");
    // first, what the stage throws at you if you shoot nothing
    let calm = parallel(seeds, |k| bot::stage_trial(1000 + k as u64, true, false));
    let m = calm.len().max(1) as f32;
    println!(
        "PACIFIST (dodges, never shoots): cleared {:.0}%, deaths/run {:.2}, avg bullets {:.0}, peak {}",
        calm.iter().filter(|r| r.cleared).count() as f32 * 100.0 / m,
        calm.iter().map(|r| r.deaths).sum::<u32>() as f32 / m,
        calm.iter().map(|r| r.bullet_frames as f32 / r.frames.max(1) as f32).sum::<f32>() / m,
        calm.iter().map(|r| r.peak).max().unwrap_or(0)
    );
    let runs = parallel(seeds, |k| bot::stage_trial(1000 + k as u64, true, true));
    let n = runs.len().max(1) as f32;
    let pct = |c: usize| c as f32 * 100.0 / n;
    let cleared = runs.iter().filter(|r| r.cleared).count();
    let clean = runs.iter().filter(|r| r.cleared && r.deaths == 0).count();
    let avg = |f: &dyn Fn(&bot::StageRun) -> f32| runs.iter().map(f).sum::<f32>() / n;
    println!("{:>8} {:>8} {:>11} {:>10} {:>7} {:>9} {:>7} {:>13} {:>5}", "CLEARED", "NO-MISS", "DEATHS/RUN", "BOMBS/RUN", "KILLED", "KILLS/RUN", "GRAZES", "AVG BULLETS", "PEAK");
    println!(
        "{:>7.0}% {:>7.0}% {:>11.2} {:>10.2} {:>6.0}% {:>9.0} {:>7.0} {:>13.0} {:>5}",
        pct(cleared),
        pct(clean),
        avg(&|r| r.deaths as f32),
        avg(&|r| r.bombs as f32),
        avg(&|r| r.killed as f32 / r.spawned.max(1) as f32) * 100.0,
        avg(&|r| r.killed as f32),
        avg(&|r| r.grazes as f32),
        avg(&|r| r.bullet_frames as f32 / r.frames.max(1) as f32),
        runs.iter().map(|r| r.peak).max().unwrap_or(0)
    );
    let mmss = |f: u32| format!("{}:{:04.1}", f / 3600, (f % 3600) as f32 / 60.0);
    println!("{:<10} {:>8} {:>18} {:>7} {:>11}", "SECTION", "PLANNED", "BEGAN", "DEATHS", "DEATHS/RUN");
    for (i, s) in SECTIONS.iter().enumerate() {
        let began: Vec<u32> = runs.iter().filter_map(|r| r.began[i]).collect();
        let span = match (began.iter().min(), began.iter().max()) {
            (Some(a), Some(b)) => format!("{} - {}", mmss(*a), mmss(*b)),
            _ => "never".into(),
        };
        let deaths: u32 = runs.iter().map(|r| r.section_deaths[i]).sum();
        println!("{:<10} {:>8} {:>18} {:>7} {:>11.2}", s.1, mmss(frames(s.0)), span, deaths, deaths as f32 / n);
    }
    let mut kills: Vec<u32> = runs.iter().filter_map(|r| r.hull_kill).collect();
    kills.sort();
    if let (Some(fast), Some(med)) = (kills.first(), kills.get(kills.len() / 2)) {
        println!("DERELICT: destroyed in {:.0}% of runs, median {:.1} s, fastest {:.1} s; it left in the rest", pct(kills.len()), *med as f32 / 60.0, *fast as f32 / 60.0);
    } else {
        println!("DERELICT: never destroyed");
    }
    use crate::tune::{BOSS_NAME, JAWS_PHASE_NAMES};
    println!("{BOSS_NAME}: destroyed in {:.0}% of runs", pct(runs.iter().filter(|r| r.boss_killed).count()));
    println!("{:<16} {:>8} {:>18} {:>11}", "PHASE", "BROKEN", "TIME (MED, RANGE)", "DEATHS/RUN");
    for p in 0..4 {
        let mut t: Vec<u32> = runs.iter().filter_map(|r| r.phase_time[p]).collect();
        t.sort();
        let time = match (t.first(), t.get(t.len() / 2), t.last()) {
            (Some(a), Some(m), Some(z)) => format!("{:.1} s ({:.1}-{:.1})", *m as f32 / 60.0, *a as f32 / 60.0, *z as f32 / 60.0),
            _ => "never".into(),
        };
        let deaths: u32 = runs.iter().map(|r| r.phase_deaths[p]).sum();
        println!("{:<16} {:>7.0}% {:>18} {:>11.2}", format!("{} {}", p + 1, JAWS_PHASE_NAMES[p]), pct(t.len()), time, deaths as f32 / n);
    }
    // the score: its spread, the extends, the phases' grades and the stage's ranks
    use crate::sim::score::Grade;
    let mut scores: Vec<u64> = runs.iter().map(|r| r.score).collect();
    scores.sort();
    let at = |k: f32| scores[((scores.len() - 1) as f32 * k) as usize];
    println!("SCORE: median {}, 10% {}, 90% {}, best {}", at(0.5), at(0.1), at(0.9), at(1.0));
    let ext = |k: u8| pct(runs.iter().filter(|r| r.extends >= k).count());
    println!("EXTENDS: the first in {:.0}% of runs, the second in {:.0}%", ext(1), ext(2));
    let grades = [Grade::S, Grade::A, Grade::B, Grade::C, Grade::D];
    let spread = |f: &dyn Fn(&StageRunRef) -> Option<Grade>| grades.iter().map(|g| format!("{} {:.0}%", g.letter(), pct(runs.iter().filter(|r| f(r) == Some(*g)).count()))).collect::<Vec<_>>().join("  ");
    for p in 0..4 {
        println!("PHASE {} GRADES: {}", p + 1, spread(&|r| r.grades[p]));
    }
    println!("STAGE RANK: {}", spread(&|r| r.rank));
    // the style rank: the share of the stage spent at each, and the best each run reached
    use crate::sim::style::RANKS;
    let total: u64 = runs.iter().map(|r| r.style_frames.iter().sum::<u64>()).sum::<u64>().max(1);
    let time: Vec<String> = (0..7).map(|k| format!("{} {:.0}%", RANKS[k], 100.0 * runs.iter().map(|r| r.style_frames[k]).sum::<u64>() as f32 / total as f32)).collect();
    println!("STYLE, TIME AT EACH: {}", time.join("  "));
    let peak: Vec<String> = (0..7).map(|k| format!("{} {:.0}%", RANKS[k], pct(runs.iter().filter(|r| r.style_peak == k as u8).count()))).collect();
    println!("STYLE, BEST REACHED: {}", peak.join("  "));
    // the kit: how often, and how soon, each piece is earned; how often the shield breaks
    let kit: Vec<String> = ["LEFT AUGMENT", "RIGHT AUGMENT", "SHIELD"]
        .iter()
        .enumerate()
        .map(|(k, name)| {
            let mut at: Vec<u32> = runs.iter().filter_map(|r| r.kit_at[k]).collect();
            at.sort_unstable();
            let med = at.get(at.len() / 2).map_or("-".to_string(), |&f| format!("{}:{:02}", f / 3600, f / 60 % 60));
            format!("{name} {:.0}% (median {med})", pct(at.len()))
        })
        .collect();
    println!("KIT EARNED: {}", kit.join("  "));
    println!("SHIELD BROKEN: {:.2} a run", runs.iter().map(|r| r.shield_breaks).sum::<u32>() as f32 / n);
    println!("done in {:.1} s", start.elapsed().as_secs_f32());
}

#[cfg(not(target_arch = "wasm32"))]
type StageRunRef = crate::sim::bot::StageRun;

/// Every sandbox pattern for 60 s without bombs (and endless lives).
#[cfg(not(target_arch = "wasm32"))]
fn pattern_report(seeds: u32) {
    use crate::sim::bot;
    use crate::tune::{SANDBOX, TICK_HZ};
    let frames = 60 * TICK_HZ;
    let presets: Vec<usize> = (0..SANDBOX.len()).filter(|&i| !SANDBOX[i].guns.is_empty()).collect();
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let start = std::time::Instant::now();
    println!("PG_SIM: {seeds} seeds x 60 s per pattern, {threads} threads, no bombs");
    println!("{:<18} {:>9} {:>11} {:>9} {:>11}", "PATTERN", "SURVIVED", "DEATHS/RUN", "GRAZES", "PEAK SHOTS");
    for &p in &presets {
        let r = parallel(seeds, |k| bot::trial(p, 1000 + k as u64, frames, false));
        let n = r.len().max(1) as f32;
        let survived = r.iter().filter(|x| x.0 == 0).count() as f32 / n;
        let deaths = r.iter().map(|x| x.0).sum::<u32>() as f32 / n;
        let grazes = r.iter().map(|x| x.1).sum::<u32>() as f32 / n;
        let peak = r.iter().map(|x| x.2).max().unwrap_or(0);
        println!("{:<18} {:>8.0}% {:>11.2} {:>9.0} {:>11}", SANDBOX[p].name, survived * 100.0, deaths, grazes, peak);
    }
    println!("done in {:.1} s", start.elapsed().as_secs_f32());
}

/// The debug keys' state (debug builds only). Most of them flip the sandbox
/// switches that the pause menu also has.
#[derive(Default)]
pub struct Keys {
    /// Frame-step mode: the game holds still and F4 advances one tick.
    pub stepping: bool,
    pub step: bool,
    pub sheet: bool,
    /// F6: skip to the next section of the stage.
    pub skip: bool,
    quarter_n: u32,
}

impl Keys {
    /// Read the F keys; returns how far to step the sandbox pattern (F7).
    pub fn read(&mut self, v: &mut View) -> i32 {
        if !cfg!(debug_assertions) {
            return 0;
        }
        v.hitboxes ^= is_key_pressed(KeyCode::F1);
        v.gray ^= is_key_pressed(KeyCode::F2);
        v.slowmo ^= is_key_pressed(KeyCode::F3);
        if is_key_pressed(KeyCode::F4) {
            self.step = self.stepping;
            self.stepping = true;
        }
        if self.stepping && is_key_pressed(KeyCode::Enter) {
            self.stepping = false;
        }
        v.invincible ^= is_key_pressed(KeyCode::F5);
        self.skip |= is_key_pressed(KeyCode::F6);
        v.fps ^= is_key_pressed(KeyCode::F8);
        self.sheet |= is_key_pressed(KeyCode::F9);
        is_key_pressed(KeyCode::F7) as i32
    }

    /// How many of the clock's ticks actually run (slow-mo runs one in four).
    pub fn filter(&mut self, ticks: u32, slowmo: bool) -> u32 {
        if self.stepping {
            return std::mem::take(&mut self.step) as u32;
        }
        if slowmo {
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
    cpu_worst: (f32, u64),
    /// Frames that ran no tick, or more than one (stutter on a 60 Hz screen), and
    /// frames slower than 20 ms.
    uneven: u64,
    slow: u64,
    /// Since the last `interval()` report: frames, dt sum, worst dt, cpu sum, worst cpu, slow frames.
    iv: (u64, f64, f32, f64, f32, u64),
}

impl Perf {
    pub fn new() -> Perf {
        Perf { dts: [1.0 / 60.0; PERF_WINDOW], cpu: [0.0; PERF_WINDOW], i: 0, ticks: 0, frames: 0, worst: 0.0, sum: 0.0, cpu_sum: 0.0, cpu_worst: (0.0, 0), uneven: 0, slow: 0, iv: (0, 0.0, 0.0, 0.0, 0.0, 0) }
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
            if cpu > self.cpu_worst.0 {
                self.cpu_worst = (cpu, self.frames);
            }
            self.uneven += (ticks != 1) as u64;
            self.slow += (dt > 0.020) as u64;
        }
        let iv = &mut self.iv;
        iv.0 += 1;
        iv.1 += dt as f64;
        iv.2 = iv.2.max(dt);
        iv.3 += cpu as f64;
        iv.4 = iv.4.max(cpu);
        iv.5 += (dt > 0.020) as u64;
    }

    /// One line on the frames since the last call (for long runs with PG_PERF), then start over.
    pub fn interval(&mut self) -> String {
        let (n, dt, worst, cpu, cpu_worst, slow) = self.iv;
        let n1 = n.max(1) as f64;
        self.iv = (0, 0.0, 0.0, 0.0, 0.0, 0);
        format!(
            "{:>6} frames  avg {:.2} ms  worst {:.1} ms  over 20 ms {:>4}  cpu avg {:.2} ms  cpu worst {:.2} ms",
            n,
            dt / n1 * 1000.0,
            worst * 1000.0,
            slow,
            cpu / n1 * 1000.0,
            cpu_worst * 1000.0
        )
    }

    /// One line for the console when a run with PG_PERF ends.
    pub fn summary(&self) -> String {
        let n = self.frames.saturating_sub(30).max(1) as f64;
        format!(
            "frames {}  avg {:.2} ms ({:.1} fps)  worst {:.2} ms  over 20 ms {}  not 1 tick {}  cpu avg {:.2} ms  cpu worst {:.2} ms (frame {})",
            self.frames,
            self.sum / n * 1000.0,
            n / self.sum.max(1e-9),
            self.worst * 1000.0,
            self.slow,
            self.uneven,
            self.cpu_sum / n * 1000.0,
            self.cpu_worst.0 * 1000.0,
            self.cpu_worst.1
        )
    }

    pub fn draw(&self, a: &Atlas, lay: &Layout, sim_frame: u64, seed: u64, shots: usize, bullets: usize, particles: usize) {
        let avg = self.dts.iter().sum::<f32>() / PERF_WINDOW as f32;
        let worst = self.dts.iter().cloned().fold(0.0, f32::max);
        let cpu = self.cpu.iter().sum::<f32>() / PERF_WINDOW as f32;
        let lines = [
            format!("FPS {:.1}  AVG {:.1}MS  MAX {:.1}MS", 1.0 / avg.max(1e-4), avg * 1000.0, worst * 1000.0),
            format!("CPU {:.2}MS  TICKS {}  SIM {}", cpu * 1000.0, self.ticks, sim_frame),
            format!("BULLETS {}  SHOTS {}  FX {}", bullets, shots, particles),
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
        assert_eq!(parse_scene("boss"), Ok(SceneReq::Boss(0)));
        assert_eq!(parse_scene("boss:2"), Ok(SceneReq::Boss(2)));
        assert!(parse_scene("boss:5").is_err());
        assert_eq!(parse_scene("sandbox:3"), Ok(SceneReq::Sandbox(3)));
        assert_eq!(parse_scene("sandbox:spiral"), Ok(SceneReq::Sandbox(SANDBOX.iter().position(|p| p.name == "SPIRAL").unwrap())));
        assert_eq!(parse_scene("sandbox:slow-pause-aim"), Ok(SceneReq::Sandbox(SANDBOX.iter().position(|p| p.name == "SLOW, PAUSE, AIM").unwrap())));
        assert!(parse_scene("sandbox:99").is_err());
        assert_eq!(parse_scene("stress"), Ok(SceneReq::Sandbox(SANDBOX.iter().position(|p| p.fill == 4000).unwrap())));
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
        let mut k = Keys::default();
        let run: u32 = (0..40).map(|_| k.filter(1, true)).sum();
        assert_eq!(run, 10);
        let mut k = Keys { stepping: true, ..Default::default() };
        assert_eq!(k.filter(1, false), 0);
        k.step = true;
        assert_eq!(k.filter(3, false), 1);
        assert_eq!(k.filter(1, false), 0);
    }
}
