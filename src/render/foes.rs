//! The enemies, drawn: engraved monochrome craft built pixel by pixel in code, lit
//! from the top left like the Pilgrim. Red only where it means danger: a gun about
//! to fire, the derelict's weak points and, once it cracks, its burning seams.
//! Also the breather's background: drifting debris and the tiny tumbling figure.

use super::art::{Atlas, Packer, grid_image};
use super::palette::{ASH, CREAM, DUSK, SIGNAL, SLATE, SOOT, WHITE_A, tone};
use crate::sim::World;
use crate::sim::enemy::{Enemy, EnemyKind};
use crate::sim::pattern::Mount;
use crate::sim::rng::Rng;
use crate::tune::{CHARGE_FRAMES, HULL_EMERGE, MUZZLE_FRAMES};
use macroquad::prelude::*;

/// An enemy sprite, its white silhouette (the hit flash) and a sample of its pixels
/// relative to its center (what it shatters into).
pub struct FoeSprite {
    pub r: Rect,
    pub sil: Rect,
    pub pixels: Vec<(i8, i8, Color)>,
}

pub struct FoeArt {
    pub drone: FoeSprite,
    pub gunship: FoeSprite,
    /// The buoy's spikes turning, 4 frames.
    pub buoy: [FoeSprite; 4],
    pub heavy: FoeSprite,
    pub hull: FoeSprite,
    pub turret: FoeSprite,
    /// Where a turret broke off: a scorched socket.
    pub socket: Rect,
    /// The derelict's crack seams, pixels relative to its center (they burn red once it cracks).
    pub cracks: Vec<(i8, i8)>,
    /// Background scrap for the breather, and the tiny figure.
    pub debris: [Rect; 6],
    pub figure: Rect,
}

impl FoeArt {
    #[cfg(test)]
    pub fn rects(&self) -> Vec<Rect> {
        let mut v = vec![self.drone.r, self.drone.sil, self.gunship.r, self.gunship.sil, self.heavy.r, self.heavy.sil, self.hull.r, self.hull.sil, self.turret.r, self.turret.sil, self.socket, self.figure];
        for b in &self.buoy {
            v.extend([b.r, b.sil]);
        }
        v.extend(self.debris);
        v
    }

    pub fn sprite(&self, kind: EnemyKind, frame: u64) -> &FoeSprite {
        match kind {
            EnemyKind::Drone => &self.drone,
            EnemyKind::Gunship => &self.gunship,
            EnemyKind::Buoy => &self.buoy[(frame / 6 % 4) as usize],
            EnemyKind::Heavy => &self.heavy,
            EnemyKind::Hull => &self.hull,
        }
    }
}

const OUTLINE: Color = Color::new(0.07, 0.07, 0.063, 1.0);
/// The brightest an enemy gets: inside the 35-75% band.
fn hi() -> Color {
    tone(0.62)
}
const EMBER_DIM: Color = Color::new(0x6A as f32 / 255.0, 0x1C as f32 / 255.0, 0x17 as f32 / 255.0, 1.0);

fn pal(c: char) -> Option<Color> {
    match c {
        'o' => Some(OUTLINE),
        'K' => Some(SOOT),
        'D' => Some(DUSK),
        'S' => Some(SLATE),
        'A' => Some(ASH),
        'H' => Some(hi()),
        'r' => Some(EMBER_DIM),
        _ => None,
    }
}

/// The right half turned away from the light.
fn darker(c: char) -> char {
    match c {
        'H' => 'A',
        'A' => 'S',
        'S' => 'D',
        'D' => 'K',
        c => c,
    }
}

/// A full grid from its left half plus the center column, the right side in shade.
fn mirror(half: &[&str]) -> Vec<Vec<char>> {
    half.iter()
        .map(|r| {
            let l: Vec<char> = r.chars().collect();
            let mut full = l.clone();
            full.extend(l.iter().rev().skip(1).map(|&c| darker(c)));
            full
        })
        .collect()
}

