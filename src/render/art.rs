//! The sprite atlas, built pixel by pixel in code at startup. Everything draws from
//! this one texture so the whole frame batches into a few draw calls.

use super::font;
use super::palette::{ASH, DUSK, SLATE, SOOT};
use super::ship::{self, ShipArt};
use macroquad::prelude::*;

const W: u16 = 512;
const H: u16 = 256;

/// Packs images into the atlas in rows, with a pixel of space around each.
pub struct Packer {
    pub img: Image,
    x: u16,
    y: u16,
    row_h: u16,
}

impl Packer {
    pub fn new() -> Packer {
        Packer { img: Image::gen_image_color(W, H, Color::new(0.0, 0.0, 0.0, 0.0)), x: 1, y: 1, row_h: 0 }
    }

    pub fn add(&mut self, src: &Image) -> Rect {
        if self.x + src.width + 1 > W {
            self.x = 1;
            self.y += self.row_h + 1;
            self.row_h = 0;
        }
        assert!(self.y + src.height < H, "the atlas is full");
        for y in 0..src.height as u32 {
            for x in 0..src.width as u32 {
                let c = src.get_pixel(x, y);
                if c.a > 0.0 {
                    self.img.set_pixel(self.x as u32 + x, self.y as u32 + y, c);
                }
            }
        }
        let r = Rect::new(self.x as f32, self.y as f32, src.width as f32, src.height as f32);
        self.x += src.width + 1;
        self.row_h = self.row_h.max(src.height);
        r
    }

    /// Add an image and its white silhouette (for flashes and afterimages).
    pub fn add_with_silhouette(&mut self, src: &Image) -> (Rect, Rect) {
        (self.add(src), self.add(&silhouette(src)))
    }
}

/// Every lit pixel turned white, keeping its alpha.
pub fn silhouette(src: &Image) -> Image {
    let mut out = Image::gen_image_color(src.width, src.height, Color::new(0.0, 0.0, 0.0, 0.0));
    for y in 0..src.height as u32 {
        for x in 0..src.width as u32 {
            let a = src.get_pixel(x, y).a;
            if a > 0.0 {
                out.set_pixel(x, y, Color::new(1.0, 1.0, 1.0, a));
            }
        }
    }
    out
}

/// An image from rows of palette characters ('.' is empty).
pub fn grid_image(rows: &[Vec<char>], pal: impl Fn(char) -> Option<Color>) -> Image {
    let w = rows.iter().map(|r| r.len()).max().unwrap_or(0) as u16;
    let mut img = Image::gen_image_color(w, rows.len() as u16, Color::new(0.0, 0.0, 0.0, 0.0));
    for (y, row) in rows.iter().enumerate() {
        for (x, &c) in row.iter().enumerate() {
            if let Some(col) = pal(c) {
                img.set_pixel(x as u32, y as u32, col);
            }
        }
    }
    img
}

pub fn rows(src: &[&str]) -> Vec<Vec<char>> {
    src.iter().map(|r| r.chars().collect()).collect()
}

fn white(a: f32) -> Color {
    Color::new(1.0, 1.0, 1.0, a)
}

/// The target dummies: engraved monochrome, inside the enemy value band.
fn dummy_images() -> [Image; 3] {
    let clear = Color::new(0.0, 0.0, 0.0, 0.0);
    let outline = Color::new(0.07, 0.07, 0.06, 1.0);

    // still: a ringed target buoy
    let mut still = Image::gen_image_color(26, 26, clear);
    for y in 0..26u32 {
        for x in 0..26u32 {
            let (dx, dy) = (x as f32 - 12.5, y as f32 - 12.5);
            let d = (dx * dx + dy * dy).sqrt();
            let lit = dx + dy < -3.0; // light from the top left
            let c = match d {
                d if d > 13.0 => continue,
                d if d > 12.0 => outline,
                d if d > 10.0 => if lit { ASH } else { SLATE },
                d if d > 8.0 => DUSK,
                d if d > 6.0 => if lit { ASH } else { SLATE },
                d if d > 3.0 => SOOT,
                d if d > 1.5 => SLATE,
                _ => ASH,
            };
            still.set_pixel(x, y, c);
        }
    }
    for k in 0..3u32 {
        // crosshair ticks on the outer ring
        still.set_pixel(12 + k % 2, 1, DUSK);
        still.set_pixel(12 + k % 2, 24, DUSK);
        still.set_pixel(1, 12 + k % 2, DUSK);
        still.set_pixel(24, 12 + k % 2, DUSK);
    }

    // moving: a diamond drone
    let mut moving = Image::gen_image_color(20, 20, clear);
    for y in 0..20i32 {
        for x in 0..20i32 {
            let d = (x - 10).abs() + (y - 10).abs();
            let c = match d {
                d if d > 10 => continue,
                10 => outline,
                9 | 8 => if x + y < 20 { ASH } else { SLATE },
                _ if (y - 10).abs() <= 1 && (x - 10).abs() <= 4 => SOOT,
                d if d > 5 => SLATE,
                _ => DUSK,
            };
            moving.set_pixel(x as u32, y as u32, c);
        }
    }

    // big: a plated hulk with bevelled corners
    let (bw, bh) = (66u32, 42u32);
    let mut big = Image::gen_image_color(bw as u16, bh as u16, clear);
    for y in 0..bh {
        for x in 0..bw {
            let cut = 6;
            let (cx, cy) = (x.min(bw - 1 - x), y.min(bh - 1 - y));
            if cx + cy < cut {
                continue;
            }
            let edge = cx == 0 || cy == 0 || cx + cy == cut;
            let c = if edge {
                outline
            } else if y == 1 || (cx + cy == cut + 1 && y < bh / 2) {
                ASH
            } else if x % 11 == 0 || y % 10 == 0 {
                DUSK
            } else if (13..29).contains(&y) && (26..40).contains(&x) {
                if (x + y) % 2 == 0 { SOOT } else { DUSK }
            } else if y > bh - 5 {
                DUSK
            } else {
                SLATE
            };
            big.set_pixel(x, y, c);
        }
    }
    [still, moving, big]
}

