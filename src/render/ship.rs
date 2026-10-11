//! The Pilgrim, after Jason's split-sword sheet (docs/reference/Split-sword
//! starship_ flight to laser.png): two bone-white blades, each with a teal channel
//! and a red heart, an engine nacelle under each on a crossbar, and a capsule
//! below on struts. Focusing closes the blades into one sword, the laser form,
//! and the beam fires from its tip.
//!
//! Each frame of the transformation is laid out pixel by pixel from a keyframe
//! table (the left half; the right mirrors it), rolled for the bank, then painted
//! in bone like the enemies. The lights are found in the painted frames and lit at
//! draw time.

use super::art::{Atlas, Packer, grid_image, rows};
use super::foes::{BONE, OUTLINE};
use super::palette::{CREAM, SIGNAL, TEAL, TEAL_HOT, VOID};
use macroquad::prelude::*;

/// Frames of the transformation, from the flight form (0) to the laser form.
pub const FORMS: usize = 5;
/// Rolls from hard left to hard right.
const BANKS: usize = 5;
/// The canvas: odd both ways, centered on the hitbox.
const SIZE: (i32, i32) = (33, 49);
const CX: i32 = SIZE.0 / 2;
const CY: i32 = SIZE.1 / 2;
/// Rows from the hitbox: the sword's tip (the beam leaves from here), where its
/// bevelled top meets the outer edge, the shoulder, and the foot of the blades.
pub const TIP: i32 = -21;
const CORNER: i32 = -13;
const SHOULDER: i32 = -8;
const FOOT: i32 = 7;
/// The channel: the point of its arch, and the step where it widens.
const ARCH: i32 = -15;
const STEP: i32 = -1;

/// One frame of the transformation, the left half.
struct Key {
    /// Columns: the blade's inner edge, its channel's core, its outer edge below
    /// and above the shoulder, and the nacelle's center.
    inner: i32,
    core: i32,
    outer: i32,
    upper: i32,
    nacelle: i32,
    /// The antenna on the bevelled top: its column, and how far it stands up (0
    /// once it has folded away).
    antenna: (i32, i32),
    /// Where the strut leaves the crossbar.
    strut: i32,
}

/// Flight form to laser form. The blades close in 3 px and slim down, their
/// channels slide together into one core, the antennas fold away and the
/// nacelles tuck in (in flight they hang under the blades' outer edges).
const KEYS: [Key; FORMS] = [
    Key { inner: -3, core: -7, outer: -12, upper: -11, nacelle: -12, antenna: (-9, 4), strut: -7 },
    Key { inner: -2, core: -5, outer: -10, upper: -9, nacelle: -11, antenna: (-7, 3), strut: -7 },
    Key { inner: -1, core: -3, outer: -8, upper: -7, nacelle: -10, antenna: (-5, 1), strut: -6 },
    Key { inner: 0, core: -1, outer: -6, upper: -6, nacelle: -10, antenna: (0, 0), strut: -6 },
    Key { inner: 0, core: 0, outer: -5, upper: -5, nacelle: -9, antenna: (0, 0), strut: -5 },
];
/// The nacelles' and the capsule's half widths.
const NACELLE_W: i32 = 3;
const POD_W: i32 = 3;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum M {
    Empty,
    /// The blade's face.
    Face,
    /// The lighter inner face, between the channel and the gap.
    Rail,
    /// The channel: its dark walls, its black core (the teal dashes), the grooves
    /// beside it below its step.
    Wall,
    Core,
    Groove,
    /// Where the blades meet, above the channel.
    Seam,
    Crack,
    /// The crossbar, the nacelles and the capsule.
    Metal,
    Window,
    Nozzle,
    Antenna,
    Strut,
    Red,
}

/// Which piece a pixel belongs to: a dark line is drawn where two pieces meet.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Piece {
    Blade,
    Bar,
    Nacelle,
    Pod,
    Strut,
}

/// A material map of the whole canvas, in pixels from the hitbox.
#[derive(Clone)]
struct Map {
    m: Vec<(M, Piece)>,
}

impl Map {
    fn new() -> Map {
        Map { m: vec![(M::Empty, Piece::Blade); (SIZE.0 * SIZE.1) as usize] }
    }

    fn at(&self, x: i32, y: i32) -> (M, Piece) {
        if x < -CX || x > CX || y < -CY || y > CY { (M::Empty, Piece::Blade) } else { self.m[((y + CY) * SIZE.0 + x + CX) as usize] }
    }

    fn set(&mut self, x: i32, y: i32, v: (M, Piece)) {
        if !(x < -CX || x > CX || y < -CY || y > CY) {
            self.m[((y + CY) * SIZE.0 + x + CX) as usize] = v;
        }
    }

    /// Lay a pixel and its mirror.
    fn put(&mut self, x: i32, y: i32, m: M, p: Piece) {
        self.set(x, y, (m, p));
        self.set(-x, y, (m, p));
    }

    /// Change the material of a laid pixel (and its mirror), keeping its piece.
    fn over(&mut self, x: i32, y: i32, m: M) {
        for x in [x, -x] {
            let (k, p) = self.at(x, y);
            if k != M::Empty {
                self.set(x, y, (m, p));
            }
        }
    }
}

/// The row where the blade's top edge crosses column `x`.
fn top_at(k: &Key, x: i32) -> i32 {
    (TIP..=FOOT).find(|&y| left_at(k, y) <= x).unwrap_or(FOOT)
}

