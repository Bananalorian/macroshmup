//! JAWSTHIC, Stage 1's boss (Jason's concept): a great white carved from moon rock,
//! a gothic cathedral for a body. Lancet windows down its flank, a fin with an arched
//! window, a segmented stone tail trailing glitches, a red diamond eye, and jaws that
//! drop open on a red crosshair. Drawn in code with the carving painter (carve.rs),
//! part by part, so the tail can swing, the jaw can drop and the fins can paddle.
//!
//! Every part faces right here; the painter mirrors them to face left. Coordinates
//! are pixels from each part's pivot, y down.

use super::art::Packer;
use super::carve::{Form, Frame, M, Part, Pose, Pt, Shape, carve, carve_with, disc, line, poly, ring, seams};
use super::palette::{CORE as CORE_C, EMBER, INK as INK_C, SIGNAL};
use crate::sim::boss::{Act, Boss, Laser};
use crate::tune::{JAWS_BLASTS, JAWS_DEATH, JAWS_DORSAL_CORE, JAWS_FALL, JAWS_INTRO, JAWS_LASER};
use macroquad::prelude::{Color, DrawTextureParams, FilterMode, Image, Rect, Texture2D, draw_texture_ex, vec2};

// Where the parts hang on the body (from its pivot, facing right): shared with the
// sim, which aims the guns and places the hurtboxes to match.
use crate::tune::{JAWS_DORSAL_AT as DORSAL_AT, JAWS_EYE as EYE, JAWS_HINGE as HINGE, JAWS_PECTORAL_AT as PECTORAL_AT, JAWS_TAIL as TAIL, JAWS_WINDOWS as WINDOWS};
/// The far pectoral fin, peeking out behind the body.
pub const FAR_FIN_AT: Pt = (25.0, 19.0);

/// The body's outline: the snout, over the back to the tail root, along the belly
/// to the throat, then the mouth line back out to the snout.
fn outline() -> Vec<Pt> {
    vec![
        (72.0, -8.0),
        (69.5, -12.5),
        (63.0, -17.5),
        (54.0, -21.5),
        (43.0, -25.0),
        (31.0, -27.5),
        (18.0, -29.0),
        (6.0, -29.0),
        (-6.0, -27.5),
        (-17.0, -24.0),
        (-26.0, -19.0),
        (-32.0, -13.5),
        (-34.5, -7.0),
        (-35.0, 0.0),
        (-32.5, 6.0),
        (-27.0, 11.5),
        (-18.0, 16.5),
        (-6.0, 20.5),
        (6.0, 23.0),
        (18.0, 24.5),
        (24.0, 24.0),
        (26.5, 21.0),
        (26.5, 14.0),
        HINGE,
        (34.0, 7.5),
        (44.0, 6.0),
        (54.0, 4.0),
        (63.0, 1.5),
        (68.0, -1.5),
        (71.0, -4.5),
    ]
}

/// The whole fish's silhouette with the jaw shut: its curve rounds the light.
fn barrel_outline() -> Vec<Pt> {
    vec![(72.0, -8.0), (63.0, -17.5), (43.0, -25.0), (18.0, -29.0), (6.0, -29.0), (-17.0, -24.0), (-32.0, -13.5), (-35.0, 0.0), (-27.0, 11.5), (-6.0, 20.5), (18.0, 24.5), (28.0, 23.5), (45.0, 17.5), (59.0, 10.0), (61.0, 6.0), (63.0, 1.5), (71.0, -4.5)]
}

/// The mouth line: the upper jaw's edge, hinge to snout.
const MOUTH: [Pt; 5] = [HINGE, (34.0, 7.5), (44.0, 6.0), (54.0, 4.0), (63.0, 1.5)];

/// The lower jaw, shut: shorter than the upper, so the snout overhangs it.
fn jaw_outline() -> Vec<Pt> {
    vec![HINGE, (34.0, 7.5), (44.0, 6.0), (54.0, 4.0), (60.0, 2.8), (61.0, 6.0), (59.0, 10.0), (53.0, 14.0), (45.0, 17.5), (36.0, 21.0), (28.0, 23.5), (24.0, 18.0)]
}

/// `p` swung `a` radians about `hinge`.
pub fn swing(p: Pt, hinge: Pt, a: f32) -> Pt {
    let (s, c) = a.sin_cos();
    let (dx, dy) = (p.0 - hinge.0, p.1 - hinge.1);
    (hinge.0 + c * dx - s * dy, hinge.1 + s * dx + c * dy)
}

