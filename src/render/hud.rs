//! Type and screen furniture, styled after the PILGRIM sheet: spaced capitals,
//! thin rules, small labels, a version stamp bottom-left and a tagline bottom-right.

use super::Renderer;
use super::art::Atlas;
use super::font::{self, text_width};
use super::layout::{IRect, Layout, Orient};
use super::palette::{ASH, CREAM, DUSK, SLATE, STRIP, VOID, WHITE_A, tone};
use macroquad::math::Rect;
use crate::input::{Button, Mode, touch_buttons};
use crate::menu::{self, ITEMS};
use crate::storage::Settings;
use crate::storage::View;
use crate::tune::{BLINK_FRAMES, SANDBOX, TITLE_SCALE, TITLE_TRACK, TRACK};

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
    r.hole(full, lay.cw as f32 * 0.5, cy, rad, super::scene::hole_bright(1.0), t as f32 / 60.0);
}

/// The title: the name, the tagline, PRESS START (drawn over the post pass).
pub fn title(r: &Renderer, lay: &Layout, t: u64, mode: Option<Mode>) {
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
            if blink_on {
                text_c(a, start_hint(mode), cx, 240, 1, TRACK, CREAM);
            }
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
            if blink_on {
                text_c(a, if mode == Some(Mode::Pad) { "PRESS A TO START" } else { "TAP TO START" }, cx, 316, 1, TRACK, CREAM);
            }
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

/// The on-screen touch buttons.
fn buttons(a: &Atlas, lay: &Layout, v: &HudView) {
    for (b, cx, cy, r) in touch_buttons(lay) {
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
            small(a, "STAGE 1  //  SANDBOX", l0, 42, SLATE);
            small(a, "LIVES", l0, 56, ASH);
            icons(a, a.r.life_icon, v.lives, l1, 56);
            small(a, "BOMBS", l0, 66, ASH);
            icons(a, a.r.bomb_icon, v.bombs, l1, 66);
            let n = SANDBOX.len();
            small(a, "PATTERN", l0, 82, ASH);
            text_r(a, &format!("{}/{}", v.pattern % n + 1, n), l1, 82, SLATE);
            small(a, SANDBOX[v.pattern % n].name, l0, 94, CREAM);
            small(a, "BULLETS", l0, 106, SLATE);
            text_r(a, &v.bullets.to_string(), l1, 106, ASH);
            small(a, "HITS", l0, 116, SLATE);
            text_r(a, &v.hits.to_string(), l1, 116, ASH);
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
            let rows = controls(v.mode);
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
            let pat = format!("{} {}/{}", SANDBOX[v.pattern % n].name, v.pattern % n + 1, n);
            text(a, &pat, 8, f.bottom() + 20, 1, 1, ASH);
            let hint = if v.mode == Mode::Pad { "START: PAUSE + TUNING" } else { "DRAG ANYWHERE TO FLY" };
            small(a, hint, 8, f.bottom() + 36, ASH);
            small(a, "2ND FINGER: FOCUS", 8, f.bottom() + 48, SLATE);
            let stats = format!("BULLETS {}  HITS {}  GRAZE {}", v.bullets, v.hits, v.grazes);
            text(a, &stats, 8, f.bottom() + 74, 1, 1, SLATE);
            buttons(a, lay, v);
        }
    }
    if let Some((msg, left)) = v.toast {
        if left > 0 && (left > 20 || left % 6 < 3) {
            text_c(a, msg, f.x + f.w / 2, f.y + 40, 1, TRACK, CREAM);
        }
    }
    if let Some(sel) = v.over {
        game_over(a, lay, v, sel);
    } else if let Some(sel) = v.menu {
        pause_menu(a, lay, v, sel);
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
        for s in ["HUMANITY STILL LOOKS UP", "A LONG WAY FROM HOME", "STAGE 1  //  SANDBOX", "PAUSE TO CHANGE", "BOT SLOW INVINCIBLE"] {
            assert!(text_width(s, 1, TRACK) <= room, "{s}");
        }
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
