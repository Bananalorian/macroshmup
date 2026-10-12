//! Type and screen furniture, styled after the PILGRIM sheet: spaced capitals,
//! thin rules, small labels, a version stamp bottom-left and a tagline bottom-right.

use super::Renderer;
use super::art::Atlas;
use super::font::{self, text_width};
use super::layout::{IRect, Layout, Orient};
use super::palette::{ASH, CORE, CREAM, DUSK, GEM, INK, SIGNAL, SLATE, STRIP, TEAL, TEAL_HOT, VOID, WHITE_A, tone};
use macroquad::math::Rect;
use crate::input::{Button, Mode, touch_buttons};
use crate::menu::{self, ITEMS};
use crate::storage::Settings;
use crate::storage::View;
use crate::tune::{BLINK_FRAMES, BOSS_NAME, JAWS_PHASE_NAMES, JAWS_REFILL, SANDBOX, SECTIONS, STAGE_NAME, TALLY_COUNT, TALLY_DONE, TALLY_LINE, TALLY_RANK, TALLY_SCORE, TALLY_STEP, TICK_HZ, TITLE_CARD, TITLE_SCALE, TITLE_TRACK, TRACK};

/// The sandbox switches that change how the game plays, as one line.
fn sandbox_flags(v: &View) -> String {
    let mut f = vec![];
    if v.autopilot {
        f.push("BOT");
    }
    if v.slowmo {
        f.push("SLOW");
    }
    if v.invincible {
        f.push("INVINCIBLE");
    }
    f.join(" ")
}
use macroquad::color::Color;

/// `V0.2 // PILGRIM`, from the crate version.
pub fn version_stamp() -> String {
    let mut v = env!("CARGO_PKG_VERSION").split('.');
    format!("V{}.{} // PILGRIM", v.next().unwrap_or("0"), v.next().unwrap_or("0"))
}

/// Draw `s` with its top-left at whole pixel `(x, y)`.
pub fn text(a: &Atlas, s: &str, x: i32, y: i32, scale: i32, track: i32, c: Color) {
    let step = (font::GW as i32 + track) * scale;
    for (i, ch) in s.chars().enumerate() {
        if let Some(g) = a.r.glyph(ch) {
            a.blit(g, x + i as i32 * step, y, scale, c);
        }
    }
}

/// Small spaced text on a VOID backing, so a passing star never reads as a glyph.
fn small(a: &Atlas, s: &str, x: i32, y: i32, c: Color) {
    a.fill(x - 1, y - 1, text_width(s, 1, TRACK) + 2, font::GH as i32 + 2, VOID);
    text(a, s, x, y, 1, TRACK, c);
}

/// Small spaced text, right edge at `x`.
fn text_r(a: &Atlas, s: &str, x: i32, y: i32, c: Color) {
    small(a, s, x - text_width(s, 1, TRACK), y, c);
}

/// Text centered on `cx`.
fn text_c(a: &Atlas, s: &str, cx: i32, y: i32, scale: i32, track: i32, c: Color) {
    let (w, x) = (text_width(s, scale, track), cx - text_width(s, scale, track) / 2);
    a.fill(x - scale, y - scale, w + 2 * scale, (font::GH as i32 + 2) * scale, VOID);
    text(a, s, x, y, scale, track, c);
}

fn hrule(a: &Atlas, x0: i32, x1: i32, y: i32) {
    a.fill(x0, y, x1 - x0, 1, DUSK);
}

/// The palette strip from the top of the sheet.
fn strip(a: &Atlas, x: i32, y: i32, sw: i32, sh: i32) {
    for (i, c) in STRIP.iter().enumerate() {
        a.fill(x + i as i32 * sw, y, sw, sh, *c);
    }
}

/// The sheet's corner mark: a cross sparkle with a couple of loose dots.
fn sparkle(a: &Atlas, x: i32, y: i32) {
    a.blit(a.r.sparkle, x, y, 1, tone(0.75));
    a.fill(x + 3, y + 15, 1, 1, tone(0.5));
    a.fill(x + 6, y + 26, 1, 1, tone(0.35));
}

/// The two stamps from the bottom of the sheet, under a full-width rule.
fn stamps(a: &Atlas, x0: i32, x1: i32, y: i32, right: bool) {
    hrule(a, x0, x1, y);
    small(a, &version_stamp(), x0, y + 6, ASH);
    small(a, "HUMANITY STILL LOOKS UP", x0, y + 16, SLATE);
    if right {
        text_r(a, "A SMALL SHIP", x1, y + 6, ASH);
        text_r(a, "A LONG WAY FROM HOME", x1, y + 16, SLATE);
    }
}

/// The stamps' long lines, broken to fit a side panel.
const LEFT_STAMP: [&str; 2] = ["HUMANITY", "STILL LOOKS UP"];
const RIGHT_STAMP: [&str; 3] = ["A SMALL SHIP", "A LONG WAY", "FROM HOME"];
/// The health bar gem's radius (px).
const GEM_R: i32 = 5;

/// A score as 8 digits, the leading zeros dimmed (the arcade way).
fn score_digits(a: &Atlas, n: u64, x: i32, y: i32, scale: i32) {
    let s = format!("{:08}", n.min(99_999_999));
    let lead = s.find(|c| c != '0').unwrap_or(s.len() - 1);
    let track = if scale > 1 { 1 } else { TRACK };
    a.fill(x - 1, y - 1, text_width(&s, scale, track) + 2, font::GH as i32 * scale + 2, VOID);
    text(a, &s[..lead], x, y, scale, track, DUSK);
    let step = (font::GW as i32 + track) * scale;
    text(a, &s[lead..], x + lead as i32 * step, y, scale, track, if scale > 1 { CREAM } else { ASH });
}

/// The stamps for a side panel: three short lines under the rule at `y`.
fn panel_stamps(a: &Atlas, x0: i32, x1: i32, y: i32, right: bool) {
    hrule(a, x0, x1, y);
    if right {
        for (i, s) in RIGHT_STAMP.iter().enumerate() {
            text_r(a, s, x1, y + 6 + i as i32 * 10, if i == 0 { ASH } else { SLATE });
        }
    } else {
        small(a, &version_stamp(), x0, y + 6, ASH);
        for (i, s) in LEFT_STAMP.iter().enumerate() {
            small(a, s, x0, y + 16 + i as i32 * 10, SLATE);
        }
    }
}

fn start_hint(mode: Option<Mode>) -> &'static str {
    match mode {
        None => "PRESS Z  //  TAP TO START",
        Some(Mode::Keys) => "PRESS Z TO START",
        Some(Mode::Pad) => "PRESS A TO START",
        Some(Mode::Touch) => "TAP TO START",
    }
}

/// The title's background (the scene pass): stars and the black hole.
pub fn title_bg(r: &Renderer, lay: &Layout, t: u64) {
    let full = IRect::new(0, 0, lay.cw, lay.ch);
    r.stars.draw(&r.atlas, full, t, 1.0);
    let (cy, rad) = match lay.orient {
        Orient::Landscape => (80.0, 17.0),
        Orient::Portrait => (112.0, 15.0),
    };
    r.hole(full, lay.cw as f32 * 0.5, cy, rad, super::scene::hole_bright(1.0), t as f32 / 60.0, 0.0);
}

/// The title's choices (Boss Practice once you've reached the boss).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleItem {
    Stage,
    Practice,
    Sandbox,
    Options,
}