/// Height of a polyline at `x` (linear between its points).
fn along(pts: &[Pt], x: f32) -> f32 {
    for w in pts.windows(2) {
        let ((x0, y0), (x1, y1)) = (w[0], w[1]);
        if (x0..=x1).contains(&x) {
            return y0 + (y1 - y0) * (x - x0) / (x1 - x0).max(1e-4);
        }
    }
    pts.last().map_or(0.0, |p| p.1)
}

/// A tooth: a sharp cone from a base on the gum line to a tip, raked toward the throat.
fn tooth(x: f32, base: f32, len: f32, down: bool) -> Shape {
    let s = if down { 1.0 } else { -1.0 };
    poly(&[(x - 1.7, base - s * 0.6), (x + 1.5, base - s * 0.6), (x - 0.4, base + s * len)], M::Tooth)
}

/// A lancet window: a pointed arch in a thick molded frame, a round eye of tracery
/// at its head, a slit of light below.
fn lancet(cx: f32, top: f32, bot: f32, half: f32, light: M) -> Vec<Shape> {
    let spring = top + half * 1.6;
    let f = 2.0;
    let outer = vec![(cx, top - f - 0.5), (cx + half * 0.8 + 0.6, top - 0.6), (cx + half + f, spring - 0.5), (cx + half + f, bot + f), (cx - half - f, bot + f), (cx - half - f, spring - 0.5), (cx - half * 0.8 - 0.6, top - 0.6)];
    let inner = vec![(cx, top + 0.3), (cx + half * 0.6, top + 1.5), (cx + half, spring + 0.6), (cx + half, bot), (cx - half, bot), (cx - half, spring + 0.6), (cx - half * 0.6, top + 1.5)];
    vec![
        poly(&outer, M::Frame),
        poly(&inner, M::Slot),
        disc((cx, spring + 0.6), 1.2, M::Frame),
        line((cx, spring + 3.5), (cx, bot - 3.0), 1.0, light),
        line((cx - half, bot - 1.5), (cx + half, bot - 1.5), 1.0, M::Recess),
    ]
}

