//! The Pilgrim, rebuilt pixel by pixel from Jason's sheet: gothic spires, the cross
//! on the spine, red reactor dots down the center, red and teal wingtip lights, a
//! teal engine. The hull and the wing pods are separate sprites so the pods can
//! lag behind and tuck in for focus.
//!
//! The grids are the left half plus the center column; the right half mirrors it.

use super::art::{Atlas, Packer, grid_image, rows};
use super::palette::{ASH, CREAM, DUSK, SIGNAL, SLATE, TEAL, VOID};
use macroquad::prelude::*;

/// Left half of the hull; column 10 is the centerline. Row 11 (the cross's arms) is the hitbox.
const BODY: [&str; 34] = [
    "..........L",
    "..........r",
    ".........OA",
    ".........AC",
    "........OAS",
    "........CSO",
    "........ASO",
    ".......OASO",
    ".......CSOO",
    ".......ASOC",
    "......OASOC",
    "......CSOCC",
    "......ASOOC",
    ".....OASOOC",
    ".....CSOOOC",
    "....OASOOOO",
    "...OSASOOCC",
    "..OSDASOCOO",
    ".OSDSASOAOO",
    "OASDSASOAOP",
    "ASDSDASOAOO",
    "SDSDSASOAOP",
    "ASDSDASOAOO",
    "OSDSOASOAOP",
    ".OSOOASOAOO",
    "..O.OASOAAA",
    "....OASOCOO",
    "....OASOOCO",
    "....OASOOOC",
    ".....OSOOOC",
    ".....OAACAC",
    "......ODSDS",
    ".......OSAA",
    "........OAN",
];
const BODY_ANCHOR: (i32, i32) = (10, 11);

/// The left wing pod; column 4 is its spine. The right pod mirrors it.
const POD: [&str; 18] = [
    "....L..",
    "....r..",
    "...OCO.",
    "...CAS.",
    "..OCAS.",
    "..OCASO",
    "..OCADO",
    "..OCASO",
    ".OSCADO",
    ".SACASO",
    "OSACADO",
    "SAOCASO",
    "SDOCADO",
    "WSOCASO",
    "M.OCADO",
    "..OSASO",
    "..OAAAO",
    "...ONO.",
];
/// The pod's spine, and how far its top sits above the hitbox.
const POD_ANCHOR: (i32, i32) = (4, 5);