/// A popcorn drone, nose down: swept spire wings and a dark visor.
const DRONE_ART: [&str; 14] = [
    "o.......",
    "oo......",
    "oHo....o",
    "oAHo..oA",
    ".oAHooAH",
    ".oAAHHAH",
    "..oSAAAS",
    "..oKKKKK",
    "...oSKKo",
    "...oSSSS",
    "....oSAS",
    ".....oSA",
    "......oS",
    ".......o",
];

/// A gunship: two spired pods slung either side of an armored, arched hull, the
/// gun at its nose.
const GUNSHIP_ART: [&str; 23] = [
    "...o...........o",
    "..oHo.........oA",
    "..oAo.........oH",
    ".oHAAo.......oAH",
    ".oAASo.......oAH",
    ".oAASo......oHAH",
    ".oASSo.....oHAAH",
    ".oASSoooooooAAAH",
    "oHASSoHHHHHHAASA",
    "oAASSoAAAAAAASSA",
    "oAASKoSSSSSSSSKA",
    "oASSKoDDDDDoooKS",
    "oASKKooooooKKKKS",
    ".oSKo....oSKSKKS",
    ".oSKo....oSSKKoS",
    ".oSo.....oSSSKoA",
    "..oo.....oDSSSoS",
    "..........oDSSSS",
    "..........oKDSSK",
    "...........oKDSK",
    "............oKKK",
    ".............oKK",
    "..............oo",
];

/// A 4x4 ordered-dither threshold in 0..1 (the black hole's stipple, in metal).
fn bayer(x: i32, y: i32) -> f32 {
    const M: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
    (M[(y & 3) as usize][(x & 3) as usize] as f32 + 0.5) / 16.0
}

/// The engraving's five tones, dark to light.
fn tone_at(i: usize) -> Color {
    [SOOT, DUSK, SLATE, ASH, hi()][i.min(4)]
}

/// Paint a shape (`mask` gives a material per pixel: 0 empty, 1 plate, 2 recess,
/// 3 dark port, 4 red core) as engraved metal lit from above and to the left: an
/// outline, a bright lip along the top, a dark underside, a rounded fall-off down
/// each column stippled between the tones, and seams every `panel` px with rivets.
fn engrave(w: u16, h: u16, panel: i32, mask: impl Fn(i32, i32) -> u8) -> Image {
    let mut img = Image::gen_image_color(w, h, Color::new(0.0, 0.0, 0.0, 0.0));
    let m = |x: i32, y: i32| if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 { 0 } else { mask(x, y) };
    // how much solid hull lies from (x, y) in a direction
    let run = |x: i32, y: i32, dx: i32, dy: i32| (1..64).take_while(|&k| m(x + dx * k, y + dy * k) != 0).count() as f32;
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let k = m(x, y);
            if k == 0 {
                continue;
            }
            let edge = m(x - 1, y) == 0 || m(x + 1, y) == 0 || m(x, y - 1) == 0 || m(x, y + 1) == 0;
            let c = if edge {
                OUTLINE
            } else {
                match k {
                    // a recess: dark, its upper lip in shadow
                    2 => if m(x, y - 1) != 2 { OUTLINE } else if bayer(x, y) < 0.3 { DUSK } else { SOOT },
                    3 => OUTLINE,
                    4 => EMBER_DIM,
                    _ => {
                        let (up, down, left, right) = (run(x, y, 0, -1), run(x, y, 0, 1), run(x, y, -1, 0), run(x, y, 1, 0));
                        let v = up / (up + down).max(1.0);
                        let hz = left / (left + right).max(1.0);
                        let mut s = 1.04 - 0.62 * v - 0.2 * hz;
                        if up < 1.5 {
                            s = 1.0; // the lit lip along the top
                        } else if down < 1.5 {
                            s = 0.1; // the underside
                        }
                        if panel > 0 && up > 1.5 && down > 1.5 {
                            if x % panel == 0 {
                                s -= 0.3; // a seam
                            } else if x % panel == 1 && y % 4 == 1 {
                                s += 0.3; // a rivet beside it
                            }
                        }
                        tone_at((s * 4.0 + bayer(x, y) - 0.5).floor().max(0.0) as usize)
                    }
                }
            };
            img.set_pixel(x as u32, y as u32, c);
        }
    }
    img
}