/// The head and body with the jaw dropped `open` radians: one image, so the jaw, the
/// teeth and the throat sit together cleanly.
pub fn body(open: f32) -> Part {
    let form = Form::barrel(&poly(&barrel_outline(), M::Hull));
    let jaw = |s: Shape| s.turned(HINGE, open);
    let mut v = vec![];

    // the mouth: dark, veined, between the upper jaw and the dropped one
    let mut cavity: Vec<Pt> = MOUTH.to_vec();
    cavity.extend(MOUTH.iter().rev().map(|&p| swing(p, HINGE, open)));
    v.push(poly(&cavity, M::Mouth));
    v.push(line((32.0, 10.5), (44.0, 13.5), 1.0, M::Vein));
    v.push(line((40.0, 9.5), (49.0, 15.5), 1.0, M::Vein));
    v.push(line((49.0, 7.5), (55.0, 11.5), 1.0, M::Vein));

    // the lower jaw: plated chin, gum line, teeth pointing up
    v.push(jaw(poly(&jaw_outline(), M::Hull)));
    for s in [seams(&[(33.0, 21.5), (37.0, 14.0), (41.0, 10.0)]), seams(&[(47.0, 16.5), (50.0, 10.5), (52.0, 7.0)]), seams(&[(56.0, 12.0), (58.0, 7.0)])] {
        v.extend(s.into_iter().map(jaw));
    }
    for (k, x) in [32.0, 35.5, 39.0, 42.5, 46.0, 49.5, 53.0, 56.5].iter().enumerate() {
        let len = [2.5, 3.0, 3.5, 4.0, 4.0, 4.0, 4.0, 3.5][k];
        v.push(jaw(tooth(*x, along(&MOUTH, *x) + 1.4, len, false)));
    }
    v.push(jaw(line((29.5, 9.8), (59.0, 4.2), 1.0, M::Gum)));
    // the throat behind the hinge
    v.push(disc(HINGE, 2.6, M::Shade));

    // the body
    v.push(poly(&outline(), M::Hull));
    for s in [
        // the head: big cracked flagstones
        seams(&[(40.0, -25.5), (41.0, -19.0), (37.0, -12.0), (33.0, -5.0), (30.0, 2.0), (28.5, 7.0)]),
        seams(&[(62.0, -18.0), (63.0, -12.0), (65.0, -6.0), (66.0, -1.0)]),
        seams(&[(33.0, -5.0), (42.0, -2.5), (52.0, -3.0), (60.0, -6.0), (65.0, -6.0)]),
        seams(&[(52.0, -21.5), (53.0, -16.0), (58.0, -14.5), (62.5, -13.0)]),
        seams(&[(44.0, -2.6), (46.0, 3.0)]),
        seams(&[(68.0, -11.0), (70.0, -8.0)]),
        // masonry over the windows
        seams(&[(-17.0, -23.5), (-17.0, -20.5)]),
        seams(&[(-7.0, -27.0), (-7.0, -21.5)]),
        seams(&[(3.0, -29.0), (3.0, -22.0)]),
        seams(&[(14.0, -29.0), (14.0, -22.0)]),
        seams(&[(27.0, -28.0), (27.5, -21.0)]),
        // and under them
        seams(&[(-17.0, 11.0), (-17.5, 16.5)]),
        seams(&[(-7.0, 12.0), (-7.5, 20.5)]),
        seams(&[(4.0, 12.5), (4.0, 22.8)]),
        // the tail root's collar
        seams(&[(-29.0, -12.5), (-32.0, -6.0), (-32.0, 1.0), (-29.0, 8.0)]),
    ] {
        v.extend(s);
    }
    // gill arches: four curved slits, the ribs between them lit down their front
    for x in [14.5, 18.0, 21.5, 25.0] {
        let arch = [(x - 0.5, -17.0), (x + 1.5, -10.0), (x + 2.2, -2.0), (x + 1.8, 5.0), (x, 11.0)];
        v.extend(arch.windows(2).map(|w| line(w[0], w[1], 1.6, M::Recess)));
    }
    // the arcade: a shadowed bay along the flank, a string course over it and a sill
    // under it, the lancet windows set in it with red and teal light inside
    v.push(poly(&[(-29.0, -15.0), (13.0, -19.5), (13.5, 11.5), (-28.0, 9.5)], M::Shade));
    v.push(poly(&[(-29.5, -17.5), (14.0, -22.0), (14.0, -20.0), (-29.0, -15.5)], M::Frame));
    v.push(poly(&[(-28.5, 9.0), (14.0, 11.0), (14.0, 13.0), (-28.0, 11.0)], M::Frame));
    for (i, &cx) in WINDOWS.iter().enumerate() {
        v.extend(lancet(cx, -13.5, 6.5, 2.0, if i % 2 == 0 { M::Red } else { M::Teal }));
    }
    // pits in the rock
    for (c, r) in [((58.0, -9.0), 1.3), ((36.0, -21.0), 1.1), ((-13.0, -22.5), 1.0), ((1.0, 19.0), 1.2), ((-24.0, 13.0), 1.0), ((48.0, -20.0), 0.9), ((67.0, -6.0), 0.8), ((-31.0, -2.0), 0.9), ((21.0, -25.0), 1.0), ((38.0, 1.0), 0.8)] {
        v.push(disc(c, r, M::Pit));
    }
    // the eye: a red diamond in a dark socket, under a hard brow
    let (ex, ey) = EYE;
    v.push(poly(&[(ex, ey - 3.4), (ex + 3.6, ey), (ex, ey + 3.2), (ex - 3.6, ey)], M::Slot));
    v.push(poly(&[(ex, ey - 1.6), (ex + 1.7, ey), (ex, ey + 1.6), (ex - 1.7, ey)], M::Red));
    v.push(line((ex - 8.0, ey - 6.0), (ex + 6.0, ey - 3.5), 2.2, M::Frame));
    // the sockets the fins plug into
    v.push(poly(&[(-13.0, -24.5), (12.0, -29.5), (13.0, -27.0), (-12.0, -22.0)], M::Frame));
    v.push(disc(PECTORAL_AT, 7.0, M::Shade));

    // the upper jaw: gum line, teeth pointing down
    v.push(line((29.0, 9.0), (62.0, 2.6), 1.0, M::Gum));
    for (k, x) in [30.0, 33.5, 37.0, 40.5, 44.0, 47.5, 51.0, 54.5, 58.0, 61.5].iter().enumerate() {
        let len = [2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5, 5.5, 5.0, 4.0][k];
        v.push(tooth(*x, along(&MOUTH, *x) + 0.6, len, true));
    }
    Part { shapes: v, form, seed: 31 }
}