/// Where each sprite sits in the atlas (CPU side, so it can be tested headlessly).
pub struct Rects {
    glyphs: [Option<Rect>; 128],
    /// A single white pixel, for rules, panels, beams and 1px particles.
    pub px: Rect,
    /// A 3x3 plus: a bright center with dimmer arms.
    pub plus: Rect,
    /// A 7x7 cross sparkle, like the one in the corner of the sheet.
    pub sparkle: Rect,
    pub ship: ShipArt,
    /// Still, moving, big; and their white silhouettes (the hit flash).
    pub dummy: [Rect; 3],
    pub dummy_sil: [Rect; 3],
}

impl Rects {
    pub fn glyph(&self, c: char) -> Option<Rect> {
        let c = c.to_ascii_uppercase();
        if c.is_ascii() { self.glyphs[c as usize] } else { None }
    }
}

pub fn build() -> (Image, Rects) {
    let mut p = Packer::new();
    let mut glyphs = [None; 128];
    let (gw, gh) = (font::GW as u16, font::GH as u16);
    for c in font::CHARS.chars() {
        let mut g = Image::gen_image_color(gw, gh, Color::new(0.0, 0.0, 0.0, 0.0));
        for y in 0..gh as usize {
            for x in 0..gw as usize {
                if font::bit(c, x, y) {
                    g.set_pixel(x as u32, y as u32, WHITE);
                }
            }
        }
        glyphs[c as usize] = Some(p.add(&g));
    }

    // a 3x3 white block; sampling its center pixel can never bleed into a neighbour
    let block = Image::gen_image_color(3, 3, WHITE);
    let b = p.add(&block);
    let px = Rect::new(b.x + 1.0, b.y + 1.0, 1.0, 1.0);

    let mut plus = Image::gen_image_color(3, 3, Color::new(0.0, 0.0, 0.0, 0.0));
    plus.set_pixel(1, 1, WHITE);
    for (dx, dy) in [(1, 0), (0, 1), (2, 1), (1, 2)] {
        plus.set_pixel(dx, dy, white(0.45));
    }
    let plus = p.add(&plus);

    let mut sp = Image::gen_image_color(7, 7, Color::new(0.0, 0.0, 0.0, 0.0));
    for d in 0..7u32 {
        let a = [0.25, 0.5, 0.8, 1.0, 0.8, 0.5, 0.25][d as usize];
        sp.set_pixel(d, 3, white(a));
        sp.set_pixel(3, d, white(a));
    }
    let sparkle = p.add(&sp);

    let ship = ship::build(&mut p);
    let ims = dummy_images();
    let mut dummy = [Rect::default(); 3];
    let mut dummy_sil = [Rect::default(); 3];
    for (i, im) in ims.iter().enumerate() {
        (dummy[i], dummy_sil[i]) = p.add_with_silhouette(im);
    }

    (p.img, Rects { glyphs, px, plus, sparkle, ship, dummy, dummy_sil })
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

    /// Draw a region mirrored left to right.
    pub fn blit_flip(&self, src: Rect, x: i32, y: i32, tint: Color) {
        draw_texture_ex(
            &self.tex,
            x as f32,
            y as f32,
            tint,
            DrawTextureParams { source: Some(src), flip_x: true, ..Default::default() },
        );
    }

    /// Draw a region centered on `(cx, cy)`, rotated by `angle` radians.
    pub fn blit_rot(&self, src: Rect, cx: f32, cy: f32, angle: f32, tint: Color) {
        draw_texture_ex(
            &self.tex,
            (cx - src.w * 0.5).round(),
            (cy - src.h * 0.5).round(),
            tint,
            DrawTextureParams { source: Some(src), rotation: angle, ..Default::default() },
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
    fn nothing_in_the_atlas_overlaps() {
        let (img, r) = build();
        assert_eq!(img.get_pixel(r.px.x as u32, r.px.y as u32), WHITE);
        let mut all: Vec<Rect> = font::CHARS.chars().filter_map(|c| r.glyph(c)).collect();
        all.extend([r.plus, r.sparkle]);
        all.extend(r.dummy);
        all.extend(r.dummy_sil);
        all.extend(r.ship.rects());
        for (i, a) in all.iter().enumerate() {
            assert!(a.x >= 0.0 && a.y >= 0.0 && a.x + a.w <= W as f32 && a.y + a.h <= H as f32);
            for b in &all[i + 1..] {
                assert!(!a.overlaps(b) || a == b, "{a:?} overlaps {b:?}");
            }
        }
    }

    #[test]
    fn dummies_sit_in_the_enemy_value_band() {
        use crate::render::palette::luma;
        let ims = dummy_images();
        for im in &ims {
            let lit: Vec<f32> = im.get_image_data().iter().filter(|p| p[3] > 0).map(|p| luma(Color::from_rgba(p[0], p[1], p[2], 255))).collect();
            let brightest = lit.iter().cloned().fold(0.0, f32::max);
            assert!(brightest <= 0.75 && brightest >= 0.35, "enemies are 35-75%: {brightest}");
        }
    }
}
