//! JAWSTHIC, Stage 1's boss (Jason's design): a great white carved from cracked moon
//! rock, a gothic cathedral for a body. A red diamond eye under a heavy brow, two
//! rows of fangs on red gums, a mouth that drops open on a red crosshair, four
//! lancet windows down its flank lit red and teal, a segmented stone tail ending in
//! a crescent fin, a dorsal fin with a lancet in it, and two long pectoral fins on
//! piped joints.
//!
//! The parts are traced from his sprite atlas at game size (tools/trace_jawsthic.py
//! writes them into jaws_px.rs). Here they're turned with a RotSprite-style rotation
//! (EPX-doubled twice, then sampled), so a swinging tail or a dropping jaw stays
//! crisp, mirrored to face left, and packed in a texture of their own: the jaw at
//! each step of its drop (put together with the body), every turn of every tail
//! plate and fin.
//!
//! The atlas draws the head biting, lips pulled back off the gums. Shut, the lips
//! of his closed-jaw picture close over them, one stone head with a slit for a
//! mouth; as the jaw drops they pull back and the fangs slide down out of the gums.
//!
//! Every part faces right here. Coordinates are pixels from the body's pivot, y down.

use super::art::Packer;
use super::jaws_px::{self as px, Grid};
use super::palette::{CORE as CORE_C, EMBER, INK as INK_C, SIGNAL};
use crate::sim::boss::{Act, Boss, Laser};
use crate::tune::{JAWS_BLASTS, JAWS_DEATH, JAWS_DORSAL_CORE, JAWS_FALL, JAWS_INTRO, JAWS_LASER, JAWS_SMEAR_LIFE, JAWS_SMEARS, JAWS_STRIP, JAWS_STRIP_LIFE, JAWS_STRIP_LIFT, JAWS_STRIP_SWIRL};
use macroquad::prelude::{Color, DrawTextureParams, FilterMode, Image, Rect, Texture2D, draw_texture_ex, vec2};

// Where the parts hang on the body (from its pivot, facing right): shared with the
// sim, which aims the guns and places the hurtboxes to match.
use crate::tune::{JAWS_DORSAL_AT as DORSAL_AT, JAWS_EYE as EYE, JAWS_HINGE as HINGE, JAWS_PECTORAL_AT as PECTORAL_AT};
/// The far pectoral fin, peeking out from behind the throat.
pub const FAR_FIN_AT: (f32, f32) = (24.0, 19.0);

pub type Pt = (f32, f32);

// ---------------------------------------------------------------- the palette

const fn rgb(c: u32) -> Color {
    Color::new(((c >> 16) & 0xFF) as f32 / 255.0, ((c >> 8) & 0xFF) as f32 / 255.0, (c & 0xFF) as f32 / 255.0, 1.0)
}

/// The stone's tones, dark to light (the lightest stays under a bullet's core).
pub const RAMP: [Color; 6] = [rgb(0x26241F), rgb(0x3E3B33), rgb(0x6E6858), rgb(0x9D9682), rgb(0xC4BDA8), rgb(0xDDD6C2)];
pub const OUTLINE: Color = rgb(0x121210);
pub const RECESS: Color = rgb(0x1E1D19);
const GUM: Color = rgb(0x8E231B);
const GUM_LIT: Color = rgb(0xC9342A);
const VEIN: Color = rgb(0x45120E);
const TEAL: Color = rgb(0x32DFB3);
const TEAL_DIM: Color = rgb(0x135746);

/// A traced letter's color (see jaws_px.rs).
fn color(c: char) -> Option<Color> {
    Some(match c {
        'o' => OUTLINE,
        'x' => RECESS,
        d @ '0'..='5' => RAMP[d as usize - '0' as usize],
        'S' => SIGNAL,
        'G' => GUM_LIT,
        'g' => GUM,
        'v' => VEIN,
        'T' => TEAL,
        't' => TEAL_DIM,
        _ => return None,
    })
}

/// Stone that cracks open and burns as it weakens (not its outline, its lights, its gums).
fn stone(c: Color) -> bool {
    RAMP[1..].contains(&c)
}

// ---------------------------------------------------------------- turning parts

/// An image and the pixel its pivot sits on.
#[derive(Clone)]
pub struct Frame {
    pub img: Image,
    pub pivot: (i32, i32),
}

fn decode(g: &Grid) -> Frame {
    let (w, h) = (g.rows[0].len(), g.rows.len());
    let mut img = Image::gen_image_color(w as u16, h as u16, Color::new(0.0, 0.0, 0.0, 0.0));
    for (y, row) in g.rows.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            if let Some(col) = color(c) {
                img.set_pixel(x as u32, y as u32, col);
            }
        }
    }
    Frame { img, pivot: g.pivot }
}

/// Columns `x0..x1` of a frame, its pivot moved to `pivot` (in the source's columns).
fn columns(f: &Frame, x0: i32, x1: i32, pivot: (i32, i32)) -> Frame {
    let h = f.img.height as u32;
    let mut img = Image::gen_image_color((x1 - x0) as u16, h as u16, Color::new(0.0, 0.0, 0.0, 0.0));
    for y in 0..h {
        for x in x0..x1 {
            img.set_pixel((x - x0) as u32, y, f.img.get_pixel(x as u32, y));
        }
    }
    Frame { img, pivot: (pivot.0 - x0, pivot.1) }
}

/// EPX: each pixel becomes four, rounding off stair-steps where neighbors agree.
fn epx(src: &Image) -> Image {
    let (w, h) = (src.width as i32, src.height as i32);
    let s = src.get_image_data();
    let at = |x: i32, y: i32| s[(y.clamp(0, h - 1) * w + x.clamp(0, w - 1)) as usize];
    let mut out = Image::gen_image_color((w * 2) as u16, (h * 2) as u16, Color::new(0.0, 0.0, 0.0, 0.0));
    let o = out.get_image_data_mut();
    for y in 0..h {
        for x in 0..w {
            let (p, a, b, c, d) = (at(x, y), at(x, y - 1), at(x + 1, y), at(x - 1, y), at(x, y + 1));
            let q = [
                if c == a && c != d && a != b { a } else { p },
                if a == b && a != c && b != d { b } else { p },
                if d == c && d != b && c != a { c } else { p },
                if b == d && b != a && d != c { d } else { p },
            ];
            let i = (2 * y * 2 * w + 2 * x) as usize;
            o[i] = q[0];
            o[i + 1] = q[1];
            o[i + (2 * w) as usize] = q[2];
            o[i + (2 * w) as usize + 1] = q[3];
        }
    }
    out
}

/// A part turned to each of `angles` (radians; positive swings its left end up, the
/// way the tail bends), RotSprite-style: sampled from its EPX-doubled-twice image,
/// so edges stay one clean pixel at any angle.
fn turns(f: &Frame, angles: &[f32]) -> Vec<Frame> {
    let big = epx(&epx(&f.img));
    let b = big.get_image_data();
    let (bw, bh) = (big.width as i32, big.height as i32);
    let (w, h) = (f.img.width as f32, f.img.height as f32);
    let (pcx, pcy) = (f.pivot.0 as f32 + 0.5, f.pivot.1 as f32 + 0.5);
    angles
        .iter()
        .map(|&a| {
            if a.abs() < 1e-4 {
                return f.clone();
            }
            let (s, c) = a.sin_cos();
            let corners = [(0.0, 0.0), (w, 0.0), (0.0, h), (w, h)].map(|(x, y): (f32, f32)| (pcx + c * (x - pcx) - s * (y - pcy), pcy + s * (x - pcx) + c * (y - pcy)));
            let x0 = corners.iter().map(|p| p.0).fold(f32::MAX, f32::min).floor() as i32;
            let y0 = corners.iter().map(|p| p.1).fold(f32::MAX, f32::min).floor() as i32;
            let x1 = corners.iter().map(|p| p.0).fold(f32::MIN, f32::max).ceil() as i32;
            let y1 = corners.iter().map(|p| p.1).fold(f32::MIN, f32::max).ceil() as i32;
            let (ow, oh) = (x1 - x0, y1 - y0);
            let mut img = Image::gen_image_color(ow as u16, oh as u16, Color::new(0.0, 0.0, 0.0, 0.0));
            let o = img.get_image_data_mut();
            for y in y0..y1 {
                for x in x0..x1 {
                    let (dx, dy) = (x as f32 + 0.5 - pcx, y as f32 + 0.5 - pcy);
                    let (sx, sy) = (pcx + c * dx + s * dy, pcy - s * dx + c * dy);
                    let (bx, by) = ((sx * 4.0).floor() as i32, (sy * 4.0).floor() as i32);
                    if bx >= 0 && by >= 0 && bx < bw && by < bh {
                        let v = b[(by * bw + bx) as usize];
                        if v[3] > 0 {
                            o[((y - y0) * ow + x - x0) as usize] = v;
                        }
                    }
                }
            }
            Frame { img, pivot: (f.pivot.0 - x0, f.pivot.1 - y0) }
        })
        .collect()
}