/// Tail plate `i` (0 is the front): a stone block on a joint, a dark gap behind it
/// with a light in it.
pub fn tail(i: usize) -> Part {
    let (len, h) = TAIL[i];
    let next = TAIL.get(i + 1).map_or(h - 2.0, |t| t.1);
    let (a, b) = (h * 0.5, next * 0.5 + 1.5);
    let block = poly(&[(0.5, -a + 1.0), (-1.0, -a), (-len + 1.0, -b), (-len - 1.0, -b + 1.5), (-len - 1.0, b - 1.5), (-len + 1.0, b), (-1.0, a), (0.5, a - 1.0)], M::Hull);
    let form = Form::barrel(&block);
    let mut v = vec![disc((0.0, 0.0), a - 1.0, M::Hull), block];
    // the shadow the plate ahead casts over this one's front
    v.push(poly(&[(1.0, -a + 1.0), (-2.5, -a + 0.5), (-2.5, a - 0.5), (1.0, a - 1.0)], M::Shade));
    // a lit cap along the top, the flank split on the bigger plates
    v.extend(seams(&[(-2.5, -a * 0.5), (-len, -b * 0.5)]));
    if i < 2 {
        v.extend(seams(&[(-len * 0.55, -a * 0.47), (-len * 0.55, a * 0.9)]));
    }
    // a keel along the narrow end
    if i >= 3 {
        v.push(line((-2.5, 0.5), (-len, 0.5), 1.0, M::Frame));
    }
    // the raised rim at its back, over the next plate
    v.push(poly(&[(-len + 1.0, -b), (-len - 1.0, -b + 1.5), (-len - 1.0, b - 1.5), (-len + 1.0, b), (-len + 2.5, b - 0.5), (-len + 2.5, -b + 0.5)], M::Frame));
    // a light glinting in the joint
    v.push(disc((-2.0, a * 0.35), 0.6, if i % 2 == 0 { M::Teal } else { M::Red }));
    if i == 2 {
        // the little second dorsal and anal fins, swept back
        v.push(poly(&[(-1.5, -a + 0.5), (-6.5, -a - 5.0), (-9.0, -a - 4.5), (-7.5, -a + 0.5)], M::Hull));
        v.push(poly(&[(-1.5, a - 0.5), (-7.0, a + 4.5), (-9.5, a + 4.0), (-7.5, a - 0.5)], M::Hull));
    }
    if i % 2 == 0 {
        v.push(disc((-len * 0.45, a * 0.45), 0.8, M::Pit));
    }
    Part { shapes: v, form, seed: 40 + i as u32 }
}

/// The tail fin: a stone crescent, its lobes ribbed.
pub fn caudal() -> Part {
    let fin = vec![
        (2.0, -7.0),
        (-1.0, -13.0),
        (-6.0, -20.0),
        (-13.0, -26.5),
        (-22.0, -32.0),
        (-25.5, -33.0),
        (-22.5, -27.0),
        (-18.5, -19.0),
        (-15.5, -11.0),
        (-14.0, -4.0),
        (-13.5, 0.0),
        (-14.5, 5.0),
        (-17.0, 12.0),
        (-21.0, 19.0),
        (-23.5, 22.0),
        (-19.0, 20.0),
        (-12.0, 15.5),
        (-6.0, 10.5),
        (-1.0, 6.0),
        (2.0, 5.0),
    ];
    let mut v = vec![disc((0.0, 0.0), 6.0, M::Hull), poly(&fin, M::Hull)];
    for s in [seams(&[(-1.0, -5.0), (-8.0, -14.0), (-17.0, -25.0)]), seams(&[(-3.0, -0.5), (-10.0, -2.0)]), seams(&[(-2.0, 4.0), (-9.0, 9.0), (-16.0, 15.0)])] {
        v.extend(s);
    }
    v.push(line((2.0, -0.5), (-7.0, -0.5), 1.0, M::Frame));
    v.push(disc((-10.0, -12.0), 1.0, M::Pit));
    v.push(disc((-8.0, 6.0), 0.8, M::Pit));
    Part { shapes: v, form: Form::Flat { tip: (-0.25, -0.3), lean: (0.0, -0.006) }, seed: 50 }
}

/// The dorsal fin: tall and falcate, swept back, an arched window in it.
pub fn dorsal() -> Part {
    let fin = vec![(13.0, 1.0), (10.0, -5.0), (6.0, -12.0), (1.0, -19.0), (-5.0, -26.0), (-11.0, -31.0), (-17.0, -34.5), (-15.0, -30.0), (-13.5, -24.0), (-12.5, -17.0), (-12.5, -10.0), (-13.5, -4.0), (-15.5, 1.0)];
    let mut v = vec![poly(&fin, M::Hull)];
    v.extend(lancet(-5.5, -20.0, -6.0, 2.2, M::Red));
    for s in [seams(&[(9.5, -4.5), (-1.0, -3.5)]), seams(&[(-10.0, -3.5), (-13.0, -4.0)]), seams(&[(1.5, -17.5), (-1.5, -18.5)]), seams(&[(-9.5, -23.5), (-13.5, -24.0)])] {
        v.extend(s);
    }
    Part { shapes: v, form: Form::Flat { tip: (-0.15, -0.15), lean: (0.0, -0.006) }, seed: 60 }
}

