//! Score popups, arcade style: points float up off kills (teal and doubled
//! point-blank, big and flashing for big kills), a boss phase's grade letter slams in
//! over the boss with its bonus counting up, and EXTEND flashes across the field.
//! Drawn under the enemy bullets, so nothing ever hides one.

use super::art::Atlas;
use super::font::{GH, text_width};
use super::hud::text;
use super::palette::{ASH, CREAM, SLATE, TEAL, TEAL_HOT, VOID};
use crate::sim::score::{Award, Grade};
use crate::tune::{BIG_POINTS, FIELD_H, FIELD_W};
use macroquad::color::Color;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Style {
    /// A kill's points.
    Small,
    /// A point-blank kill's (doubled) points.
    Blank,
    /// Shards collected.
    Shards,
    /// A big kill's points: twice the size, flashing.
    Big,
    /// A boss phase's grade and bonus.
    Grade(u8, Grade),
    Extend,
}

impl Style {
    /// Frames it stays.
    fn life(self) -> u32 {
        match self {
            Style::Small | Style::Blank => 42,
            Style::Shards => 36,
            Style::Big => 72,
            Style::Grade(..) => 120,
            Style::Extend => 120,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Popup {
    x: f32,
    y: f32,
    n: u32,
    style: Style,
    t: u32,
}

/// The most on screen at once (the oldest go first).
const CAP: usize = 48;

pub struct Popups {
    list: Vec<Popup>,
}

/// `1234567` as `1,234,567`.
pub fn commas(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// The grade's own color: S in the Pilgrim's hot teal, A cream, then fading to slate.
pub fn grade_color(g: Grade) -> Color {
    match g {
        Grade::S => TEAL_HOT,
        Grade::A => CREAM,
        Grade::B => ASH,
        Grade::C | Grade::D => SLATE,
    }
}

/// What the grade means, over its letter.
pub fn grade_word(g: Grade) -> &'static str {
    match g {
        Grade::S | Grade::A => "CAPTURED",
        Grade::B => "BROKEN",
        Grade::C | Grade::D => "TIME OUT",
    }
}

/// Text with a one-pixel ink outline, so it reads over anything.
pub fn outlined(a: &Atlas, s: &str, x: i32, y: i32, scale: i32, track: i32, c: Color) {
    let ink = Color { a: c.a, ..VOID };
    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1), (scale, scale)] {
        text(a, s, x + dx, y + dy, scale, track, ink);
    }
    text(a, s, x, y, scale, track, c);
}

/// A grade letter's size as it slams in: huge, then settling at 6x in a few frames.
pub fn slam_scale(t: u32) -> i32 {
    [12, 10, 9, 8, 7].get(t as usize).copied().unwrap_or(6)
}

impl Popups {
    pub fn new() -> Popups {
        Popups { list: Vec::with_capacity(CAP) }
    }

    fn push(&mut self, p: Popup) {
        if self.list.len() >= CAP {
            self.list.remove(0);
        }
        self.list.push(p);
    }

    /// Points scored at `(x, y)` in playfield px.
    pub fn points(&mut self, x: f32, y: f32, n: u32, how: Award) {
        let style = match how {
            Award::Graze => return,
            Award::Grade(p, g) => Style::Grade(p, g),
            Award::Shards => Style::Shards,
            _ if n >= BIG_POINTS => Style::Big,
            Award::Blank => Style::Blank,
            _ => Style::Small,
        };
        // a grade sits where it can be read whole, wherever the boss was
        let (x, y) = match style {
            Style::Grade(..) => (x.clamp(70.0, FIELD_W as f32 - 70.0), (y + 74.0).clamp(90.0, FIELD_H as f32 * 0.5)),
            Style::Shards => (x, y - 26.0),
            _ => (x, y),
        };
        self.push(Popup { x, y, n, style, t: 0 });
    }

    pub fn extend(&mut self) {
        // low in the field, clear of a grade slamming in over the boss
        self.push(Popup { x: FIELD_W as f32 * 0.5, y: FIELD_H as f32 * 0.6, n: 0, style: Style::Extend, t: 0 });
    }