/// The blade's outermost column on row `y`.
fn left_at(k: &Key, y: i32) -> i32 {
    if y < CORNER {
        k.inner + ((k.upper - k.inner) as f32 * (y - TIP) as f32 / (CORNER - TIP) as f32).round() as i32
    } else if y < SHOULDER {
        k.upper
    } else {
        k.outer
    }
}

/// Lay out one frame of the transformation, unrolled.
fn layout(k: &Key) -> Map {
    let mut map = Map::new();
    // the blade, row by row, its bevelled top running from the tip at its inner edge
    // down to the outer corner
    for y in TIP..=FOOT {
        for x in left_at(k, y)..=k.inner {
            map.put(x, y, M::Face, Piece::Blade);
        }
    }
    // the lighter inner face
    let c = k.core;
    for y in TIP..=FOOT {
        for x in c + 2..=k.inner {
            map.over(x, y, M::Rail);
        }
    }
    // the channel: a lancet arch at its top, grooved and wider below its step
    for y in ARCH..=FOOT {
        let (l, r) = if y == ARCH { (c, c) } else if y < STEP { (c - 1, c + 1) } else { (c - 2, c + 1) };
        for x in l..=r {
            map.over(x, y, if x == c { M::Core } else if x == c - 2 { M::Groove } else { M::Wall });
        }
    }
    map.over(c, 0, M::Red);
    // once they touch, the seam between the blades, from just under the tip to the arch
    if k.inner == 0 {
        for y in TIP + 2..ARCH {
            map.over(0, y, M::Seam);
        }
    }
    for (dx, y) in [(2, -6), (3, -5), (2, 2)] {
        map.over(k.outer + dx, y, M::Crack);
    }
    let (ax, ah) = k.antenna;
    let base = top_at(k, ax);
    for i in 1..=ah {
        map.put(ax, base - i, M::Antenna, Piece::Blade);
    }
    // the crossbar under the blades, out to the nacelles
    let n = k.nacelle;
    for y in FOOT + 1..=FOOT + 3 {
        for x in n + NACELLE_W..=k.inner {
            map.put(x, y, M::Metal, Piece::Bar);
        }
    }
    // the nacelles: rounded, a window, a red light on top, a nozzle beneath
    for y in 8..=14 {
        let w = if y == 8 || y == 14 { NACELLE_W - 1 } else { NACELLE_W };
        for x in n - w..=n + w {
            map.put(x, y, M::Metal, Piece::Nacelle);
        }
    }
    for y in 10..=13 {
        map.put(n, y, M::Window, Piece::Nacelle);
    }
    map.put(n, 8, M::Red, Piece::Nacelle);
    for x in n - 1..=n + 1 {
        map.put(x, 15, M::Nozzle, Piece::Nacelle);
    }
    // the capsule, on struts
    for y in 11..=21 {
        let w = if y == 11 || y == 21 { POD_W - 1 } else { POD_W };
        for x in -w..=w {
            map.put(x, y, M::Metal, Piece::Pod);
        }
    }
    for y in 13..=16 {
        map.put(0, y, M::Window, Piece::Pod);
    }
    map.put(0, 19, M::Red, Piece::Pod);
    map.put(-POD_W, 16, M::Red, Piece::Pod);
    for x in -1..=1 {
        map.put(x, 22, M::Nozzle, Piece::Pod);
    }
    for (x, y) in line((k.strut, 11), (-POD_W - 1, 13)) {
        if map.at(x, y).0 == M::Empty {
            map.put(x, y, M::Strut, Piece::Strut);
        }
    }
    map
}

/// The pixels on a line, ends included.
fn line(a: (i32, i32), b: (i32, i32)) -> Vec<(i32, i32)> {
    let n = (b.0 - a.0).abs().max((b.1 - a.1).abs()).max(1);
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            ((a.0 as f32 + (b.0 - a.0) as f32 * t).round() as i32, (a.1 as f32 + (b.1 - a.1) as f32 * t).round() as i32)
        })
        .collect()
}

/// The columns a roll drops from the dipping side, one per step of the roll: plain
/// stone from the middle of the blade's outer face (never a channel, a window, a
/// light or the antenna), so the face foreshortens and everything outboard of it
/// slides in whole.
fn dropped(map: &Map, bank: i8) -> Vec<i32> {
    let side = bank.signum() as i32;
    let precious = |x: i32| (-CY..=CY).any(|y| matches!(map.at(x, y).0, M::Core | M::Wall | M::Groove | M::Window | M::Red | M::Antenna | M::Seam));
    let face: Vec<i32> = (1..=CX).rev().map(|x| x * side).filter(|&x| map.at(x, 0).0 == M::Face).collect();
    let plain: Vec<i32> = face.iter().skip(1).copied().filter(|&x| !precious(x)).collect();
    let mid = plain.len() / 2;
    match bank.unsigned_abs() {
        0 => vec![],
        1 => plain.get(mid).into_iter().copied().collect(),
        _ => plain.iter().skip(mid.saturating_sub(1)).take(2).copied().collect(),
    }
}

/// Where a column of the unrolled ship lands at a roll.
fn roll_x(x: i32, drop: &[i32]) -> i32 {
    let side = x.signum();
    x - side * drop.iter().filter(|&&d| d.signum() == side && d.abs() < x.abs()).count() as i32
}

