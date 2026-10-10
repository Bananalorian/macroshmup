//! The palette, sampled from the PILGRIM sheet (SPRINTS.md §2).
//! Red means danger, teal means the Pilgrim; everything else is monochrome.

use macroquad::color::Color;

const fn hex(c: u32) -> Color {
    Color::new(
        ((c >> 16) & 0xFF) as f32 / 255.0,
        ((c >> 8) & 0xFF) as f32 / 255.0,
        (c & 0xFF) as f32 / 255.0,
        1.0,
    )
}

/// The warm black behind everything.
pub const VOID: Color = hex(0x080907);
/// Highlights, text, hull lights.
pub const CREAM: Color = hex(0xDAD7C3);
/// Light hull tones.
pub const ASH: Color = hex(0x7F7F71);
/// Mid tones, enemies.
pub const SLATE: Color = hex(0x54564C);
/// Dark tones, outlines on light shapes.
pub const SOOT: Color = hex(0x2B2B28);
/// Panel lines, rules.
pub const DUSK: Color = hex(0x363531);
/// Red: enemy bullet rims, the Pilgrim's running lights, boss highlights.
pub const SIGNAL: Color = hex(0xF83C30);
/// Teal: the Pilgrim's engines and trails, shards, graze sparks, the hitbox dot.
pub const TEAL: Color = hex(0x32DFB3);

/// Enemy bullet cores: CREAM pushed to the brightest pixels on screen.
pub const CORE: Color = hex(0xF4F1E6);
/// A lighter red for the bullets' wobble frame.
pub const EMBER: Color = hex(0xFF6A50);
/// The bullets' outline: as dark as it gets.
pub const INK: Color = hex(0x000000);

/// Plain white at full strength (a sprite's own colors, untinted).
pub const WHITE_A: Color = Color::new(1.0, 1.0, 1.0, 1.0);

/// The strip of swatches on the sheet, left to right.
pub const STRIP: [Color; 7] = [CREAM, ASH, SLATE, SOOT, SIGNAL, DUSK, TEAL];

/// Perceived brightness (Rec. 709 luma on the sRGB values), 0..1. The same
/// formula the grayscale screenshots use, so the value budget can be tested.
pub fn luma(c: Color) -> f32 {
    0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
}

/// A monochrome tone at `value` brightness (0..1), mixed from VOID toward CREAM
/// so it keeps the warm cast of the reference art.
pub fn tone(value: f32) -> Color {
    let t = ((value - luma(VOID)) / (luma(CREAM) - luma(VOID))).clamp(0.0, 1.0);
    Color::new(
        VOID.r + (CREAM.r - VOID.r) * t,
        VOID.g + (CREAM.g - VOID.g) * t,
        VOID.b + (CREAM.b - VOID.b) * t,
        1.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_hex(c: Color) -> u32 {
        let b = |v: f32| (v * 255.0).round() as u32;
        (b(c.r) << 16) | (b(c.g) << 8) | b(c.b)
    }

    #[test]
    fn tokens_match_the_art_bible() {
        assert_eq!(to_hex(VOID), 0x080907);
        assert_eq!(to_hex(CREAM), 0xDAD7C3);
        assert_eq!(to_hex(ASH), 0x7F7F71);
        assert_eq!(to_hex(SLATE), 0x54564C);
        assert_eq!(to_hex(SOOT), 0x2B2B28);
        assert_eq!(to_hex(DUSK), 0x363531);
        assert_eq!(to_hex(SIGNAL), 0xF83C30);
        assert_eq!(to_hex(TEAL), 0x32DFB3);
    }

    /// Non-negotiable 3: bullet cores are the brightest thing on screen.
    #[test]
    fn bullet_cores_outshine_everything_else() {
        for c in [CREAM, ASH, SLATE, SOOT, DUSK, SIGNAL, TEAL, EMBER, VOID] {
            assert!(luma(CORE) > luma(c) + 0.08, "{c:?}");
        }
        assert!(luma(INK) == 0.0, "a 0% outline");
    }

    #[test]
    fn tone_hits_the_value_it_asks_for() {
        for v in [0.05, 0.1, 0.2, 0.3, 0.5, 0.8] {
            assert!((luma(tone(v)) - v).abs() < 0.005, "{v}");
        }
        assert_eq!(to_hex(tone(1.0)), 0xDAD7C3, "tone tops out at cream");
    }
}
