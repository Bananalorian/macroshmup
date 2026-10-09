//! Type and screen furniture, styled after the PILGRIM sheet: spaced capitals,
//! thin rules, small labels, a version stamp bottom-left and a tagline bottom-right.

use super::Renderer;
use super::art::Atlas;
use super::font::{self, text_width};
use super::layout::{IRect, Layout, Orient};
use super::palette::{ASH, CREAM, DUSK, SLATE, STRIP, VOID, tone};
use crate::tune::{BLINK_FRAMES, TITLE_SCALE, TITLE_TRACK, TRACK};
use macroquad::color::Color;

/// `V0.1 // PILGRIM`, from the crate version.
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

/// Small spaced text on a VOID backing, so a passing star never reads as a glyph.
fn small(a: &Atlas, s: &str, x: i32, y: i32, c: Color) {
    a.fill(x - 1, y - 1, text_width(s, 1, TRACK) + 2, font::GH as i32 + 2, VOID);
    text(a, s, x, y, 1, TRACK, c);
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

/// The placeholder title: the name, the tagline, PRESS START.
pub fn title(r: &Renderer, lay: &Layout, t: u64) {
    let a = &r.atlas;
    r.stars.draw(a, IRect::new(0, 0, lay.cw, lay.ch), t);
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
                text_c(a, "PRESS Z  //  TAP TO START", cx, 240, 1, TRACK, CREAM);
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
                text_c(a, "TAP TO START", cx, 316, 1, TRACK, CREAM);
            }
            stamps(a, m, lay.cw - m, lay.ch - 32, false);
        }
    }
}

/// The empty playfield with its starfield, framed by the HUD panels.
pub fn stage(r: &Renderer, lay: &Layout, frame: u64) {
    let a = &r.atlas;
    let f = lay.field;
    r.stars.draw(a, f, frame);
    match lay.orient {
        Orient::Landscape => {
            // rules down both sides of the playfield
            a.fill(f.x - 1, 0, 1, lay.ch, DUSK);
            a.fill(f.right(), 0, 1, lay.ch, DUSK);

            let (l0, l1) = (12, f.x - 13);
            small(a, "PILGRIM", l0, 12, CREAM);
            small(a, "TYPE-07", l0, 22, ASH);
            hrule(a, l0, l1, 34);
            small(a, "STAGE 1", l0, 42, SLATE);
            small(a, "ESC   TITLE", l0, 290, SLATE);
            small(a, "F11   FULLSCREEN", l0, 300, SLATE);
            stamps(a, l0, l1, lay.ch - 32, false);

            let (r0, r1) = (f.right() + 13, lay.cw - 12);
            for (i, s) in ["SOMEWHERE", "BEYOND", "THE", "EVENT HORIZON"].iter().enumerate() {
                small(a, s, r0, 12 + i as i32 * 10, ASH);
            }
            sparkle(a, r1 - 7, 12);
            hrule(a, r0, r1, lay.ch - 32);
            text_r(a, "A SMALL SHIP", r1, lay.ch - 26, ASH);
            text_r(a, "A LONG WAY FROM HOME", r1, lay.ch - 16, SLATE);
        }
        Orient::Portrait => {
            small(a, "PILGRIM", 8, 8, CREAM);
            text_r(a, &version_stamp(), lay.cw - 8, 8, ASH);
            hrule(a, 0, lay.cw, f.y - 1);
            hrule(a, 0, lay.cw, f.bottom());
            let cx = lay.cw / 2;
            text_c(a, "A SMALL SHIP", cx, f.bottom() + 36, 1, TRACK, SLATE);
            text_c(a, "A LONG WAY FROM HOME", cx, f.bottom() + 46, 1, TRACK, SLATE);
        }
    }
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
        for s in ["HUMANITY STILL LOOKS UP", "THE EVENT HORIZON", "A LONG WAY FROM HOME"] {
            assert!(text_width(s, 1, TRACK) <= port, "{s}");
        }
        // the side panels in landscape are 185 px wide with 12 px margins
        for s in ["HUMANITY STILL LOOKS UP", "A LONG WAY FROM HOME", "F11   FULLSCREEN"] {
            assert!(text_width(s, 1, TRACK) <= 185 - 25, "{s}");
        }
    }
}