/// The ship rolled by `bank`: the dipping side loses its dropped columns.
fn roll(map: &Map, bank: i8) -> Map {
    let drop = dropped(map, bank);
    let mut out = Map::new();
    for y in -CY..=CY {
        for x in -CX..=CX {
            if !drop.contains(&x) {
                out.set(roll_x(x, &drop), y, map.at(x, y));
            }
        }
    }
    out
}

const VOIDISH: Color = rgb(0x0B0B0A);
/// The lights' unlit colors; they're lit over the top at draw time.
const RED_DIM: Color = rgb(0x6A1C17);

const fn rgb(c: u32) -> Color {
    Color::new(((c >> 16) & 0xFF) as f32 / 255.0, ((c >> 8) & 0xFF) as f32 / 255.0, (c & 0xFF) as f32 / 255.0, 1.0)
}

/// A fixed per-pixel noise in 0..1 (the stone's speckle).
fn hash(x: i32, y: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA77) ^ 0x5EED_0007;
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h & 0xFFFF) as f32 / 65536.0
}

/// Thin parts stand alone: no outline, and they don't break the outline of what
/// they stand on.
fn solid(m: M) -> bool {
    !matches!(m, M::Empty | M::Antenna | M::Strut)
}

/// Paint a rolled map in bone, lit from the front. Selective outlines: ink along
/// the bottoms and outer sides and where pieces meet, a mid-tone rim along the top
/// edges (so the point of the sword doesn't vanish into the black), none down the
/// blades' inner edges (the bright rail meets the gap). A bright lip under each top
/// edge, darker bevels down the outer edges, speckled stone. The dipping side is a
/// tone darker.
fn paint(map: &Map, bank: i8) -> Image {
    let mut img = Image::gen_image_color(SIZE.0 as u16, SIZE.1 as u16, Color::new(0.0, 0.0, 0.0, 0.0));
    let bare = |x: i32, y: i32| !solid(map.at(x, y).0);
    let ink = |x: i32, y: i32| -> bool {
        let (m, p) = map.at(x, y);
        if !solid(m) || matches!(m, M::Red | M::Nozzle) {
            return false;
        }
        let out = if x < 0 { -1 } else { 1 };
        let meets = |dx: i32, dy: i32| {
            let (n, q) = map.at(x + dx, y + dy);
            solid(n) && q != p
        };
        let side = if x == 0 { bare(-1, y) || bare(1, y) } else { bare(x + out, y) || (p != Piece::Blade && bare(x - out, y)) };
        bare(x, y + 1) || (side && !bare(x, y - 1)) || meets(0, 1) || (x != 0 && meets(out, 0)) || m == M::Seam
    };
    // a top edge catching the light
    let rim = |x: i32, y: i32| solid(map.at(x, y).0) && bare(x, y - 1) && !ink(x, y);
    for y in -CY..=CY {
        for x in -CX..=CX {
            let (m, p) = map.at(x, y);
            if m == M::Empty {
                continue;
            }
            let out = if x < 0 { -1 } else { 1 };
            let dip = (x * bank as i32 > 0) as i32;
            let speck = hash(x, y);
            let tone = |t: i32| BONE[(t - dip).clamp(0, 4) as usize];
            let c = if ink(x, y) {
                OUTLINE
            } else if rim(x, y) && !matches!(m, M::Red | M::Nozzle | M::Antenna | M::Strut) {
                tone(2)
            } else {
                match m {
                    // cream stone stippled gray, its outer bevel a step darker,
                    // shadowed just above the crossbar
                    M::Face => {
                        let t = if rim(x, y - 1) || ink(x, y - 1) {
                            4
                        } else if x != 0 && ink(x + out, y) {
                            3
                        } else if y == FOOT - 1 {
                            2
                        } else {
                            4
                        };
                        tone(t - (speck < 0.16) as i32 - (speck < 0.03) as i32)
                    }
                    M::Rail => tone(4 - (speck < 0.08) as i32),
                    M::Wall => BONE[0],
                    M::Core | M::Window => VOIDISH,
                    M::Groove => if y % 2 == 0 { BONE[0] } else { OUTLINE },
                    M::Seam => OUTLINE,
                    M::Crack => BONE[1],
                    M::Metal => {
                        let up = map.at(x, y - 1);
                        let t = if rim(x, y - 1) || ink(x, y - 1) || up.1 != p {
                            4
                        } else if ink(x, y + 1) {
                            1
                        } else if ink(x + out, y) {
                            2
                        } else {
                            3
                        };
                        tone(t - (speck < 0.08) as i32)
                    }
                    M::Nozzle => if map.at(x - 1, y).0 == M::Nozzle && map.at(x + 1, y).0 == M::Nozzle { BONE[0] } else { BONE[1] },
                    M::Antenna => {
                        let above = map.at(x, y - 1).0 == M::Antenna;
                        let second = above && map.at(x, y - 2).0 != M::Antenna;
                        if !above { BONE[4] } else if second { BONE[1] } else { tone(3) }
                    }
                    M::Strut => tone(2),
                    M::Red => RED_DIM,
                    M::Empty => continue,
                }
            };
            img.set_pixel((x + CX) as u32, (y + CY) as u32, c);
        }
    }
    img
}

/// What a light is, and so how it behaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glow {
    /// The teal dashes running up a channel.
    Channel,
    /// The red heart in each channel (one, in the laser form).
    Heart,
    /// The red lights on the nacelles, taking turns.
    Beacon,
    /// The capsule's red light underneath, and its strobes either side.
    Keel,
    Strobe,
    /// The teal dashes in the nacelles' and the capsule's windows.
    Window,
}