impl TitleItem {
    pub fn label(self) -> &'static str {
        match self {
            TitleItem::Stage => "STAGE 1",
            TitleItem::Practice => "BOSS PRACTICE",
            TitleItem::Sandbox => "PATTERN SANDBOX",
            TitleItem::Options => "OPTIONS",
        }
    }
}

pub fn title_items(practice: bool) -> Vec<TitleItem> {
    let mut v = vec![TitleItem::Stage];
    if practice {
        v.push(TitleItem::Practice);
    }
    v.extend([TitleItem::Sandbox, TitleItem::Options]);
    v
}

/// The title menu's geometry: (center x, first row's y, row height).
pub fn title_rows(lay: &Layout) -> (i32, i32, i32) {
    match lay.orient {
        Orient::Landscape => (lay.cw / 2, 204, 13),
        Orient::Portrait => (lay.cw / 2, 262, 15),
    }
}

/// Which of `n` title rows a tap at canvas `(x, y)` lands on.
pub fn title_tap(lay: &Layout, n: usize, x: f32, y: f32) -> Option<usize> {
    let (cx, y0, row_h) = title_rows(lay);
    let (x, y) = (x as i32, y as i32);
    if (x - cx).abs() > 90 || y < y0 - 4 {
        return None;
    }
    let i = ((y - y0 + 4) / row_h) as usize;
    (i < n).then_some(i)
}

fn title_menu(a: &Atlas, lay: &Layout, items: &[TitleItem], sel: usize, blink_on: bool) {
    let (cx, y0, row_h) = title_rows(lay);
    for (i, item) in items.iter().enumerate() {
        let label = item.label();
        let y = y0 + i as i32 * row_h;
        let on = i == sel;
        text_c(a, label, cx, y, 1, TRACK, if on { CREAM } else { SLATE });
        if on && blink_on {
            let w = text_width(label, 1, TRACK);
            text(a, ">", cx - w / 2 - 12, y, 1, 1, CREAM);
            text(a, "<", cx + w / 2 + 6, y, 1, 1, CREAM);
        }
    }
}

/// The best score, centered on `cx`.
fn hiscore_line(a: &Atlas, cx: i32, y: i32, hiscore: u64) {
    text_c(a, &format!("HI-SCORE  {}", super::popups::commas(hiscore)), cx, y, 1, TRACK, ASH);
}

/// The title: the name, the tagline, the menu, the best score (drawn over the post pass).
pub fn title(r: &Renderer, lay: &Layout, t: u64, mode: Option<Mode>, items: &[TitleItem], sel: usize, hiscore: u64) {
    let a = &r.atlas;
    let blink_on = t % (BLINK_FRAMES * 3) < BLINK_FRAMES * 2;
    let cx = lay.cw / 2;
    match lay.orient {
        Orient::Landscape => {
            let m = 16;
            small(a, "TYPE-07  \"PILGRIM\"", m, m, ASH);
            small(a, "HUMAN PILOTED CRAFT // DEEP SPACE PROTOCOL", m, m + 10, SLATE);
            strip(a, m, m + 24, 12, 6);
            sparkle(a, lay.cw - m - 7, m);

            let ty = 126;
            text_c(a, "PILGRIM", cx, ty, TITLE_SCALE, TITLE_TRACK, CREAM);
            let w = text_width("PILGRIM", TITLE_SCALE, TITLE_TRACK);
            hrule(a, cx - w / 2, cx + w / 2, ty + 32);
            text_c(a, "SOMEWHERE BEYOND THE EVENT HORIZON", cx, ty + 42, 1, TRACK, ASH);
            title_menu(a, lay, items, sel, blink_on);
            text_c(a, start_hint(mode), cx, 266, 1, TRACK, SLATE);
            hiscore_line(a, cx, 284, hiscore);
            stamps(a, m, lay.cw - m, lay.ch - 32, true);
        }
        Orient::Portrait => {
            let m = 12;
            small(a, "TYPE-07  \"PILGRIM\"", m, m, ASH);
            strip(a, m, m + 12, 10, 4);
            sparkle(a, lay.cw - m - 7, m);

            let ty = 170;
            text_c(a, "PILGRIM", cx, ty, TITLE_SCALE, TITLE_TRACK, CREAM);
            let w = text_width("PILGRIM", TITLE_SCALE, TITLE_TRACK);
            hrule(a, cx - w / 2, cx + w / 2, ty + 32);
            text_c(a, "SOMEWHERE BEYOND", cx, ty + 42, 1, TRACK, ASH);
            text_c(a, "THE EVENT HORIZON", cx, ty + 52, 1, TRACK, ASH);
            title_menu(a, lay, items, sel, blink_on);
            text_c(a, if mode == Some(Mode::Pad) { "PRESS A TO START" } else { "TAP TO START" }, cx, 338, 1, TRACK, SLATE);
            hiscore_line(a, cx, 356, hiscore);
            stamps(a, m, lay.cw - m, lay.ch - 32, false);
        }
    }
}

/// The options screen, over the title's sky.
pub fn options(r: &Renderer, lay: &Layout, s: &Settings, mode: Mode, sel: usize) {
    menu_panel(&r.atlas, lay, s, mode, &menu::OPTIONS, sel, "OPTIONS");
}

/// Everything the stage HUD shows besides the playfield.
pub struct HudView<'a> {
    pub settings: &'a Settings,
    pub mode: Mode,
    /// The pause menu's selected row, when paused.
    pub menu: Option<usize>,
    pub toast: Option<(&'a str, u32)>,
    pub bomb_flash: u32,
    pub slam_flash: u32,
    /// Ships left (including the one flying) and bombs, and the rush gauge (slam
    /// links, 0 to 3).
    pub lives: u8,
    pub bombs: u8,
    pub rush: f32,
    /// The game-over menu's selected row, once the last ship is gone.
    pub over: Option<usize>,
    /// The sandbox: pattern number, bullets on screen, hits, grazes.
    pub pattern: usize,
    pub bullets: usize,
    pub hits: u32,
    pub grazes: u32,
    /// Stage 1's state (none in the sandbox).
    pub stage: Option<StageView>,
    /// The score, and the best so far.
    pub score: u64,
    pub hiscore: u64,
    /// Frames since the run ended (the tally and game over count up on it), whether
    /// it set a new record, and the stage's tally.
    pub end_t: u32,
    pub record: bool,
    pub tally: Option<crate::sim::score::Tally>,
    /// The title's demo: the bot plays, PRESS START over it.
    pub demo: bool,
    /// The run counts for the hi-score (Stage 1 from the top).
    pub ranked: bool,
}

impl HudView<'_> {
    /// The best score to show: this run's, once it passes the saved one, if it counts.
    fn best(&self) -> u64 {
        if self.ranked { self.hiscore.max(self.score) } else { self.hiscore }
    }
}

/// What the HUD shows of Stage 1.
#[derive(Clone, Copy, Debug, Default)]
pub struct StageView {
    /// Frames since the stage began, and the section it's in.
    pub t: u32,
    pub section: usize,
    pub killed: u32,
    pub spawned: u32,
    /// Reached the end.
    pub over: bool,
    /// The derelict's health while it's here: the hull, and each turret (0 when broken).
    pub boss: Option<(f32, [f32; 3])>,
    /// Frames since the WARNING sounded.
    pub warned: Option<u32>,
    /// JAWSTHIC's bar while it's here.
    pub jaws: Option<JawsView>,
}

