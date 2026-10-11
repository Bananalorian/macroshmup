//! Type and screen furniture, styled after the PILGRIM sheet: spaced capitals,
//! thin rules, small labels, a version stamp bottom-left and a tagline bottom-right.

use super::Renderer;
use super::art::Atlas;
use super::font::{self, text_width};
use super::layout::{IRect, Layout, Orient};
use super::palette::{ASH, CREAM, DUSK, SIGNAL, SLATE, STRIP, VOID, WHITE_A, tone};
use macroquad::math::Rect;
use crate::input::{Button, Mode, touch_buttons};
use crate::menu::{self, ITEMS};
use crate::storage::Settings;
use crate::storage::View;
use crate::tune::{BLINK_FRAMES, BOSS_NAME, JAWS_PHASES, JAWS_PHASE_NAMES, SANDBOX, SECTIONS, STAGE_NAME, TICK_HZ, TITLE_CARD, TITLE_SCALE, TITLE_TRACK, TRACK};

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

/// The title's choices.
pub const TITLE_ITEMS: [&str; 2] = ["STAGE 1", "PATTERN SANDBOX"];

/// The title menu's geometry: (center x, first row's y, row height).
pub fn title_rows(lay: &Layout) -> (i32, i32, i32) {
    match lay.orient {
        Orient::Landscape => (lay.cw / 2, 226, 13),
        Orient::Portrait => (lay.cw / 2, 290, 15),
    }
}

/// Which title row a tap at canvas `(x, y)` lands on.
pub fn title_tap(lay: &Layout, x: f32, y: f32) -> Option<usize> {
    let (cx, y0, row_h) = title_rows(lay);
    let (x, y) = (x as i32, y as i32);
    if (x - cx).abs() > 90 || y < y0 - 4 {
        return None;
    }
    let i = ((y - y0 + 4) / row_h) as usize;
    (i < TITLE_ITEMS.len()).then_some(i)
}

fn title_menu(a: &Atlas, lay: &Layout, sel: usize, blink_on: bool) {
    let (cx, y0, row_h) = title_rows(lay);
    for (i, label) in TITLE_ITEMS.iter().enumerate() {
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

/// The title: the name, the tagline, the menu (drawn over the post pass).
pub fn title(r: &Renderer, lay: &Layout, t: u64, mode: Option<Mode>, sel: usize) {
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

            let ty = 132;
            text_c(a, "PILGRIM", cx, ty, TITLE_SCALE, TITLE_TRACK, CREAM);
            let w = text_width("PILGRIM", TITLE_SCALE, TITLE_TRACK);
            hrule(a, cx - w / 2, cx + w / 2, ty + 32);
            text_c(a, "SOMEWHERE BEYOND THE EVENT HORIZON", cx, ty + 42, 1, TRACK, ASH);
            title_menu(a, lay, sel, blink_on);
            text_c(a, start_hint(mode), cx, 260, 1, TRACK, SLATE);
            stamps(a, m, lay.cw - m, lay.ch - 32, true);
        }
        Orient::Portrait => {
            let m = 12;
            small(a, "TYPE-07  \"PILGRIM\"", m, m, ASH);
            strip(a, m, m + 12, 10, 4);
            sparkle(a, lay.cw - m - 7, m);

            let ty = 176;
            text_c(a, "PILGRIM", cx, ty, TITLE_SCALE, TITLE_TRACK, CREAM);
            let w = text_width("PILGRIM", TITLE_SCALE, TITLE_TRACK);
            hrule(a, cx - w / 2, cx + w / 2, ty + 32);
            text_c(a, "SOMEWHERE BEYOND", cx, ty + 42, 1, TRACK, ASH);
            text_c(a, "THE EVENT HORIZON", cx, ty + 52, 1, TRACK, ASH);
            title_menu(a, lay, sel, blink_on);
            text_c(a, if mode == Some(Mode::Pad) { "PRESS A TO START" } else { "TAP TO START" }, cx, 330, 1, TRACK, SLATE);
            stamps(a, m, lay.cw - m, lay.ch - 32, false);
        }
    }
}

/// Everything the stage HUD shows besides the playfield.
pub struct HudView<'a> {
    pub settings: &'a Settings,
    pub mode: Mode,
    /// The pause menu's selected row, when paused.
    pub menu: Option<usize>,
    pub toast: Option<(&'a str, u32)>,
    pub bomb_flash: u32,
    /// Ships left (including the one flying) and bombs.
    pub lives: u8,
    pub bombs: u8,
    /// The game-over menu's selected row, once the last ship is gone.
    pub over: Option<usize>,
    /// The sandbox: pattern number, bullets on screen, hits, grazes.
    pub pattern: usize,
    pub bullets: usize,
    pub hits: u32,
    pub grazes: u32,
    /// Stage 1's state (none in the sandbox).
    pub stage: Option<StageView>,
}