/// A pectoral fin on its piped joint: a broad falcate blade swept back, slots of teal
/// light in it.
pub fn pectoral() -> Part {
    let blade = vec![(5.0, -3.0), (0.0, 2.0), (-7.0, 8.0), (-15.0, 14.0), (-24.0, 21.0), (-33.0, 29.0), (-40.0, 36.0), (-36.0, 35.5), (-30.0, 33.5), (-22.0, 29.0), (-14.0, 23.5), (-7.0, 17.0), (-2.0, 10.0), (3.0, 5.0)];
    let mut v = vec![poly(&blade, M::Hull)];
    for s in [seams(&[(-6.0, 8.5), (-7.5, 17.5)]), seams(&[(-15.0, 15.0), (-16.5, 25.0)]), seams(&[(-24.0, 22.0), (-25.5, 31.0)])] {
        v.extend(s);
    }
    for (x, y) in [(-10.5, 14.5), (-19.5, 21.0), (-28.5, 28.0)] {
        v.push(poly(&[(x + 2.0, y - 1.5), (x - 0.5, y - 1.5), (x - 2.0, y + 1.5), (x + 0.5, y + 1.5)], M::Slot));
        v.push(disc((x, y), 0.5, M::Teal));
    }
    v.push(disc((0.0, 0.0), 5.6, M::Pipe));
    v.push(ring((0.0, 0.0), 2.6, 3.4, M::Recess));
    v.push(disc((0.0, 0.0), 1.4, M::Red));
    Part { shapes: v, form: Form::Flat { tip: (-0.1, -0.35), lean: (0.0, 0.0) }, seed: 70 }
}

/// What's left of a pectoral fin: its joint, a ragged root of the blade.
pub fn pectoral_stump() -> Part {
    let root = vec![(5.0, -3.0), (0.0, 2.0), (-6.0, 7.0), (-9.0, 12.0), (-6.5, 11.0), (-5.0, 15.0), (-2.0, 10.0), (3.0, 5.0)];
    let v = vec![
        poly(&root, M::Hull),
        poly(&[(-5.5, 8.0), (-8.0, 11.0), (-6.5, 10.5), (-5.0, 13.0), (-3.5, 9.5)], M::Recess),
        disc((0.0, 0.0), 5.6, M::Pipe),
        ring((0.0, 0.0), 2.6, 3.4, M::Recess),
        disc((0.0, 0.0), 1.4, M::Slot),
    ];
    Part { shapes: v, form: Form::Flat { tip: (-0.1, -0.35), lean: (0.0, 0.0) }, seed: 71 }
}

// ---------------------------------------------------------------- in the game

/// A carved frame in the boss's texture, and the pixel its pivot sits on.
#[derive(Clone, Copy, Debug, Default)]
pub struct Sprite {
    pub r: Rect,
    pub pivot: (i32, i32),
}

/// The jaw's frames, shut to wide open (radians).
pub const JAW_FRAMES: usize = 7;
pub const JAW_OPEN: f32 = 0.52;
/// Turned parts are carved every `STEP` radians out to their reach either way: the
/// tail's plates, then its fin; the pectoral fins' paddle.
const STEP: f32 = 0.05;
const REACH: [f32; 6] = [0.1, 0.2, 0.32, 0.46, 0.62, 0.84];
const PADDLE: f32 = 0.2;

fn turns(reach: f32) -> usize {
    (reach / STEP).round() as usize * 2 + 1
}

/// Which carved turn suits angle `a`.
fn turn_of(a: f32, reach: f32) -> usize {
    ((a.clamp(-reach, reach) + reach) / STEP).round() as usize
}

/// JAWSTHIC, carved: every frame it can show, in a texture of its own.
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
    /// [plate, 5 the tail fin][facing left][turn]
    tail: [[Vec<Sprite>; 2]; 6],
    /// [facing left]
    dorsal: [Sprite; 2],
    /// [facing left][turn], and the stump [facing left]
    pectoral: [Vec<Sprite>; 2],
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
}

fn pack(p: &mut Packer, f: &Frame) -> Sprite {
    Sprite { r: p.add(&f.img), pivot: f.pivot }
}

impl JawsArt {
    pub fn new() -> JawsArt {
        let (img, frames) = Self::sheet();
        let tex = Texture2D::from_image(&img);
        tex.set_filter(FilterMode::Nearest);
        JawsArt { tex, frames }
    }