#[derive(Clone, Copy, Debug)]
pub struct Light {
    /// Offset from the hitbox.
    pub dx: i32,
    pub dy: i32,
    pub glow: Glow,
}

/// Every light in a rolled map.
fn lights(map: &Map) -> Vec<Light> {
    let mut v = vec![];
    for y in -CY..=CY {
        for x in -CX..=CX {
            let (m, p) = map.at(x, y);
            let glow = match (m, p) {
                (M::Core, _) => Glow::Channel,
                (M::Window, _) if y % 2 == 0 => Glow::Window,
                (M::Red, Piece::Blade) => Glow::Heart,
                (M::Red, Piece::Nacelle) => Glow::Beacon,
                (M::Red, Piece::Pod) if x == 0 => Glow::Keel,
                (M::Red, _) => Glow::Strobe,
                _ => continue,
            };
            v.push(Light { dx: x, dy: y, glow });
        }
    }
    v
}

/// One frame of the ship: a form at a roll.
pub struct Frame {
    pub r: Rect,
    pub sil: Rect,
    pub lights: Vec<Light>,
    /// From the hitbox: the nacelles' nozzles and the capsule's (the flames hang
    /// below them), the blades' tips (the bolts' muzzle flashes; one, once they
    /// touch) and the antenna tips (the pod shots').
    pub nozzles: [(i32, i32); 3],
    pub tips: [(i32, i32); 2],
    pub guns: [(i32, i32); 2],
}

pub struct ShipArt {
    /// `frames[form][bank + 2]`.
    pub frames: Vec<Vec<Frame>>,
    pub flame_idle: [Rect; 4],
    pub flame_thrust: [Rect; 4],
    pub jet_idle: [Rect; 4],
    pub jet_thrust: [Rect; 4],
    pub muzzle: [Rect; 4],
    pub bolt: [Rect; 4],
    pub pod_shot: [Rect; 2],
    pub dot: Rect,
    /// Every pixel of the ship in its flight form, lights lit, relative to the
    /// hitbox: the death shatters it into exactly these.
    pub pixels: Vec<(i8, i8, Color)>,
}

impl ShipArt {
    /// The frame for a focus blend (0 flight, 1 laser) and a roll.
    pub fn frame(&self, form: f32, bank: i8) -> &Frame {
        &self.frames[form_index(form)][(bank.clamp(-2, 2) + 2) as usize]
    }

    /// Every atlas region the ship uses (for the overlap test).
    #[cfg(test)]
    pub fn rects(&self) -> Vec<Rect> {
        let mut v: Vec<Rect> = self.frames.iter().flatten().flat_map(|f| [f.r, f.sil]).collect();
        v.push(self.dot);
        for set in [&self.flame_idle, &self.flame_thrust, &self.jet_idle, &self.jet_thrust, &self.muzzle, &self.bolt] {
            v.extend(set);
        }
        v.extend(self.pod_shot);
        v
    }
}

/// Which frame of the transformation a focus blend shows.
pub fn form_index(form: f32) -> usize {
    (form.clamp(0.0, 1.0) * (FORMS - 1) as f32).round() as usize
}

/// The nacelles' twin flames under their nozzles, idle flicker and full thrust, 4
/// frames each. Column 1 is the nozzle's center.
const FLAME_IDLE: [&[&str]; 4] = [
    &["T.T", "t.t", "..t"],
    &["T.T", "T.t", "t..", "..t"],
    &["T.T", "t.T", "..t"],
    &["T.T", "t.t", "t..", "...", "t.."],
];
const FLAME_THRUST: [&[&str]; 4] = [
    &["CTC", "T.T", "T.T", "t.T", "..t", "...", "..t"],
    &["CTC", "T.T", "T.t", "t.T", "t..", "..t", "...", "t.."],
    &["CTC", "T.T", "T.T", "T.t", "..t", "t..", "...", "..t"],
    &["CTC", "T.T", "t.T", "T.t", "t..", "...", "t.."],
];
/// The capsule's jet. Column 1 is the nozzle's center.
const JET_IDLE: [&[&str]; 4] = [&[".T.", ".t."], &[".T.", ".t.", ".t."], &[".T.", ".t."], &[".T.", ".t.", "...", ".t."]];
const JET_THRUST: [&[&str]; 4] = [
    &["tCt", ".T.", ".T.", ".t.", "...", ".t."],
    &["tCt", ".T.", ".T.", ".t.", ".t.", "...", ".t."],
    &["tCt", ".T.", ".T.", ".T.", ".t.", "...", "..."],
    &["tCt", ".T.", ".t.", ".t.", "...", ".t."],
];
/// Muzzle flashes, 4 frames (the fire animation). Column 2 is the center, the bottom row sits on the muzzle.
const MUZZLE: [&[&str]; 4] = [
    &["..R..", ".RCR.", "RCCCR", ".RCR."],
    &["..R..", ".RCR.", ".RCR."],
    &[".R.R.", "..C..", ".RCR."],
    &["..r..", ".rRr."],
];
/// The twin red bolt, 4 frames (from the sheet's PROJECTILE row): a hot head, a streak, a dotted tail.
const BOLT: [&[&str]; 4] = [
    &[".R.", "RCR", "RCR", "RCR", "RRR", "RRR", ".R.", ".R.", ".R.", ".R.", ".r.", ".r.", "...", ".r.", "...", "...", ".r.", "..."],
    &[".R.", "RCR", "RCR", "RCR", "RRR", ".R.", ".R.", ".R.", ".R.", ".r.", ".r.", ".r.", "...", "...", ".r.", "...", "...", ".r."],
    &[".R.", "RCR", "RCR", "RRR", "RRR", ".R.", ".R.", ".R.", ".R.", ".R.", ".r.", "...", ".r.", "...", "...", ".r.", "...", "..."],
    &[".R.", "RCR", "RCR", "RCR", "RRR", "RRR", ".R.", ".R.", ".R.", ".r.", ".r.", "...", "...", ".r.", "...", "...", "...", ".r."],
];
/// The thinner pod shot.
const POD_SHOT: [&[&str]; 2] = [
    &["R.", "CR", "RR", "R.", "R.", "R.", "r.", "r.", "r.", "..", "r.", "..", "..", "r."],
    &[".R", "RC", "RR", ".R", ".R", ".R", ".r", ".r", ".r", "..", ".r", "..", "r.", ".."],
];
/// The hitbox dot (2 px radius), shown while focused.
const DOT: [&str; 5] = [".TTT.", "TCCCT", "TCCCT", "TCCCT", ".TTT."];