/// A frame mirrored to face left.
fn mirrored(f: &Frame) -> Frame {
    let (w, h) = (f.img.width as u32, f.img.height as u32);
    let mut img = Image::gen_image_color(w as u16, h as u16, Color::new(0.0, 0.0, 0.0, 0.0));
    for y in 0..h {
        for x in 0..w {
            img.set_pixel(w - 1 - x, y, f.img.get_pixel(x, y));
        }
    }
    Frame { img, pivot: (w as i32 - 1 - f.pivot.0, f.pivot.1) }
}

/// Frames laid together, each with its pivot at its offset from the first's: the
/// first is the base whose pivot the result keeps; later ones go over earlier ones.
fn layered(layers: &[(&Frame, (i32, i32))]) -> Frame {
    let bounds = |(f, (dx, dy)): &(&Frame, (i32, i32))| (dx - f.pivot.0, dy - f.pivot.1, dx - f.pivot.0 + f.img.width as i32, dy - f.pivot.1 + f.img.height as i32);
    let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
    for l in layers {
        let b = bounds(l);
        (x0, y0, x1, y1) = (x0.min(b.0), y0.min(b.1), x1.max(b.2), y1.max(b.3));
    }
    let (w, h) = (x1 - x0, y1 - y0);
    let mut img = Image::gen_image_color(w as u16, h as u16, Color::new(0.0, 0.0, 0.0, 0.0));
    for l in layers {
        let (bx, by, ..) = bounds(l);
        let f = l.0;
        for y in 0..f.img.height {
            for x in 0..f.img.width {
                let c = f.img.get_pixel(x as u32, y as u32);
                if c.a > 0.0 {
                    img.set_pixel((bx - x0 + x as i32) as u32, (by - y0 + y as i32) as u32, c);
                }
            }
        }
    }
    Frame { img, pivot: (-x0, -y0) }
}

fn round(p: Pt) -> (i32, i32) {
    (p.0.round() as i32, p.1.round() as i32)
}

// ---------------------------------------------------------------- the rig

/// The jaw's frames, shut to wide open. Wide open is the atlas's pose; shut, it's
/// turned up by `JAW_SHUT` radians (tools/trace_jawsthic.py fits the closed lips to
/// it).
pub const JAW_FRAMES: usize = 7;
pub const JAW_SHUT: f32 = -0.55;
/// How far the lips slide back (px), and how far open the jaw is when they're all
/// the way back and the fangs all the way out.
const LIPS: (f32, f32) = (14.0, 0.4);
/// The tail's joints in the traced tail, root to the fin, on its centerline (Jason
/// drew it rising toward the fin). Each plate runs from its joint back to the next
/// one; the tail fin is everything behind the last. Each plate keeps a few columns
/// of the one behind it, so the joints never gap as they bend.
const JOINTS: [(i32, i32); 6] = [(80, 46), (72, 47), (64, 46), (55, 43), (46, 40), (39, 34)];
const OVERLAP: i32 = 3;
/// Turned parts are turned every `STEP` radians out to their reach either way: the
/// pectoral fins' paddle. (The tail isn't turned: it bends column by column.)
const STEP: f32 = 0.05;
const PADDLE: f32 = 0.2;

/// Steps either side of a turned part's rest angle.
fn half(reach: f32) -> i32 {
    (reach / STEP).round() as i32
}

#[cfg(test)]
fn turns_of(reach: f32) -> usize {
    half(reach) as usize * 2 + 1
}

/// The angles a part is turned to: `STEP` apart, out to its reach either side of
/// `rest`.
fn angles(rest: f32, reach: f32) -> Vec<f32> {
    (-half(reach)..=half(reach)).map(|t| rest + t as f32 * STEP).collect()
}

/// Which of those turns suits angle `a` (the nearest, held at the ends).
fn turn_of(a: f32, rest: f32, reach: f32) -> usize {
    let h = half(reach);
    ((((a - rest) / STEP).round() as i32).clamp(-h, h) + h) as usize
}

/// What's in a frame slid `dy` px down (up if negative), kept only where the frame
/// was: a lip drawn back into the head, the fangs up into the gums.
fn slid(f: &Frame, dy: i32) -> Frame {
    let (w, h) = (f.img.width as i32, f.img.height as i32);
    let mut img = Image::gen_image_color(w as u16, h as u16, Color::new(0.0, 0.0, 0.0, 0.0));
    for y in 0..h {
        for x in 0..w {
            let sy = y - dy;
            if f.img.get_pixel(x as u32, y as u32).a > 0.0 && (0..h).contains(&sy) {
                let c = f.img.get_pixel(x as u32, sy as u32);
                if c.a > 0.0 {
                    img.set_pixel(x as u32, y as u32, c);
                }
            }
        }
    }
    Frame { img, pivot: f.pivot }
}

/// The head's moving parts: the body (its fangs off), the jaw, the upper fangs (and
/// how far they slide to be all the way up in the gums: their longest), and the
/// closed lips, the lower one pivoting on the hinge.
struct Mouth {
    body: Frame,
    jaw: Frame,
    fangs: Frame,
    reach: f32,
    lip: Frame,
    lower_lip: Frame,
}

impl Mouth {
    fn new() -> Mouth {
        let fangs = decode(&px::FANGS);
        let reach = (0..fangs.img.width as u32).map(|x| (0..fangs.img.height as u32).filter(|&y| fangs.img.get_pixel(x, y).a > 0.0).count()).max().unwrap_or(0) as f32;
        Mouth { body: decode(&px::BODY), jaw: decode(&px::JAW), fangs, reach, lip: decode(&px::LIP), lower_lip: decode(&px::LOWER_LIP) }
    }

    /// How far back the lips are at openness `open` (0 shut, 1 gone).
    fn back(open: f32) -> f32 {
        (open / LIPS.1).min(1.0)
    }

    /// The body with the jaw dropped to frame `k` of `JAW_FRAMES`, put together: the
    /// jaw under the body, the fangs as far out as the lips are back, and the lips
    /// (the lower one swung down with the jaw) drawn back as far.
    fn frame(&self, k: usize) -> Frame {
        let open = k as f32 / (JAW_FRAMES - 1) as f32;
        let back = Mouth::back(open);
        let jaw = &turns(&self.jaw, &[JAW_SHUT * (1.0 - open)])[0];
        let fangs = slid(&self.fangs, -(self.reach * (1.0 - back)).round() as i32);
        if back >= 1.0 {
            return layered(&[(jaw, round(HINGE)), (&self.body, (0, 0)), (&fangs, (0, 0))]);
        }
        let d = (LIPS.0 * back).round() as i32;
        let lower = &turns(&slid(&self.lower_lip, d), &[-JAW_SHUT * open])[0];
        let lip = slid(&self.lip, -d);
        layered(&[(jaw, round(HINGE)), (&self.body, (0, 0)), (&fangs, (0, 0)), (lower, round(HINGE)), (&lip, (0, 0))])
    }
}

/// Where column `c` of the traced tail falls between its joints: the plate it's on
/// (0 at the root), how far along that plate (0..1), and the drawn centerline's row.
/// The tail fin's columns, behind the last joint, go with that joint: the fin moves
/// with the tail, never on its own.
fn along_tail(c: i32) -> (usize, f32, f32) {
    let i = (0..5).find(|&i| c >= JOINTS[i + 1].0).unwrap_or(4);
    let (a, z) = (JOINTS[i], JOINTS[i + 1]);
    let t = ((a.0 - c) as f32 / (a.0 - z.0) as f32).clamp(0.0, 1.0);
    (i, t, a.1 as f32 + (z.1 - a.1) as f32 * t)
}

/// The tail's plates (front to back) and its fin, as frames pivoting on their front.
fn tail_parts() -> Vec<Frame> {
    let tail = decode(&px::TAIL);
    let mut v: Vec<Frame> = JOINTS.windows(2).map(|j| columns(&tail, (j[1].0 - OVERLAP).max(0), j[0].0 + 1, j[0])).collect();
    v.push(columns(&tail, 0, JOINTS[5].0 + 1, JOINTS[5]));
    v
}