/// What the HUD shows of Stage 1.
#[derive(Clone, Copy, Debug, Default)]
pub struct StageView {
    /// Frames since the stage began, and the section it's in.
    pub t: u32,
    pub section: usize,
    pub killed: u32,
    pub spawned: u32,
    pub bombs_used: u32,
    /// Reached the end.
    pub over: bool,
    /// The derelict's health while it's here: the hull, and each turret (0 when broken).
    pub boss: Option<(f32, [f32; 3])>,
    /// Frames since the WARNING sounded.
    pub warned: Option<u32>,
    /// JAWSTHIC's bar while it's here.
    pub jaws: Option<JawsView>,
    /// How the boss fight ended: destroyed (or lost to the hole), phases broken, and
    /// phases got through without a death or a bomb.
    pub jaws_end: Option<(bool, u8, u8)>,
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
            ("[ ]", "PATTERN"),
            ("ESC", "PAUSE"),
            ("M", "SOUND"),
            ("F11", "FULLSCREEN"),
        ],
        Mode::Pad => &[("STICK", "MOVE"), ("A", "FIRE"), ("RB / RT", "FOCUS"), ("A+RB", "BEAM"), ("B", "BOMB"), ("LB", "PATTERN"), ("START", "PAUSE")],
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
        let lit = b == Button::Bomb && v.bomb_flash > 0;
        for k in 0..n {
            let ang = k as f32 / n as f32 * std::f32::consts::TAU;
            a.fill(cx + (ang.cos() * r).round() as i32, cy + (ang.sin() * r).round() as i32, 1, 1, if lit { CREAM } else { SLATE });
        }
        match b {
            Button::Bomb => text_c(a, "BOMB", cx, cy - 3, 1, 1, if lit { CREAM } else { ASH }),
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

            let (l0, l1) = (12, f.x - 13);
            small(a, "PILGRIM", l0, 12, CREAM);
            small(a, "TYPE-07", l0, 22, ASH);
            hrule(a, l0, l1, 34);
            let place = match v.stage {
                Some(_) => format!("STAGE 1  //  {STAGE_NAME}"),
                None => "PATTERN SANDBOX".to_string(),
            };
            small(a, &place, l0, 42, SLATE);
            small(a, "LIVES", l0, 56, ASH);
            icons(a, a.r.life_icon, v.lives, l1, 56);
            small(a, "BOMBS", l0, 66, ASH);
            icons(a, a.r.bomb_icon, v.bombs, l1, 66);
            if let Some(s) = v.stage {
                small(a, "SECTION", l0, 82, ASH);
                text_r(a, &clock(s.t), l1, 82, SLATE);
                small(a, SECTIONS[s.section].1, l0, 94, CREAM);
                small(a, "KILLS", l0, 106, SLATE);
                text_r(a, &format!("{}/{}", s.killed, s.spawned), l1, 106, ASH);
                small(a, "MISSES", l0, 116, SLATE);
                text_r(a, &v.hits.to_string(), l1, 116, ASH);
            } else {
                let n = SANDBOX.len();
                small(a, "PATTERN", l0, 82, ASH);
                text_r(a, &format!("{}/{}", v.pattern % n + 1, n), l1, 82, SLATE);
                small(a, SANDBOX[v.pattern % n].name, l0, 94, CREAM);
                small(a, "BULLETS", l0, 106, SLATE);
                text_r(a, &v.bullets.to_string(), l1, 106, ASH);
                small(a, "HITS", l0, 116, SLATE);
                text_r(a, &v.hits.to_string(), l1, 116, ASH);
            }
            small(a, "GRAZE", l0, 126, SLATE);
            text_r(a, &v.grazes.to_string(), l1, 126, ASH);
            let flags = sandbox_flags(&v.settings.view);
            if !flags.is_empty() {
                small(a, &flags, l0, 138, CREAM);
            }
            small(a, "TUNING", l0, 152, ASH);
            for (i, (k, val)) in tuning_lines(v.settings).iter().enumerate() {
                let y = 164 + i as i32 * 10;
                small(a, k, l0, y, SLATE);
                text_r(a, val, l1, y, CREAM);
            }
            let hint = match v.mode {
                Mode::Keys => "1-6 TO CHANGE",
                _ => "PAUSE TO CHANGE",
            };
            small(a, hint, l0, 196, SLATE);
            // the pattern keys only matter in the sandbox
            let rows: Vec<&(&str, &str)> = controls(v.mode).iter().filter(|(_, act)| v.stage.is_none() || *act != "PATTERN").collect();
            let y0 = lay.ch - 42 - rows.len() as i32 * 10;
            for (i, (k, act)) in rows.iter().enumerate() {
                let y = y0 + i as i32 * 10;
                small(a, k, l0, y, ASH);
                text_r(a, act, l1, y, SLATE);
            }
            stamps(a, l0, l1, lay.ch - 32, false);

            let (r0, r1) = (f.right() + 13, lay.cw - 12);
            for (i, s) in ["SOMEWHERE", "BEYOND", "THE", "EVENT HORIZON"].iter().enumerate() {
                small(a, s, r0, 12 + i as i32 * 10, ASH);
            }
            sparkle(a, r1 - 7, 12);
            hrule(a, r0, r1, lay.ch - 32);
            text_r(a, "A SMALL SHIP", r1, lay.ch - 26, ASH);
            text_r(a, "A LONG WAY FROM HOME", r1, lay.ch - 16, SLATE);
            if v.mode == Mode::Touch {
                buttons(a, lay, v);
            }
        }
        Orient::Portrait => {
            a.fill(0, 0, lay.cw, f.y, VOID);
            a.fill(0, f.bottom(), lay.cw, lay.ch - f.bottom(), VOID);
            small(a, "PILGRIM", 8, 8, CREAM);
            icons_left(a, a.r.life_icon, v.lives, 64, 8);
            icons_left(a, a.r.bomb_icon, v.bombs, 64 + 3 * 9 + 8, 8);
            text_r(a, &version_stamp(), lay.cw - 32, 8, ASH);
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
        pause_menu(a, lay, v, sel);
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

/// JAWSTHIC's bar along the top of the playfield: its name, the bar cut into its
/// phases' shares (the current one cream, those to come ash, those gone empty), the
/// phase's name under it, and the seconds left, red in the last ten.
fn jaws_bar(a: &Atlas, lay: &Layout, j: &JawsView, t: u32) {
    let f = lay.field;
    let (x0, x1, y) = (f.x + 10, f.right() - 10, f.y + 6);
    small(a, BOSS_NAME, x0, y, ASH);
    let bx = x0 + text_width(BOSS_NAME, 1, TRACK) + 6;
    let bar_end = x1 - text_width("00", 1, TRACK) - 6;
    let total = (bar_end - bx) as f32;
    let shown = j.fill * total;
    let mut sx = bx as f32;
    for (i, &(share, ..)) in JAWS_PHASES.iter().enumerate() {
        let w = share * total;
        let (left, width) = (sx.round() as i32, (w.round() as i32 - 2).max(1));
        a.fill(left, y + 2, width, 3, DUSK);
        let reach = ((shown - (sx - bx as f32)) / w).clamp(0.0, 1.0);
        let lit = ((width as f32) * (j.bar[i] * reach)).round() as i32;
        let current = j.phase as usize == i + 1;
        let c = if current { CREAM } else { ASH };
        a.fill(left, y + 2, lit, 3, c);
        if current {
            a.fill(left, y + 6, width, 1, Color { a: 0.6, ..CREAM });
        }
        sx += w;
    }
    if j.phase > 0 {
        small(a, JAWS_PHASE_NAMES[j.phase as usize - 1], bx, y + 10, SLATE);
        let secs = j.left.ceil() as i32;
        let urgent = j.left <= 10.0;
        if !urgent || t % 30 < 22 {
            text_r(a, &format!("{secs:02}"), x1, y, if urgent { SIGNAL } else { CREAM });
        }
    }
}

/// The stage-end panel's geometry: (panel, first row's y, row height).
pub fn end_box(lay: &Layout) -> (IRect, i32, i32) {
    let f = lay.field;
    let panel = IRect::new(f.x + 20, f.y + f.h / 2 - 76, f.w - 40, 152);
    (panel, panel.y + 116, 14)
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

fn stage_end(a: &Atlas, lay: &Layout, v: &HudView, sel: usize) {
    let s = v.stage.unwrap_or_default();
    let (p, row0, row_h) = end_box(lay);
    let cx = p.x + p.w / 2;
    a.fill(p.x, p.y, p.w, p.h, Color { a: 0.92, ..VOID });
    hrule(a, p.x, p.right(), p.y);
    hrule(a, p.x, p.right(), p.bottom() - 1);
    text_c(a, STAGE_NAME, cx, p.y + 10, 2, 3, CREAM);
    let verdict = match s.jaws_end {
        Some((true, ..)) => format!("{BOSS_NAME} DESTROYED"),
        Some((false, ..)) => format!("{BOSS_NAME} LOST TO THE HOLE"),
        None => "STAGE CLEAR".to_string(),
    };
    text_c(a, &verdict, cx, p.y + 32, 1, 1, if s.jaws_end.is_some_and(|e| e.0) { CREAM } else { ASH });
    let pct = s.killed * 100 / s.spawned.max(1);
    let (broken, clean) = s.jaws_end.map_or((0, 0), |e| (e.1, e.2));
    let lines = [
        ("KILLS", format!("{}/{}  {}%", s.killed, s.spawned, pct)),
        ("PHASES BROKEN", format!("{broken}/4  CLEAN {clean}")),
        ("MISSES", v.hits.to_string()),
        ("BOMBS USED", s.bombs_used.to_string()),
        ("GRAZE", v.grazes.to_string()),
    ];
    for (i, (k, val)) in lines.iter().enumerate() {
        let y = p.y + 50 + i as i32 * 12;
        text(a, k, p.x + 14, y, 1, 1, SLATE);
        let w = text_width(val, 1, 1);
        text(a, val, p.right() - 14 - w, y, 1, 1, CREAM);
    }
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

/// `n` icons ending at `right` (the side panel's right edge).
fn icons(a: &Atlas, icon: Rect, n: u8, right: i32, y: i32) {
    for i in 0..n as i32 {
        a.blit(icon, right - 7 - i * 9, y, 1, WHITE_A);
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
    let panel = IRect::new(f.x + 30, f.y + f.h / 2 - 50, f.w - 60, 100);
    (panel, panel.y + 52, 16)
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
    a.fill(p.x, p.y, p.w, p.h, Color { a: 0.92, ..VOID });
    hrule(a, p.x, p.right(), p.y);
    hrule(a, p.x, p.right(), p.bottom() - 1);
    text_c(a, "GAME OVER", p.x + p.w / 2, p.y + 12, 2, 3, CREAM);
    text_c(a, &format!("HITS {}  GRAZE {}", v.hits, v.grazes), p.x + p.w / 2, p.y + 34, 1, 1, SLATE);
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

/// The pause menu's geometry, shared by drawing and tapping.
pub struct MenuBox {
    pub panel: IRect,
    pub row0: i32,
    pub row_h: i32,
    /// The value column: taps left of its middle step down, right of it step up.
    pub val0: i32,
    pub val1: i32,
}

pub fn menu_box(lay: &Layout) -> MenuBox {
    let f = lay.field;
    let row_h = 14;
    let h = 32 + ITEMS.len() as i32 * row_h + 22;
    let panel = IRect::new(f.x + 14, f.y + (f.h - h) / 2, f.w - 28, h);
    MenuBox { panel, row0: panel.y + 28, row_h, val0: panel.x + panel.w / 2 - 10, val1: panel.right() - 8 }
}

/// What a tap at canvas `(x, y)` means in the menu: (row, step) where step is
/// -1 or 1 on a value, 0 elsewhere on the row.
pub fn menu_tap(lay: &Layout, x: f32, y: f32) -> Option<(usize, i32)> {
    let m = menu_box(lay);
    let (x, y) = (x as i32, y as i32);
    if x < m.panel.x || x >= m.panel.right() || y < m.row0 - 4 {
        return None;
    }
    let row = ((y - m.row0 + 4) / m.row_h) as usize;
    let item = *ITEMS.get(row)?;
    let step = if item.adjustable() && x >= m.val0 - 8 {
        if x < (m.val0 + m.val1) / 2 { -1 } else { 1 }
    } else {
        0
    };
    Some((row, step))
}

fn pause_menu(a: &Atlas, lay: &Layout, v: &HudView, sel: usize) {
    let m = menu_box(lay);
    let p = m.panel;
    a.fill(p.x, p.y, p.w, p.h, Color { a: 0.94, ..VOID });
    hrule(a, p.x, p.right(), p.y);
    hrule(a, p.x, p.right(), p.bottom() - 1);
    text(a, "PAUSED // TUNING", p.x + 10, p.y + 10, 1, TRACK, CREAM);
    for (i, item) in ITEMS.iter().enumerate() {
        let y = m.row0 + i as i32 * m.row_h;
        let on = i == sel;
        let c = if on { CREAM } else { ASH };
        if on {
            a.fill(p.x + 4, y - 3, p.w - 8, m.row_h - 3, Color { a: 0.5, ..DUSK });
            text(a, ">", p.x + 6, y, 1, 1, CREAM);
        }
        text(a, item.label(), p.x + 16, y, 1, 1, c);
        if item.adjustable() {
            let val = menu::value(v.settings, *item);
            let cxv = (m.val0 + m.val1) / 2;
            let w = text_width(&val, 1, 1);
            text(a, &val, cxv - w / 2, y, 1, 1, c);
            text(a, "<", m.val0 - 4, y, 1, 1, if on { CREAM } else { SLATE });
            text(a, ">", m.val1 - 5, y, 1, 1, if on { CREAM } else { SLATE });
        }
    }
    let hint = match v.mode {
        Mode::Keys => "ARROWS CHANGE  Z OK  ESC BACK",
        Mode::Pad => "D-PAD CHANGE  A OK  B BACK",
        Mode::Touch => "TAP < > TO CHANGE",
    };
    text(a, hint, p.x + 10, p.bottom() - 14, 1, 1, SLATE);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tune::{CANVAS_W, PORTRAIT_W};

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
        // the landscape side panels are 185 px wide with 12 px margins
        let room = 185 - 25;
        for s in ["HUMANITY STILL LOOKS UP", "A LONG WAY FROM HOME", "PATTERN SANDBOX", "STAGE 1  //  OUTER DISK", "PAUSE TO CHANGE", "BOT SLOW INVINCIBLE"] {
            assert!(text_width(s, 1, TRACK) <= room, "{s}");
        }
        for (_, name) in SECTIONS {
            assert!(text_width(name, 1, TRACK) <= room, "{name}");
        }
        assert!(text_width("KILLS 999/999  MISSES 99  GRAZE 9999", 1, 1) <= 270 - 16);
        for p in SANDBOX {
            assert!(text_width(p.name, 1, TRACK) <= room, "{}", p.name);
            // the portrait thumb zone line, beside the buttons
            assert!(text_width(&format!("{} 13/13", p.name), 1, 1) <= 270 - 8 - 100, "{}", p.name);
            assert!(text_width("BULLETS 8192  HITS 999  GRAZE 9999", 1, 1) <= 270 - 16);
        }
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
        all.extend(TITLE_ITEMS.iter().map(|s| s.to_string()));
        all.extend(["DERELICT", "WARNING", "BOMBS USED", "MISSES", "PHASES BROKEN", "CLEAN", "STAGE CLEAR", BOSS_NAME, STAGE_NAME].map(String::from));
        all.extend(JAWS_PHASE_NAMES.iter().map(|s| s.to_string()));
        all.extend([format!("{BOSS_NAME} DESTROYED"), format!("{BOSS_NAME} LOST TO THE HOLE")]);
        all.extend(ITEMS.iter().map(|i| i.label().to_string()));
        let s = crate::storage::Settings::default();
        all.extend(ITEMS.iter().map(|i| menu::value(&s, *i)));
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
            assert!(text_width(STAGE_NAME, 2, 3) <= p.w && text_width("THE BOSS ARRIVES IN V0.7", 1, 1) <= p.w - 8);
            assert!(text_width("WARNING", 2, 4) <= lay.field.w - 20);
            let (cx, y0, rh) = title_rows(&lay);
            for i in 0..TITLE_ITEMS.len() {
                assert_eq!(title_tap(&lay, cx as f32, (y0 + i as i32 * rh + 2) as f32), Some(i));
            }
        }
    }

    #[test]
    fn the_menu_fits_and_taps_land_on_rows() {
        for (w, h) in [(1280, 720), (390, 844), (844, 390)] {
            let lay = Layout::fit(w, h);
            let m = menu_box(&lay);
            assert!(m.panel.y >= lay.field.y && m.panel.bottom() <= lay.field.bottom());
            for (i, item) in ITEMS.iter().enumerate() {
                let y = (m.row0 + i as i32 * m.row_h + 3) as f32;
                assert_eq!(menu_tap(&lay, (m.panel.x + 20) as f32, y), Some((i, 0)));
                if item.adjustable() {
                    assert_eq!(menu_tap(&lay, m.val0 as f32, y), Some((i, -1)));
                    assert_eq!(menu_tap(&lay, (m.val1 - 2) as f32, y), Some((i, 1)));
                }
            }
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