/// A turret buoy: an armored ring around a dark eye, four spikes turning.
fn buoy(frame: u32) -> Image {
    let s = 23u16;
    let mut img = Image::gen_image_color(s, s, Color::new(0.0, 0.0, 0.0, 0.0));
    let c = 11.0;
    let spin = frame as f32 * std::f32::consts::FRAC_PI_8 * 0.5;
    for y in 0..s as i32 {
        for x in 0..s as i32 {
            let (dx, dy) = (x as f32 - c, y as f32 - c);
            let d = dx.hypot(dy);
            let a = dy.atan2(dx) - spin;
            let lit = dx + dy < -2.0;
            // spikes every quarter turn
            let q = (a / std::f32::consts::FRAC_PI_2).round() * std::f32::consts::FRAC_PI_2;
            let off = (a - q).abs() * d;
            let col = if d > 10.0 && d <= 11.6 && off < 1.0 {
                if off < 0.5 { if lit { hi() } else { ASH } } else { OUTLINE }
            } else if d > 10.0 {
                continue;
            } else if d > 9.2 {
                OUTLINE
            } else if d > 7.2 {
                // the ring, with rivets that turn with the spikes
                let r = (a / std::f32::consts::FRAC_PI_4).round() * std::f32::consts::FRAC_PI_4;
                if (a - r).abs() * d < 0.6 && d > 7.8 && d < 8.8 {
                    if lit { hi() } else { ASH }
                } else if lit {
                    ASH
                } else {
                    SLATE
                }
            } else if d > 6.4 {
                SOOT
            } else if d > 3.2 {
                if lit { SLATE } else if (x + y) % 3 == 0 { SOOT } else { DUSK }
            } else if d > 1.6 {
                SOOT
            } else {
                OUTLINE
            };
            img.set_pixel(x as u32, y as u32, col);
        }
    }
    img
}

/// The heavy: a broad armored gunboat with spired flank towers, a ring launcher in
/// the middle and a needle gun on each tower.
fn heavy_mask(x: i32, y: i32) -> u8 {
    let (x, y) = (x - 24, y - 17);
    let ax = x.abs();
    // flank towers, spired at the top, gun barrels below
    let tower = (13..=19).contains(&ax) && y >= -17 + (ax - 16).abs() * 3 && y <= 9;
    let barrel = (15..=17).contains(&ax) && (9..=14).contains(&y);
    // the hull: tapering to a nose
    let half = if y > 4 { 22 - (y - 4) * 2 } else { 22 };
    let hull = y >= -9 && y <= 16 && ax <= half && !(y < -6 && ax > 19);
    if !(tower || barrel || hull) {
        return 0;
    }
    if barrel && y >= 12 && ax == 16 {
        return 3; // muzzles
    }
    let d = ((x * x + (y - 3) * (y - 3)) as f32).sqrt();
    if d <= 2.0 {
        return 3; // the ring launcher's port
    }
    if d <= 6.5 {
        return 2;
    }
    // slots down the towers
    if tower && ax == 16 && (-8..=4).contains(&y) && y % 4 == 0 {
        return 2;
    }
    1
}