/// Engine plumes: idle flicker and full thrust, 4 frames each. Column 2 is the center.
const PLUME_IDLE: [&[&str]; 4] = [
    &[".tTt.", "..T..", "..t.."],
    &[".tTt.", ".tTt.", "..T..", "..t.."],
    &[".tCt.", "..T..", "..T..", "..t.."],
    &[".tTt.", "..T..", "..t..", "..t.."],
];
const PLUME_THRUST: [&[&str]; 4] = [
    &["tTCTt", ".TCT.", ".TCT.", "..C..", "..T..", "..T..", "..t..", "..t..", "...t."],
    &["tTCTt", "tTCTt", ".TCT.", ".TCT.", "..C..", "..T..", ".tT..", "..t..", "..t..", "..t.."],
    &["tTCTt", ".TCT.", ".TCT.", ".TCT.", "..C..", "..T..", "..T..", "..t..", ".t...", "..t.."],
    &["tTCTt", ".TCT.", ".TCT.", "..C..", "..T..", "..Tt.", "..t..", "..t..", "..t.."],
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
    let rgb = |r: u8, g: u8, b: u8| Some(Color::from_rgba(r, g, b, 255));
    match c {
        'O' => rgb(0x12, 0x12, 0x10),
        'D' => Some(DUSK),
        'S' => Some(SLATE),
        'A' => Some(ASH),
        'C' => Some(CREAM),
        'R' => Some(SIGNAL),
        'T' => Some(TEAL),
        // the lights' unlit colors; they're lit over the top at draw time
        'r' | 'L' | 'W' => rgb(0x6A, 0x1C, 0x17),
        'P' => rgb(0x7A, 0x22, 0x1B),
        't' | 'M' | 'N' => rgb(0x13, 0x57, 0x46),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glow {
    /// The red light on the nose.
    Nose,
    /// The red lights on the pod tips.
    PodTip,
    /// The red wingtip strobes.
    Wing,
    /// The teal wingtip lights.
    Teal,
    /// The reactor dots down the spine.
    Reactor,
    /// The teal engine nozzles.
    Nozzle,
}

#[derive(Clone, Copy, Debug)]
pub struct Light {
    /// Offset from the sprite's anchor.
    pub dx: i32,
    pub dy: i32,
    pub glow: Glow,
}

pub struct Sprite {
    pub r: Rect,
    pub sil: Rect,
    /// The anchor pixel inside the sprite (the hitbox for the hull, the spine top for pods).
    pub ax: i32,
    pub ay: i32,
    pub lights: Vec<Light>,
}

pub struct ShipArt {
    /// The hull rolled -2 (hard left) to 2 (hard right), indexed by bank + 2.
    pub body: Vec<Sprite>,
    /// The left pod; the right one is drawn mirrored.
    pub pod: Sprite,
    pub plume_idle: [Rect; 4],
    pub plume_thrust: [Rect; 4],
    pub muzzle: [Rect; 4],
    pub bolt: [Rect; 4],
    pub pod_shot: [Rect; 2],
    pub dot: Rect,
}

impl ShipArt {
    /// Every atlas region the ship uses (for the overlap test).
    #[cfg(test)]
    pub fn rects(&self) -> Vec<Rect> {
        let mut v: Vec<Rect> = self.body.iter().flat_map(|s| [s.r, s.sil]).collect();
        v.extend([self.pod.r, self.pod.sil, self.dot]);
        v.extend(self.plume_idle);
        v.extend(self.plume_thrust);
        v.extend(self.muzzle);
        v.extend(self.bolt);
        v.extend(self.pod_shot);
        v
    }
}

/// The full hull, mirrored from the left half.
pub fn body_rows() -> Vec<Vec<char>> {
    BODY.iter()
        .map(|r| {
            let l: Vec<char> = r.chars().collect();
            let mut full = l.clone();
            full.extend(l.iter().rev().skip(1));
            full
        })
        .collect()
}

fn darker(c: char) -> char {
    match c {
        'C' => 'A',
        'A' => 'S',
        'S' => 'D',
        c => c,
    }
}

/// The hull rolled toward the right by `b` (1 or 2): the dipping right wing
/// foreshortens (columns drop out) and turns away from the light.
fn roll_right(full: &[Vec<char>], b: usize) -> Vec<Vec<char>> {
    let w = full[0].len();
    // mirrored index m = w-1-c; drop wing columns, keep the outline and the spine
    let drop: &[usize] = if b == 1 { &[3] } else { &[2, 4] };
    full.iter()
        .map(|row| {
            let mut out: Vec<char> = row.iter().enumerate().filter(|(c, _)| !drop.contains(&(w - 1 - c))).map(|(_, ch)| *ch).collect();
            let n = out.len();
            for (c, ch) in out.iter_mut().enumerate() {
                if c + 6 >= n && *ch != 'L' && *ch != 'W' && *ch != 'M' {
                    *ch = darker(*ch);
                }
            }
            out.resize(w, '.');
            out
        })
        .collect()
}

fn lights(grid: &[Vec<char>], ax: i32, ay: i32) -> Vec<Light> {
    let mut v = vec![];
    for (y, row) in grid.iter().enumerate() {
        for (x, &c) in row.iter().enumerate() {
            let glow = match (c, y) {
                ('L', 0) if grid.len() > 20 => Glow::Nose,
                ('L', _) => Glow::PodTip,
                ('W', _) => Glow::Wing,
                ('M', _) => Glow::Teal,
                ('P', _) => Glow::Reactor,
                ('N', _) => Glow::Nozzle,
                _ => continue,
            };
            v.push(Light { dx: x as i32 - ax, dy: y as i32 - ay, glow });
        }
    }
    v
}

fn sprite(p: &mut Packer, grid: &[Vec<char>], ax: i32, ay: i32) -> Sprite {
    let (r, sil) = p.add_with_silhouette(&grid_image(grid, pal));
    Sprite { r, sil, ax, ay, lights: lights(grid, ax, ay) }
}

fn small(p: &mut Packer, src: &[&str]) -> Rect {
    p.add(&grid_image(&rows(src), pal))
}

pub fn build(p: &mut Packer) -> ShipArt {
    let full = body_rows();
    let flip = |g: &[Vec<char>]| -> Vec<Vec<char>> { g.iter().map(|r| r.iter().rev().cloned().collect()).collect() };
    let (ax, ay) = BODY_ANCHOR;
    let body = vec![
        sprite(p, &flip(&roll_right(&flip(&full), 2)), ax, ay),
        sprite(p, &flip(&roll_right(&flip(&full), 1)), ax, ay),
        sprite(p, &full, ax, ay),
        sprite(p, &roll_right(&full, 1), ax, ay),
        sprite(p, &roll_right(&full, 2), ax, ay),
    ];
    let pod = sprite(p, &rows(&POD), POD_ANCHOR.0, 0);
    ShipArt {
        body,
        pod,
        plume_idle: PLUME_IDLE.map(|g| small(p, g)),
        plume_thrust: PLUME_THRUST.map(|g| small(p, g)),
        muzzle: MUZZLE.map(|g| small(p, g)),
        bolt: BOLT.map(|g| small(p, g)),
        pod_shot: POD_SHOT.map(|g| small(p, g)),
        dot: small(p, &DOT),
    }
}

/// Everything about how the ship looks on one frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct Pose {
    pub bank: i8,
    /// Pod offsets from the hitbox (left, right).
    pub pods: [(f32, f32); 2],
    pub thrust: bool,
    /// Frames spent shooting in a row (0 when not shooting).
    pub fire_t: u32,
    pub beaming: bool,
    /// Hull flash frames left (counts down from BUMP_FLASH).
    pub hit_t: u32,
    pub frame: u64,
}

/// A cheap per-frame flicker in 0..1 (cosmetic, from the frame number).
fn flicker(frame: u64, salt: u64) -> f32 {
    let mut z = frame.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ salt.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z ^= z >> 29;
    (z % 1000) as f32 / 1000.0
}

fn light_on(glow: Glow, frame: u64, dx: i32, pose: &Pose) -> Option<(Color, f32)> {
    let f = frame;
    match glow {
        Glow::Nose => (f % 60 < 48).then_some((SIGNAL, 1.0)),
        // the two pod tips alternate
        Glow::PodTip => (((f / 30) % 2 == 0) == (dx < 0)).then_some((SIGNAL, 1.0)),
        // wingtips: a red double strobe, then teal
        Glow::Wing => (f % 90 < 4 || (8..12).contains(&(f % 90))).then_some((SIGNAL, 1.0)),
        Glow::Teal => ((45..75).contains(&(f % 90))).then_some((TEAL, 1.0)),
        Glow::Reactor => {
            let base = if pose.fire_t > 0 { 0.75 } else { 0.35 };
            let pulse = 0.5 + 0.5 * ((f as f32 * 0.12) + dx as f32).sin();
            Some((SIGNAL, (base + 0.25 * pulse).min(1.0)))
        }
        Glow::Nozzle => Some((TEAL, 0.55 + 0.45 * flicker(f, dx as u64 + 7))),
    }
}

fn draw_lights(a: &Atlas, s: &Sprite, x: i32, y: i32, flip: bool, pose: &Pose) {
    for l in &s.lights {
        let dx = if flip { -l.dx } else { l.dx };
        if let Some((c, k)) = light_on(l.glow, pose.frame, dx, pose) {
            a.fill(x + dx, y + l.dy, 1, 1, Color { a: k, ..c });
        }
    }
}

/// Where the hull's top-left goes for a hitbox at `(x, y)`.
fn hull_origin(s: &Sprite, x: i32, y: i32) -> (i32, i32) {
    (x - s.ax, y - s.ay)
}

/// Draw the Pilgrim with its hitbox (the cross) at whole pixel `(x, y)`.
pub fn draw(a: &Atlas, x: i32, y: i32, pose: &Pose) {
    let art = &a.r.ship;
    let b = pose.bank.clamp(-2, 2);
    let hull = &art.body[(b + 2) as usize];
    let (hx, hy) = hull_origin(hull, x, y);

    // engine plume under everything, hanging from the nozzle
    let plume = if pose.thrust { art.plume_thrust[(pose.frame / 3 % 4) as usize] } else { art.plume_idle[(pose.frame / 4 % 4) as usize] };
    a.blit(plume, x - 2, hy + hull.r.h as i32, 1, WHITE);

    // the pods, rolled with the hull: the dipping side tucks in more
    for (i, side) in [-1i32, 1].into_iter().enumerate() {
        let (ox, oy) = pose.pods[i];
        let dip = (b as i32 * side).max(0);
        let rise = (-(b as i32) * side).max(0);
        let inward = dip + rise / 2;
        let px = x + ox.round() as i32 - side * inward;
        let py = y + oy.round() as i32 - POD_ANCHOR.1 + dip.min(1);
        let shade = if dip > 0 { Color::new(0.8, 0.8, 0.8, 1.0) } else { WHITE };
        let pod = &art.pod;
        let flip = side > 0;
        let left = if flip { px - (pod.r.w as i32 - 1 - pod.ax) } else { px - pod.ax };
        if flip {
            a.blit_flip(pod.r, left, py, shade);
        } else {
            a.blit(pod.r, left, py, 1, shade);
        }
        draw_lights(a, pod, px, py, flip, pose);
        if pose.hit_t > 0 {
            hit_overlay(a, pod.sil, left, py, flip, pose.hit_t);
        }
        // pod muzzle flash while firing
        if pose.fire_t > 0 && !pose.beaming {
            let m = art.muzzle[((pose.fire_t + 2) % 4) as usize];
            a.blit(m, px - 2, py - m.h as i32 + 1, 1, Color { a: 0.8, ..WHITE });
        }
    }

    a.blit(hull.r, hx, hy, 1, WHITE);
    draw_lights(a, hull, x, y, false, pose);
    if pose.hit_t > 0 {
        hit_overlay(a, hull.sil, hx, hy, false, pose.hit_t);
    }
    if pose.fire_t > 0 {
        let m = art.muzzle[(pose.fire_t % 4) as usize];
        a.blit(m, x - 2, hy - m.h as i32 + 2, 1, WHITE);
    }
}

/// The hit flash from the sheet: red, redder, a cream blaze, a red afterglow.
fn hit_overlay(a: &Atlas, sil: Rect, x: i32, y: i32, flip: bool, hit_t: u32) {
    let phase = (crate::tune::BUMP_FLASH.saturating_sub(hit_t)) * 4 / crate::tune::BUMP_FLASH.max(1);
    let c = match phase {
        0 => Color { a: 0.35, ..SIGNAL },
        1 => Color { a: 0.7, ..SIGNAL },
        2 => Color { a: 0.85, ..CREAM },
        _ => Color { a: 0.25, ..SIGNAL },
    };
    if flip {
        a.blit_flip(sil, x, y, c);
    } else {
        a.blit(sil, x, y, 1, c);
    }
}

/// Teal afterimages of the hull at earlier positions (drawn under the ship).
pub fn draw_afterimages(a: &Atlas, bank: i8, at: &[(i32, i32)]) {
    let hull = &a.r.ship.body[(bank.clamp(-2, 2) + 2) as usize];
    for (k, &(x, y)) in at.iter().enumerate() {
        let alpha = [0.22, 0.13, 0.07][k.min(2)];
        let (hx, hy) = hull_origin(hull, x, y);
        a.blit(hull.sil, hx, hy, 1, Color { a: alpha, ..TEAL });
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

    #[test]
    fn grids_are_rectangular_and_use_the_palette() {
        for (name, g) in [("body", &BODY[..]), ("pod", &POD[..]), ("dot", &DOT[..])] {
            let w = g[0].len();
            assert!(g.iter().all(|r| r.len() == w), "{name} rows differ in width");
            assert!(g.iter().flat_map(|r| r.chars()).all(|c| c == '.' || pal(c).is_some()), "{name} uses an unknown color");
        }
        for set in [&PLUME_IDLE[..], &PLUME_THRUST[..], &MUZZLE[..], &BOLT[..], &POD_SHOT[..]] {
            for g in set {
                assert!(g.iter().all(|r| r.len() == g[0].len()));
            }
        }
    }

    #[test]
    fn the_pilgrim_is_the_planned_size() {
        let full = body_rows();
        // the pods reach POD_ANCHOR.0 px outward from their spines
        let (w, h) = (2.0 * (crate::tune::POD_X + POD_ANCHOR.0 as f32) + 1.0, full.len());
        assert_eq!(full[0].len(), 21);
        assert!((28.0..=32.0).contains(&w), "{w} wide with pods");
        assert!((34..=40).contains(&h), "{h} tall");
        assert_eq!(full[BODY_ANCHOR.1 as usize][BODY_ANCHOR.0 as usize], 'C', "the hitbox sits on the cross");
        assert_eq!(full[BODY_ANCHOR.1 as usize][BODY_ANCHOR.0 as usize - 1], 'C', "on its arms");
    }

    /// The value budget: the Pilgrim tops out at 85%.
    #[test]
    fn the_hull_stays_inside_its_value_budget() {
        for c in BODY.iter().chain(POD.iter()).flat_map(|r| r.chars()) {
            if let Some(col) = pal(c) {
                assert!(luma(col) <= 0.85, "{c} is too bright");
            }
        }
    }

    #[test]
    fn rolling_keeps_the_anchor_and_finds_every_light() {
        let full = body_rows();
        let straight = lights(&full, 10, 11);
        assert_eq!(straight.iter().filter(|l| l.glow == Glow::Reactor).count(), 3);
        assert_eq!(straight.iter().filter(|l| l.glow == Glow::Nose).count(), 1);
        for b in 1..=2 {
            let r = roll_right(&full, b);
            assert_eq!(r[0].len(), 21);
            assert_eq!(r[BODY_ANCHOR.1 as usize][BODY_ANCHOR.0 as usize], 'C', "the cross doesn't move");
            assert_eq!(lights(&r, 10, 11).len(), straight.len());
            let width = |g: &[Vec<char>]| g.iter().map(|row| row.iter().rposition(|c| *c != '.').unwrap_or(0)).max().unwrap();
            assert_eq!(width(&r), width(&full) - b, "the dipping side foreshortens");
        }
        let pod = lights(&rows(&POD), 4, 0);
        let kinds: Vec<Glow> = pod.iter().map(|l| l.glow).collect();
        assert!(kinds.contains(&Glow::PodTip) && kinds.contains(&Glow::Wing) && kinds.contains(&Glow::Teal) && kinds.contains(&Glow::Nozzle));
    }
}