    pub fn tick(&mut self) {
        for p in &mut self.list {
            p.t += 1;
        }
        self.list.retain(|p| p.t < p.style.life());
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// Draw them, the playfield's top-left at canvas `(ox, oy)`.
    pub fn draw(&self, a: &Atlas, ox: i32, oy: i32) {
        for p in &self.list {
            let life = p.style.life();
            let fade = ((life - p.t) as f32 / 12.0).min(1.0);
            match p.style {
                Style::Small | Style::Blank | Style::Shards => {
                    let rise = p.t as f32 * if p.style == Style::Shards { 0.6 } else { 0.4 };
                    let s = if p.style == Style::Shards { format!("+{}", commas(p.n as u64)) } else { commas(p.n as u64) };
                    let c = if p.style == Style::Small { CREAM } else { TEAL_HOT };
                    let x = ox + p.x.round() as i32 - text_width(&s, 1, 1) / 2;
                    let y = oy + (p.y - rise).round() as i32 - 3;
                    outlined(a, &s, x, y, 1, 1, Color { a: fade, ..c });
                }
                Style::Big => {
                    let s = commas(p.n as u64);
                    let flash = p.t < 30 && p.t % 6 < 3;
                    let c = if flash { TEAL_HOT } else { CREAM };
                    let x = ox + p.x.round() as i32 - text_width(&s, 2, 1) / 2;
                    let y = oy + (p.y - p.t as f32 * 0.25).round() as i32 - 7;
                    outlined(a, &s, x, y, 2, 1, Color { a: fade, ..c });
                }
                Style::Grade(_, g) => draw_grade(a, ox + p.x.round() as i32, oy + p.y.round() as i32, g, p.n, p.t, fade),
                Style::Extend => {
                    if p.t < 60 && p.t % 8 >= 5 {
                        continue;
                    }
                    let s = "EXTEND";
                    let x = ox + p.x as i32 - text_width(s, 2, 3) / 2;
                    let y = oy + p.y as i32;
                    outlined(a, s, x, y, 2, 3, Color { a: fade, ..TEAL_HOT });
                    let sub = "1UP";
                    outlined(a, sub, ox + p.x as i32 - text_width(sub, 1, 2) / 2, y + 20, 1, 2, Color { a: fade, ..CREAM });
                }
            }
        }
    }
}

/// A grade, arcade style, centered on `(cx, cy)` (canvas px), `t` frames in: the
/// letter slams in, the word over it, the bonus counting up under it; then it fades.
pub fn draw_grade(a: &Atlas, cx: i32, cy: i32, g: Grade, bonus: u32, t: u32, fade: f32) {
    let (y, h) = slam_letter(a, cx, cy, g, t, fade);
    let c = grade_color(g);
    if t >= 4 {
        let word = grade_word(g);
        outlined(a, word, cx - text_width(word, 1, 2) / 2, y - 12, 1, 2, Color { a: fade, ..ASH });
        if bonus > 0 {
            let k = ((t - 4) as f32 / 30.0).min(1.0);
            let shown = format!("+{}", commas((bonus as f32 * k) as u64 / 10 * 10));
            outlined(a, &shown, cx - text_width(&shown, 1, 1) / 2, y + h + 6, 1, 1, Color { a: fade, ..c });
        }
    }
}

/// A grade letter slamming in at `(cx, cy)`, `t` frames in: huge and white, settling
/// in a few frames; a streak of light across and a ring bursting out; an S glowing
/// teal; a glint running across it now and then. Returns its top and height.
pub fn slam_letter(a: &Atlas, cx: i32, cy: i32, g: Grade, t: u32, fade: f32) -> (i32, i32) {
    let s = slam_scale(t);
    let (w, h) = (text_width(g.letter(), s, 0), GH as i32 * s);
    let (x, y) = (cx - w / 2, cy - h / 2);
    let c = grade_color(g);
    // the streak and the ring, in the first frames
    if t < 6 {
        let k = 1.0 - t as f32 / 6.0;
        a.fill(cx - 160, cy - 1, 320, 2, Color { a: 0.7 * k, ..TEAL_HOT });
        a.fill(cx - 160, cy, 320, 1, Color { a: 0.9 * k, ..CREAM });
    }
    if t < 18 {
        let r = 8.0 + t as f32 * 3.5;
        let n = 40;
        for k in 0..n {
            let ang = k as f32 / n as f32 * std::f32::consts::TAU;
            a.fill(cx + (ang.cos() * r) as i32, cy + (ang.sin() * r) as i32, 1, 1, Color { a: 1.0 - t as f32 / 18.0, ..c });
        }
    }
    // an S glows teal round its edges
    if g == Grade::S {
        for (dx, dy) in [(-2, 0), (2, 0), (0, -2), (0, 2)] {
            text(a, g.letter(), x + dx, y + dy, s, 0, Color { a: 0.35 * fade, ..TEAL });
        }
    }
    let hot = t < 3 || (t > 20 && t % 36 < 2);
    outlined(a, g.letter(), x, y, s, 0, Color { a: fade, ..if hot { CREAM } else { c } });
    // a glint: a bright diagonal running across the letter now and then
    let gt = t % 36;
    if (8..14).contains(&gt) && t > 20 {
        let gx = x + (gt as i32 - 8) * w / 5;
        for d in 0..h {
            a.fill(gx + d / 3, y + d, 1, 1, Color { a: 0.55 * fade, ..CREAM });
        }
    }
    (y, h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_read_like_an_arcade_score() {
        assert_eq!(commas(0), "0");
        assert_eq!(commas(999), "999");
        assert_eq!(commas(1000), "1,000");
        assert_eq!(commas(12_345_678), "12,345,678");
    }

    #[test]
    fn a_grade_slams_in_then_settles() {
        let sizes: Vec<i32> = (0..8).map(slam_scale).collect();
        assert!(sizes.windows(2).all(|w| w[1] <= w[0]), "{sizes:?}");
        assert_eq!(sizes[0], 12);
        assert_eq!(*sizes.last().unwrap(), 6);
    }

    #[test]
    fn popups_pick_their_style_and_go() {
        let mut p = Popups::new();
        p.points(10.0, 10.0, 500, Award::Kill);
        p.points(10.0, 10.0, 1000, Award::Blank);
        p.points(10.0, 10.0, BIG_POINTS, Award::Kill);
        p.points(10.0, 10.0, 300_000, Award::Grade(4, Grade::S));
        p.points(10.0, 10.0, 100, Award::Graze);
        let styles: Vec<Style> = p.list.iter().map(|q| q.style).collect();
        assert_eq!(styles, vec![Style::Small, Style::Blank, Style::Big, Style::Grade(4, Grade::S)], "grazes add up quietly");
        assert!(p.list[3].x >= 70.0, "a grade stays inside the field");
        for _ in 0..200 {
            p.tick();
        }
        assert_eq!(p.len(), 0);
        for k in 0..100 {
            p.points(k as f32, 0.0, 100, Award::Kill);
        }
        assert_eq!(p.len(), CAP, "capped");
    }
}