/// The derelict: a long gothic capital ship lying broadside, a castle of arched
/// windows amidships, spires along its back, plating torn open, its stern broken
/// off into bare ribs.
fn hull_mask(x: i32, y: i32, holes: &[(i32, i32, i32, i32)]) -> u8 {
    let (x, y) = (x - 58, y - 25);
    let ax = x.abs();
    // the hull's half-height: full amidships, tapering to a pointed bow (left)
    let half = if x < -36 { 13 - (-36 - x) * 3 / 5 } else { 13 };
    let body = ax <= 57 && y >= -half && y <= half + 2;
    // the castle amidships, crenellated
    let castle = (-24..=20).contains(&x) && y < -half && y >= -20 && !(y == -20 && x.rem_euclid(4) == 0);
    // spires: from the castle roof, and two from the deck
    let spire = |sx: i32, base: i32| {
        let dx = (x - sx).abs();
        dx <= 2 && y < base && y >= -25 + dx * 3
    };
    let spires = spire(-16, -20) || spire(-2, -20) || spire(12, -20) || spire(-34, -half) || spire(30, -half);
    // a keel ridge under the middle
    let keel = ax <= 30 && y > half + 2 && y <= half + 5 - ax / 15;
    if !(body || castle || spires || keel) {
        return 0;
    }
    // the castle's arched windows
    if castle && (-18..=-15).contains(&y) && x.rem_euclid(6) <= 1 && x > -22 && x < 18 {
        return if y == -18 && x.rem_euclid(6) == 1 { 2 } else { 3 };
    }
    // the broken stern: plating gone, ribs left standing
    if x > 42 {
        let rib = (x - 42) % 4 == 0;
        let jag = y > -half + 2 + ((x * 7) % 5) && y < half - ((x * 3) % 4);
        if !rib && jag {
            return 0;
        }
        if !rib {
            return 2;
        }
    }
    // holes torn in the plating
    for &(hx, hy, rx, ry) in holes {
        let (dx, dy) = ((x - hx) as f32 / rx as f32, (y - hy) as f32 / ry as f32);
        if dx * dx + dy * dy <= 1.0 {
            return 2;
        }
    }
    // portholes down the length
    if y == -4 && x % 6 == 0 && ax < 40 {
        return 3;
    }
    1
}

/// A weak-point turret: an armored dome around a red core, barrel down.
fn turret_mask(x: i32, y: i32) -> u8 {
    let (dx, dy) = (x - 8, y - 8);
    let d = ((dx * dx + dy * dy) as f32).sqrt();
    let barrel = dx.abs() <= 1 && (4..=8).contains(&dy);
    if d > 7.6 && !barrel {
        return 0;
    }
    if barrel && dy >= 7 {
        return 3;
    }
    if d <= 2.4 {
        return 4;
    }
    if d <= 3.6 {
        return 2;
    }
    1
}

/// A sample of an image's lit pixels relative to its center, `every` n-th one.
fn pixels(img: &Image, every: usize) -> Vec<(i8, i8, Color)> {
    let (w, h) = (img.width as i32, img.height as i32);
    let mut v = vec![];
    let mut k = 0;
    for y in 0..h {
        for x in 0..w {
            let c = img.get_pixel(x as u32, y as u32);
            if c.a > 0.0 {
                if k % every == 0 {
                    v.push(((x - w / 2) as i8, (y - h / 2) as i8, Color { a: 1.0, ..c }));
                }
                k += 1;
            }
        }
    }
    v
}

fn foe(p: &mut Packer, img: &Image, every: usize) -> FoeSprite {
    let (r, sil) = p.add_with_silhouette(img);
    FoeSprite { r, sil, pixels: pixels(img, every) }
}

/// The derelict's crack seams: a few jagged lines wandering along the hull.
fn cracks(holes: &[(i32, i32, i32, i32)]) -> Vec<(i8, i8)> {
    let mut rng = Rng::new(0xC4AC);
    let mut v = vec![];
    for &(hx, hy, _, _) in holes {
        for dir in [-1, 1] {
            let (mut x, mut y) = (hx, hy);
            for _ in 0..rng.range(10.0, 20.0) as i32 {
                x += dir;
                if rng.chance(0.4) {
                    y += if rng.chance(0.5) { 1 } else { -1 };
                }
                y = y.clamp(-12, 14);
                if hull_mask(x + 58, y + 25, &[]) == 1 {
                    v.push((x as i8, y as i8));
                }
            }
        }
    }
    v
}