fn pal(c: char) -> Option<Color> {
    match c {
        'C' => Some(CREAM),
        'R' => Some(SIGNAL),
        'T' => Some(TEAL),
        'r' => Some(RED_DIM),
        't' => Some(rgb(0x13_57_46)),
        _ => None,
    }
}

fn small(p: &mut Packer, src: &[&str]) -> Rect {
    p.add(&grid_image(&rows(src), pal))
}

/// A light's color when lit (the shatter's, and the sheet's).
fn lit(g: Glow) -> Color {
    match g {
        Glow::Channel | Glow::Window => TEAL,
        _ => SIGNAL,
    }
}

/// One frame, painted, with its lights and its points.
fn frame_image(form: usize, bank: i8) -> (Image, Vec<Light>, [(i32, i32); 3], [(i32, i32); 2], [(i32, i32); 2]) {
    let k = &KEYS[form];
    let flat = layout(k);
    let drop = dropped(&flat, bank);
    let map = roll(&flat, bank);
    let at = |x: i32, y: i32| (roll_x(x, &drop), y);
    let n = k.nacelle;
    let nozzles = [at(n, 16), at(-n, 16), at(0, 23)];
    let tips = [at(k.inner, TIP), at(-k.inner, TIP)];
    let (ax, ah) = k.antenna;
    let gun = if ah > 0 { (ax, top_at(k, ax) - ah) } else { (k.upper + 1, CORNER - 1) };
    let guns = [at(gun.0, gun.1), at(-gun.0, gun.1)];
    (paint(&map, bank), lights(&map), nozzles, tips, guns)
}

pub fn build(p: &mut Packer) -> ShipArt {
    let frames = (0..FORMS)
        .map(|form| {
            (0..BANKS)
                .map(|i| {
                    let bank = i as i8 - 2;
                    let (img, lights, nozzles, tips, guns) = frame_image(form, bank);
                    let (r, sil) = p.add_with_silhouette(&img);
                    Frame { r, sil, lights, nozzles, tips, guns }
                })
                .collect()
        })
        .collect();
    ShipArt {
        frames,
        flame_idle: FLAME_IDLE.map(|g| small(p, g)),
        flame_thrust: FLAME_THRUST.map(|g| small(p, g)),
        jet_idle: JET_IDLE.map(|g| small(p, g)),
        jet_thrust: JET_THRUST.map(|g| small(p, g)),
        muzzle: MUZZLE.map(|g| small(p, g)),
        bolt: BOLT.map(|g| small(p, g)),
        pod_shot: POD_SHOT.map(|g| small(p, g)),
        dot: small(p, &DOT),
        pixels: ship_pixels(),
    }
}

/// Every pixel of the flight form, lights lit, relative to the hitbox.
fn ship_pixels() -> Vec<(i8, i8, Color)> {
    let (img, lights, ..) = frame_image(0, 0);
    let mut v = vec![];
    for y in 0..SIZE.1 {
        for x in 0..SIZE.0 {
            let c = img.get_pixel(x as u32, y as u32);
            if c.a > 0.0 {
                let (dx, dy) = (x - CX, y - CY);
                let c = lights.iter().find(|l| (l.dx, l.dy) == (dx, dy) && (l.glow != Glow::Channel || dy % 2 == 0)).map_or(c, |l| lit(l.glow));
                v.push((dx as i8, dy as i8, c));
            }
        }
    }
    v
}

/// Everything about how the ship looks on one frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct Pose {
    pub bank: i8,
    /// The focus blend: 0 is the flight form, 1 the laser form.
    pub form: f32,
    pub thrust: bool,
    /// Frames spent shooting in a row (0 when not shooting).
    pub fire_t: u32,
    pub beaming: bool,
    /// Hull flash frames left (counts down from BUMP_FLASH).
    pub hit_t: u32,
    /// Frames since the blades locked together (0 while they're apart).
    pub locked: u32,
    pub frame: u64,
}

/// A cheap per-frame flicker in 0..1 (cosmetic, from the frame number).
fn flicker(frame: u64, salt: u64) -> f32 {
    let mut z = frame.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ salt.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z ^= z >> 29;
    (z % 1000) as f32 / 1000.0
}