// ---------------------------------------------------------------- in the game

/// A frame in the boss's texture, and the pixel its pivot sits on.
#[derive(Clone, Copy, Debug, Default)]
pub struct Sprite {
    pub r: Rect,
    pub pivot: (i32, i32),
}

/// JAWSTHIC: every frame it can show, in a texture of its own.
pub struct JawsArt {
    pub tex: Texture2D,
    frames: JawsFrames,
}

impl std::ops::Deref for JawsArt {
    type Target = JawsFrames;
    fn deref(&self) -> &JawsFrames {
        &self.frames
    }
}

/// Where each of its frames sits in its texture, and what was read off its stone.
pub struct JawsFrames {
    /// [facing left][jaw frame]
    body: [[Sprite; JAW_FRAMES]; 2],
    /// [facing left]: the whole tail, its fin too, as traced, one image pivoting on
    /// its root joint (it's bent column by column as it swims).
    tail: [Sprite; 2],
    /// [facing left]
    dorsal: [Sprite; 2],
    /// [facing left][turn]: the near pectoral fin and the far one; the near one's
    /// stump [facing left].
    pectoral: [Vec<Sprite>; 2],
    far: [Vec<Sprite>; 2],
    stump: [Sprite; 2],
    /// Each lancet window's glass, pixels from the body's pivot (facing right).
    pub windows: [Vec<(i32, i32)>; 4],
    /// Seams that crack open and burn as it weakens, pixels from the pivot (facing right).
    pub cracks: Vec<Vec<(i32, i32)>>,
    /// A sample of its body's pixels, its dorsal fin's and its pectoral fin's, from
    /// their pivots (facing right): what they shatter into.
    pub shards: Vec<(i8, i8, Color)>,
    pub dorsal_shards: Vec<(i8, i8, Color)>,
    pub fin_shards: Vec<(i8, i8, Color)>,
    /// The same for each tail plate and (last) the tail fin.
    pub tail_shards: [Vec<(i8, i8, Color)>; 6],
    /// Its outline along its back (-1) and its belly (1), every other column, from
    /// the pivot (facing right): where the black hole strips specks off it. The
    /// tail's the same, by column of the traced tail, rows from its centerline.
    pub rim: Vec<(i32, i32, i32)>,
    pub tail_rim: Vec<(i32, i32, i32)>,
}

/// The topmost and bottommost pixel of every other column of `f`, from its pivot
/// (-1 the back, 1 the belly); not the jaw's underside, which drops away.
fn rim(f: &Frame) -> Vec<(i32, i32, i32)> {
    let mut v = vec![];
    for x in (0..f.img.width as i32).step_by(2) {
        let solid: Vec<i32> = (0..f.img.height as i32).filter(|&y| f.img.get_pixel(x as u32, y as u32).a > 0.0).collect();
        if let (Some(&top), Some(&bottom)) = (solid.first(), solid.last()) {
            let x = x - f.pivot.0;
            v.push((x, top - f.pivot.1, -1));
            if (x as f32) < HINGE.0 {
                v.push((x, bottom - f.pivot.1, 1));
            }
        }
    }
    v
}

/// The traced tail's top and bottom, every other column: (its column, the row from
/// its centerline there, -1 the top or 1 the bottom).
fn tail_rim(f: &Frame) -> Vec<(i32, i32, i32)> {
    let mut v = vec![];
    for c in (0..f.img.width as i32).step_by(2) {
        let solid: Vec<i32> = (0..f.img.height as i32).filter(|&y| f.img.get_pixel(c as u32, y as u32).a > 0.0).collect();
        if let (Some(&top), Some(&bottom)) = (solid.first(), solid.last()) {
            let rest = along_tail(c).2.round() as i32;
            v.push((c, top - rest, -1));
            v.push((c, bottom - rest, 1));
        }
    }
    v
}

fn pack(p: &mut Packer, f: &Frame) -> Sprite {
    Sprite { r: p.add(&f.img), pivot: f.pivot }
}

/// A sample of a frame's pixels (every `every`th), from its pivot.
fn sample(f: &Frame, every: usize) -> Vec<(i8, i8, Color)> {
    let mut v = vec![];
    let mut k = 0;
    for y in 0..f.img.height as i32 {
        for x in 0..f.img.width as i32 {
            let c = f.img.get_pixel(x as u32, y as u32);
            if c.a > 0.0 {
                if k % every == 0 {
                    v.push(((x - f.pivot.0) as i8, (y - f.pivot.1) as i8, c));
                }
                k += 1;
            }
        }
    }
    v
}

impl JawsArt {
    pub fn new() -> JawsArt {
        let (img, frames) = Self::sheet();
        let tex = Texture2D::from_image(&img);
        tex.set_filter(FilterMode::Nearest);
        JawsArt { tex, frames }
    }

    /// Turn, mirror and pack everything (CPU only, so tests can check it): the
    /// texture's image, and where each frame is in it.
    fn sheet() -> (Image, JawsFrames) {
        let mut p = Packer::sized(2048, 1024);
        let mouth = Mouth::new();
        let bodies: Vec<Frame> = (0..JAW_FRAMES).map(|k| mouth.frame(k)).collect();
        let shut = bodies[0].clone();
        let parts = tail_parts();
        let tail = Frame { pivot: JOINTS[0], ..decode(&px::TAIL) };
        let dorsal = decode(&px::DORSAL);
        let paddle = angles(0.0, PADDLE);
        let (near, far) = (turns(&decode(&px::PECTORAL), &paddle), turns(&decode(&px::FAR_FIN), &paddle));
        let stump = decode(&px::STUMP);

        // both facings: right as traced, left mirrored
        let face = |f: &Frame, left: bool| if left { mirrored(f) } else { f.clone() };
        let mut out_body = [[Sprite::default(); JAW_FRAMES]; 2];
        let mut out_tail = [Sprite::default(); 2];
        let mut out_dorsal = [Sprite::default(); 2];
        let mut out_near: [Vec<Sprite>; 2] = Default::default();
        let mut out_far: [Vec<Sprite>; 2] = Default::default();
        let mut out_stump = [Sprite::default(); 2];
        for (s, left) in [false, true].into_iter().enumerate() {
            for (k, b) in bodies.iter().enumerate() {
                out_body[s][k] = pack(&mut p, &face(b, left));
            }
            out_tail[s] = pack(&mut p, &face(&tail, left));
            out_dorsal[s] = pack(&mut p, &face(&dorsal, left));
            out_near[s] = near.iter().map(|f| pack(&mut p, &face(f, left))).collect();
            out_far[s] = far.iter().map(|f| pack(&mut p, &face(f, left))).collect();
            out_stump[s] = pack(&mut p, &face(&stump, left));
        }

        let windows = std::array::from_fn(|w| px::GLASS[w].iter().map(|&(x, y)| (x as i32, y as i32)).collect());
        // seams that crack open as it weakens: walks across its stone from these
        // starts (on the head, the flank, the tail root, the belly)
        let at = |x: i32, y: i32| {
            let (fx, fy) = (x + shut.pivot.0, y + shut.pivot.1);
            (fx >= 0 && fy >= 0 && fx < shut.img.width as i32 && fy < shut.img.height as i32).then(|| shut.img.get_pixel(fx as u32, fy as u32))
        };
        let mut rng = crate::sim::rng::Rng::new(0x5EA4);
        let starts = [(48.0, -26.0, -1.0, 0.5), (36.0, -12.0, -1.0, -0.3), (12.0, -28.0, -0.6, 1.0), (-14.0, -22.0, -1.0, 0.3), (-48.0, 6.0, 1.0, 0.2), (-6.0, 20.0, 1.0, -0.3), (62.0, -18.0, -1.0, 0.5), (-36.0, -18.0, 1.0, 0.5)];
        let cracks = starts
            .iter()
            .map(|&(sx, sy, dx, dy)| {
                let (mut x, mut y) = (sx, sy);
                let mut seam = vec![];
                for _ in 0..rng.range(14.0, 24.0) as i32 {
                    x += dx + rng.range(-0.5, 0.5);
                    y += dy + rng.range(-0.6, 0.6);
                    let q = (x.round() as i32, y.round() as i32);
                    if at(q.0, q.1).is_some_and(stone) && !seam.contains(&q) {
                        seam.push(q);
                    }
                }
                seam
            })
            .collect();
        let tail_shards = std::array::from_fn(|i| sample(&parts[i], 2));
        let frames = JawsFrames {
            body: out_body,
            tail: out_tail,
            dorsal: out_dorsal,
            pectoral: out_near,
            far: out_far,
            stump: out_stump,
            windows,
            cracks,
            shards: sample(&shut, 5),
            dorsal_shards: sample(&dorsal, 2),
            fin_shards: sample(&decode(&px::PECTORAL), 2),
            tail_shards,
            rim: rim(&shut),
            tail_rim: tail_rim(&tail),
        };
        (p.img, frames)
    }
}