/// Scrap and rock for the breather's debris field: dim, inside the background's budget.
fn debris(i: usize) -> Image {
    let mut rng = Rng::new(0xDEB0 + i as u64);
    let s = [5u16, 7, 9, 6, 11, 8][i];
    let mut img = Image::gen_image_color(s, s, Color::new(0.0, 0.0, 0.0, 0.0));
    let c = s as f32 * 0.5;
    let bumps: Vec<f32> = (0..6).map(|_| rng.range(0.55, 1.0)).collect();
    for y in 0..s as i32 {
        for x in 0..s as i32 {
            let (dx, dy) = (x as f32 + 0.5 - c, y as f32 + 0.5 - c);
            let a = dy.atan2(dx).rem_euclid(std::f32::consts::TAU);
            let r = bumps[((a / std::f32::consts::TAU * 6.0) as usize).min(5)] * c;
            let d = dx.hypot(dy);
            if d > r {
                continue;
            }
            let v = if dx + dy < -c * 0.3 { 0.24 } else if d > r - 1.0 { 0.1 } else { 0.16 };
            img.set_pixel(x as u32, y as u32, tone(v));
        }
    }
    img
}

/// The tiny figure from Jason's black hole image, arms up, sneakers first.
const FIGURE: [&str; 11] = [
    "a.....a",
    ".a...a.",
    "..hh...",
    "..hh...",
    ".bbbb..",
    "..bb...",
    "..bbb..",
    ".b..b..",
    ".b...b.",
    "s....s.",
    "ss...ss",
];

pub fn build(p: &mut Packer) -> FoeArt {
    let rows = |g: Vec<Vec<char>>| grid_image(&g, pal);
    let drone = rows(mirror(&DRONE_ART));
    let gunship = rows(mirror(&GUNSHIP_ART));
    let buoys = [0, 1, 2, 3].map(buoy);
    let heavy = engrave(49, 35, 6, heavy_mask);
    let holes = [(-22, 4, 4, 3), (8, -8, 3, 2), (24, 6, 5, 3)];
    let hull = engrave(117, 51, 12, |x, y| hull_mask(x, y, &holes));
    let turret = engrave(17, 17, 0, turret_mask);
    // a broken turret's socket: a scorched ring
    let mut socket = Image::gen_image_color(13, 9, Color::new(0.0, 0.0, 0.0, 0.0));
    for y in 0..9i32 {
        for x in 0..13i32 {
            let (dx, dy) = ((x - 6) as f32 / 6.4, (y - 4) as f32 / 4.4);
            let d = dx * dx + dy * dy;
            if d <= 1.0 {
                socket.set_pixel(x as u32, y as u32, if d > 0.55 { OUTLINE } else if (x + y) % 2 == 0 { SOOT } else { DUSK });
            }
        }
    }
    let figure_pal = |c: char| match c {
        'h' => Some(tone(0.26)),
        'a' => Some(tone(0.2)),
        'b' => Some(tone(0.15)),
        's' => Some(tone(0.3)),
        _ => None,
    };
    let figure = grid_image(&FIGURE.iter().map(|r| r.chars().collect()).collect::<Vec<_>>(), figure_pal);
    FoeArt {
        drone: foe(p, &drone, 1),
        gunship: foe(p, &gunship, 2),
        buoy: [foe(p, &buoys[0], 2), foe(p, &buoys[1], 2), foe(p, &buoys[2], 2), foe(p, &buoys[3], 2)],
        heavy: foe(p, &heavy, 3),
        hull: foe(p, &hull, 5),
        turret: foe(p, &turret, 2),
        socket: p.add(&socket),
        cracks: cracks(&holes),
        debris: [0, 1, 2, 3, 4, 5].map(|i| p.add(&debris(i))),
        figure: p.add(&figure),
    }
}

fn snap(v: f32) -> i32 {
    v.round() as i32
}