/// JAWSTHIC's health bar: each phase's segment (0..1 full), the phase it's in (0 in
/// the intro), seconds left in it, and how far the bar has filled (it fills as the
/// boss arrives).
#[derive(Clone, Copy, Debug, Default)]
pub struct JawsView {
    pub bar: [f32; 4],
    pub phase: u8,
    pub left: f32,
    pub fill: f32,
    /// How far through the calm between phases (0..1), if it's in one.
    pub calm: Option<f32>,
}

/// What the boss's bar shows: the phase whose gem sits in the middle (0-3), how full
/// the bar is either side of it (0..1), and how white the gem flashes. Each phase
/// gets the whole bar, which drains from both ends in to the gem; in the calm after
/// it the gem turns the next phase's color with a flash and the bar refills outward.
pub fn jaws_gauge(j: &JawsView) -> (usize, f32, f32) {
    match (j.phase, j.calm) {
        (0, _) => (0, j.fill, 0.0),
        (p, Some(c)) => {
            let refill = ((c - JAWS_REFILL.0) / (JAWS_REFILL.1 - JAWS_REFILL.0)).clamp(0.0, 1.0);
            (p as usize % 4, refill * refill * (3.0 - 2.0 * refill), (1.0 - c / JAWS_REFILL.0).max(0.0))
        }
        (p, None) => (p as usize - 1, j.bar[p as usize - 1], 0.0),
    }
}

/// The gem: a faceted diamond lit from the top left, a glint on it, a faint glow
/// round it; `flash` blanches it.
fn gem(a: &Atlas, cx: i32, cy: i32, c: Color, flash: f32) {
    let mix = |c: Color, to: Color, k: f32| Color::new(c.r + (to.r - c.r) * k, c.g + (to.g - c.g) * k, c.b + (to.b - c.b) * k, 1.0);
    let white = Color::new(1.0, 1.0, 1.0, 1.0);
    let r = GEM_R;
    for dy in -r - 1..=r + 1 {
        for dx in -r - 1..=r + 1 {
            let d = dx.abs() + dy.abs();
            let col = if d == r + 1 {
                Color { a: 0.3, ..c }
            } else if d > r {
                continue;
            } else if d == r {
                INK
            } else if d <= 1 {
                mix(c, white, 0.3)
            } else {
                match (dx > 0, dy > 0) {
                    (false, false) => mix(c, white, 0.45),
                    (true, false) => c,
                    (false, true) => mix(c, INK, 0.25),
                    (true, true) => mix(c, INK, 0.5),
                }
            };
            let col = if col.a < 1.0 { col } else { mix(col, white, flash) };
            a.fill(cx + dx, cy + dy, 1, 1, col);
        }
    }
    a.fill(cx - 1, cy - 2, 1, 1, mix(CORE, white, flash));
}

/// m:ss
fn clock(t: u32) -> String {
    let s = t / TICK_HZ;
    format!("{}:{:02}", s / 60, s % 60)
}

fn controls(mode: Mode) -> &'static [(&'static str, &'static str)] {
    match mode {
        Mode::Keys => &[
            ("ARROWS", "MOVE"),
            ("Z", "FIRE"),
            ("SHIFT", "FOCUS"),
            ("Z+SHIFT", "BEAM"),
            ("X", "BOMB"),
            ("C", "SLAM"),
            ("[ ]", "PATTERN"),
            ("ESC", "PAUSE"),
            ("M", "SOUND"),
            ("F11", "FULLSCREEN"),
        ],
        Mode::Pad => &[("STICK", "MOVE"), ("A", "FIRE"), ("RB / RT", "FOCUS"), ("A+RB", "BEAM"), ("B", "BOMB"), ("X", "SLAM"), ("LB", "PATTERN"), ("START", "PAUSE")],
        Mode::Touch => &[("DRAG", "FLY + FIRE"), ("2ND FINGER", "FOCUS"), ("NEXT", "PATTERN"), ("II", "PAUSE")],
    }
}