/// Draw `s` with its pivot at canvas `(x, y)`, cut off below canvas row `clip`, its
/// rows in each glitch band (canvas rows y0..y1) shifted `dx` px.
fn put(tex: &Texture2D, s: Sprite, x: i32, y: i32, tint: Color, clip: Option<i32>, bands: &[(i32, i32, i32)]) {
    let (left, top) = (x - s.pivot.0, y - s.pivot.1);
    let bottom = clip.map_or(top + s.r.h as i32, |c| c.min(top + s.r.h as i32));
    let mut row = top;
    while row < bottom {
        // the band this row is in (and its shift), and where the stretch ends
        let band = bands.iter().find(|b| (b.0..b.1).contains(&row));
        let end = match band {
            Some(b) => b.1.min(bottom),
            None => bands.iter().filter(|b| b.0 > row).map(|b| b.0).min().unwrap_or(bottom).min(bottom),
        };
        let shift = band.map_or(0, |b| b.2);
        let src = Rect::new(s.r.x, s.r.y + (row - top) as f32, s.r.w, (end - row) as f32);
        draw_texture_ex(tex, (left + shift) as f32, row as f32, tint, DrawTextureParams { source: Some(src), dest_size: Some(vec2(src.w, src.h)), ..Default::default() });
        row = end;
    }
}

/// `put`, bent: each column of `s` moved down `bend(its x from the pivot, facing
/// right)` px (the sprite faces `facing`), so the body swims without its stone ever
/// being resampled. Columns bent alike go in one strip.
#[allow(clippy::too_many_arguments)]
fn put_bent(tex: &Texture2D, s: Sprite, x: i32, y: i32, tint: Color, clip: Option<i32>, bands: &[(i32, i32, i32)], facing: f32, bend: &impl Fn(f32) -> f32) {
    let w = s.r.w as i32;
    let dy = |c: i32| bend(facing * (c - s.pivot.0) as f32).round() as i32;
    let mut c0 = 0;
    while c0 < w {
        let d = dy(c0);
        let mut c1 = c0 + 1;
        while c1 < w && dy(c1) == d {
            c1 += 1;
        }
        let strip = Sprite { r: Rect::new(s.r.x + c0 as f32, s.r.y, (c1 - c0) as f32, s.r.h), pivot: (s.pivot.0 - c0, s.pivot.1) };
        put(tex, strip, x, y + d, tint, clip, bands);
        c0 = c1;
    }
}

/// Which way its eye looks from `eye` toward `you`: its pupil's step from the middle
/// of the socket, -1 to 1 each way.
pub fn gaze(eye: Pt, you: Pt) -> (i32, i32) {
    let (dx, dy) = (you.0 - eye.0, you.1 - eye.1);
    let d = dx.hypot(dy).max(1.0);
    (((dx / d) * 1.45).round().clamp(-1.0, 1.0) as i32, ((dy / d) * 1.45).round().clamp(-1.0, 1.0) as i32)
}

/// A cosmetic hash of a few numbers, 0..1 (glitches pick bands and shifts with it).
fn noise(a: u64, b: u64) -> f32 {
    let mut z = a.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ b.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    z ^= z >> 29;
    z = z.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z ^= z >> 32;
    (z % 10_000) as f32 / 10_000.0
}

/// How hard it glitches now: a little all the time, more as it weakens, all through a
/// turn, and coming apart when it dies or falls.
fn glitchiness(b: &Boss) -> f32 {
    match b.act {
        Act::Intro => 0.04,
        Act::Fight | Act::Break if b.turn > 0 => 1.0,
        Act::Fight | Act::Break => [0.0, 0.03, 0.05, 0.08, 0.16][b.phase as usize],
        Act::Dying => 0.35,
        Act::Falling => 0.7,
        Act::Gone => 0.0,
    }
}

/// This frame's glitch bands (canvas rows and shifts) over a fish centered at canvas
/// `(x, y)`.
fn glitch_bands(b: &Boss, x: i32, y: i32, frame: u64) -> Vec<(i32, i32, i32)> {
    let g = glitchiness(b);
    let tick = frame / 3;
    if noise(tick, 1) >= g {
        return vec![];
    }
    let n = 1 + (noise(tick, 2) * 3.0) as u64;
    let reach = if b.turn > 0 { 9.0 } else { 5.0 };
    (0..n)
        .map(|k| {
            let y0 = y - 46 + (noise(tick, 10 + k) * 92.0) as i32;
            let h = 2 + (noise(tick, 20 + k) * 7.0) as i32;
            let dx = ((noise(tick, 30 + k) - 0.5) * 2.0 * reach).round() as i32;
            (y0, y0 + h, if dx == 0 { 1 } else { dx })
        })
        .filter(|_| x > -1000)
        .collect()
}

