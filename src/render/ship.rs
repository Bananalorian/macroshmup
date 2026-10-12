//! The Pilgrim, redrawn pixel by pixel at game size from Jason's sprites
//! (docs/reference/ship/: the modular atlas and the firing strip, after the
//! split-sword sheet). Two bone blades, each with a bevelled top, an antenna, two
//! plates stepping out down its outer edge, and a teal channel near its inner edge
//! with a red heart. An engine nacelle hangs at each end of the crossbar beneath,
//! and a capsule rides below on struts. Focusing slides the blades together, folding
//! their plates and antennas away and tucking the nacelles in, until they lock into
//! the laser form's sword: a long seamed point, two stepped shoulders, one channel.
//! The beam fires from its tip.
//!
//! Every frame is put together from those parts and never scaled. A roll folds the
//! dipping side's plates (the sword's shoulders) and tucks its nacelle in, so the
//! white stone down every edge stays whole. The lights are found in the frames and
//! lit at draw time.

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
/// Rows from the hitbox: the tip (the beam leaves from here) and the blades' last
/// row.
pub const TIP: i32 = -22;
const FOOT: i32 = 6;
/// The crossbar's middle row (sparks fly off it as the blades lock).
pub const BAR: i32 = FOOT + 2;
/// The row where a blade's 45-degree bevel meets its outer edge (the channel's arch
/// sits on it), and where each plate steps out down the outer edge.
const CORNER: i32 = TIP + 7;
const PLATES: [i32; 2] = [-7, 0];
/// The rows where the sword's shoulders step out.
const SHOULDERS: [i32; 2] = [-4, 2];
/// The nacelles' top row and the capsule's.
const NACELLE_TOP: i32 = FOOT + 1;
const POD_TOP: i32 = FOOT + 4;

/// One frame of the transformation.
struct Form {
    /// The left blade's inner edge (the right blade mirrors it).
    inner: i32,
    /// Plates out down each blade's outer edge (the sword's shoulders).
    plates: i32,
    /// How far the antennas stand.
    antenna: i32,
    /// The left nacelle's center.
    nacelle: i32,
}

/// Flight form to laser form: the blades close 3 px, folding a plate a frame and
/// their antennas, until their outer edges are the sword's. The nacelles tuck in.
const FORM: [Form; FORMS] = [
    Form { inner: -3, plates: 2, antenna: 4, nacelle: -11 },
    Form { inner: -2, plates: 1, antenna: 3, nacelle: -11 },
    Form { inner: -1, plates: 0, antenna: 1, nacelle: -10 },
    Form { inner: 0, plates: 0, antenna: 0, nacelle: -10 },
    Form { inner: 0, plates: 2, antenna: 0, nacelle: -10 },
];
/// The laser form: the sword in place of the blades.
const SWORD: usize = FORMS - 1;

/// The nacelle (the left one; the right mirrors it), 7 wide around its center,
/// from `NACELLE_TOP`. `b` its beacon, `v` its dark window, `n` the window's lights.
const NACELLE: [&str; 8] = [".o2b2o.", "o34443o", "o3vnv2o", "o3vvv2o", "o3vnv2o", "o3vvv2o", "o34442o", ".o222o."];
/// The capsule, 7 wide around the center, from `POD_TOP`: its window, a strobe
/// either side (`s`), the keel light (`k`).
const POD: [&str; 13] = [
    "..o2o..", ".o343o.", "o34443o", "o3vnv3o", "o3vvv3o", "s3vnv3s", "o3vvv3o", "o3vnv3o", "o34443o", "o33k33o", "o34443o", ".o343o.", "..o2o..",
];

/// A pixel of a part: `(x, y, letter)`. The letters: `o` outline, `0`-`4` bone dark
/// to light, `c` channel core, `v` dark window, and the lights `h` heart, `b`
/// beacon, `k` keel, `s` strobe, `n` window.
type Px = (i32, i32, char);

/// A fixed per-pixel noise in 0..1 (the stone's speckle).
fn hash(x: i32, y: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA77) ^ 0x5EED_0007;
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h & 0xFFFF) as f32 / 65536.0
}

/// Speckle a part's lightest stone by its own coordinates, so the speckle moves
/// with the part.
fn speckle(px: &mut [Px]) {
    for p in px.iter_mut() {
        if p.2 == '4' && hash(p.0, p.1) < 0.14 {
            p.2 = '3';
        }
    }
}

/// The channel's core on row `y`: dark, its heart on the hitbox's row, vents below.
fn core(y: i32) -> char {
    match y {
        ..0 => 'c',
        0 => 'h',
        _ if y % 2 != 0 => 'o',
        _ => '1',
    }
}