/// How a light looks this frame, if it's on.
fn light_on(l: &Light, pose: &Pose) -> Option<(Color, f32)> {
    let f = pose.frame;
    match l.glow {
        Glow::Channel => {
            // a charge running up to the tip as the blades lock
            if (1..=8).contains(&pose.locked) {
                let front = FOOT + 1 - pose.locked as i32 * 4;
                if (l.dy - front).abs() <= 2 {
                    return Some((TEAL_HOT, 1.0));
                }
            }
            // dashes flowing up the channel: slowly at rest, fast while it fires
            let (every, k) = if pose.beaming { (1, 1.0) } else if pose.fire_t > 0 { (3, 1.0) } else { (8, 0.8) };
            let phase = (f / every) as i32;
            ((l.dy + phase).rem_euclid(4) < 2).then_some((if pose.beaming { TEAL_HOT } else { TEAL }, k))
        }
        Glow::Heart => {
            let base = if pose.fire_t > 0 { 0.75 } else { 0.4 };
            let pulse = 0.5 + 0.5 * (f as f32 * 0.12 + l.dx as f32).sin();
            Some((SIGNAL, (base + 0.25 * pulse).min(1.0)))
        }
        // the two nacelles take turns
        Glow::Beacon => (((f / 30) % 2 == 0) == (l.dx < 0)).then_some((SIGNAL, 1.0)),
        Glow::Keel => (f % 60 < 48).then_some((SIGNAL, 1.0)),
        // a red double strobe
        Glow::Strobe => (f % 90 < 4 || (8..12).contains(&(f % 90))).then_some((SIGNAL, 1.0)),
        Glow::Window => Some((TEAL, 0.5 + 0.35 * flicker(f / 3, (l.dx * 31 + l.dy) as u64))),
    }
}

/// Draw the Pilgrim with its hitbox at whole pixel `(x, y)`.
pub fn draw(a: &Atlas, x: i32, y: i32, pose: &Pose) {
    let art = &a.r.ship;
    let fr = art.frame(pose.form, pose.bank);

    // the flames under everything, hanging from the nozzles
    for (i, &(nx, ny)) in fr.nozzles.iter().enumerate() {
        let src = match (i < 2, pose.thrust) {
            (true, true) => art.flame_thrust[((pose.frame / 3 + i as u64) % 4) as usize],
            (true, false) => art.flame_idle[((pose.frame / 4 + i as u64) % 4) as usize],
            (false, true) => art.jet_thrust[(pose.frame / 3 % 4) as usize],
            (false, false) => art.jet_idle[(pose.frame / 4 % 4) as usize],
        };
        a.blit(src, x + nx - 1, y + ny, 1, WHITE);
    }

    let (ox, oy) = (x - CX, y - CY);
    a.blit(fr.r, ox, oy, 1, WHITE);
    for l in &fr.lights {
        if let Some((c, k)) = light_on(l, pose) {
            a.fill(x + l.dx, y + l.dy, 1, 1, Color { a: k, ..c });
        }
    }
    if pose.hit_t > 0 {
        hit_overlay(a, fr.sil, ox, oy, pose.hit_t);
    }

    // the blades lock: a glint down the seam
    if (1..=6).contains(&pose.locked) {
        let k = (7 - pose.locked) as f32 / 6.0;
        a.fill(x, y + TIP, 1, FOOT - TIP, Color { a: 0.6 * k, ..TEAL_HOT });
    }
    if pose.beaming {
        let k = if pose.frame % 4 < 2 { 0.95 } else { 0.7 };
        a.blit(a.r.plus, x - 1, y + TIP - 2, 1, Color { a: k, ..TEAL_HOT });
    } else if pose.fire_t > 0 {
        let m = art.muzzle[(pose.fire_t % 4) as usize];
        for &(tx, ty) in &fr.tips {
            a.blit(m, x + tx - 2, y + ty - m.h as i32 + 1, 1, WHITE);
        }
        let m = art.muzzle[((pose.fire_t + 2) % 4) as usize];
        for &(gx, gy) in &fr.guns {
            a.blit(m, x + gx - 2, y + gy - m.h as i32 + 1, 1, Color { a: 0.8, ..WHITE });
        }
    }
}

/// The hit flash from the sheet: red, redder, a cream blaze, a red afterglow.
fn hit_overlay(a: &Atlas, sil: Rect, x: i32, y: i32, hit_t: u32) {
    let phase = (crate::tune::BUMP_FLASH.saturating_sub(hit_t)) * 4 / crate::tune::BUMP_FLASH.max(1);
    let c = match phase {
        0 => Color { a: 0.35, ..SIGNAL },
        1 => Color { a: 0.7, ..SIGNAL },
        2 => Color { a: 0.85, ..CREAM },
        _ => Color { a: 0.25, ..SIGNAL },
    };
    a.blit(sil, x, y, 1, c);
}

/// The ship's silhouette at `(x, y)` (its hitbox), in one color: afterimages,
/// the respawn blink.
pub fn draw_silhouette(a: &Atlas, form: f32, bank: i8, x: i32, y: i32, c: Color) {
    a.blit(a.r.ship.frame(form, bank).sil, x - CX, y - CY, 1, c);
}

/// Teal afterimages of the ship at earlier positions (drawn under the ship).
pub fn draw_afterimages(a: &Atlas, form: f32, bank: i8, at: &[(i32, i32)]) {
    for (k, &(x, y)) in at.iter().enumerate() {
        let alpha = [0.14, 0.08, 0.04][k.min(2)];
        draw_silhouette(a, form, bank, x, y, Color { a: alpha, ..TEAL });
    }
}