fn tuning_lines(s: &Settings) -> [(&'static str, String); 3] {
    use crate::menu::Item;
    [("SPEED", menu::value(s, Item::Speed)), ("FOCUS", menu::value(s, Item::Focus)), ("BLEND", menu::value(s, Item::Blend))]
}

/// The on-screen touch buttons (NEXT steps the sandbox's patterns, so not in the stage).
fn buttons(a: &Atlas, lay: &Layout, v: &HudView) {
    for (b, cx, cy, r) in touch_buttons(lay).into_iter().filter(|(b, ..)| v.stage.is_none() || *b != Button::Next) {
        let (cx, cy) = (cx as i32, cy as i32);
        let n = (r * 1.6) as i32;
        let lit = (b == Button::Bomb && v.bomb_flash > 0) || (b == Button::Slam && v.slam_flash > 0);
        for k in 0..n {
            let ang = k as f32 / n as f32 * std::f32::consts::TAU;
            a.fill(cx + (ang.cos() * r).round() as i32, cy + (ang.sin() * r).round() as i32, 1, 1, if lit { CREAM } else { SLATE });
        }
        match b {
            Button::Bomb => text_c(a, "BOMB", cx, cy - 3, 1, 1, if lit { CREAM } else { ASH }),
            Button::Slam => text_c(a, "SLAM", cx, cy - 3, 1, 1, if lit { CREAM } else { ASH }),
            Button::Next => text_c(a, "NEXT", cx, cy - 3, 1, 1, ASH),
            Button::Pause => {
                a.fill(cx - 3, cy - 4, 2, 9, ASH);
                a.fill(cx + 2, cy - 4, 2, 9, ASH);
            }
        }
    }
}

/// The stage's frame: masks over the playfield's overflow, then the panels.
pub fn stage(r: &Renderer, lay: &Layout, v: &HudView) {
    let a = &r.atlas;
    let f = lay.field;
    match lay.orient {
        Orient::Landscape => {
            a.fill(0, 0, f.x, lay.ch, VOID);
            a.fill(f.right(), 0, lay.cw - f.right(), lay.ch, VOID);
            a.fill(f.x - 1, 0, 1, lay.ch, DUSK);
            a.fill(f.right(), 0, 1, lay.ch, DUSK);

            // after the sheet: the left panel is the ship (score, lives, bombs, how
            // it's going, the controls), the right the voyage (where, the tuning, the
            // palette strip, the stamps)
            let (l0, l1) = (12, f.x - 13);
            small(a, "PILGRIM", l0, 12, CREAM);
            small(a, "TYPE-07", l0, 22, ASH);
            hrule(a, l0, l1, 34);
            let mut y = 42;
            if v.stage.is_some() {
                small(a, "SCORE", l0, y, ASH);
                score_digits(a, v.score, l0, y + 11, 2);
                small(a, "HI-SCORE", l0, y + 32, SLATE);
                score_digits(a, v.best(), l0, y + 42, 1);
                y += 58;
            } else {
                small(a, "PATTERN SANDBOX", l0, y, SLATE);
                y += 16;
            }
            hrule(a, l0, l1, y);
            small(a, "LIVES", l0, y + 8, ASH);
            icons(a, a.r.life_icon, v.lives, l1, y + 8);
            small(a, "BOMBS", l0, y + 18, ASH);
            icons(a, a.r.bomb_icon, v.bombs, l1, y + 18);
            small(a, "RUSH", l0, y + 28, ASH);
            rush_gauge(a, v.rush, l1, y + 29);
            y += 42;
            hrule(a, l0, l1, y);
            let stats: [(&str, String); 3] = match v.stage {
                Some(s) => [("GRAZE", v.grazes.to_string()), ("KILLS", format!("{}/{}", s.killed, s.spawned)), ("MISSES", v.hits.to_string())],
                None => [("BULLETS", v.bullets.to_string()), ("HITS", v.hits.to_string()), ("GRAZE", v.grazes.to_string())],
            };
            for (i, (k, val)) in stats.iter().enumerate() {
                let ry = y + 8 + i as i32 * 10;
                small(a, k, l0, ry, SLATE);
                text_r(a, val, l1, ry, ASH);
            }
            y += 40;
            let flags = sandbox_flags(&v.settings.view);
            if !flags.is_empty() {
                // set tighter than the rest, so all three fit
                a.fill(l0 - 1, y - 1, text_width(&flags, 1, 1) + 2, font::GH as i32 + 2, VOID);
                text(a, &flags, l0, y, 1, 1, CREAM);
            }
            // the pattern keys only matter in the sandbox
            let rows: Vec<&(&str, &str)> = controls(v.mode).iter().filter(|(_, act)| v.stage.is_none() || *act != "PATTERN").collect();
            let y0 = lay.ch - 14 - rows.len() as i32 * 10;
            hrule(a, l0, l1, y0 - 8);
            for (i, (k, act)) in rows.iter().enumerate() {
                let ry = y0 + i as i32 * 10;
                small(a, k, l0, ry, ASH);
                text_r(a, act, l1, ry, SLATE);
            }

            let (r0, r1) = (f.right() + 13, lay.cw - 12);
            for (i, s) in ["SOMEWHERE", "BEYOND", "THE", "EVENT HORIZON"].iter().enumerate() {
                small(a, s, r0, 12 + i as i32 * 10, ASH);
            }
            sparkle(a, r1 - 7, 12);
            hrule(a, r0, r1, 56);
            match v.stage {
                Some(s) => {
                    small(a, "STAGE 1", r0, 64, SLATE);
                    small(a, STAGE_NAME, r0, 74, SLATE);
                    small(a, "SECTION", r0, 90, ASH);
                    text_r(a, &clock(s.t), r1, 90, SLATE);
                    small(a, SECTIONS[s.section].1, r0, 102, CREAM);
                }
                None => {
                    let n = SANDBOX.len();
                    small(a, "PATTERN", r0, 64, ASH);
                    text_r(a, &format!("{}/{}", v.pattern % n + 1, n), r1, 64, SLATE);
                    small(a, SANDBOX[v.pattern % n].name, r0, 76, CREAM);
                }
            }
            hrule(a, r0, r1, 118);
            small(a, "TUNING", r0, 126, ASH);
            for (i, (k, val)) in tuning_lines(v.settings).iter().enumerate() {
                let ry = 138 + i as i32 * 10;
                small(a, k, r0, ry, SLATE);
                text_r(a, val, r1, ry, CREAM);
            }
            let hint = match v.mode {
                Mode::Keys => "1-6 TO CHANGE",
                _ => "PAUSE TO CHANGE",
            };
            small(a, hint, r0, 170, SLATE);
            strip(a, r0, 190, 12, 6);
            panel_stamps(a, r0, r1, lay.ch - 42, false);
            if v.mode == Mode::Touch {
                buttons(a, lay, v);
            }
        }
        Orient::Portrait => {
            a.fill(0, 0, lay.cw, f.y, VOID);
            a.fill(0, f.bottom(), lay.cw, lay.ch - f.bottom(), VOID);
            // the compact strip: the score, lives and bombs, the best
            if v.stage.is_some() {
                score_digits(a, v.score, 8, 8, 1);
                text_r(a, &format!("HI {:08}", v.best().min(99_999_999)), lay.cw - 30, 8, ASH);
            } else {
                small(a, "PILGRIM", 8, 8, CREAM);
                text_r(a, &version_stamp(), lay.cw - 32, 8, ASH);
            }
            icons_left(a, a.r.life_icon, v.lives, 72, 8);
            icons_left(a, a.r.bomb_icon, v.bombs, 72 + 3 * 9 + 8, 8);
            hrule(a, 0, lay.cw, f.y - 1);
            hrule(a, 0, lay.cw, f.bottom());
            let t = tuning_lines(v.settings);
            let line = format!("SPD {}  FOC {}  BLD {}", t[0].1.trim_end_matches(" PX"), t[1].1, t[2].1.trim_end_matches(" F"));
            text(a, &line, 8, f.bottom() + 8, 1, 1, SLATE);
            let n = SANDBOX.len();
            let pat = match v.stage {
                Some(s) => format!("{} {}", SECTIONS[s.section].1, clock(s.t)),
                None => format!("{} {}/{}", SANDBOX[v.pattern % n].name, v.pattern % n + 1, n),
            };
            text(a, &pat, 8, f.bottom() + 20, 1, 1, ASH);
            let hint = if v.mode == Mode::Pad { "START: PAUSE + TUNING" } else { "DRAG ANYWHERE TO FLY" };
            small(a, hint, 8, f.bottom() + 36, ASH);
            small(a, "2ND FINGER: FOCUS", 8, f.bottom() + 48, SLATE);
            let stats = match v.stage {
                Some(s) => format!("KILLS {}/{}  MISSES {}  GRAZE {}", s.killed, s.spawned, v.hits, v.grazes),
                None => format!("BULLETS {}  HITS {}  GRAZE {}", v.bullets, v.hits, v.grazes),
            };
            text(a, &stats, 8, f.bottom() + 74, 1, 1, SLATE);
            buttons(a, lay, v);
        }
    }
    if let Some(s) = v.stage {
        stage_overlays(a, lay, &s);
    }
    if let Some((msg, left)) = v.toast {
        if left > 0 && (left > 20 || left % 6 < 3) {
            text_c(a, msg, f.x + f.w / 2, f.y + 40, 1, TRACK, CREAM);
        }
    }
    let cleared = v.stage.is_some_and(|s| s.over);
    if let Some(sel) = v.over {
        if cleared {
            stage_end(a, lay, v, sel);
        } else {
            game_over(a, lay, v, sel);
        }
    } else if let Some(sel) = v.menu {
        menu_panel(a, lay, v.settings, v.mode, &ITEMS, sel, "PAUSED // TUNING");
    }
    if v.demo {
        let cx = f.x + f.w / 2;
        text_c(a, "DEMO", cx, f.y + 36, 2, 4, ASH);
        if (v.end_t / BLINK_FRAMES as u32) % 3 < 2 {
            text_c(a, "PRESS START", cx, f.bottom() - 48, 1, TRACK, CREAM);
        }
    }
}

/// Over the playfield: the stage title, the derelict's health, WARNING.
fn stage_overlays(a: &Atlas, lay: &Layout, s: &StageView) {
    let f = lay.field;
    let cx = f.x + f.w / 2;
    // the stage title, typed in, held, blinked out
    if s.t < TITLE_CARD {
        let t = s.t as i32;
        let out = TITLE_CARD as i32 - t;
        if out > 30 || out % 6 < 3 {
            let y = f.y + 132;
            let head = "STAGE 1";
            let shown = (t / 4).clamp(0, head.len() as i32) as usize;
            text_c(a, &head[..shown], cx, y, 2, 4, CREAM);
            let w = ((t - 20).clamp(0, 30) * text_width(STAGE_NAME, 1, TRACK + 1) / 30).max(0);
            if w > 0 {
                hrule(a, cx - w / 2 - 8, cx + w / 2 + 8, y + 22);
            }
            if t > 36 {
                let name = STAGE_NAME;
                let n = ((t - 36) / 3).clamp(0, name.len() as i32) as usize;
                text_c(a, &name[..n], cx, y + 30, 1, TRACK + 1, ASH);
            }
        }
    }
    // the derelict's health: the hull, and a notch for each turret still standing
    if let Some((hull, turrets)) = s.boss {
        let (x0, x1, y) = (f.x + 12, f.right() - 12, f.y + 6);
        small(a, "DERELICT", x0, y, ASH);
        let bx = x0 + text_width("DERELICT", 1, TRACK) + 6;
        let bw = x1 - bx;
        a.fill(bx, y + 2, bw, 3, DUSK);
        a.fill(bx, y + 2, (bw as f32 * hull).round() as i32, 3, CREAM);
        for (i, k) in turrets.iter().enumerate() {
            let tx = bx + i as i32 * 8;
            a.fill(tx, y + 8, 6, 2, if *k > 0.0 { SIGNAL } else { DUSK });
        }
    }
    if let Some(j) = s.jaws {
        jaws_bar(a, lay, &j, s.t);
    }
    // WARNING: red bands crawling across, the word blinking
    if let Some(w) = s.warned.filter(|w| *w < 8 * TICK_HZ) {
        let w = w as i32;
        let y = f.y + f.h / 2 - 24;
        for band in [y, y + 40] {
            a.fill(f.x, band, f.w, 1, SIGNAL);
            a.fill(f.x, band + 7, f.w, 1, SIGNAL);
            for k in 0..f.w / 8 + 2 {
                let x = f.x + k * 8 - (w / 2) % 8;
                for d in 0..4 {
                    if x + d + 1 >= f.x && x + d + 1 < f.right() {
                        a.fill(x + d + 1, band + 2 + d.min(3), 1, 1, Color { a: 0.7, ..SIGNAL });
                    }
                }
            }
        }
        if w % 40 < 28 {
            text_c(a, "WARNING", cx, y + 15, 2, 4, SIGNAL);
        }
    }
}

/// JAWSTHIC's bar along the top of the playfield: its name, then the phase's bar
/// draining from both ends in to the gem in its middle (the gem's color says which
/// phase), the phase's name under it, and the seconds left, red in the last ten.
fn jaws_bar(a: &Atlas, lay: &Layout, j: &JawsView, t: u32) {
    let f = lay.field;
    let (x0, x1, y) = (f.x + 10, f.right() - 10, f.y + 6);
    small(a, BOSS_NAME, x0, y, ASH);
    let bx = x0 + text_width(BOSS_NAME, 1, TRACK) + 6;
    let bar_end = x1 - text_width("00", 1, TRACK) - 6;
    let cx = (bx + bar_end) / 2;
    let (shown, full, flash) = jaws_gauge(j);
    // each side: its track from the gem out, lit from the gem out as far as it's full
    let side = (bar_end - bx) / 2 - GEM_R - 3;
    let lit = (side as f32 * full).round() as i32;
    let (l0, r0) = (cx - GEM_R - 3, cx + GEM_R + 4);
    a.fill(l0 - side, y + 2, side, 3, DUSK);
    a.fill(r0, y + 2, side, 3, DUSK);
    a.fill(l0 - lit, y + 2, lit, 3, CREAM);
    a.fill(r0, y + 2, lit, 3, CREAM);
    if lit > 0 {
        // bright caps on the ends as they close in
        for x in [l0 - lit, r0 + lit - 1] {
            a.fill(x, y + 1, 1, 5, Color { a: 0.8, ..CREAM });
        }
    }
    gem(a, cx, y + 3, GEM[shown], flash);
    if j.phase > 0 {
        small(a, JAWS_PHASE_NAMES[shown], bx, y + 10, SLATE);
        let secs = j.left.ceil() as i32;
        let urgent = j.left <= 10.0;
        if !urgent || t % 30 < 22 {
            text_r(a, &format!("{secs:02}"), x1, y, if urgent { SIGNAL } else { CREAM });
        }
    }
}

/// The stage-clear tally's geometry: (panel, first menu row's y, row height).
pub fn end_box(lay: &Layout) -> (IRect, i32, i32) {
    let f = lay.field;
    let h = 252;
    let panel = IRect::new(f.x + 24, f.y + (f.h - h) / 2, f.w - 48, h);
    (panel, panel.y + h - 30, 14)
}

/// Which row of the game-over or stage-end menu a tap lands on.
pub fn ended_tap(lay: &Layout, cleared: bool, x: f32, y: f32) -> Option<usize> {
    if !cleared {
        return over_tap(lay, x, y);
    }
    let (p, row0, row_h) = end_box(lay);
    let (x, y) = (x as i32, y as i32);
    if x < p.x || x >= p.right() || y < row0 - 4 {
        return None;
    }
    let i = ((y - row0 + 4) / row_h) as usize;
    (i < OVER_ITEMS.len()).then_some(i)
}

/// `n` counted up from 0 over `TALLY_COUNT` frames, starting at frame `from`.
fn counted(n: u64, e: u32, from: u32) -> u64 {
    let k = (e.saturating_sub(from) as f32 / TALLY_COUNT as f32).min(1.0);
    (n as f64 * k as f64) as u64
}

/// The tally, arcade style, `e` frames after the stage ended: the lines reveal one
/// by one, counting up; the boss's grades pop in; the score counts up; the rank
/// slams in with its flare; the last line types in; then the menu. Any button
/// before then skips to the end.
fn stage_end(a: &Atlas, lay: &Layout, v: &HudView, sel: usize) {
    let Some(t) = v.tally else { return };
    let e = v.end_t;
    let (p, row0, row_h) = end_box(lay);
    let cx = p.x + p.w / 2;
    let (x0, x1) = (p.x + 14, p.right() - 14);
    a.fill(p.x, p.y, p.w, p.h, Color { a: 0.92, ..VOID });
    hrule(a, p.x, p.right(), p.y);
    hrule(a, p.x, p.right(), p.bottom() - 1);
    text_c(a, STAGE_NAME, cx, p.y + 10, 2, 3, CREAM);
    let verdict = if t.destroyed { format!("{BOSS_NAME} DESTROYED") } else { format!("{BOSS_NAME} LOST TO THE HOLE") };
    text_c(a, &verdict, cx, p.y + 30, 1, 1, if t.destroyed { CREAM } else { ASH });
    hrule(a, x0, x1, p.y + 44);
    let row = |i: u32| p.y + 52 + i as i32 * 12;
    let at = |i: u32| TALLY_STEP * (i + 1);
    let line = |i: u32, k: &str, val: &str, c: Color| {
        if e >= at(i) {
            text(a, k, x0, row(i), 1, 1, SLATE);
            text(a, val, x1 - text_width(val, 1, 1), row(i), 1, 1, c);
        }
    };
    let pct = t.killed * 100 / t.spawned.max(1);
    line(0, "KILLS", &format!("{}/{}  {}%", counted(t.killed as u64, e, at(0)), t.spawned, pct), CREAM);
    // the boss's phases, each grade popping in
    if e >= at(1) {
        text(a, BOSS_NAME, x0, row(1), 1, 1, SLATE);
        for (i, g) in t.grades.iter().enumerate() {
            let pop = at(1) + 4 * i as u32;
            if e < pop {
                continue;
            }
            let gx = x1 - (3 - i as i32) * 12 - 5;
            let (letter, c) = g.map_or(("-", SLATE), |g| (g.letter(), super::popups::grade_color(g)));
            let s = if e - pop < 2 { 2 } else { 1 };
            text(a, letter, gx - (s - 1) * 2, row(1) - (s - 1) * 3, s, 0, c);
        }
    }
    let bonus = |n: u32, i: u32| format!("+{}", super::popups::commas(counted(n as u64, e, at(i))));
    if t.no_miss > 0 {
        line(2, "NO MISS", &bonus(t.no_miss, 2), TEAL_HOT);
    } else {
        line(2, &format!("MISSES {}", t.misses), "+0", SLATE);
    }
    if t.no_bomb > 0 {
        line(3, "NO BOMB", &bonus(t.no_bomb, 3), TEAL_HOT);
    } else {
        line(3, &format!("BOMBS {}", t.bombs), "+0", SLATE);
    }
    line(4, "GRAZE", &counted(t.grazes as u64, e, at(4)).to_string(), CREAM);
    if e >= TALLY_SCORE {
        hrule(a, x0, x1, p.y + 114);
        text(a, "SCORE", x0, p.y + 122, 1, 1, ASH);
        let sc = super::popups::commas(counted(t.total, e, TALLY_SCORE));
        text(a, &sc, x1 - text_width(&sc, 1, 1), p.y + 122, 1, 1, CREAM);
        // a new record climbs with the score
        let hi = if v.record { sc.clone() } else { super::popups::commas(v.best()) };
        let label = if v.record { "NEW RECORD" } else { "HI-SCORE" };
        let lit = !v.record || e < TALLY_RANK || e % 20 < 14;
        if lit {
            text(a, label, x0, p.y + 134, 1, 1, if v.record { TEAL_HOT } else { SLATE });
        }
        text(a, &hi, x1 - text_width(&hi, 1, 1), p.y + 134, 1, 1, if v.record { TEAL_HOT } else { ASH });
    }
    if e >= TALLY_RANK {
        text(a, "RANK", cx - 64, p.y + 168, 1, TRACK, ASH);
        super::popups::slam_letter(a, cx + 6, p.y + 172, t.rank, e - TALLY_RANK, 1.0);
    }
    if e >= TALLY_LINE {
        let line = "SOMEWHERE BEYOND THE EVENT HORIZON";
        let n = ((e - TALLY_LINE) as usize / 2).min(line.len());
        let w = text_width(line, 1, 1);
        text(a, &line[..n], cx - w / 2, p.y + 202, 1, 1, ASH);
    }
    if e >= TALLY_DONE {
        for (i, label) in OVER_ITEMS.iter().enumerate() {
            let y = row0 + i as i32 * row_h;
            let on = i == sel;
            if on {
                a.fill(p.x + 4, y - 3, p.w - 8, row_h - 3, Color { a: 0.5, ..DUSK });
                text(a, ">", p.x + 10, y, 1, 1, CREAM);
            }
            text(a, label, p.x + 22, y, 1, 1, if on { CREAM } else { ASH });
        }
    }
}

/// `n` icons ending at `right` (the side panel's right edge).
fn icons(a: &Atlas, icon: Rect, n: u8, right: i32, y: i32) {
    for i in 0..n as i32 {
        a.blit(icon, right - 7 - i * 9, y, 1, WHITE_A);
    }
}

/// The rush gauge, right-aligned at `right`: a segment a slam link, teal when full,
/// filling as kills come in.
fn rush_gauge(a: &Atlas, rush: f32, right: i32, y: i32) {
    for i in 0..crate::tune::SLAM_LINKS as i32 {
        let x = right - (crate::tune::SLAM_LINKS as i32 - i) * 9 + 2;
        let fill = (rush - i as f32).clamp(0.0, 1.0);
        a.fill(x, y, 7, 5, SLATE);
        a.fill(x + 1, y + 1, 5, 3, VOID);
        let w = (fill * 5.0).floor() as i32;
        if w > 0 {
            a.fill(x + 1, y + 1, w, 3, if fill >= 1.0 { TEAL } else { ASH });
        }
    }
}

/// `n` icons starting at `x`.
fn icons_left(a: &Atlas, icon: Rect, n: u8, x: i32, y: i32) {
    for i in 0..n.min(3) as i32 {
        a.blit(icon, x + i * 9, y, 1, WHITE_A);
    }
}

/// The game-over menu's rows.
pub const OVER_ITEMS: [&str; 2] = ["RETRY", "QUIT TO TITLE"];

/// The game-over panel's geometry: (panel, first row's y, row height).
pub fn over_box(lay: &Layout) -> (IRect, i32, i32) {
    let f = lay.field;
    let panel = IRect::new(f.x + 40, f.y + f.h / 2 - 60, f.w - 80, 120);
    (panel, panel.y + 80, 16)
}

/// Which game-over row a tap at canvas `(x, y)` lands on.
pub fn over_tap(lay: &Layout, x: f32, y: f32) -> Option<usize> {
    let (p, row0, row_h) = over_box(lay);
    let (x, y) = (x as i32, y as i32);
    if x < p.x || x >= p.right() || y < row0 - 4 {
        return None;
    }
    let i = ((y - row0 + 4) / row_h) as usize;
    (i < OVER_ITEMS.len()).then_some(i)
}

fn game_over(a: &Atlas, lay: &Layout, v: &HudView, sel: usize) {
    let (p, row0, row_h) = over_box(lay);
    let (x0, x1) = (p.x + 14, p.right() - 14);
    a.fill(p.x, p.y, p.w, p.h, Color { a: 0.92, ..VOID });
    hrule(a, p.x, p.right(), p.y);
    hrule(a, p.x, p.right(), p.bottom() - 1);
    text_c(a, "GAME OVER", p.x + p.w / 2, p.y + 12, 2, 3, CREAM);
    if v.stage.is_some() {
        let sc = super::popups::commas(counted(v.score, v.end_t, 0));
        text(a, "SCORE", x0, p.y + 38, 1, 1, ASH);
        text(a, &sc, x1 - text_width(&sc, 1, 1), p.y + 38, 1, 1, CREAM);
        let hi = super::popups::commas(v.best());
        if !v.record || v.end_t % 20 < 14 {
            text(a, if v.record { "NEW RECORD" } else { "HI-SCORE" }, x0, p.y + 50, 1, 1, if v.record { TEAL_HOT } else { SLATE });
        }
        text(a, &hi, x1 - text_width(&hi, 1, 1), p.y + 50, 1, 1, if v.record { TEAL_HOT } else { ASH });
    } else {
        text_c(a, &format!("HITS {}  GRAZE {}", v.hits, v.grazes), p.x + p.w / 2, p.y + 40, 1, 1, SLATE);
    }
    hrule(a, x0, x1, p.y + 66);
    for (i, label) in OVER_ITEMS.iter().enumerate() {
        let y = row0 + i as i32 * row_h;
        let on = i == sel;
        if on {
            a.fill(p.x + 4, y - 3, p.w - 8, row_h - 3, Color { a: 0.5, ..DUSK });
            text(a, ">", p.x + 10, y, 1, 1, CREAM);
        }
        text(a, label, p.x + 22, y, 1, 1, if on { CREAM } else { ASH });
    }
}

/// A menu panel's geometry, shared by drawing and tapping.
pub struct MenuBox {
    pub panel: IRect,
    pub row0: i32,
    pub row_h: i32,
    /// The value column: taps left of its middle step down, right of it step up.
    pub val0: i32,
    pub val1: i32,
}

/// The panel for a menu of `n` rows (the pause menu, the options).
pub fn menu_box(lay: &Layout, n: usize) -> MenuBox {
    let f = lay.field;
    let row_h = 14;
    let h = 32 + n as i32 * row_h + 22;
    let panel = IRect::new(f.x + 14, f.y + (f.h - h) / 2, f.w - 28, h);
    MenuBox { panel, row0: panel.y + 28, row_h, val0: panel.x + panel.w / 2 - 10, val1: panel.right() - 8 }
}

/// What a tap at canvas `(x, y)` means in a menu of `items`: (row, step) where step
/// is -1 or 1 on a value, 0 elsewhere on the row.
pub fn menu_tap(lay: &Layout, items: &[menu::Item], x: f32, y: f32) -> Option<(usize, i32)> {
    let m = menu_box(lay, items.len());
    let (x, y) = (x as i32, y as i32);
    if x < m.panel.x || x >= m.panel.right() || y < m.row0 - 4 {
        return None;
    }
    let row = ((y - m.row0 + 4) / m.row_h) as usize;
    let item = *items.get(row)?;
    let step = if item.adjustable() && x >= m.val0 - 8 {
        if x < (m.val0 + m.val1) / 2 { -1 } else { 1 }
    } else {
        0
    };
    Some((row, step))
}

/// A menu panel: its title, a row per item with its value between < and >.
fn menu_panel(a: &Atlas, lay: &Layout, s: &Settings, mode: Mode, items: &[menu::Item], sel: usize, title: &str) {
    let m = menu_box(lay, items.len());
    let p = m.panel;
    a.fill(p.x, p.y, p.w, p.h, Color { a: 0.94, ..VOID });
    hrule(a, p.x, p.right(), p.y);
    hrule(a, p.x, p.right(), p.bottom() - 1);
    text(a, title, p.x + 10, p.y + 10, 1, TRACK, CREAM);
    for (i, item) in items.iter().enumerate() {
        let y = m.row0 + i as i32 * m.row_h;
        let on = i == sel;
        let c = if on { CREAM } else { ASH };
        if on {
            a.fill(p.x + 4, y - 3, p.w - 8, m.row_h - 3, Color { a: 0.5, ..DUSK });
            text(a, ">", p.x + 6, y, 1, 1, CREAM);
        }
        text(a, item.label(), p.x + 16, y, 1, 1, c);
        if item.adjustable() {
            let val = menu::value(s, *item);
            let cxv = (m.val0 + m.val1) / 2;
            let w = text_width(&val, 1, 1);
            text(a, &val, cxv - w / 2, y, 1, 1, c);
            text(a, "<", m.val0 - 4, y, 1, 1, if on { CREAM } else { SLATE });
            text(a, ">", m.val1 - 5, y, 1, 1, if on { CREAM } else { SLATE });
        }
    }
    let hint = match mode {
        Mode::Keys => "ARROWS CHANGE  Z OK  ESC BACK",
        Mode::Pad => "D-PAD CHANGE  A OK  B BACK",
        Mode::Touch => "TAP < > TO CHANGE",
    };
    text(a, hint, p.x + 10, p.bottom() - 14, 1, 1, SLATE);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tune::{CANVAS_H, CANVAS_W, FIELD_W, PORTRAIT_W};

    /// The boss's bar: each phase gets the whole bar, draining from both ends in to
    /// the gem; at zero the calm turns the gem the next phase's color with a flash and
    /// refills the bar outward; the gems run green to red.
    #[test]
    fn the_boss_bar_drains_to_its_gem_and_the_gem_turns_each_phase() {
        let view = |phase: u8, bar: [f32; 4], calm: Option<f32>| JawsView { bar, phase, left: 20.0, fill: 1.0, calm };
        assert_eq!(jaws_gauge(&view(1, [1.0, 1.0, 1.0, 1.0], None)), (0, 1.0, 0.0), "a fresh phase: full, the first gem");
        assert_eq!(jaws_gauge(&view(1, [0.4, 1.0, 1.0, 1.0], None)), (0, 0.4, 0.0), "both ends drain together");
        assert_eq!(jaws_gauge(&view(2, [0.0, 0.25, 1.0, 1.0], None)).1, 0.25, "phase 2's own bar, not the whole fight's");
        let (gem, full, flash) = jaws_gauge(&view(1, [0.0, 1.0, 1.0, 1.0], Some(0.0)));
        assert_eq!((gem, full), (1, 0.0), "at zero the next phase's gem, the bar empty");
        assert!(flash > 0.9, "the gem flashes as it turns");
        let mid = jaws_gauge(&view(1, [0.0, 1.0, 1.0, 1.0], Some(0.5))).1;
        assert!(mid > 0.0 && mid < 1.0, "it refills through the calm: {mid}");
        assert_eq!(jaws_gauge(&view(1, [0.0, 1.0, 1.0, 1.0], Some(0.99))).1, 1.0, "full by the next phase");
        assert_eq!(jaws_gauge(&view(4, [0.0, 0.0, 0.0, 0.7], None)).0, 3, "the last phase's gem");
        assert_eq!(jaws_gauge(&view(0, [1.0; 4], None)), (0, 1.0, 0.0), "arriving, the bar fills as it breaches");
        let g = GEM[0];
        assert!(g.g > g.r && g.g > g.b, "it starts green");
        assert_eq!(GEM[3], SIGNAL, "and ends red");
        assert!(GEM.windows(2).all(|w| w[0] != w[1]), "every phase its own color");
    }

    #[test]
    fn the_stamp_reads_like_the_sheet() {
        let v = version_stamp();
        assert!(v.starts_with('V') && v.ends_with(" // PILGRIM"), "{v}");
        let ver = env!("CARGO_PKG_VERSION");
        let minor = ver.split('.').take(2).collect::<Vec<_>>().join(".");
        assert_eq!(v, format!("V{minor} // PILGRIM"));
    }

    #[test]
    fn title_lines_fit_their_layouts() {
        let land = (CANVAS_W as i32) - 32;
        for s in ["HUMAN PILOTED CRAFT // DEEP SPACE PROTOCOL", "SOMEWHERE BEYOND THE EVENT HORIZON", "PRESS Z  //  TAP TO START"] {
            assert!(text_width(s, 1, TRACK) <= land, "{s}");
        }
        let port = (PORTRAIT_W as i32) - 24;
        assert!(text_width("PILGRIM", TITLE_SCALE, TITLE_TRACK) <= port);
        for s in ["HUMANITY STILL LOOKS UP", "THE EVENT HORIZON", "A LONG WAY FROM HOME", "DRAG ANYWHERE TO FLY", "START: PAUSE + TUNING"] {
            assert!(text_width(s, 1, TRACK) <= port, "{s}");
        }
    }

    #[test]
    fn panel_text_fits_the_side_panels() {
        // the landscape side panels, with 12 px margins and 13 by the playfield
        let room = (CANVAS_W - FIELD_W) as i32 / 2 - 25;
        assert_eq!(room, 115);
        let stamp = version_stamp();
        let lines = LEFT_STAMP.iter().chain(RIGHT_STAMP.iter()).copied();
        for s in lines.chain(["PATTERN SANDBOX", "STAGE 1", STAGE_NAME, "PAUSE TO CHANGE", "EVENT HORIZON", &stamp]) {
            assert!(text_width(s, 1, TRACK) <= room, "{s}");
        }
        assert!(text_width("BOT SLOW INVINCIBLE", 1, 1) <= room, "the flags, set tighter");
        for (_, name) in SECTIONS {
            assert!(text_width(name, 1, TRACK) <= room, "{name}");
        }
        let port = PORTRAIT_W as i32;
        assert!(text_width("KILLS 999/999  MISSES 99  GRAZE 9999", 1, 1) <= port - 16);
        for p in SANDBOX {
            assert!(text_width(p.name, 1, TRACK) <= room, "{}", p.name);
            // the portrait thumb zone line, beside the buttons
            assert!(text_width(&format!("{} 13/13", p.name), 1, 1) <= port - 8 - 100, "{}", p.name);
            assert!(text_width("BULLETS 8192  HITS 999  GRAZE 9999", 1, 1) <= port - 16);
        }
        // the left panel's rows (the score, lives/bombs/rush, the stats, then the
        // sandbox flags) clear the controls listed under them
        let most = controls(Mode::Keys).len() as i32;
        let flags = 42 + 58 + 42 + 40;
        assert!(flags + font::GH as i32 + 2 <= CANVAS_H as i32 - 14 - most * 10 - 8, "the panel's rows and the controls overlap");
        for mode in [Mode::Keys, Mode::Pad, Mode::Touch] {
            for (k, act) in controls(mode) {
                assert!(text_width(k, 1, TRACK) + 8 + text_width(act, 1, TRACK) <= room, "{k} {act}");
            }
        }
    }

    /// Every label the HUD draws uses glyphs the font has (a missing one draws as a gap).
    #[test]
    fn every_label_has_its_glyphs() {
        let ok = |s: &str| s.chars().all(|c| c == ' ' || font::CHARS.contains(c.to_ascii_uppercase()));
        let mut all: Vec<String> = vec![];
        for mode in [Mode::Keys, Mode::Pad, Mode::Touch] {
            for (k, act) in controls(mode) {
                all.push(k.to_string());
                all.push(act.to_string());
            }
        }
        all.extend(SANDBOX.iter().map(|p| p.name.to_string()));
        all.extend(SECTIONS.iter().map(|s| s.1.to_string()));
        all.extend(title_items(true).iter().map(|i| i.label().to_string()));
        all.extend(["DERELICT", "WARNING", "MISSES 3", "BOMBS 2", "NO MISS", "NO BOMB", "KILLS", "GRAZE", "SCORE", "HI-SCORE", "NEW RECORD", "RANK", "DEMO", "PRESS START", "OPTIONS", "+12,345", "SOMEWHERE BEYOND THE EVENT HORIZON", BOSS_NAME, STAGE_NAME].map(String::from));
        all.extend(JAWS_PHASE_NAMES.iter().map(|s| s.to_string()));
        all.extend([format!("{BOSS_NAME} DESTROYED"), format!("{BOSS_NAME} LOST TO THE HOLE")]);
        all.extend(ITEMS.iter().chain(menu::OPTIONS.iter()).map(|i| i.label().to_string()));
        let mut s = crate::storage::Settings::default();
        all.extend(ITEMS.iter().chain(menu::OPTIONS.iter()).map(|i| menu::value(&s, *i)));
        (s.hold_beam, s.hitbox_always, s.music_vol) = (true, true, 0);
        all.extend(menu::OPTIONS.iter().map(|i| menu::value(&s, *i)));
        all.extend(["S", "A", "B", "C", "D", "CAPTURED", "BROKEN", "TIME OUT", "EXTEND", "1UP"].map(String::from));
        for t in all {
            assert!(ok(&t), "{t:?} has a character the font can't draw");
        }
    }

    #[test]
    fn the_game_over_menu_fits_and_taps_land() {
        for (w, h) in [(1280, 720), (390, 844)] {
            let lay = Layout::fit(w, h);
            let (p, row0, row_h) = over_box(&lay);
            assert!(p.x >= lay.field.x && p.right() <= lay.field.right() && p.bottom() <= lay.field.bottom());
            for i in 0..OVER_ITEMS.len() {
                assert_eq!(over_tap(&lay, (p.x + 30) as f32, (row0 + i as i32 * row_h + 2) as f32), Some(i));
            }
            assert!(text_width("GAME OVER", 2, 3) <= p.w);
        }
    }

    #[test]
    fn the_stage_panels_fit_and_taps_land() {
        for (w, h) in [(1280, 720), (390, 844)] {
            let lay = Layout::fit(w, h);
            let (p, row0, row_h) = end_box(&lay);
            assert!(p.x >= lay.field.x && p.right() <= lay.field.right() && p.y >= lay.field.y && p.bottom() <= lay.field.bottom());
            assert!(row0 + 2 * row_h <= p.bottom());
            for i in 0..OVER_ITEMS.len() {
                assert_eq!(ended_tap(&lay, true, (p.x + 30) as f32, (row0 + i as i32 * row_h + 2) as f32), Some(i));
            }
            assert!(text_width(STAGE_NAME, 2, 3) <= p.w && text_width("SOMEWHERE BEYOND THE EVENT HORIZON", 1, 1) <= p.w - 8);
            assert!(text_width("WARNING", 2, 4) <= lay.field.w - 20);
            // the rank's letter, at its biggest, stays inside the panel
            assert!(text_width("S", super::super::popups::slam_scale(0), 0) <= p.w);
            let (cx, y0, rh) = title_rows(&lay);
            let items = title_items(true);
            for i in 0..items.len() {
                assert_eq!(title_tap(&lay, items.len(), cx as f32, (y0 + i as i32 * rh + 2) as f32), Some(i));
            }
        }
    }

    #[test]
    fn the_menu_fits_and_taps_land_on_rows() {
        for (w, h) in [(1280, 720), (390, 844), (844, 390)] {
            let lay = Layout::fit(w, h);
            for items in [&ITEMS[..], &menu::OPTIONS[..]] {
                let m = menu_box(&lay, items.len());
                assert!(m.panel.y >= lay.field.y && m.panel.bottom() <= lay.field.bottom());
                for (i, item) in items.iter().enumerate() {
                    let y = (m.row0 + i as i32 * m.row_h + 3) as f32;
                    assert_eq!(menu_tap(&lay, items, (m.panel.x + 20) as f32, y), Some((i, 0)));
                    if item.adjustable() {
                        assert_eq!(menu_tap(&lay, items, m.val0 as f32, y), Some((i, -1)));
                        assert_eq!(menu_tap(&lay, items, (m.val1 - 2) as f32, y), Some((i, 1)));
                    }
                }
            }
            let m = menu_box(&lay, ITEMS.len());
            for mode in [Mode::Keys, Mode::Pad, Mode::Touch] {
                let hint = match mode {
                    Mode::Keys => "ARROWS CHANGE  Z OK  ESC BACK",
                    Mode::Pad => "D-PAD CHANGE  A OK  B BACK",
                    Mode::Touch => "TAP < > TO CHANGE",
                };
                assert!(text_width(hint, 1, 1) <= m.panel.w - 20);
            }
        }
    }
}