    /// Carve everything (CPU only, so tests can check it): the texture's image, and
    /// where each frame is in it.
    fn sheet() -> (Image, JawsFrames) {
        let mut p = Packer::sized(1024, 1024);
        let pose = |flip: bool, angle: f32| Pose { angle, flip };
        let mut body = [[Sprite::default(); JAW_FRAMES]; 2];
        let mut tail: [[Vec<Sprite>; 2]; 6] = Default::default();
        let mut dorsal = [Sprite::default(); 2];
        let mut pectoral: [Vec<Sprite>; 2] = Default::default();
        let mut stump = [Sprite::default(); 2];
        for (f, flip) in [false, true].into_iter().enumerate() {
            for k in 0..JAW_FRAMES {
                body[f][k] = pack(&mut p, &carve(&self::body(k as f32 / (JAW_FRAMES - 1) as f32 * JAW_OPEN), pose(flip, 0.0)));
            }
            for (i, reach) in REACH.iter().enumerate() {
                let part = if i < TAIL.len() { self::tail(i) } else { caudal() };
                tail[i][f] = (0..turns(*reach)).map(|t| pack(&mut p, &carve(&part, pose(flip, -reach + t as f32 * STEP)))).collect();
            }
            dorsal[f] = pack(&mut p, &carve(&self::dorsal(), pose(flip, 0.0)));
            let fin = self::pectoral();
            pectoral[f] = (0..turns(PADDLE)).map(|t| pack(&mut p, &carve(&fin, pose(flip, -PADDLE + t as f32 * STEP)))).collect();
            stump[f] = pack(&mut p, &carve(&pectoral_stump(), pose(flip, 0.0)));
        }
        // the windows' glass and the stone that cracks, read off the body facing right
        let (frame, mats) = carve_with(&self::body(0.0), pose(false, 0.0));
        let w = frame.img.width as i32;
        let at = |i: usize| ((i as i32 % w) - frame.pivot.0, (i as i32 / w) - frame.pivot.1);
        let mut windows: [Vec<(i32, i32)>; 4] = Default::default();
        for (i, m) in mats.iter().enumerate() {
            let (x, y) = at(i);
            if *m == M::Slot && (-17..=8).contains(&y) {
                if let Some(k) = WINDOWS.iter().position(|cx| (x as f32 - cx).abs() <= 2.6) {
                    windows[k].push((x, y));
                }
            }
        }
        let stone = |x: i32, y: i32| {
            let (px, py) = (x + frame.pivot.0, y + frame.pivot.1);
            px >= 0 && py >= 0 && px < w && py < frame.img.height as i32 && mats[(py * w + px) as usize] == M::Hull
        };
        let mut rng = crate::sim::rng::Rng::new(0x5EA4);
        let starts = [(52.0, -18.0, -1.0, 0.4), (40.0, -8.0, -1.0, -0.3), (20.0, -24.0, -0.6, 1.0), (-8.0, -23.0, -1.0, 0.3), (-26.0, 12.0, 1.0, 0.2), (6.0, 18.0, 1.0, -0.3), (62.0, -4.0, -1.0, 0.5), (-20.0, -20.0, 1.0, 0.5)];
        let cracks = starts
            .iter()
            .map(|&(sx, sy, dx, dy)| {
                let (mut x, mut y) = (sx, sy);
                let mut seam = vec![];
                for _ in 0..rng.range(12.0, 22.0) as i32 {
                    x += dx + rng.range(-0.5, 0.5);
                    y += dy + rng.range(-0.6, 0.6);
                    let (px, py) = (x.round() as i32, y.round() as i32);
                    if stone(px, py) && !seam.contains(&(px, py)) {
                        seam.push((px, py));
                    }
                }
                seam
            })
            .collect();
        let sample = |f: &Frame, every: usize| {
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
        };
        let shards = sample(&frame, 5);
        let dorsal_shards = sample(&carve(&self::dorsal(), pose(false, 0.0)), 2);
        let fin_shards = sample(&carve(&self::pectoral(), pose(false, 0.0)), 2);
        let tail_shards = std::array::from_fn(|i| sample(&carve(&if i < TAIL.len() { self::tail(i) } else { caudal() }, pose(false, 0.0)), 2));
        (p.img, JawsFrames { body, tail, dorsal, pectoral, stump, windows, cracks, shards, dorsal_shards, fin_shards, tail_shards })
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

    // back to front: the far fin, the tail fin, the tail, the dorsal fin, the body, the near fin
    let far = art.pectoral[f][turn_of(-paddle + 0.15, PADDLE)];
    if !fins_out[0] && !gone(7) {
        let (x, y) = swing(FAR_FIN_AT);
        put(tex, far, x, y, Color::new(k * 0.42, k * 0.42, k * 0.45, 1.0), clip, &bands);
    }
    let joints = b.tail();
    for i in (0..6).rev() {
        // the tail fin goes with the first blast (or when its turret breaks), then
        // the plates from the back
        if gone(if i == 5 { 0 } else { 5 - i }) || (i == 5 && fins_out[1]) {
            continue;
        }
        let (jx, jy, ang) = joints[i];
        let s = art.tail[i][f][turn_of(ang, REACH[i])];
        let (x, y) = px((jx, jy));
        put(tex, s, x, y, tint, clip, &bands);
    }
    let (dx, dy) = swing(DORSAL_AT);
    if !gone(6) {
        put(tex, art.dorsal[f], dx, dy, tint, clip, &bands);
    }
    let body = art.body[f][jaw];
    put(tex, body, bx, by, tint, clip, &bands);
    let (fx, fy) = swing(PECTORAL_AT);
    let near = if fins_out[0] { art.stump[f] } else { art.pectoral[f][turn_of(paddle, PADDLE)] };
    if !gone(7) {
        put(tex, near, fx, fy, tint, clip, &bands);
    }
    if clip.is_some_and(|c| c < by - 60) {
        return;
    }

    // the glows: lit windows before they fire, the turrets' cores, the eye, the jaws
    let mirror = |(x, y): (i32, i32)| (bx + if flip { -x } else { x }, by + y);
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
        a.fill(sx - 1, sy - 3, 3, 7, Color { a: 0.85 * k, ..super::carve::RECESS });
        for e in 0..3 {
            if (frame / 5 + e) % 3 != 0 {
                a.fill(sx - 1 + (e as i32 % 2), sy - 2 + 2 * e as i32, 1, 1, Color { a: 0.8 * k, ..SIGNAL });
            }
        }
    }
    // the dorsal fin's window: a red light, steady
    let (wx, wy) = swing(JAWS_DORSAL_CORE);
    a.fill(wx, wy, 1, 2, Color { a: 0.6 * k, ..SIGNAL });
    let (ex, ey) = swing(EYE);
    let pulse = 0.55 + 0.35 * ((frame as f32) * 0.08).sin();
    a.blit(a.r.discs[1], ex - 3, ey - 3, 1, Color { a: pulse * 0.6 * k, ..SIGNAL });
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
            put(tex, body, bx, by, Color::new(0.45, 0.05, 0.03, 0.2), clip, &[]);
        }
        for (i, q) in b.fins.iter().enumerate() {
            if q.flash > 0 && q.alive() {
                let (s, (x, y)) = if i == 0 { (near, (fx, fy)) } else { (art.tail[5][f][turn_of(joints[5].2, REACH[5])], px((joints[5].0, joints[5].1))) };
                put(tex, s, x, y, Color::new(0.7, 0.08, 0.05, 0.45), clip, &[]);
            }
        }
        if !bands.is_empty() {
            put(tex, body, bx - 2, by, Color::new(0.6, 0.05, 0.03, 0.34), clip, &bands);
            put(tex, body, bx + 2, by, Color::new(0.04, 0.45, 0.34, 0.28), clip, &bands);
        }
        r.additive(false);
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
    use super::super::carve::stamp;
    use super::super::carve::to_screen;
    use super::*;
    use crate::tune::JAWS_TAIL_ROOT as TAIL_ROOT;

    /// The whole fish posed: jaw open `open` rad, the tail's wave at `phase`, facing left if `flip`.
    fn shark(img: &mut Image, x: i32, y: i32, open: f32, phase: f32, amp: f32, flip: bool) {
        let pose = Pose { angle: 0.0, flip };
        let at = |p: Pt| {
            let (dx, dy) = to_screen(p.0, p.1, pose);
            (x + dx.round() as i32, y + dy.round() as i32)
        };
        let put = |img: &mut Image, f: &Frame, (px, py): (i32, i32)| stamp(img, f, px, py);
        // far fin, dimmed
        let mut far = carve(&pectoral(), Pose { angle: 0.25, flip });
        for py in 0..far.img.height as u32 {
            for px in 0..far.img.width as u32 {
                let c = far.img.get_pixel(px, py);
                if c.a > 0.0 {
                    far.img.set_pixel(px, py, Color::new(c.r * 0.45, c.g * 0.45, c.b * 0.45, 1.0));
                }
            }
        }
        put(img, &far, at(FAR_FIN_AT));
        // the tail, back to front
        let mut joints = vec![(TAIL_ROOT, 0.0f32)];
        let mut ang = 0.0f32;
        for (i, &(len, _)) in TAIL.iter().enumerate() {
            let k = (i + 1) as f32 / 7.0;
            ang += amp * (0.3 + k) * (phase - i as f32 * 0.55).sin() * 0.16;
            let (p, _) = *joints.last().unwrap();
            joints.last_mut().unwrap().1 = ang;
            joints.push(((p.0 - len * ang.cos(), p.1 - len * ang.sin()), ang));
        }
        let tip_ang = ang + amp * 1.3 * (phase - 6.0 * 0.55).sin() * 0.16;
        let (tip, _) = *joints.last().unwrap();
        put(img, &carve(&caudal(), Pose { angle: tip_ang, flip }), at(tip));
        for i in (0..TAIL.len()).rev() {
            let (p, a) = joints[i];
            put(img, &carve(&tail(i), Pose { angle: a, flip }), at(p));
        }
        put(img, &carve(&dorsal(), pose), at(DORSAL_AT));
        put(img, &carve(&body(open), pose), at((0.0, 0.0)));
        put(img, &carve(&pectoral(), Pose { angle: 0.0, flip }), at(PECTORAL_AT));
    }

    /// Every frame the game needs is carved at startup: it fits its texture, and it's quick.
    #[test]
    fn every_frame_fits_its_texture_and_carves_quickly() {
        let t = std::time::Instant::now();
        let (img, art) = JawsArt::sheet();
        let took = t.elapsed().as_secs_f32();
        println!("carved in {took:.3} s");
        assert!(took < 1.5, "{took} s");
        assert_eq!((img.width, img.height), (1024, 1024));
        // the windows' glass and the cracks were found in the stone
        assert!(art.windows.iter().all(|w| w.len() > 20), "{:?}", art.windows.iter().map(|w| w.len()).collect::<Vec<_>>());
        assert!(art.cracks.iter().map(|c| c.len()).sum::<usize>() > 60);
        // every turn of every part is there, in both facings
        for f in 0..2 {
            for (i, reach) in REACH.iter().enumerate() {
                assert_eq!(art.tail[i][f].len(), turns(*reach));
            }
        }
    }

    /// The boss is bone white like Stage 1's enemies: its stone never outshines a bullet's core.
    #[test]
    fn its_stone_is_bone_white_and_never_outshines_a_bullet() {
        use crate::render::palette::{CORE, luma};
        let f = carve(&body(0.0), Pose::default());
        let lumas: Vec<f32> = f.img.get_image_data().iter().filter(|p| p[3] > 0).map(|p| luma(Color::from_rgba(p[0], p[1], p[2], 255))).collect();
        let top = lumas.iter().cloned().fold(0.0, f32::max);
        assert!(top < luma(CORE) - 0.08 && top > 0.8, "{top}");
        let bright = lumas.iter().filter(|&&l| l > 0.55).count() as f32 / lumas.len() as f32;
        assert!(bright > 0.3, "only {:.0}% of it is bright", bright * 100.0);
    }

    /// The review sheet: the fish big, then posed at 1x.
    /// `cargo test --release jawsthic_sheet -- --ignored` writes shots/jawsthic_sheet.png.
    #[test]
    #[ignore]
    fn jawsthic_sheet() {
        let bg = Color::new(0.03, 0.035, 0.027, 1.0);
        let (w, h) = (700u16, 640u16);
        let mut sheet = Image::gen_image_color(w, h, bg);
        // big: 3x
        let mut big = Image::gen_image_color(230, 125, bg);
        shark(&mut big, 125, 62, 0.0, 0.0, 0.0, false);
        for y in 0..big.height as u32 * 3 {
            for x in 0..big.width as u32 * 3 {
                if x < w as u32 && y < h as u32 {
                    sheet.set_pixel(x, y, big.get_pixel(x / 3, y / 3));
                }
            }
        }
        // posed at 1x, as in the game
        let poses = [(0.0, 0.0, 1.0, false), (0.22, 1.5, 1.0, false), (0.5, 3.4, 1.0, false), (0.1, 0.8, 1.0, true), (0.35, 2.2, 1.0, true), (0.5, 4.6, 1.4, true)];
        for (k, &(open, ph, amp, flip)) in poses.iter().enumerate() {
            let (col, row) = (k % 3, k / 3);
            let x = 10 + col as i32 * 233 + if flip { 80 } else { 120 };
            shark(&mut sheet, x, 445 + row as i32 * 120, open, ph, amp, flip);
        }
        let _ = std::fs::create_dir_all("shots");
        let mut out = Image::gen_image_color(w * 2, h * 2, bg);
        for y in 0..h as u32 * 2 {
            for x in 0..w as u32 * 2 {
                // export_png writes the bottom row first
                out.set_pixel(x, y, sheet.get_pixel(x / 2, h as u32 - 1 - y / 2));
            }
        }
        out.export_png("shots/jawsthic_sheet.png");
    }
}