/// The left blade, `x` from its inner edge. A 45-degree bevel runs from the tip (its
/// inner top corner) down to the outer edge. Outward from the inner edge: the inner
/// face, the channel (walls either side of its core), three columns of white stone,
/// the outline, and down the lower part `plates` plates stepping out.
fn blade(plates: i32, antenna: i32) -> Vec<Px> {
    let mut v = vec![];
    for y in TIP..=FOOT {
        if y <= CORNER {
            let outer = TIP - y;
            v.extend((outer..=0).map(|u| (u, y, if u == outer { '2' } else { '4' })));
        } else {
            v.extend([(-7, y, 'o'), (-6, y, '3'), (-5, y, '4'), (-4, y, '4'), (0, y, '4')]);
            for (i, &top) in PLATES.iter().enumerate().take(plates as usize) {
                let edge = -8 - i as i32;
                if y >= top {
                    // a lit step on top, then the plate's shaded face along the outline
                    v.extend([(edge, y, if y == top { '2' } else { 'o' }), (edge + 1, y, '2')]);
                }
            }
        }
        if y == CORNER {
            v.push((-2, y, '0'));
        } else if y > CORNER {
            v.extend([(-3, y, '0'), (-1, y, '0'), (-2, y, core(y))]);
        }
    }
    v.extend([(-5, -10, '1'), (-6, -9, '1'), (-5, -3, '2')]);
    v.extend((0..antenna).map(|k| (-6, CORNER - 2 - k, if k == antenna - 1 { '3' } else { '2' })));
    speckle(&mut v);
    v
}