/// How far the derelict has risen out of the disk: 0 (a black shape) to 1.
pub fn emerged(e: &Enemy) -> f32 {
    if e.kind == EnemyKind::Hull { (e.t as f32 / HULL_EMERGE as f32).min(1.0) } else { 1.0 }
}

/// Every enemy, the big ones underneath, with hit flashes, broken sockets, burning
/// seams and the red charge-up on every gun about to fire.
pub fn draw(a: &Atlas, w: &World, ox: i32, oy: i32) {
    let art = &a.r.foes;
    let f = w.frame;
    for kind in [EnemyKind::Hull, EnemyKind::Heavy, EnemyKind::Buoy, EnemyKind::Gunship, EnemyKind::Drone] {
        for e in w.foes.list.iter().filter(|e| e.kind == kind && e.alive()) {
            let s = art.sprite(kind, f);
            let (x, y) = (ox + snap(e.x - s.r.w * 0.5), oy + snap(e.y - s.r.h * 0.5));
            let k = emerged(e);
            let tint = Color::new(k, k, k, 1.0);
            a.blit(s.r, x, y, 1, tint);
            // popcorn that's going to shoot has a dim red eye until it does
            if kind == EnemyKind::Drone && e.variant == 1 && e.guns.first().is_some_and(|g| g.e.since == u32::MAX) {
                a.fill(ox + snap(e.x), oy + snap(e.y) + 1, 1, 1, Color { a: 0.8, ..SIGNAL });
            }
            if kind == EnemyKind::Hull {
                draw_hull_parts(a, e, ox, oy, tint, f);
            }
            if e.parts[0].flash > 0 {
                a.blit(s.sil, x, y, 1, Color { a: 0.7, ..CREAM });
            }
            draw_telegraphs(a, e, ox, oy, f);
        }
    }
}

fn draw_hull_parts(a: &Atlas, e: &Enemy, ox: i32, oy: i32, tint: Color, f: u64) {
    let art = &a.r.foes;
    // the seams burn once it's cracked
    if e.phase == 2 {
        let pulse = 0.55 + 0.35 * ((f as f32) * 0.15).sin();
        for &(dx, dy) in &art.cracks {
            a.fill(ox + snap(e.x) + dx as i32, oy + snap(e.y) + dy as i32, 1, 1, Color { a: pulse, ..SIGNAL });
        }
    }
    for q in &e.parts[1..] {
        let (cx, cy) = (ox + snap(e.x + q.dx), oy + snap(e.y + q.dy));
        if !q.alive() {
            a.blit(art.socket, cx - 6, cy - 9, 1, tint);
            continue;
        }
        let t = &art.turret;
        let (x, y) = (cx - (t.r.w * 0.5) as i32, cy - (t.r.h * 0.5) as i32);
        a.blit(t.r, x, y, 1, tint);
        // the weak point's red core, brighter as the turret wears down
        let hurt = 1.0 - q.hp / q.max_hp;
        a.fill(cx - 1, cy - 1, 3, 3, Color { a: (0.35 + 0.5 * hurt) * tint.r, ..SIGNAL });
        if q.flash > 0 {
            a.blit(t.sil, x, y, 1, Color { a: 0.7, ..CREAM });
        }
    }
}