/// The boss and everything on it: the fish, its burning cracks, its eye, the glow in
/// its windows and jaws before they fire, the hit flash, and its glitches.
pub fn draw(r: &super::Renderer, b: &Boss, ox: i32, oy: i32, frame: u64) {
    if b.act == Act::Gone {
        return;
    }
    let art = &r.jaws;
    let a = &r.atlas;
    let f = (b.facing < 0.0) as usize;
    let flip = b.facing < 0.0;
    let px = |p: (f32, f32)| (ox + p.0.round() as i32, oy + p.1.round() as i32);
    let (bx, by) = px((b.x, b.y));
    // it shudders as it dies
    let (bx, by) = if b.act == Act::Dying { (bx + [0, 1, 0, -1][(frame % 4) as usize], by + [0, 0, 1, 0][(frame / 2 % 4) as usize]) } else { (bx, by) };
    let clip = b.surface().map(|s| oy + s.round() as i32);
    // rising out of the disk it's dark, and it lights up as it breaches; dying, it darkens
    let k = match b.act {
        Act::Intro => (0.62 + 0.38 * (b.t as f32 / JAWS_INTRO[2] as f32).powi(2)).min(1.0),
        Act::Dying => 1.0 - 0.5 * (b.t as f32 / JAWS_DEATH as f32),
        Act::Falling => 1.0 - 0.6 * (b.t as f32 / JAWS_FALL as f32),
        _ => 1.0,
    };
    let tint = Color::new(k, k, k, 1.0);
    let bands = glitch_bands(b, bx, by, frame);
    let tex = &art.tex;
    let swing = |p: Pt| px(b.at(p));
    let paddle = 0.14 * (b.swim * 0.5).sin();
    let jaw = ((b.jaw * (JAW_FRAMES - 1) as f32).round() as usize).min(JAW_FRAMES - 1);
    // phase 2's turrets, once broken: the pectoral fins (near and far), the tail fin
    let fins_out = [b.phase >= 2 && !b.fins[0].alive(), b.phase >= 2 && !b.fins[1].alive()];
    // dying, it comes apart from the tail up as the blasts walk along it
    let blown = b.blown();
    if blown >= JAWS_BLASTS.len() {
        return;
    }
    let gone = |piece: usize| blown > piece;

    let joints = b.tail();
    // under it all, the specks the hole is stripping off it (the hole is behind it,
    // so they vanish behind its stone as they fall in)
    if !gone(8) {
        stripped(a, b, &art.rim, if gone(0) { &[] } else { &art.tail_rim }, &joints, (bx, by), (ox, oy), clip, k, frame);
    }
    // back to front: the far fin, the tail fin, the tail, the dorsal fin, the body, the near fin
    if !gone(7) {
        let (x, y) = swing(FAR_FIN_AT);
        put(tex, art.far[f][turn_of(-paddle, 0.0, PADDLE)], x, y, Color::new(k * 0.72, k * 0.72, k * 0.74, 1.0), clip, &bands);
    }
    // the tail, fin and all: one image, each column slid up or down to where the
    // sim's tail runs now, so it swims as one piece and its pixels are never
    // resampled; the fin rides the last joint. The blasts take the fin first (or its
    // turret breaks), then the plates from the back.
    let tail_cols = |c: i32| -> Option<(Sprite, i32, i32)> {
        let (i, t, rest) = along_tail(c);
        let fin = c < JOINTS[5].0;
        if (fin && (gone(0) || fins_out[1])) || (!fin && gone(5 - i)) {
            return None;
        }
        let y = joints[i].1 + (joints[i + 1].1 - joints[i].1) * t;
        let s = art.tail[f];
        let sx = if flip { s.r.w as i32 - 1 - c } else { c };
        let col = Sprite { r: Rect::new(s.r.x + sx as f32, s.r.y, 1.0, s.r.h), pivot: (0, rest.round() as i32) };
        Some((col, px((joints[0].0, joints[0].1)).0 + if flip { JOINTS[0].0 - c } else { c - JOINTS[0].0 }, oy + y.round() as i32))
    };
    for c in 0..art.tail[f].r.w as i32 {
        if let Some((col, x, y)) = tail_cols(c) {
            put(tex, col, x, y, tint, clip, &bands);
        }
    }
    let (dx, dy) = swing(DORSAL_AT);
    if !gone(6) {
        put(tex, art.dorsal[f], dx, dy, tint, clip, &bands);
    }
    let body = art.body[f][jaw];
    let bend = |lx: f32| b.bend(lx);
    put_bent(tex, body, bx, by, tint, clip, &bands, b.facing, &bend);
    // where a pixel of the body (from its pivot, facing right) is drawn now
    let mirror = |(x, y): (i32, i32)| (bx + if flip { -x } else { x }, by + y + b.bend(x as f32).round() as i32);
    // its eye follows you, and rolls back white as it bites: the socket, then the
    // red diamond in it looking your way
    let (ex, ey) = mirror(round(EYE));
    let (gx, gy) = gaze(((ex - ox) as f32, (ey - oy) as f32), b.look);
    if clip.is_none_or(|c| ey + 3 < c) {
        let dim = |c: Color| Color::new(c.r * k, c.g * k, c.b * k, 1.0);
        let pupil = if b.biting() { RAMP[4] } else { SIGNAL };
        for dy in -3i32..=3 {
            for dx in -3i32..=3 {
                let d = dx.abs() + dy.abs();
                if d <= 3 {
                    a.fill(ex + dx, ey + dy, 1, 1, dim(OUTLINE));
                }
                if (dx - gx).abs() + (dy - gy).abs() <= 1 {
                    a.fill(ex + dx, ey + dy, 1, 1, dim(pupil));
                }
            }
        }
    }
    let (fx, fy) = swing(PECTORAL_AT);
    let near = if fins_out[0] { art.stump[f] } else { art.pectoral[f][turn_of(paddle, 0.0, PADDLE)] };
    if !gone(7) {
        put(tex, near, fx, fy, tint, clip, &bands);
    }
    smears(a, b, &joints, (ox, oy), clip, k, frame, |i| !gone(if i == 5 { 0 } else { 5 - i }) && !(i == 5 && fins_out[1]));
    if clip.is_some_and(|c| c < by - 60) {
        return;
    }

    // the glows: lit windows before they fire, the turrets' cores, the eye, the jaws
    let charges = b.charges();
    for (w, glass) in art.windows.iter().enumerate() {
        let c = charges.windows[w];
        if c > 0.0 {
            let flicker = if frame % 4 < 2 { 1.0 } else { 0.8 };
            for &p in glass {
                let (x, y) = mirror(p);
                a.fill(x, y, 1, 1, Color { a: (0.25 + 0.65 * c) * flicker * k, ..SIGNAL });
            }
        }
    }
    if b.phase == 2 && b.act == Act::Fight {
        let cores = [swing(PECTORAL_AT), px((joints[5].0, joints[5].1))];
        for (i, &(x, y)) in cores.iter().enumerate() {
            if b.fins[i].alive() {
                let r = (2.0 + 1.0 * ((frame as f32) * 0.3 + i as f32).sin()).round() as usize;
                a.blit(a.r.discs[r.clamp(1, 16) - 1], x - r as i32 - 1, y - r as i32 - 1, 1, Color { a: 0.75, ..SIGNAL });
            }
        }
    }
    // where the tail fin was shot off: a scorched socket, embers in it
    if fins_out[1] && !gone(0) {
        let (sx, sy) = px((joints[5].0, joints[5].1));
        a.fill(sx - 1, sy - 3, 3, 7, Color { a: 0.85 * k, ..RECESS });
        for e in 0..3 {
            if (frame / 5 + e) % 3 != 0 {
                a.fill(sx - 1 + (e as i32 % 2), sy - 2 + 2 * e as i32, 1, 1, Color { a: 0.8 * k, ..SIGNAL });
            }
        }
    }
    // the dorsal fin's window: a red light, steady
    let (wx, wy) = swing(JAWS_DORSAL_CORE);
    a.fill(wx, wy, 1, 2, Color { a: 0.6 * k, ..SIGNAL });
    let pulse = 0.55 + 0.35 * ((frame as f32) * 0.08).sin();
    if !b.biting() {
        a.blit(a.r.discs[1], ex + gx - 3, ey + gy - 3, 1, Color { a: pulse * 0.6 * k, ..SIGNAL });
    }
    if b.jaw > 0.25 {
        // the crosshair in its mouth, hotter as a shot comes
        let (mx, my) = px(b.muzzle(crate::sim::boss::Muzzle::Mouth));
        let heat = (b.jaw - 0.25) / 0.75 * (0.45 + 0.55 * charges.mouth);
        crosshair(a, mx, my, 1.0 + 3.0 * b.jaw, heat * k, frame);
    }
    // seams cracking open as it weakens, burning
    let open = match (b.act, b.phase) {
        (Act::Dying | Act::Falling, _) => art.cracks.len(),
        (_, 3) => 3,
        (_, 4) => art.cracks.len(),
        _ => 0,
    };
    let burn = 0.5 + 0.4 * ((frame as f32) * 0.17).sin();
    for seam in &art.cracks[..open] {
        for &p in seam {
            let (x, y) = mirror(p);
            a.fill(x, y, 1, 1, Color { a: burn * k, ..SIGNAL });
        }
    }

    // a hit flushes it red; a glitch splits it red and teal
    let flash = b.flash > 0;
    if flash || !bands.is_empty() || b.fins.iter().any(|q| q.flash > 0) {
        r.additive(true);
        if flash {
            put_bent(tex, body, bx, by, Color::new(0.45, 0.05, 0.03, 0.2), clip, &[], b.facing, &bend);
        }
        for (i, q) in b.fins.iter().enumerate() {
            if q.flash > 0 && q.alive() {
                let red = Color::new(0.7, 0.08, 0.05, 0.45);
                if i == 0 {
                    put(tex, near, fx, fy, red, clip, &[]);
                } else {
                    for c in 0..JOINTS[5].0 {
                        if let Some((col, x, y)) = tail_cols(c) {
                            put(tex, col, x, y, red, clip, &[]);
                        }
                    }
                }
            }
        }
        if !bands.is_empty() {
            put_bent(tex, body, bx - 2, by, Color::new(0.6, 0.05, 0.03, 0.34), clip, &bands, b.facing, &bend);
            put_bent(tex, body, bx + 2, by, Color::new(0.04, 0.45, 0.34, 0.28), clip, &bands, b.facing, &bend);
        }
        r.additive(false);
    }
}

/// Torn pixels streaming off its tail, the way Jason drew it glitching: stone chips
/// and red and teal scanlines trailing back from the plates and the fin. A few
/// always, more as it weakens, a storm through a turn and as it dies or falls.
/// `shown(i)`: tail piece `i` is still there.
#[allow(clippy::too_many_arguments)]
fn smears(a: &super::art::Atlas, b: &Boss, joints: &[(f32, f32, f32); 6], (ox, oy): (i32, i32), clip: Option<i32>, k: f32, frame: u64, shown: impl Fn(usize) -> bool) {
    let n = match b.act {
        Act::Fight | Act::Break if b.turn > 0 => JAWS_SMEARS.1,
        Act::Fight | Act::Break | Act::Intro => JAWS_SMEARS.0[b.phase.clamp(1, 4) as usize - 1],
        Act::Dying | Act::Falling => JAWS_SMEARS.1,
        Act::Gone => 0,
    };
    // how far either side of each joint they tear off: the plates, then the tall fin
    const SPREAD: [f32; 6] = [11.0, 10.0, 9.0, 8.0, 7.0, 28.0];
    let back = -b.facing;
    let life = JAWS_SMEAR_LIFE as u64;
    for s in 0..n as u64 {
        // each smear lives `life` frames, drifting back as it fades
        let t = frame + s * 13;
        let (age, id) = (t % life, t / life * 64 + s);
        let piece = 1 + (noise(id, 1) * 5.0) as usize;
        if !shown(piece) {
            continue;
        }
        let (jx, jy, _) = joints[piece];
        let x = jx + back * (noise(id, 2) * 10.0 + age as f32 * 0.7);
        let y = jy + (noise(id, 3) - 0.5) * 2.0 * SPREAD[piece];
        let (sx, sy) = (ox + x.round() as i32, oy + y.round() as i32);
        if clip.is_some_and(|c| sy >= c) {
            continue;
        }
        let len = 2 + (noise(id, 4) * 8.0) as i32;
        let fade = (1.0 - age as f32 / life as f32) * k;
        let kind = noise(id, 5);
        let left = if back < 0.0 { sx - len } else { sx };
        if kind < 0.55 {
            // a chip of stone, its dark edge under it
            a.fill(left, sy, len, 2, Color { a: 0.85 * fade, ..RAMP[4 + (kind < 0.2) as usize] });
            a.fill(left, sy + 2, len, 1, Color { a: 0.7 * fade, ..OUTLINE });
        } else {
            let c = if kind < 0.8 { SIGNAL } else { TEAL };
            a.fill(left, sy, len, 1, Color { a: 0.6 * fade, ..c });
        }
    }
}

