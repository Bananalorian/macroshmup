//! Where the low-res canvas goes on the real screen: landscape or portrait, the
//! biggest whole-number scale that fits, centered with letterboxing.

use crate::tune::{CANVAS_H, CANVAS_W, FIELD_H, FIELD_W, PORTRAIT_H, PORTRAIT_HUD_H, PORTRAIT_W};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orient {
    /// 640x360: the playfield in the middle, HUD panels either side.
    Landscape,
    /// 270x480 (phones): a HUD strip on top, the playfield, a thumb zone below.
    Portrait,
}

/// A rectangle in whole canvas pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl IRect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> IRect {
        IRect { x, y, w, h }
    }
    pub fn right(&self) -> i32 {
        self.x + self.w
    }
    pub fn bottom(&self) -> i32 {
        self.y + self.h
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    pub orient: Orient,
    /// Canvas size in low-res pixels.
    pub cw: i32,
    pub ch: i32,
    /// The playfield inside the canvas.
    pub field: IRect,
    /// Whole-number upscale: each canvas pixel is `scale` x `scale` screen pixels.
    pub scale: i32,
    /// Top-left of the scaled canvas on the screen, in screen pixels.
    pub ox: i32,
    pub oy: i32,
}

impl Layout {
    /// Fit the canvas into a screen of `sw` x `sh` physical pixels.
    pub fn fit(sw: u32, sh: u32) -> Layout {
        let (sw, sh) = (sw.max(1) as i32, sh.max(1) as i32);
        let orient = if sh > sw { Orient::Portrait } else { Orient::Landscape };
        let (cw, ch, field) = match orient {
            Orient::Landscape => {
                let fx = (CANVAS_W - FIELD_W) as i32 / 2;
                (CANVAS_W as i32, CANVAS_H as i32, IRect::new(fx, 0, FIELD_W as i32, FIELD_H as i32))
            }
            Orient::Portrait => (
                PORTRAIT_W as i32,
                PORTRAIT_H as i32,
                IRect::new(0, PORTRAIT_HUD_H as i32, FIELD_W as i32, FIELD_H as i32),
            ),
        };
        // never below 1: a tiny window crops the canvas rather than blurring it
        let scale = (sw / cw).min(sh / ch).max(1);
        let ox = (sw - cw * scale) / 2;
        let oy = (sh - ch * scale) / 2;
        Layout { orient, cw, ch, field, scale, ox, oy }
    }

    /// Screen pixel -> canvas pixel (for touch and mouse).
    #[allow(dead_code)] // touch steering arrives in Sprint 1
    pub fn to_canvas(&self, x: f32, y: f32) -> (f32, f32) {
        ((x - self.ox as f32) / self.scale as f32, (y - self.oy as f32) / self.scale as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_screens_get_the_art_bible_scales() {
        for (w, h, s) in [(1280, 720, 2), (1920, 1080, 3), (2560, 1440, 4), (3840, 2160, 6)] {
            let l = Layout::fit(w, h);
            assert_eq!(l.orient, Orient::Landscape);
            assert_eq!(l.scale, s, "{w}x{h}");
            assert_eq!((l.ox, l.oy), (0, 0), "{w}x{h} fills exactly");
        }
    }

    #[test]
    fn odd_screens_letterbox_evenly() {
        let l = Layout::fit(1366, 768);
        assert_eq!(l.scale, 2);
        assert_eq!((l.ox, l.oy), (43, 24));
        let l = Layout::fit(1440, 900); // 16:10
        assert_eq!(l.scale, 2);
        assert_eq!((l.ox, l.oy), (80, 90));
    }

    #[test]
    fn phones_get_portrait() {
        let l = Layout::fit(390, 844);
        assert_eq!(l.orient, Orient::Portrait);
        assert_eq!((l.cw, l.ch, l.scale), (270, 480, 1));
        let l = Layout::fit(1170, 2532); // the same phone in physical pixels
        assert_eq!(l.scale, 4);
        assert_eq!(l.ox, (1170 - 270 * 4) / 2);
    }

    #[test]
    fn the_playfield_is_the_same_everywhere() {
        for (w, h) in [(1280, 720), (390, 844), (800, 600), (1080, 1920), (300, 200)] {
            let l = Layout::fit(w, h);
            assert_eq!((l.field.w, l.field.h), (270, 360));
            assert!(l.field.x >= 0 && l.field.right() <= l.cw);
            assert!(l.field.y >= 0 && l.field.bottom() <= l.ch);
        }
        let l = Layout::fit(1280, 720);
        assert_eq!(l.field.x, (640 - 270) / 2, "centered in landscape");
    }

    #[test]
    fn scale_is_always_a_whole_number_at_least_one() {
        for w in (100..4000).step_by(37) {
            for h in (100..3000).step_by(41) {
                let l = Layout::fit(w, h);
                assert!(l.scale >= 1);
                if l.cw * l.scale <= w as i32 && l.ch * l.scale <= h as i32 {
                    // the biggest that fits
                    assert!(l.cw * (l.scale + 1) > w as i32 || l.ch * (l.scale + 1) > h as i32);
                }
            }
        }
    }

    #[test]
    fn screen_to_canvas_round_trips() {
        let l = Layout::fit(1366, 768);
        assert_eq!(l.to_canvas(43.0, 24.0), (0.0, 0.0));
        assert_eq!(l.to_canvas(43.0 + 640.0 * 2.0, 24.0 + 360.0 * 2.0), (640.0, 360.0));
    }
}