/// A gun about to fire glows red at its muzzle, swelling until the shot (the
/// telegraph); the moment it fires, a red flash.
fn draw_telegraphs(a: &Atlas, e: &Enemy, ox: i32, oy: i32, f: u64) {
    for g in &e.guns {
        if !e.parts[g.part as usize].alive() || (g.phase != 0 && g.phase != e.phase) {
            continue;
        }
        let Mount::Owner { dx, dy } = g.e.gun.mount else { continue };
        let (x, y) = (ox + snap(e.x + dx), oy + snap(e.y + dy));
        let left = g.e.until_next();
        if e.armed && left <= CHARGE_FRAMES {
            let k = 1.0 - left as f32 / CHARGE_FRAMES as f32;
            let flicker = if f % 4 < 2 { 1.0 } else { 0.8 };
            let r = (1.0 + k * 2.4).round() as usize;
            let disc = a.r.discs[r.clamp(1, 16) - 1];
            // red only: a white center would read as a bullet's core
            a.blit(disc, x - r as i32 - 1, y - r as i32 - 1, 1, Color { a: (0.2 + 0.5 * k) * flicker, ..SIGNAL });
        }
        if g.e.since < MUZZLE_FRAMES {
            let k = 1.0 - g.e.since as f32 / MUZZLE_FRAMES as f32;
            a.fill(x - 1, y - 1, 3, 3, Color { a: k, ..SIGNAL });
            a.fill(x - 3, y, 7, 1, Color { a: k * 0.6, ..SIGNAL });
            a.fill(x, y - 3, 1, 7, Color { a: k * 0.6, ..SIGNAL });
        }
    }
}

/// The derelict after its death, `k` of the way (0 to 1) to the final blast: the wreck
/// shudders, its seams blaze and it darkens as the blasts eat through it.
pub fn draw_wreck(a: &Atlas, ox: i32, oy: i32, x: f32, y: f32, k: f32, f: u64) {
    let art = &a.r.foes;
    let s = &art.hull;
    let jx = [0, 1, 0, -1][(f % 4) as usize];
    let jy = [0, 0, 1, 0][(f / 2 % 4) as usize];
    let (cx, cy) = (ox + snap(x) + jx, oy + snap(y) + jy);
    let dark = 1.0 - 0.6 * k;
    a.blit(s.r, cx - (s.r.w * 0.5) as i32, cy - (s.r.h * 0.5) as i32, 1, Color::new(dark, dark, dark, 1.0));
    let burn = 0.6 + 0.4 * ((f as f32) * 0.6).sin().abs();
    for &(dx, dy) in &art.cracks {
        a.fill(cx + dx as i32, cy + dy as i32, 1, 1, Color { a: burn, ..SIGNAL });
    }
    // the blasts flash it white now and then
    if f % 6 < 2 {
        a.blit(s.sil, cx - (s.r.w * 0.5) as i32, cy - (s.r.h * 0.5) as i32, 1, Color { a: 0.35 * (1.0 - k), ..CREAM });
    }
}

/// Engine exhausts (offsets from the center): enemies fly nose down, so their engines face up.
pub fn engines(kind: EnemyKind) -> &'static [(f32, f32)] {
    match kind {
        EnemyKind::Drone => &[(0.0, -6.0)],
        EnemyKind::Gunship => &[(-13.0, -11.0), (13.0, -11.0)],
        EnemyKind::Buoy => &[],
        EnemyKind::Heavy => &[(-16.0, -17.0), (16.0, -17.0), (0.0, -10.0)],
        EnemyKind::Hull => &[],
    }
}

/// The breather's background: scrap drifting down through the far distance, `t`
/// frames into it (pure function of the time, like the stars).
pub fn draw_debris(a: &Atlas, ox: i32, oy: i32, t: u32, fade: f32) {
    let mut rng = Rng::new(0x5C4A9);
    for i in 0..26 {
        let x = rng.range(10.0, 260.0);
        let speed = rng.range(0.25, 0.9);
        let start = rng.range(0.0, 400.0);
        let spin = rng.range(-0.04, 0.04);
        let src = a.r.foes.debris[i % 6];
        let y = (start + t as f32 * speed) % 420.0 - 40.0;
        let drift = (t as f32 * 0.01 + i as f32).sin() * 4.0;
        let ang = t as f32 * spin + i as f32;
        a.blit_rot(src, ox as f32 + x + drift, oy as f32 + y, ang, Color { a: fade, ..WHITE_A });
    }
}