/// Where a speck the hole strips off it is, `u` (0 to 1) into its life: off `start`
/// on its outline it lifts `lift` px out (`side` -1 up, 1 down), easing to a stop;
/// then the hole at `hole` takes it, faster and faster, swirling `swirl` px round on
/// the way in, the way a bomb's shards fly to you.
fn strip_path(start: Pt, side: f32, lift: f32, swirl: f32, hole: Pt, u: f32) -> Pt {
    let l = (u / JAWS_STRIP_LIFT).clamp(0.0, 1.0);
    let p = (start.0, start.1 + side * lift * (1.0 - (1.0 - l) * (1.0 - l)));
    let q = ((u - JAWS_STRIP_LIFT) / (1.0 - JAWS_STRIP_LIFT)).clamp(0.0, 1.0).powi(2);
    let (dx, dy) = (hole.0 - p.0, hole.1 - p.1);
    let d = dx.hypot(dy).max(1.0);
    let w = swirl * (std::f32::consts::PI * q).sin();
    (p.0 + dx * q - dy / d * w, p.1 + dy * q + dx / d * w)
}

/// The black hole taking it apart: specks lifting off its outline (its back, its
/// belly, its tail) and falling into the hole like the shards a bomb sends to you, a
/// short trail behind each, reddening and dimming as they near it. Bone mostly, some
/// red, a little teal. Some always, more as it weakens, a storm through a turn and
/// its end.
#[allow(clippy::too_many_arguments)]
fn stripped(a: &super::art::Atlas, b: &Boss, rim: &[(i32, i32, i32)], tail_rim: &[(i32, i32, i32)], joints: &[(f32, f32, f32); 6], (bx, by): (i32, i32), (ox, oy): (i32, i32), clip: Option<i32>, k: f32, frame: u64) {
    if rim.is_empty() {
        return;
    }
    let n = match b.act {
        Act::Fight | Act::Break if b.turn > 0 => JAWS_STRIP.1,
        Act::Fight | Act::Break | Act::Intro => JAWS_STRIP.0[b.phase.clamp(1, 4) as usize - 1],
        Act::Dying | Act::Falling => JAWS_STRIP.1,
        Act::Gone => 0,
    };
    let hole = (ox as f32 + crate::tune::HOLE_X, oy as f32 + crate::tune::HOLE_Y);
    let life = JAWS_STRIP_LIFE as u64;
    for s in 0..n as u64 {
        let t = frame + s * 17;
        let (age, id) = (t % life, t / life * 64 + s);
        // a third off the tail, where it is now; the rest off the body
        let (start, side) = if !tail_rim.is_empty() && noise(id, 6) < 0.3 {
            let (c, dy, side) = tail_rim[(noise(id, 1) * tail_rim.len() as f32) as usize % tail_rim.len()];
            let (i, t, _) = along_tail(c);
            let y = joints[i].1 + (joints[i + 1].1 - joints[i].1) * t;
            ((ox as f32 + joints[0].0.round() + b.facing * (c - JOINTS[0].0) as f32, oy as f32 + y.round() + dy as f32), side)
        } else {
            let (rx, ry, side) = rim[(noise(id, 1) * rim.len() as f32) as usize % rim.len()];
            ((bx as f32 + b.facing * rx as f32, (by + ry) as f32 + b.bend(rx as f32).round()), side)
        };
        let (lift, swirl) = (3.0 + noise(id, 2) * 9.0, (noise(id, 4) - 0.5) * 2.0 * JAWS_STRIP_SWIRL);
        let at = |a: f32| strip_path(start, side as f32, lift, swirl, hole, a / life as f32);
        let (x, y) = at(age as f32);
        if clip.is_some_and(|c| y as i32 >= c) {
            continue;
        }
        // falling in, it reddens and goes out
        let u = age as f32 / life as f32;
        let q = ((u - JAWS_STRIP_LIFT) / (1.0 - JAWS_STRIP_LIFT)).clamp(0.0, 1.0);
        let kind = noise(id, 3);
        let base = if kind < 0.7 { RAMP[5 - (kind < 0.25) as usize] } else if kind < 0.92 { SIGNAL } else { TEAL };
        let red = q * q;
        let c = Color::new(base.r + (SIGNAL.r - base.r) * red, base.g + (SIGNAL.g - base.g) * red, base.b + (SIGNAL.b - base.b) * red, 1.0);
        let fade = k * (1.0 - q.powi(4));
        // the head, and a trail back the way it came, longer as it speeds up
        let (px0, py0) = at(age as f32 - 1.0);
        let v = (x - px0).hypot(y - py0).max(0.01);
        let (ux, uy) = ((x - px0) / v, (y - py0) / v);
        let len = (1.0 + v * 1.5).min(5.0) as i32;
        for s in 1..=len {
            let m = 0.5 * (1.0 - s as f32 / (len + 1) as f32);
            a.fill((x - ux * s as f32).round() as i32, (y - uy * s as f32).round() as i32, 1, 1, Color { a: m * fade, ..c });
        }
        a.fill(x.round() as i32, y.round() as i32, 1, 1, Color { a: 0.95 * fade, ..c });
    }
}

/// The red crosshair in its jaws: a ring of radius `r`, a cross through it, a hot center.
fn crosshair(a: &super::art::Atlas, x: i32, y: i32, r: f32, heat: f32, frame: u64) {
    if heat <= 0.02 {
        return;
    }
    let c = Color { a: heat.min(1.0), ..SIGNAL };
    let n = (r * 4.0).round() as i32;
    for k in 0..n {
        let ang = k as f32 / n as f32 * std::f32::consts::TAU;
        a.fill(x + (ang.cos() * r).round() as i32, y + (ang.sin() * r).round() as i32, 1, 1, c);
    }
    // the cross's arms tick in and out
    let (inner, arm) = (r.round() as i32 + 1, 1 + (frame / 4 % 2) as i32);
    a.fill(x - inner - arm + 1, y, arm, 1, c);
    a.fill(x + inner, y, arm, 1, c);
    a.fill(x, y - inner - arm + 1, 1, arm, c);
    a.fill(x, y + inner, 1, arm, c);
    a.fill(x, y, 1, 1, Color { a: (heat * 1.2).min(1.0), ..EMBER });
}

/// Phase 3's laser: a blinking red warning line along where it starts, a faint line
/// where it stops and dots sweeping between; then the beam, a white-hot core in a red
/// rim in black, the way enemy bullets are drawn.
pub fn draw_laser(a: &super::art::Atlas, l: &Laser, ox: i32, oy: i32, frame: u64) {
    let (x0, y0) = (ox as f32 + l.x, oy as f32 + l.y);
    let len = 520.0;
    if !l.firing() {
        if l.t >= JAWS_LASER.0 {
            return;
        }
        let k = l.t as f32 / JAWS_LASER.0 as f32;
        let blink = if frame % 8 < 5 { 0.9 } else { 0.45 };
        dashes(a, x0, y0, l.from, len, 4, 3, Color { a: blink, ..SIGNAL });
        dashes(a, x0, y0, l.to, len, 2, 6, Color { a: 0.28, ..SIGNAL });
        // dots running from start to stop, showing which way it'll go
        for d in 0..6 {
            let u = ((k * 2.0 + d as f32 / 6.0) % 1.0) * (l.to - l.from);
            let (s, c) = (l.from + u).to_radians().sin_cos();
            for rr in [36.0, 64.0] {
                a.fill((x0 + c * rr).round() as i32, (y0 + s * rr).round() as i32, 1, 1, Color { a: 0.6, ..SIGNAL });
            }
        }
        return;
    }
    let ang = l.angle().to_radians();
    let wob = (frame % 3) as f32 * 0.5;
    let w = JAWS_LASER.3;
    for (width, col) in [(w + wob, INK_C), (w - 2.0 + wob, SIGNAL), (w - 5.0, EMBER), (2.0, CORE_C)] {
        draw_texture_ex(
            &a.tex,
            x0,
            y0 - width * 0.5,
            col,
            DrawTextureParams { source: Some(a.r.px), dest_size: Some(vec2(len, width)), rotation: ang, pivot: Some(vec2(x0, y0)), ..Default::default() },
        );
    }
    let r = 5 + (frame % 4 < 2) as i32;
    a.blit(a.r.discs[r as usize - 1], x0 as i32 - r - 1, y0 as i32 - r - 1, 1, Color { a: 0.9, ..SIGNAL });
}

