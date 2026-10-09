//! The sprite atlas, built pixel by pixel in code at startup. Everything draws from
//! this one texture so the whole frame batches into a few draw calls.

use super::font;
use macroquad::prelude::*;

const W: u16 = 256;
const H: u16 = 32;

/// Where each sprite sits in the atlas (CPU side, so it can be tested headlessly).
pub struct Rects {
    glyphs: [Option<Rect>; 128],
    /// A single white pixel, for rules, panels and 1px stars.
    pub px: Rect,
    /// A 3x3 plus: a bright center with dimmer arms.
    pub plus: Rect,
    /// A 7x7 cross sparkle, like the one in the corner of the sheet.
    pub sparkle: Rect,
}

impl Rects {
    pub fn glyph(&self, c: char) -> Option<Rect> {
        let c = c.to_ascii_uppercase();
        if c.is_ascii() { self.glyphs[c as usize] } else { None }
    }
}

fn white(a: f32) -> Color {
    Color::new(1.0, 1.0, 1.0, a)
}

pub fn build() -> (Image, Rects) {
    let mut img = Image::gen_image_color(W, H, Color::new(0.0, 0.0, 0.0, 0.0));
    let mut glyphs = [None; 128];
    let (gw, gh) = (font::GW as u32, font::GH as u32);
    for (i, c) in font::CHARS.chars().enumerate() {
        let (x0, y0) = (1 + (i as u32 % 32) * (gw + 2), 1 + (i as u32 / 32) * (gh + 2));
        for y in 0..gh {
            for x in 0..gw {
                if font::bit(c, x as usize, y as usize) {
                    img.set_pixel(x0 + x, y0 + y, WHITE);
                }
            }
        }
        glyphs[c as usize] = Some(Rect::new(x0 as f32, y0 as f32, gw as f32, gh as f32));
    }

    // a 3x3 white block; sampling its center pixel can never bleed into a neighbour
    for y in 0..3 {
        for x in 0..3 {
            img.set_pixel(240 + x, 1 + y, WHITE);
        }
    }
    let px = Rect::new(241.0, 2.0, 1.0, 1.0);

    let (ux, uy) = (240, 6);
    img.set_pixel(ux + 1, uy + 1, WHITE);
    for (dx, dy) in [(1, 0), (0, 1), (2, 1), (1, 2)] {
        img.set_pixel(ux + dx, uy + dy, white(0.45));
    }
    let plus = Rect::new(ux as f32, uy as f32, 3.0, 3.0);

    let (sx, sy) = (240, 11);
    for d in 0..7u32 {
        let a = [0.25, 0.5, 0.8, 1.0, 0.8, 0.5, 0.25][d as usize];
        img.set_pixel(sx + d, sy + 3, white(a));
        img.set_pixel(sx + 3, sy + d, white(a));
    }
    let sparkle = Rect::new(sx as f32, sy as f32, 7.0, 7.0);

    (img, Rects { glyphs, px, plus, sparkle })
}

pub struct Atlas {
    pub tex: Texture2D,
    pub r: Rects,
}

impl Atlas {
    pub fn new() -> Atlas {
        let (img, r) = build();
        let tex = Texture2D::from_image(&img);
        tex.set_filter(FilterMode::Nearest);
        Atlas { tex, r }
    }

    /// Draw an atlas region at whole-pixel `(x, y)`, `scale` times its size, tinted.
    pub fn blit(&self, src: Rect, x: i32, y: i32, scale: i32, tint: Color) {
        draw_texture_ex(
            &self.tex,
            x as f32,
            y as f32,
            tint,
            DrawTextureParams {
                source: Some(src),
                dest_size: Some(vec2(src.w * scale as f32, src.h * scale as f32)),
                ..Default::default()
            },
        );
    }

    /// A solid rectangle in whole pixels (from the atlas, so it batches with sprites).
    pub fn fill(&self, x: i32, y: i32, w: i32, h: i32, c: Color) {
        if w > 0 && h > 0 {
            draw_texture_ex(
                &self.tex,
                x as f32,
                y as f32,
                c,
                DrawTextureParams { source: Some(self.r.px), dest_size: Some(vec2(w as f32, h as f32)), ..Default::default() },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_glyph_matches_the_font() {
        let (img, r) = build();
        for c in font::CHARS.chars() {
            let g = r.glyph(c).unwrap_or_else(|| panic!("{c:?} missing from the atlas"));
            for y in 0..font::GH {
                for x in 0..font::GW {
                    let p = img.get_pixel(g.x as u32 + x as u32, g.y as u32 + y as u32);
                    assert_eq!(p.a > 0.5, font::bit(c, x, y), "{c:?} at {x},{y}");
                }
            }
        }
        assert_eq!(r.glyph('q'), r.glyph('Q'));
        assert!(r.glyph('é').is_none());
    }

    #[test]
    fn sprites_fit_and_dont_overlap_glyphs() {
        let (img, r) = build();
        assert_eq!(img.get_pixel(r.px.x as u32, r.px.y as u32), WHITE);
        let glyph_right = font::CHARS.chars().filter_map(|c| r.glyph(c)).map(|g| g.x + g.w).fold(0.0, f32::max);
        for s in [r.px, r.plus, r.sparkle] {
            assert!(s.x >= glyph_right, "sprite overlaps the glyphs");
            assert!(s.x + s.w <= W as f32 && s.y + s.h <= H as f32);
        }
    }
}