/// The tiny figure tumbling toward the hole, trailing red, green and blue streaks.
/// `k` runs 0 to 1 across its fall; it fades as it nears the hole.
pub fn draw_figure(a: &Atlas, ox: i32, oy: i32, k: f32, hole: (f32, f32)) {
    let from = (214.0, 330.0);
    let ease = k * k * (3.0 - 2.0 * k);
    let (x, y) = (from.0 + (hole.0 - from.0) * ease, from.1 + (hole.1 + 14.0 - from.1) * ease);
    let fade = (1.0 - ((k - 0.75) / 0.25).clamp(0.0, 1.0)) * (k / 0.08).min(1.0);
    if fade <= 0.0 {
        return;
    }
    // streaks behind it, along the way it came
    let (dx, dy) = (from.0 - hole.0, from.1 - hole.1);
    let d = dx.hypot(dy);
    let (ux, uy) = (dx / d, dy / d);
    for (i, (c, a0)) in [(SIGNAL, 0.5), (Color::new(0.2, 0.8, 0.4, 1.0), 0.22), (Color::new(0.3, 0.45, 1.0, 1.0), 0.22)].iter().enumerate() {
        let side = i as f32 - 0.6;
        for s in 0..14 {
            if (s + i) % 3 == 2 {
                continue;
            }
            let along = 4.0 + s as f32 * 1.3;
            let px = x + ux * along - uy * side * 1.5;
            let py = y + uy * along + ux * side * 1.5;
            let fall = 1.0 - s as f32 / 14.0;
            a.fill(ox + snap(px), oy + snap(py), 1, 1, Color { a: a0 * fall * fade, ..*c });
        }
    }
    a.blit_rot(a.r.foes.figure, ox as f32 + x, oy as f32 + y, k * 9.0, Color { a: fade, ..WHITE_A });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::palette::luma;

    fn brightest(img: &Image) -> f32 {
        img.get_image_data().iter().filter(|p| p[3] > 0).map(|p| luma(Color::from_rgba(p[0], p[1], p[2], 255))).fold(0.0, f32::max)
    }

    #[test]
    fn enemies_sit_in_the_enemy_value_band() {
        let holes = [(-22, 4, 4, 3), (8, -8, 3, 2), (24, 6, 5, 3)];
        let ims = [
            grid_image(&mirror(&DRONE_ART), pal),
            grid_image(&mirror(&GUNSHIP_ART), pal),
            buoy(0),
            engrave(49, 35, 6, heavy_mask),
            engrave(117, 51, 12, |x, y| hull_mask(x, y, &holes)),
            engrave(17, 17, 0, turret_mask),
        ];
        for (i, im) in ims.iter().enumerate() {
            let b = brightest(im);
            assert!((0.35..=0.75).contains(&b), "sprite {i}: {b}");
        }
        assert_eq!(ims[0].width, 15);
        assert_eq!(ims[1].width, 31);
    }

    #[test]
    fn the_background_extras_stay_in_the_background_budget() {
        for i in 0..6 {
            assert!(brightest(&debris(i)) <= 0.3);
        }
        let fig = grid_image(&FIGURE.iter().map(|r| r.chars().collect()).collect::<Vec<_>>(), |c| match c {
            '.' => None,
            _ => Some(tone(0.3)),
        });
        assert!(brightest(&fig) <= 0.31);
    }

    #[test]
    fn sprites_are_as_big_as_their_hurtboxes() {
        use crate::tune::{BUOY, DRONE, GUNSHIP, HEAVY, HULL};
        let holes = [(0, 0, 1, 1)];
        let sizes = [
            (grid_image(&mirror(&DRONE_ART), pal), DRONE),
            (grid_image(&mirror(&GUNSHIP_ART), pal), GUNSHIP),
            (buoy(0), BUOY),
            (engrave(49, 35, 6, heavy_mask), HEAVY),
            (engrave(117, 51, 7, |x, y| hull_mask(x, y, &holes)), HULL),
        ];
        for (im, (_, w, h)) in sizes {
            assert!((im.width as f32 - w).abs() <= 3.0 && (im.height as f32 - h).abs() <= 3.0, "{}x{} vs {w}x{h}", im.width, im.height);
        }
    }
}