/// A dashed line from `(x, y)` along `deg`, `on` px lit then `off` px dark.
fn dashes(a: &super::art::Atlas, x: f32, y: f32, deg: f32, len: f32, on: i32, off: i32, c: Color) {
    let (s, co) = deg.to_radians().sin_cos();
    let mut d = 8;
    while (d as f32) < len {
        for k in 0..on {
            let t = (d + k) as f32;
            a.fill((x + co * t).round() as i32, (y + s * t).round() as i32, 1, 1, c);
        }
        d += on + off;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::palette::{CORE, luma};
    use crate::tune::{JAWS_MOUTH, JAWS_MOUTH_OPEN, JAWS_TAIL as TAIL, JAWS_TAIL_ART as ART, JAWS_TAIL_ROOT, JAWS_TAIL_TILT as TILT};

    /// `f`'s pixel at `p` from its pivot, if it has one there.
    fn at(f: &Frame, p: (i32, i32)) -> Option<Color> {
        let (x, y) = (p.0 + f.pivot.0, p.1 + f.pivot.1);
        (x >= 0 && y >= 0 && x < f.img.width as i32 && y < f.img.height as i32).then(|| f.img.get_pixel(x as u32, y as u32)).filter(|c| c.a > 0.0)
    }

    /// Every frame the game needs is turned and packed at startup: it fits its
    /// texture, and it's quick.
    #[test]
    fn every_frame_fits_its_texture_and_turns_quickly() {
        let t = std::time::Instant::now();
        let (img, art) = JawsArt::sheet();
        let took = t.elapsed().as_secs_f32();
        println!("turned in {took:.3} s");
        assert!(took < 3.0, "{took} s");
        assert_eq!((img.width, img.height), (2048, 1024));
        // the windows' glass and the cracks were found in the stone
        assert!(art.windows.iter().all(|w| w.len() > 20), "{:?}", art.windows.iter().map(|w| w.len()).collect::<Vec<_>>());
        assert!(art.cracks.iter().map(|c| c.len()).sum::<usize>() > 60);
        // every turn of every part is there, in both facings
        for f in 0..2 {
            assert_eq!(art.tail[f].r.w as usize, px::TAIL.rows[0].len(), "the tail, fin and all, in one piece");
            assert_eq!(art.pectoral[f].len(), turns_of(PADDLE));
            assert_eq!(art.far[f].len(), turns_of(PADDLE));
        }
    }

    /// The boss is bone white like Stage 1's enemies: its stone never outshines a
    /// bullet's core, and much of it is light.
    #[test]
    fn its_stone_is_bone_white_and_never_outshines_a_bullet() {
        let f = decode(&px::BODY);
        let lumas: Vec<f32> = f.img.get_image_data().iter().filter(|p| p[3] > 0).map(|p| luma(Color::from_rgba(p[0], p[1], p[2], 255))).collect();
        let top = lumas.iter().cloned().fold(0.0, f32::max);
        assert!(top < luma(CORE) - 0.08 && top > 0.8, "{top}");
        let bright = lumas.iter().filter(|&&l| l > 0.55).count() as f32 / lumas.len() as f32;
        assert!(bright > 0.3, "only {:.0}% of it is bright", bright * 100.0);
    }

    /// Shut, the jaw's stone closes the mouth; wide open, the dark of the mouth sits
    /// where the crosshair goes.
    #[test]
    fn the_jaw_shuts_on_its_fangs_and_opens_on_the_dark() {
        let mouth = Mouth::new();
        let (shut, open) = (mouth.frame(0), mouth.frame(JAW_FRAMES - 1));
        let mouth = round(JAWS_MOUTH);
        assert!(at(&shut, mouth).is_some(), "shut, the mouth is closed");
        let cross = round((JAWS_MOUTH.0 + JAWS_MOUTH_OPEN.0, JAWS_MOUTH.1 + JAWS_MOUTH_OPEN.1));
        let dark = at(&open, cross).map(luma);
        assert!(dark.is_some_and(|l| l < 0.15), "open, the crosshair sits in the dark of the mouth: {dark:?}");
        assert!(at(&open, (mouth.0, mouth.1 + 14)).is_some(), "open, the jaw hangs below");
    }

    /// Shut, it's one stone head like the closed-jaw picture: from the corner of the
    /// mouth to the snout, every column is solid from the brow to the chin (no gap
    /// between the jaws), not a fang shows, and the gums show only as specks in the
    /// slit.
    #[test]
    fn shut_its_lips_close_over_its_fangs() {
        let mouth = Mouth::new();
        let shut = mouth.frame(0);
        let gum = |c: Color| ['S', 'G', 'g'].into_iter().any(|k| color(k) == Some(c));
        let mut specks = 0;
        for x in 22..=66 {
            let rows: Vec<i32> = (-20..40).filter(|&y| at(&shut, (x, y)).is_some()).collect();
            let (top, bottom) = (rows[0], *rows.last().unwrap());
            assert!(bottom - top + 1 == rows.len() as i32, "column {x}: a gap between the jaws");
            specks += rows.iter().filter(|&&y| at(&shut, (x, y)).is_some_and(gum)).count();
        }
        assert!(specks < 12, "{specks} px of gum show");
        // the fangs are all the way up in the gums
        let up = slid(&mouth.fangs, -mouth.reach as i32);
        assert!(up.img.get_image_data().iter().all(|p| p[3] == 0), "a fang shows");
    }

    /// As the jaw drops, the lips pull back for good: from `LIPS.1` open on, every
    /// frame is the atlas's open mouth, the fangs all out where they were traced.
    #[test]
    fn the_lips_pull_back_as_it_opens() {
        let mouth = Mouth::new();
        let mut was = 0.0;
        for k in 0..JAW_FRAMES {
            let back = Mouth::back(k as f32 / (JAW_FRAMES - 1) as f32);
            assert!(back >= was, "frame {k}");
            was = back;
        }
        assert_eq!(Mouth::back(0.0), 0.0);
        let open = mouth.frame(JAW_FRAMES - 1);
        for y in 0..mouth.fangs.img.height as i32 {
            for x in 0..mouth.fangs.img.width as i32 {
                let c = mouth.fangs.img.get_pixel(x as u32, y as u32);
                if c.a > 0.0 {
                    let p = (x - mouth.fangs.pivot.0, y - mouth.fangs.pivot.1);
                    assert_eq!(at(&open, p), Some(c), "wide open, the fang at {p:?} is out");
                }
            }
        }
    }

    /// A speck the hole strips off it lifts off its outline first (out, never back
    /// across its stone), hangs, then falls into the hole faster and faster.
    #[test]
    fn a_stripped_speck_lifts_off_then_falls_in() {
        let (start, hole) = ((100.0, 50.0), (60.0, 90.0));
        for (side, swirl) in [(-1.0, 10.0), (1.0, -10.0)] {
            let at = |u: f32| strip_path(start, side, 5.0, swirl, hole, u);
            assert_eq!(at(0.0), start);
            let lifted = at(JAWS_STRIP_LIFT);
            assert!((lifted.1 - start.1) * side > 4.9 && lifted.0 == start.0, "it lifts straight out: {lifted:?}");
            let end = at(1.0);
            assert!((end.0 - hole.0).abs() < 0.01 && (end.1 - hole.1).abs() < 0.01, "it ends in the hole");
            let step = |u: f32| {
                let (a, b) = (at(u), at(u + 0.01));
                (b.0 - a.0).hypot(b.1 - a.1)
            };
            let mut was = 0.0;
            for i in 0..10 {
                let u = JAWS_STRIP_LIFT + (1.0 - JAWS_STRIP_LIFT) * (0.05 + i as f32 * 0.09);
                assert!(step(u) > was, "faster and faster: {u}");
                was = step(u);
            }
        }
    }

    /// Its eye looks your way: straight down at you below it, across at you beside
    /// it, never out of its socket.
    #[test]
    fn its_eye_follows_you() {
        assert_eq!(gaze((100.0, 50.0), (100.0, 300.0)), (0, 1));
        assert_eq!(gaze((100.0, 50.0), (20.0, 50.0)), (-1, 0));
        assert_eq!(gaze((100.0, 50.0), (300.0, 250.0)), (1, 1));
        assert_eq!(gaze((100.0, 50.0), (100.0, 50.0)), (0, 0));
        for a in 0..32 {
            let t = a as f32 / 32.0 * std::f32::consts::TAU;
            let (x, y) = gaze((0.0, 0.0), (t.cos() * 80.0, t.sin() * 80.0));
            assert!(x.abs() + y.abs() <= 2 && x.abs() <= 1 && y.abs() <= 1);
        }
    }

    /// Every angle the sim can paddle a fin to has a turned frame.
    #[test]
    fn every_paddle_has_a_frame() {
        for a in [-1.0, -0.2, 0.0, 0.2, 1.0] {
            assert!(turn_of(a, 0.0, PADDLE) < turns_of(PADDLE));
        }
    }

    /// The tail swims as one piece: every column between two joints moves with them,
    /// on the line drawn between them, and the fin's columns all ride the last joint,
    /// so the fin never moves on its own.
    #[test]
    fn the_tail_bends_as_one_piece_fin_and_all() {
        for c in 0..px::TAIL.rows[0].len() as i32 {
            let (i, t, rest) = along_tail(c);
            assert!((0.0..=1.0).contains(&t), "column {c}");
            let (a, z) = (JOINTS[i], JOINTS[i + 1]);
            assert!(rest >= a.1.min(z.1) as f32 && rest <= a.1.max(z.1) as f32, "column {c}");
            if c < JOINTS[5].0 {
                assert_eq!((i, t), (4, 1.0), "fin column {c} rides the last joint");
            }
        }
        assert_eq!(along_tail(JOINTS[0].0), (0, 0.0, JOINTS[0].1 as f32));
    }

    /// The tail's plates are cut where the sim's joints are: each as long as the
    /// sim's plate and sloped as drawn, so at rest every plate's back meets the next
    /// one's front on the tail's centerline; the tail fin hangs off the last.
    #[test]
    fn the_tail_is_cut_where_its_joints_are() {
        let parts = tail_parts();
        assert_eq!(parts.len(), TAIL.len() + 1);
        for (i, &(len, _)) in TAIL.iter().enumerate() {
            let ((x0, y0), (x1, y1)) = (JOINTS[i], JOINTS[i + 1]);
            let (dx, up) = ((x0 - x1) as f32, (y0 - y1) as f32);
            assert!((dx.hypot(up) - len).abs() < 0.01, "plate {i}: {} long, the sim says {len}", dx.hypot(up));
            assert!((up.atan2(dx) - ART[i]).abs() < 0.01, "plate {i}: sloped {}, the sim says {}", up.atan2(dx), ART[i]);
            // the plate's stone reaches its back joint
            assert!(at(&parts[i], (x1 - x0, y1 - y0)).is_some(), "plate {i} reaches back");
        }
        assert!(JAWS_TAIL_ROOT.0 < -60.0, "the tail hangs off the back of the body");
    }

    /// The review sheet: the fish posed big and at 1x, jaw shut and open, tail
    /// waving, facing either way. `cargo test --release jawsthic_sheet -- --ignored`
    /// writes shots/jawsthic_review.png.
    #[test]
    #[ignore]
    fn jawsthic_sheet() {
        let bg = Color::new(0.03, 0.035, 0.027, 1.0);
        let mouth = Mouth::new();
        let shark = |img: &mut Image, x: i32, y: i32, open: usize, phase: f32, left: bool| {
            let face = |f: &Frame| if left { mirrored(f) } else { f.clone() };
            let put = |img: &mut Image, f: &Frame, p: Pt| {
                let (px, py) = (x + if left { -p.0 } else { p.0 }.round() as i32, y + p.1.round() as i32);
                for sy in 0..f.img.height as i32 {
                    for sx in 0..f.img.width as i32 {
                        let c = f.img.get_pixel(sx as u32, sy as u32);
                        let (tx, ty) = (px - f.pivot.0 + sx, py - f.pivot.1 + sy);
                        if c.a > 0.0 && tx >= 0 && ty >= 0 && tx < img.width as i32 && ty < img.height as i32 {
                            img.set_pixel(tx as u32, ty as u32, c);
                        }
                    }
                }
            };
            let dim = |f: &Frame| {
                let mut g = f.clone();
                for p in g.img.get_image_data_mut() {
                    for k in 0..3 {
                        p[k] = (p[k] as f32 * 0.72) as u8;
                    }
                }
                g
            };
            put(img, &face(&dim(&decode(&px::FAR_FIN))), FAR_FIN_AT);
            let mut joints = vec![(JAWS_TAIL_ROOT, 0.0f32)];
            let mut wave = 0.0f32;
            for i in 0..6 {
                wave += crate::tune::JAWS_BEND[i] * (phase - i as f32 * 0.6).sin();
                let ang = ART[i] + TILT + wave;
                joints.last_mut().unwrap().1 = TILT + wave;
                if let Some(&(len, _)) = TAIL.get(i) {
                    let (p, _) = *joints.last().unwrap();
                    joints.push(((p.0 - len * ang.cos(), p.1 - len * ang.sin()), 0.0));
                }
            }
            let tail = decode(&px::TAIL);
            for c in 0..tail.img.width as i32 {
                let (i, t, rest) = along_tail(c);
                let y = joints[i].0.1 + (joints[i + 1].0.1 - joints[i].0.1) * t;
                let col = columns(&tail, c, c + 1, (c, rest.round() as i32));
                put(img, &face(&col), ((c - JOINTS[0].0) as f32 + JAWS_TAIL_ROOT.0, y));
            }
            put(img, &face(&decode(&px::DORSAL)), DORSAL_AT);
            put(img, &face(&mouth.frame(open)), (0.0, 0.0));
            put(img, &face(&decode(&px::PECTORAL)), PECTORAL_AT);
        };
        let (w, h) = (900u16, 680u16);
        let mut sheet = Image::gen_image_color(w, h, bg);
        // big: 2x, jaw open
        let mut big = Image::gen_image_color(260, 140, bg);
        shark(&mut big, 175, 62, JAW_FRAMES - 1, 0.0, false);
        for y in 0..big.height as u32 * 2 {
            for x in 0..big.width as u32 * 2 {
                sheet.set_pixel(x, y, big.get_pixel(x / 2, y / 2));
            }
        }
        // at 1x, as in the game
        let poses = [(0, 0.0, false), (3, 1.5, false), (JAW_FRAMES - 1, 3.4, false), (1, 0.8, true), (4, 2.2, true), (6, 4.6, true)];
        for (k, &(open, ph, left)) in poses.iter().enumerate() {
            let (col, row) = (k % 3, k / 3);
            let x = 10 + col as i32 * 296 + if left { 100 } else { 190 };
            shark(&mut sheet, x, 352 + row as i32 * 128, open, ph, left);
        }
        // the jaw shutting to wide open, frame by frame: the head of each
        for k in 0..JAW_FRAMES {
            let f = mouth.frame(k);
            let (x0, y0) = (15 + k as i32 * 126, 610);
            for y in -30..60 {
                for x in -5..85 {
                    let (sx, sy) = (x + f.pivot.0, y + f.pivot.1);
                    if sx >= 0 && sy >= 0 && sx < f.img.width as i32 && sy < f.img.height as i32 {
                        let c = f.img.get_pixel(sx as u32, sy as u32);
                        if c.a > 0.0 {
                            sheet.set_pixel((x0 + x) as u32, (y0 + y) as u32, c);
                        }
                    }
                }
            }
        }
        let _ = std::fs::create_dir_all("shots");
        let mut out = Image::gen_image_color(w * 2, h * 2, bg);
        for y in 0..h as u32 * 2 {
            for x in 0..w as u32 * 2 {
                // export_png writes the bottom row first
                out.set_pixel(x, y, sheet.get_pixel(x / 2, h as u32 - 1 - y / 2));
            }
        }
        out.export_png("shots/jawsthic_review.png");
    }
}