/// The laser form's sword, `x` from the center: a long point with a seam down it,
/// straight below, then a shoulder stepping out for each of `plates` (per side,
/// `[left, right]`). One channel down its middle.
fn sword(plates: [i32; 2]) -> Vec<Px> {
    // the point's rows, outer edge first; the last letter is the center column
    const POINT: [&str; 11] = ["2", "o2", "4o", "34o", "44o", "344o", "444o", "2444o", "4444o", "34444o", "44444o"];
    let mut v = vec![];
    for (side, plates) in [(-1, plates[0]), (1, plates[1])] {
        for y in TIP..=FOOT {
            let r = (y - TIP) as usize;
            if let Some(row) = POINT.get(r) {
                let n = row.len() as i32;
                v.extend(row.chars().enumerate().map(|(i, c)| ((n - 1 - i as i32) * side, y, c)));
                continue;
            }
            let w = 5 + SHOULDERS.iter().take(plates as usize).filter(|&&s| y >= s).count() as i32;
            v.extend((0..w).map(|x| (x * side, y, '4')));
            // each shoulder lit on top, the step it makes shaded
            let top = SHOULDERS.contains(&y) && w > 5;
            v.extend([(w * side, y, if top { '2' } else { 'o' }), ((w - 1) * side, y, if w > 5 { '2' } else { '3' })]);
        }
    }
    v.extend((TIP + 2..CORNER + 1).map(|y| (0, y, 'o')));
    for y in CORNER + 1..=FOOT {
        v.extend([(-1, y, '0'), (1, y, '0'), (0, y, core(y))]);
    }
    speckle(&mut v);
    v
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

/// A frame being put together: a letter per pixel of the canvas, from the hitbox.
#[derive(Clone)]
struct Canvas {
    px: Vec<char>,
}

impl Canvas {
    fn new() -> Canvas {
        Canvas { px: vec!['.'; (SIZE.0 * SIZE.1) as usize] }
    }

    fn at(&self, x: i32, y: i32) -> char {
        if x < -CX || x > CX || y < -CY || y > CY { '.' } else { self.px[((y + CY) * SIZE.0 + x + CX) as usize] }
    }

    fn set(&mut self, x: i32, y: i32, c: char) {
        if c != '.' && !(x < -CX || x > CX || y < -CY || y > CY) {
            self.px[((y + CY) * SIZE.0 + x + CX) as usize] = c;
        }
    }

    /// Lay a part's pixels, moved `dx` and, on the right side (`side` 1), mirrored.
    fn stamp(&mut self, px: &[Px], dx: i32, side: i32) {
        for &(x, y, c) in px {
            let x = x + dx;
            self.set(if side > 0 { -x } else { x }, y, c);
        }
    }

    /// Lay a grid with its top left at `(x0, y0)`, mirrored about the center on the
    /// right side.
    fn grid(&mut self, rows: &[&str], x0: i32, y0: i32, side: i32) {
        for (r, row) in rows.iter().enumerate() {
            for (i, c) in row.chars().enumerate() {
                let x = x0 + i as i32;
                self.set(if side > 0 { -x } else { x }, y0 + r as i32, c);
            }
        }
    }
}

/// Where each side's nacelle hangs (its center) and how many plates its blade (or
/// the sword's shoulder) shows, at a roll: the dipping side folds a plate and tucks
/// its nacelle in a pixel for each step of the roll. `[left, right]`.
fn rolled(f: &Form, bank: i8) -> ([i32; 2], [i32; 2]) {
    let dip = |s: usize| if (bank < 0) == (s == 0) && bank != 0 { bank.unsigned_abs() as i32 } else { 0 };
    ([f.nacelle + dip(0), f.nacelle + dip(1)], [(f.plates - dip(0)).max(0), (f.plates - dip(1)).max(0)])
}

/// Put one frame together: a form at a roll.
fn compose(form: usize, bank: i8) -> Canvas {
    let f = &FORM[form];
    let (nacelle, plates) = rolled(f, bank);
    let mut c = Canvas::new();
    for (s, side) in [(0, -1), (1, 1)] {
        // a strut from under the crossbar to the capsule's shoulder, behind it all
        for (x, y) in line(((f.inner - 4).min(-6), BAR + 2), (-4, POD_TOP + 2)) {
            c.set(x * -side, y, '2');
        }
        let n = nacelle[s];
        c.grid(&NACELLE, n - 3, NACELLE_TOP, side);
        // the crossbar, from the nacelle in to the blade's inner edge
        for x in n + 3..=f.inner {
            for (y, k) in [(BAR - 1, '2'), (BAR, '4'), (BAR + 1, 'o')] {
                c.set(x * -side, y, k);
            }
        }
        if form != SWORD {
            c.stamp(&blade(plates[s], f.antenna), f.inner, side);
        }
    }
    if form == SWORD {
        c.stamp(&sword(plates), 0, -1);
    } else if f.inner == 0 {
        // the blades meet: a seam down the point
        for y in TIP + 2..CORNER {
            c.set(0, y, 'o');
        }
    }
    c.grid(&POD, -3, POD_TOP, -1);
    c
}

const VOIDISH: Color = rgb(0x0B0B0A);
/// The lights' unlit colors; they're lit over the top at draw time.
const RED_DIM: Color = rgb(0x6A1C17);

const fn rgb(c: u32) -> Color {
    Color::new(((c >> 16) & 0xFF) as f32 / 255.0, ((c >> 8) & 0xFF) as f32 / 255.0, (c & 0xFF) as f32 / 255.0, 1.0)
}

/// A frame's letters in color, the lights unlit.
fn paint(c: &Canvas) -> Image {
    let mut img = Image::gen_image_color(SIZE.0 as u16, SIZE.1 as u16, Color::new(0.0, 0.0, 0.0, 0.0));
    for y in -CY..=CY {
        for x in -CX..=CX {
            let col = match c.at(x, y) {
                'o' => OUTLINE,
                d @ '0'..='4' => BONE[d as usize - '0' as usize],
                'c' | 'v' | 'n' => VOIDISH,
                'h' | 'b' | 'k' | 's' => RED_DIM,
                _ => continue,
            };
            img.set_pixel((x + CX) as u32, (y + CY) as u32, col);
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

/// Every light in a frame.
fn lights(c: &Canvas) -> Vec<Light> {
    let mut v = vec![];
    for y in -CY..=CY {
        for x in -CX..=CX {
            let glow = match c.at(x, y) {
                'c' => Glow::Channel,
                'n' => Glow::Window,
                'h' => Glow::Heart,
                'b' => Glow::Beacon,
                'k' => Glow::Keel,
                's' => Glow::Strobe,
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
    let f = &FORM[form];
    let c = compose(form, bank);
    let (nacelle, _) = rolled(f, bank);
    let below = NACELLE_TOP + NACELLE.len() as i32;
    let nozzles = [(nacelle[0], below), (-nacelle[1], below), (0, POD_TOP + POD.len() as i32)];
    let tips = [(f.inner, TIP), (-f.inner, TIP)];
    // the antennas' tips, or where they stood; the sword's point at its shoulders
    let gun = if form == SWORD { (-4, CORNER + 1) } else { (f.inner - 6, CORNER - 1 - f.antenna) };
    (paint(&c), lights(&c), nozzles, tips, [gun, (-gun.0, gun.1)])
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
            // steady dashes in flight, lit like Jason's sprites (they never move, so
            // they can't fight the banking); on the sword they flow up to the tip
            // while the laser fires
            if !pose.beaming {
                return (l.dy.rem_euclid(4) < 2).then_some((TEAL, 0.85));
            }
            ((l.dy + f as i32).rem_euclid(4) < 2).then_some((TEAL_HOT, 1.0))
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
        Glow::Window => Some((TEAL, 0.7)),
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

    /// The columns a frame covers on a row.
    fn span(c: &Canvas, y: i32) -> Vec<i32> {
        (-CX..=CX).filter(|&x| c.at(x, y) != '.').collect()
    }

    #[test]
    fn grids_are_rectangular_and_use_the_palette() {
        for set in [&FLAME_IDLE[..], &FLAME_THRUST[..], &JET_IDLE[..], &JET_THRUST[..], &MUZZLE[..], &BOLT[..], &POD_SHOT[..], &[&DOT[..]][..]] {
            for g in set {
                assert!(g.iter().all(|r| r.len() == g[0].len()), "{g:?}");
                assert!(g.iter().flat_map(|r| r.chars()).all(|c| c == '.' || pal(c).is_some()), "{g:?}");
            }
        }
        for g in [&NACELLE[..], &POD[..]] {
            assert!(g.iter().all(|r| r.len() == 7), "{g:?}");
            assert!(g.iter().flat_map(|r| r.chars()).all(|c| ".o01234cvnhbks".contains(c)), "{g:?}");
        }
    }

    #[test]
    fn the_pilgrim_is_the_planned_size() {
        let c = compose(0, 0);
        let cols: Vec<i32> = (-CY..=CY).flat_map(|y| span(&c, y)).collect();
        let (lo, hi) = (*cols.iter().min().unwrap(), *cols.iter().max().unwrap());
        assert_eq!(lo, -hi, "symmetric about the hitbox");
        assert!((27..=31).contains(&(hi - lo + 1)), "{} wide with its nacelles", hi - lo + 1);
        let rows: Vec<i32> = (-CY..=CY).filter(|&y| !span(&c, y).is_empty()).collect();
        let h = rows.last().unwrap() - rows[0] + 1;
        assert!((42..=48).contains(&h), "{h} tall");
        assert_eq!(rows[0], TIP, "the blades' tips are the top row");
        assert_eq!(rows.last(), Some(&(POD_TOP + POD.len() as i32 - 1)), "the capsule is the bottom row");
    }

    /// Flight form: the hitbox sits in the gap between the blades, level with their
    /// red hearts. Laser form: one sword, its heart on the hitbox and its tip
    /// straight above it.
    #[test]
    fn the_hitbox_sits_between_the_hearts_then_on_the_heart() {
        let flight = compose(0, 0);
        assert_eq!(flight.at(0, 0), '.');
        let heart = FORM[0].inner - 2;
        assert_eq!((flight.at(heart, 0), flight.at(-heart, 0)), ('h', 'h'));
        let laser = compose(SWORD, 0);
        assert_eq!(laser.at(0, 0), 'h');
        assert_eq!(span(&laser, TIP), vec![0], "one point");
        assert_eq!(laser.at(0, TIP - 1), '.');
    }

    /// Frame by frame the blades close, fold their plates and antennas, and the
    /// nacelles tuck in; the two channels become the sword's one.
    #[test]
    fn the_blades_close_into_one_sword() {
        let gap = |f: usize| {
            let c = compose(f, 0);
            (TIP..=FOOT).filter(|&y| c.at(0, y) == '.').count()
        };
        let reach = |f: usize| (-CY..=CY).flat_map(|y| span(&compose(f, 0), y)).max().unwrap();
        for f in 1..FORMS {
            let (a, b) = (&FORM[f - 1], &FORM[f]);
            assert!(b.inner >= a.inner && b.antenna <= a.antenna && b.nacelle >= a.nacelle, "frame {f}");
            assert!(gap(f) <= gap(f - 1) && reach(f) <= reach(f - 1), "frame {f}");
        }
        assert_eq!(gap(0), (FOOT - TIP + 1) as usize, "flight: split from tip to foot");
        assert_eq!(gap(SWORD), 0, "laser: one blade");
        assert!(reach(SWORD) < reach(0), "the laser form is slimmer");
        let cores = |f: usize| (-CX..=CX).filter(|&x| compose(f, 0).at(x, -5) == 'c').collect::<Vec<_>>();
        assert_eq!(cores(0), vec![FORM[0].inner - 2, 2 - FORM[0].inner]);
        assert_eq!(cores(SWORD), vec![0], "the channels become one");
    }

    /// The closed blades are exactly as wide as the sword's base, so they lock into
    /// it without a jump.
    #[test]
    fn the_blades_lock_into_the_sword_without_a_jump() {
        let (closed, sword) = (compose(SWORD - 1, 0), compose(SWORD, 0));
        for y in SHOULDERS[1]..=FOOT {
            assert_eq!(span(&closed, y), span(&sword, y), "row {y}");
        }
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
            let flat = compose(form, 0);
            let kinds = |c: &Canvas| {
                let mut k: Vec<Glow> = lights(c).iter().map(|l| l.glow).collect();
                k.sort_by_key(|g| *g as u8);
                k.dedup();
                k
            };
            for bank in [-2i8, -1, 1, 2] {
                let r = compose(form, bank);
                for y in -CY..=CY {
                    assert_eq!(r.at(0, y), flat.at(0, y), "form {form} bank {bank}: the center column stays");
                }
                assert_eq!(kinds(&r), kinds(&flat), "form {form} bank {bank}");
                let reach = |c: &Canvas, side: i32| (-CY..=CY).flat_map(|y| span(c, y)).map(|x| x * side).max().unwrap();
                let side = bank.signum() as i32;
                assert!(reach(&r, side) < reach(&flat, side), "form {form} bank {bank}: the dipping side narrows");
                assert_eq!(reach(&r, -side), reach(&flat, -side));
            }
        }
    }

    /// A roll folds plates and tucks the nacelle in, and never cuts into the stone
    /// outside a channel: the white face stays between the channel and the outline
    /// on the dipping side. (Before v0.8.3, banked hard in flight, that side lost it
    /// and showed only the dark channel against the outline.)
    #[test]
    fn rolling_keeps_the_white_down_the_dipping_side() {
        for form in 0..FORMS {
            for bank in [-2i8, -1, 1, 2] {
                let side = bank.signum() as i32;
                let c = compose(form, bank);
                let img = paint(&c);
                let shade = |x: i32, y: i32| luma(img.get_pixel((x + CX) as u32, (y + CY) as u32));
                for y in [-12, -9, -4, 0, 4, 6] {
                    // inward from the outline to the channel
                    let blade: Vec<i32> = (1..=CX).rev().map(|x| x * side).filter(|&x| c.at(x, y) != '.').collect();
                    let stone: Vec<i32> = blade[1..].iter().copied().take_while(|&x| !matches!(c.at(x, y), '0' | 'c' | 'h')).collect();
                    assert!(stone.len() >= 3, "form {form} bank {bank} row {y}: {} columns of stone", stone.len());
                    assert!(stone.iter().filter(|&&x| shade(x, y) >= luma(BONE[3])).count() >= 2, "form {form} bank {bank} row {y}: the stone stays white");
                }
            }
        }
    }

    /// In flight, firing or not and mid-transformation, the channels' dashes and the
    /// windows hold perfectly still, so nothing moves down the blades as the ship
    /// banks; the channels flow up the sword only while the laser fires.
    #[test]
    fn the_channels_hold_still_in_flight_and_flow_while_the_laser_fires() {
        let ship = frame_image(0, 0).1;
        let channel = |pose: &Pose| ship.iter().filter(|l| l.glow == Glow::Channel).map(|l| light_on(l, pose).map(|(c, k)| (l.dx, l.dy, c.g, k))).collect::<Vec<_>>();
        let lit = channel(&Pose::default());
        assert!(lit.iter().filter(|l| l.is_some()).count() >= 8, "the blades' channels are lit");
        for (fire_t, form, frame) in [(0, 0.0, 3), (9, 0.0, 40), (9, 0.6, 7), (0, 1.0, 11)] {
            assert_eq!(channel(&Pose { fire_t, form, frame, ..Default::default() }), lit, "fire_t {fire_t}, form {form}, frame {frame}: steady");
        }
        let windows = |frame: u64| ship.iter().filter(|l| l.glow == Glow::Window).map(|l| light_on(l, &Pose { frame, ..Default::default() }).map(|(_, k)| k)).collect::<Vec<_>>();
        assert!((0..20).all(|f| windows(f) == windows(0)), "the windows hold steady");
        let sword = frame_image(SWORD, 0).1;
        let laser = |frame: u64| sword.iter().filter_map(|l| light_on(l, &Pose { beaming: true, fire_t: 9, form: 1.0, frame, ..Default::default() }).map(|(c, _)| (l.dx, l.dy, c.g))).collect::<Vec<_>>();
        assert!(!laser(0).is_empty() && laser(0) != laser(1), "the sword's channel flows while it fires");
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