/// The hitbox dot and its ring, faded in with the focus blend.
pub fn draw_hitbox(a: &Atlas, x: i32, y: i32, blend: f32, frame: u64) {
    if blend <= 0.0 {
        return;
    }
    // a dashed ring that turns slowly
    let r = 9.0;
    for k in 0..12 {
        let ang = k as f32 / 12.0 * std::f32::consts::TAU + frame as f32 * 0.03;
        let (dx, dy) = ((ang.cos() * r).round() as i32, (ang.sin() * r).round() as i32);
        a.fill(x + dx, y + dy, 1, 1, Color { a: 0.75 * blend, ..TEAL });
    }
    a.fill(x - 3, y - 3, 7, 7, Color { a: 0.5 * blend, ..VOID });
    a.blit(a.r.ship.dot, x - 2, y - 2, 1, Color { a: blend, ..WHITE });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::palette::luma;

    /// The material at a point of an unrolled form.
    fn mat(form: usize, x: i32, y: i32) -> M {
        layout(&KEYS[form]).at(x, y).0
    }

    /// The columns a form covers on a row.
    fn span(map: &Map, y: i32) -> Vec<i32> {
        (-CX..=CX).filter(|&x| map.at(x, y).0 != M::Empty).collect()
    }

    #[test]
    fn grids_are_rectangular_and_use_the_palette() {
        for set in [&FLAME_IDLE[..], &FLAME_THRUST[..], &JET_IDLE[..], &JET_THRUST[..], &MUZZLE[..], &BOLT[..], &POD_SHOT[..], &[&DOT[..]][..]] {
            for g in set {
                assert!(g.iter().all(|r| r.len() == g[0].len()), "{g:?}");
                assert!(g.iter().flat_map(|r| r.chars()).all(|c| c == '.' || pal(c).is_some()), "{g:?}");
            }
        }
    }

    #[test]
    fn the_pilgrim_is_the_planned_size() {
        let map = layout(&KEYS[0]);
        let cols: Vec<i32> = (TIP - 4..=CY).flat_map(|y| span(&map, y)).collect();
        let (lo, hi) = (*cols.iter().min().unwrap(), *cols.iter().max().unwrap());
        assert_eq!(lo, -hi, "symmetric about the hitbox");
        assert!((29..=33).contains(&(hi - lo + 1)), "{} wide with its nacelles", hi - lo + 1);
        let rows: Vec<i32> = (-CY..=CY).filter(|&y| !span(&map, y).is_empty()).collect();
        let h = rows.last().unwrap() - rows[0] + 1;
        assert!((42..=48).contains(&h), "{h} tall");
        assert_eq!(rows.last(), Some(&22), "the capsule's nozzle is the bottom row");
    }

    /// Flight form: the hitbox sits in the gap between the blades, level with their
    /// red hearts. Laser form: the blades are one sword, its heart on the hitbox and
    /// its tip straight above it.
    #[test]
    fn the_hitbox_sits_between_the_hearts_then_on_the_heart() {
        assert_eq!(mat(0, 0, 0), M::Empty);
        assert_eq!((mat(0, -7, 0), mat(0, 7, 0)), (M::Red, M::Red));
        let last = FORMS - 1;
        assert_eq!(mat(last, 0, 0), M::Red);
        assert_ne!(mat(last, 0, TIP), M::Empty);
        assert_eq!(span(&layout(&KEYS[last]), TIP), vec![0], "one point");
        assert_eq!(mat(last, 0, TIP - 1), M::Empty);
    }

    /// Frame by frame the blades close, slim down and lose their antennas, the
    /// channels slide together into one core, and the nacelles tuck in.
    #[test]
    fn the_blades_close_into_one_sword() {
        let gap = |f: usize| (TIP..=FOOT).filter(|&y| mat(f, 0, y) == M::Empty).count();
        let width = |f: usize| span(&layout(&KEYS[f]), 0).iter().filter(|&&x| x < 0 && mat(f, x, 0) != M::Empty).count();
        let antenna = |f: usize| (-CY..=CY).flat_map(|y| (-CX..=CX).map(move |x| (x, y))).filter(|&(x, y)| mat(f, x, y) == M::Antenna).count();
        for f in 1..FORMS {
            let (a, b) = (&KEYS[f - 1], &KEYS[f]);
            assert!(b.inner >= a.inner && b.core > a.core && b.outer > a.outer && b.nacelle >= a.nacelle, "frame {f}");
            assert!(gap(f) <= gap(f - 1) && antenna(f) <= antenna(f - 1), "frame {f}");
        }
        assert_eq!(gap(0), (FOOT - TIP + 1) as usize, "flight: split from tip to foot");
        assert_eq!(gap(FORMS - 1), 0, "laser: one blade");
        assert!(antenna(0) > 0 && antenna(FORMS - 1) == 0);
        assert!(KEYS[FORMS - 1].nacelle - KEYS[0].nacelle >= 3, "the nacelles tuck in");
        assert!(width(FORMS - 1) < width(0), "the laser form is slimmer");
        let cores = |f: usize| (-CX..=CX).filter(|&x| mat(f, x, -5) == M::Core).collect::<Vec<_>>();
        assert_eq!(cores(0), vec![-7, 7]);
        assert_eq!(cores(FORMS - 1), vec![0], "the channels become one");
    }

    /// The value budget: the Pilgrim tops out at 85%.
    #[test]
    fn the_hull_stays_inside_its_value_budget() {
        for form in 0..FORMS {
            for bank in -2..=2 {
                let img = frame_image(form, bank).0;
                for p in img.get_image_data().iter().filter(|p| p[3] > 0) {
                    assert!(luma(Color::from_rgba(p[0], p[1], p[2], 255)) <= 0.85, "form {form} bank {bank}: {p:?}");
                }
            }
        }
    }

    #[test]
    fn the_shatter_uses_every_pixel_of_the_ship() {
        let px = ship_pixels();
        let (img, ..) = frame_image(0, 0);
        assert_eq!(px.len(), img.get_image_data().iter().filter(|p| p[3] > 0).count());
        assert!(px.iter().any(|p| p.2 == TEAL) && px.iter().any(|p| p.2 == SIGNAL), "its lights are lit");
        let (lo, hi) = (px.iter().map(|p| p.0).min().unwrap(), px.iter().map(|p| p.0).max().unwrap());
        assert_eq!(lo, -hi, "symmetric about the hitbox");
    }

    #[test]
    fn rolling_keeps_the_hitbox_and_every_kind_of_light() {
        for form in 0..FORMS {
            let flat = roll(&layout(&KEYS[form]), 0);
            let kinds = |m: &Map| {
                let mut k: Vec<Glow> = lights(m).iter().map(|l| l.glow).collect();
                k.sort_by_key(|g| *g as u8);
                k.dedup();
                k
            };
            for bank in [-2i8, -1, 1, 2] {
                let r = roll(&layout(&KEYS[form]), bank);
                for y in -CY..=CY {
                    assert_eq!(r.at(0, y), flat.at(0, y), "form {form} bank {bank}: the center column stays");
                }
                assert_eq!(kinds(&r), kinds(&flat), "form {form} bank {bank}");
                let reach = |m: &Map, side: i32| (-CY..=CY).flat_map(|y| span(m, y)).map(|x| x * side).max().unwrap();
                let side = bank.signum() as i32;
                assert!(reach(&r, side) < reach(&flat, side), "form {form} bank {bank}: the dipping side narrows");
                assert_eq!(reach(&r, -side), reach(&flat, -side));
            }
        }
    }

    #[test]
    fn every_frame_has_a_frame_index() {
        assert_eq!(form_index(0.0), 0);
        assert_eq!(form_index(1.0), FORMS - 1);
        assert_eq!(form_index(2.0), FORMS - 1);
        let seen: Vec<usize> = (0..=5).map(|i| form_index(i as f32 / 5.0)).collect();
        assert!(seen.windows(2).all(|w| w[1] >= w[0]));
    }

    /// A review sheet of every frame, 6x, lights lit: `cargo test --release
    /// pilgrim_sheet -- --ignored` writes shots/pilgrim_frames.png.
    #[test]
    #[ignore]
    fn pilgrim_sheet() {
        let s = 6u32;
        let (cw, ch) = (SIZE.0 as u32 + 4, SIZE.1 as u32 + 12);
        let mut out = Image::gen_image_color((cw * BANKS as u32 * s) as u16, (ch * FORMS as u32 * s) as u16, VOID);
        for form in 0..FORMS {
            for (bi, bank) in (-2..=2i8).enumerate() {
                let (img, lights, nozzles, ..) = frame_image(form, bank);
                let mut cell = Image::gen_image_color(cw as u16, ch as u16, VOID);
                for y in 0..SIZE.1 as u32 {
                    for x in 0..SIZE.0 as u32 {
                        let c = img.get_pixel(x, y);
                        if c.a > 0.0 {
                            cell.set_pixel(x + 2, y + 2, c);
                        }
                    }
                }
                let put = |cell: &mut Image, dx: i32, dy: i32, c: Color| {
                    let (x, y) = (dx + CX + 2, dy + CY + 2);
                    if x >= 0 && y >= 0 && (x as u32) < cw && (y as u32) < ch {
                        cell.set_pixel(x as u32, y as u32, c);
                    }
                };
                for l in &lights {
                    if l.glow != Glow::Channel || l.dy.rem_euclid(4) < 2 {
                        put(&mut cell, l.dx, l.dy, lit(l.glow));
                    }
                }
                for (i, &(nx, ny)) in nozzles.iter().enumerate() {
                    let g = rows(if i < 2 { FLAME_IDLE[0] } else { JET_IDLE[0] });
                    for (yy, row) in g.iter().enumerate() {
                        for (xx, &c) in row.iter().enumerate() {
                            if let Some(c) = pal(c) {
                                put(&mut cell, nx - 1 + xx as i32, ny + yy as i32, c);
                            }
                        }
                    }
                }
                for y in 0..ch * s {
                    for x in 0..cw * s {
                        let c = cell.get_pixel(x / s, y / s);
                        out.set_pixel(bi as u32 * cw * s + x, form as u32 * ch * s + y, c);
                    }
                }
            }
        }
        // Image::export_png writes the bottom row first
        let (w, h) = (out.width as u32, out.height as u32);
        let mut flipped = Image::gen_image_color(w as u16, h as u16, VOID);
        for y in 0..h {
            for x in 0..w {
                flipped.set_pixel(x, h - 1 - y, out.get_pixel(x, y));
            }
        }
        let _ = std::fs::create_dir_all("shots");
        flipped.export_png("shots/pilgrim_frames.png");
    }
}
