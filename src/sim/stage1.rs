//! Stage 1, OUTER DISK: the script, as data. Every cue is timed in seconds from the
//! start, on 7 lanes, and the two edge lanes are never used (except by schools coming
//! in from beyond the sides, warned at that edge first). The Toaplan pattern: gunships
//! alternate sides, popcorn fills in around them, waves overlap, and the high-HP
//! enemies come one at a time. Since v0.12.0 (the Swarm) it's lots of fodder and few
//! bullets, and each new enemy is introduced alone, then in numbers, then mixed in.
//!
//!   0:00  fly-in: the title, the first drones, a school of remoras (no bullets for 6 s)
//!   0:10  gunships alternate sides, popcorn around them; the first lancers take aim
//!   0:40  turret buoys join: static plus aimed makes lanes; lancers flank them
//!   1:10  a breather: the disk flares, scenery drifts down to shoot, a tiny figure
//!         tumbles past
//!   1:25  the derelict hull rises out of the disk (the mid-boss)
//!   1:55  heavies one at a time through swarms, rising to an overrun
//!   2:30  calm, WARNING
//!   2:32  JAWSTHIC rises out of the disk; the stage ends when it's gone

use super::Event;
use super::enemy::{EnemyKind, Foes, Path, Spline};
use crate::tune::*;

/// Popcorn flight paths. Each bends toward its side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flight {
    /// Straight down the screen with an S in it.
    Dive,
    /// Down, a long sweep across, and back off the top on the far side.
    Swoop,
    /// Down and a tight U-turn back up.
    Arc,
    /// A diagonal right across the screen and out the bottom.
    Cross,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cue {
    /// `n` drones down `lane`, `gap` frames apart, flying `path` bending toward `side`;
    /// with `fire`, each fires one aimed shot.
    Drones { lane: u8, path: Flight, side: Side, n: u8, gap: u8, fire: bool },
    /// `n` couriers dart straight down `lane`, one after another; with `fire`, each
    /// fires one aimed needle.
    Couriers { lane: u8, n: u8, fire: bool },
    /// An usher flies in to wait at height `y`, firing pairs.
    Usher { lane: u8, y: f32 },
    /// A gunship flies in to hover at height `y`.
    Gunship { lane: u8, y: f32 },
    /// A turret buoy drifts down: rings, or a spiral.
    Buoy { lane: u8, spiral: bool },
    /// A heavy flies in to hover at height `y`.
    Heavy { lane: u8, y: f32 },
    /// The derelict hull rises out of the disk.
    Derelict,
    /// A lancer flies in to height `y`, takes aim at you and rushes.
    Lancer { lane: u8, y: f32 },
    /// A school of `n` remoras swims across from `side` at height `y`; with `fire`,
    /// every other one fires an aimed shot.
    Remoras { side: Side, y: f32, n: u8, fire: bool },
    /// Scenery drifts down `lane`: debris, a satellite or a monolith (`what` 0-2).
    Wreck { lane: u8, what: u8 },
}

use Cue::*;
use Flight::*;
use Side::*;

const fn pop(lane: u8, path: Flight, side: Side, n: u8, fire: bool) -> Cue {
    Drones { lane, path, side, n, gap: 9, fire }
}

/// Frames between couriers in a line.
const COURIER_GAP: u32 = 12;

/// The whole stage. Sections start at `tune::SECTIONS`.
pub const SCRIPT: &[(f32, Cue)] = &[
    // FLY-IN: drones, then the first school of remoras; the first shots come after 6 s
    (1.5, pop(2, Swoop, Right, 5, false)),
    (3.5, pop(4, Swoop, Left, 5, false)),
    (4.5, Couriers { lane: 3, n: 3, fire: false }),
    (5.5, pop(1, Arc, Right, 4, false)),
    (5.5, pop(5, Arc, Left, 4, false)),
    (6.5, Remoras { side: Left, y: 60.0, n: 8, fire: false }),
    (7.0, pop(3, Dive, Right, 4, true)),
    (8.5, pop(2, Swoop, Right, 5, true)),
    // GUNSHIPS: alternating sides, popcorn around them; the lancer comes in alone,
    // then in pairs, and the schools start firing
    (10.0, Gunship { lane: 1, y: 70.0 }),
    (11.0, pop(4, Swoop, Left, 5, true)),
    (12.5, Gunship { lane: 5, y: 64.0 }),
    (13.0, Usher { lane: 2, y: 112.0 }),
    (13.0, Usher { lane: 4, y: 112.0 }),
    (14.0, pop(2, Swoop, Right, 5, true)),
    (15.5, Lancer { lane: 3, y: 92.0 }),
    (17.0, Gunship { lane: 4, y: 70.0 }),
    (18.5, Remoras { side: Right, y: 70.0, n: 8, fire: false }),
    (19.0, pop(5, Arc, Left, 4, true)),
    (20.5, Couriers { lane: 4, n: 4, fire: true }),
    (21.0, Lancer { lane: 2, y: 96.0 }),
    (22.0, Gunship { lane: 1, y: 62.0 }),
    (23.0, pop(4, Arc, Left, 4, true)),
    (24.0, Remoras { side: Left, y: 64.0, n: 10, fire: true }),
    (25.5, Lancer { lane: 1, y: 92.0 }),
    (25.5, Lancer { lane: 5, y: 92.0 }),
    (26.5, Gunship { lane: 3, y: 58.0 }),
    (27.5, pop(4, Cross, Left, 5, true)),
    (28.5, Wreck { lane: 2, what: 0 }),
    (29.5, Remoras { side: Right, y: 58.0, n: 10, fire: true }),
    (30.5, Gunship { lane: 5, y: 72.0 }),
    (31.0, pop(2, Cross, Right, 5, true)),
    (32.5, Lancer { lane: 2, y: 100.0 }),
    (32.5, Lancer { lane: 4, y: 100.0 }),
    (33.5, Gunship { lane: 1, y: 70.0 }),
    (34.5, Couriers { lane: 2, n: 4, fire: true }),
    (35.5, Remoras { side: Left, y: 76.0, n: 10, fire: false }),
    (36.5, Gunship { lane: 4, y: 64.0 }),
    (37.5, pop(5, Swoop, Left, 5, true)),
    (38.5, Lancer { lane: 3, y: 100.0 }),
    (39.0, pop(1, Arc, Right, 4, true)),
    // BUOYS: static fire joins the aimed; lancers flank them, schools cross between
    (40.0, Buoy { lane: 3, spiral: false }),
    (41.0, Lancer { lane: 1, y: 92.0 }),
    (41.0, Lancer { lane: 5, y: 92.0 }),
    (42.0, pop(2, Swoop, Right, 5, true)),
    (43.0, Buoy { lane: 1, spiral: true }),
    (44.0, Remoras { side: Right, y: 66.0, n: 10, fire: true }),
    (45.0, Gunship { lane: 4, y: 72.0 }),
    (46.0, Buoy { lane: 5, spiral: false }),
    (47.0, Usher { lane: 1, y: 118.0 }),
    (47.0, Usher { lane: 5, y: 118.0 }),
    (48.0, pop(2, Dive, Right, 4, true)),
    (49.0, Buoy { lane: 2, spiral: true }),
    (50.0, Wreck { lane: 4, what: 0 }),
    (50.5, Lancer { lane: 3, y: 100.0 }),
    (51.5, Remoras { side: Left, y: 56.0, n: 10, fire: true }),
    (52.5, Buoy { lane: 4, spiral: false }),
    (53.5, Gunship { lane: 1, y: 64.0 }),
    (54.0, pop(3, Cross, Left, 5, true)),
    (55.0, Buoy { lane: 3, spiral: true }),
    (56.0, Lancer { lane: 2, y: 96.0 }),
    (56.0, Lancer { lane: 4, y: 96.0 }),
    (57.0, Remoras { side: Right, y: 74.0, n: 10, fire: false }),
    (58.0, Buoy { lane: 1, spiral: false }),
    (58.5, Usher { lane: 2, y: 110.0 }),
    (58.5, Usher { lane: 4, y: 110.0 }),
    (59.5, Gunship { lane: 5, y: 60.0 }),
    (60.0, Wreck { lane: 2, what: 1 }),
    (61.0, Buoy { lane: 5, spiral: true }),
    (62.0, Remoras { side: Left, y: 62.0, n: 12, fire: true }),
    (63.0, Couriers { lane: 3, n: 4, fire: true }),
    (64.0, Lancer { lane: 1, y: 94.0 }),
    (64.0, Lancer { lane: 5, y: 94.0 }),
    (65.0, Gunship { lane: 3, y: 66.0 }),
    (66.0, pop(5, Arc, Left, 4, true)),
    (67.5, pop(2, Swoop, Right, 5, false)),
    // BREATHER: nothing fires; scenery drifts down to shoot, and harmless flights
    (71.0, Wreck { lane: 2, what: 0 }),
    (72.0, Wreck { lane: 4, what: 1 }),
    (73.0, Wreck { lane: 3, what: 2 }),
    (74.5, Wreck { lane: 1, what: 0 }),
    (74.5, Wreck { lane: 5, what: 0 }),
    (76.0, pop(2, Swoop, Right, 6, false)),
    (79.0, pop(4, Swoop, Left, 6, false)),
    (80.5, Remoras { side: Left, y: 70.0, n: 10, fire: false }),
    // DERELICT
    (85.0, Derelict),
    // HEAVIES: one at a time, through swarms that rise to an overrun: schools from
    // both sides, lancers three and four at once
    (115.0, Heavy { lane: 2, y: 80.0 }),
    (116.0, pop(4, Swoop, Left, 6, true)),
    (117.5, Remoras { side: Right, y: 60.0, n: 10, fire: true }),
    (119.0, Lancer { lane: 1, y: 100.0 }),
    (119.0, Lancer { lane: 5, y: 100.0 }),
    (120.5, Couriers { lane: 4, n: 4, fire: true }),
    (121.5, Gunship { lane: 4, y: 56.0 }),
    (123.5, Heavy { lane: 4, y: 74.0 }),
    (124.5, Remoras { side: Left, y: 66.0, n: 10, fire: true }),
    (125.5, Lancer { lane: 1, y: 100.0 }),
    (125.5, Lancer { lane: 3, y: 110.0 }),
    (125.5, Lancer { lane: 5, y: 100.0 }),
    (127.0, Usher { lane: 3, y: 124.0 }),
    (127.0, pop(5, Swoop, Left, 6, true)),
    (128.5, Remoras { side: Right, y: 74.0, n: 12, fire: true }),
    (130.0, Buoy { lane: 2, spiral: true }),
    (132.0, Heavy { lane: 3, y: 70.0 }),
    (133.0, Remoras { side: Left, y: 56.0, n: 12, fire: true }),
    (133.0, Remoras { side: Right, y: 86.0, n: 12, fire: false }),
    (134.5, Lancer { lane: 1, y: 100.0 }),
    (134.5, Lancer { lane: 5, y: 100.0 }),
    (135.5, Gunship { lane: 1, y: 58.0 }),
    (136.5, pop(4, Dive, Right, 4, true)),
    (137.5, Lancer { lane: 2, y: 104.0 }),
    (137.5, Lancer { lane: 3, y: 112.0 }),
    (137.5, Lancer { lane: 4, y: 104.0 }),
    (138.5, Remoras { side: Right, y: 64.0, n: 14, fire: true }),
    (140.5, Heavy { lane: 2, y: 78.0 }),
    (141.5, Remoras { side: Left, y: 70.0, n: 14, fire: true }),
    (142.5, Lancer { lane: 1, y: 98.0 }),
    (142.5, Lancer { lane: 2, y: 108.0 }),
    (142.5, Lancer { lane: 4, y: 108.0 }),
    (142.5, Lancer { lane: 5, y: 98.0 }),
    (143.5, Usher { lane: 1, y: 116.0 }),
    (143.5, Usher { lane: 5, y: 116.0 }),
    (144.5, Remoras { side: Right, y: 58.0, n: 14, fire: true }),
    (144.5, Remoras { side: Left, y: 86.0, n: 14, fire: false }),
    (146.0, Lancer { lane: 1, y: 100.0 }),
    (146.0, Lancer { lane: 3, y: 110.0 }),
    (146.0, Lancer { lane: 5, y: 100.0 }),
    (147.0, Remoras { side: Left, y: 62.0, n: 14, fire: true }),
    (147.5, pop(2, Swoop, Right, 6, false)),
    // WARNING: calm
];

/// Frames, from seconds.
pub fn frames(secs: f32) -> u32 {
    (secs * TICK_HZ as f32).round() as u32
}

/// The center of lane `lane` (0-6) in playfield px.
pub fn lane_x(lane: u8) -> f32 {
    (lane as f32 + 0.5) * FIELD_W as f32 / LANES as f32
}

/// The section of the stage at frame `t`.
pub fn section_at(t: u32) -> usize {
    SECTIONS.iter().rposition(|s| frames(s.0) <= t).unwrap_or(0)
}

/// A popcorn flight's spline from lane x `lx`, bending toward `s` (+1 right, -1 left).
fn flight(path: Flight, lx: f32, s: f32) -> Spline {
    let p = |dx: f32, y: f32| (lx + dx * s, y);
    match path {
        Dive => Spline::new(&[p(0.0, -12.0), p(14.0, 110.0), p(-14.0, 230.0), p(0.0, 380.0)], DRONE_SPEED),
        Swoop => Spline::new(&[p(0.0, -12.0), p(0.0, 60.0), p(48.0, 118.0), p(128.0, 122.0), p(200.0, 70.0), p(235.0, -30.0)], DRONE_SPEED),
        Arc => Spline::new(&[p(0.0, -12.0), p(0.0, 90.0), p(40.0, 140.0), p(85.0, 110.0), p(93.0, -30.0)], DRONE_SPEED),
        Cross => Spline::new(&[p(0.0, -12.0), p(93.0, 140.0), p(213.0, 260.0), p(293.0, 400.0)], DRONE_SPEED),
    }
}

/// A remora's swim across from `side` at height `y`: in from beyond the edge, a long
/// wave across the screen, and out the other side.
fn school(side: Side, y: f32) -> Spline {
    let w = FIELD_W as f32;
    let x = |d: f32| if side == Left { d } else { w - d };
    Spline::new(&[(x(-14.0), y), (x(60.0), y + 24.0), (x(150.0), y - 16.0), (x(240.0), y + 24.0), (x(320.0), y - 8.0), (x(w + 30.0), y)], REMORA_SPEED)
}

/// Where an arrival comes in from beyond a side of the screen, if it does: that side,
/// and the height.
pub fn side_entry(a: &Arrival) -> Option<(Side, f32)> {
    let (x, y) = a.path.pos(0);
    if x < 0.0 {
        Some((Left, y))
    } else if x > FIELD_W as f32 {
        Some((Right, y))
    } else {
        None
    }
}

/// One enemy's arrival.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Arrival {
    pub frame: u32,
    pub kind: EnemyKind,
    pub variant: u8,
    pub path: Path,
}

/// The script expanded into single arrivals, in time order.
pub fn arrivals() -> Vec<Arrival> {
    let mut out = vec![];
    for &(at, cue) in SCRIPT {
        let f = frames(at);
        match cue {
            Drones { lane, path, side, n, gap, fire } => {
                let s = if side == Right { 1.0 } else { -1.0 };
                for i in 0..n {
                    let path = Path::Spline(flight(path, lane_x(lane), s));
                    out.push(Arrival { frame: f + i as u32 * gap as u32, kind: EnemyKind::Drone, variant: fire as u8, path });
                }
            }
            Couriers { lane, n, fire } => {
                let lx = lane_x(lane);
                let s = if lane < 3 { 1.0 } else { -1.0 };
                for i in 0..n {
                    let path = Path::Spline(Spline::new(&[(lx, -14.0), (lx + 10.0 * s, 140.0), (lx - 6.0 * s, 260.0), (lx, 390.0)], COURIER_SPEED));
                    out.push(Arrival { frame: f + i as u32 * COURIER_GAP, kind: EnemyKind::Courier, variant: fire as u8, path });
                }
            }
            Usher { lane, y } => {
                let x = lane_x(lane);
                let path = Path::Hover { from: (x, -20.0), at: (x, y), enter: USHER_ENTER, stay: USHER_STAY, sway: 0.0 };
                out.push(Arrival { frame: f, kind: EnemyKind::Usher, variant: 0, path });
            }
            Gunship { lane, y } => {
                let x = lane_x(lane);
                let path = Path::Hover { from: (x, -24.0), at: (x, y), enter: ENTER_FRAMES, stay: GUNSHIP_STAY, sway: GUNSHIP_SWAY };
                out.push(Arrival { frame: f, kind: EnemyKind::Gunship, variant: 0, path });
            }
            Buoy { lane, spiral } => {
                let path = Path::Drift { x: lane_x(lane), y: -14.0, vy: BUOY_DRIFT, sway: BUOY_SWAY };
                out.push(Arrival { frame: f, kind: EnemyKind::Buoy, variant: spiral as u8, path });
            }
            Heavy { lane, y } => {
                let x = lane_x(lane);
                let path = Path::Hover { from: (x, -30.0), at: (x, y), enter: ENTER_FRAMES, stay: HEAVY_STAY, sway: HEAVY_SWAY };
                out.push(Arrival { frame: f, kind: EnemyKind::Heavy, variant: 0, path });
            }
            Derelict => {
                let path = Path::Hover { from: (HOLE_X, HOLE_Y), at: (HOLE_X, HULL_Y), enter: HULL_EMERGE, stay: HULL_STAY - HULL_EMERGE, sway: HULL_SWAY };
                out.push(Arrival { frame: f, kind: EnemyKind::Hull, variant: 0, path });
            }
            Lancer { lane, y } => {
                let x = lane_x(lane);
                let (enter, wait, speed) = LANCER_RUSH;
                let path = Path::Rush { from: (x, -16.0), at: (x, y), enter, wait, speed, dir: (0.0, 1.0) };
                out.push(Arrival { frame: f, kind: EnemyKind::Lancer, variant: 0, path });
            }
            Remoras { side, y, n, fire } => {
                for i in 0..n {
                    // a loose school: each a little above or below the one before
                    let yy = y + [0.0, 5.0, -4.0, 8.0, -7.0, 3.0][i as usize % 6];
                    out.push(Arrival { frame: f + i as u32 * REMORA_GAP, kind: EnemyKind::Remora, variant: (fire && i % 2 == 0) as u8, path: Path::Spline(school(side, yy)) });
                }
            }
            Wreck { lane, what } => {
                let kind = [EnemyKind::Debris, EnemyKind::Satellite, EnemyKind::Monolith][what as usize % 3];
                let path = Path::Drift { x: lane_x(lane), y: -20.0, vy: WRECK_DRIFT, sway: WRECK_SWAY };
                out.push(Arrival { frame: f, kind, variant: 0, path });
            }
        }
    }
    out.sort_by_key(|a| a.frame);
    out
}

/// The script's fairness rules, broken: nothing may appear within `SPAWN_EDGE` px of a
/// side (coming in from beyond one is fine, in the upper half: it's warned at that
/// edge first), and high-HP enemies arrive at least `HEAVY_GAP` seconds apart.
pub fn unfair(queue: &[Arrival]) -> Option<String> {
    let mut last_heavy: Option<u32> = None;
    for a in queue {
        let (x, _) = a.path.pos(0);
        if let Some((_, y)) = side_entry(a) {
            if y > FIELD_H as f32 * 0.5 {
                return Some(format!("{:?} comes in from a side {y} px down, in the lower half", a.kind));
            }
        } else if x < SPAWN_EDGE || x > FIELD_W as f32 - SPAWN_EDGE {
            return Some(format!("{:?} arrives {x} px across, too close to the edge", a.kind));
        }
        if a.kind.heavy() {
            if let Some(t) = last_heavy.filter(|&t| a.frame - t < frames(HEAVY_GAP)) {
                return Some(format!("two heavies at frames {t} and {}", a.frame));
            }
            last_heavy = Some(a.frame);
        }
    }
    None
}

/// How many sections the stage has.
pub const N: usize = SECTIONS.len();

/// Where the stage is: its clock, who's still to come, and the tally.
#[derive(Clone)]
pub struct Stage {
    /// Frames since the stage began (it can jump ahead when the derelict dies early).
    pub t: u32,
    queue: Vec<Arrival>,
    next: usize,
    pub section: usize,
    /// When each section actually began, and deaths in each.
    pub began: [Option<u32>; N],
    pub deaths: [u32; N],
    pub spawned: u32,
    pub killed: u32,
    /// The boss has been called up (it comes once).
    pub boss_called: bool,
    /// Reached the end: the boss is gone.
    pub over: bool,
}

impl Stage {
    pub fn new() -> Stage {
        let mut began = [None; N];
        began[0] = Some(0);
        let queue = arrivals();
        debug_assert_eq!(unfair(&queue), None);
        Stage { t: 0, queue, next: 0, section: 0, began, deaths: [0; N], spawned: 0, killed: 0, boss_called: false, over: false }
    }

    /// Who's coming in from beyond a side within `within` frames: the side, the height,
    /// and frames to go (the warnings at the edges).
    pub fn incoming(&self, within: u32) -> impl Iterator<Item = (Side, f32, u32)> + '_ {
        let t = self.t;
        self.queue[self.next..].iter().take_while(move |a| a.frame <= t + within).filter_map(move |a| side_entry(a).map(|(s, y)| (s, y, a.frame - t)))
    }

    /// Enemies may fire (not in the opening seconds).
    pub fn fire_ok(&self) -> bool {
        self.t >= frames(FIRST_FIRE)
    }

    /// Jump so the next tick is frame `t`, skipping everyone due before it.
    pub fn skip_to(&mut self, t: u32) {
        self.t = t.saturating_sub(1).max(self.t);
        self.next = self.queue.iter().position(|a| a.frame > self.t).unwrap_or(self.queue.len());
    }

    /// The derelict went down early: bring the next section forward, to at most
    /// `SECTION_EARLY` seconds ahead of the timeline.
    pub fn hurry(&mut self) {
        if let Some(next) = SECTIONS.get(self.section + 1) {
            let at = frames(next.0 - SECTION_EARLY);
            if self.t + 1 < at {
                self.skip_to(at);
            }
        }
    }

    /// One frame of the timeline: arrivals due now, a new section, the warning.
    pub fn tick(&mut self, foes: &mut Foes, events: &mut Vec<Event>) {
        self.t += 1;
        while let Some(a) = self.queue.get(self.next).filter(|a| a.frame <= self.t) {
            // in a firing flight, only some of the popcorn fire (which ones changes every run)
            let quiet = matches!(a.kind, EnemyKind::Drone | EnemyKind::Courier) && a.variant == 1 && !foes.rng.chance(DRONE_FIRE_SHARE);
            foes.add(a.kind, if quiet { 0 } else { a.variant }, a.path);
            // scenery isn't an enemy: it doesn't count toward the stage's kills
            self.spawned += !a.kind.scenery() as u32;
            self.next += 1;
        }
        let s = section_at(self.t);
        if s != self.section {
            self.section = s;
            self.began[s] = Some(self.t);
            events.push(Event::Section(s as u8));
            if s == WARNING_SECTION {
                events.push(Event::Warning);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_spawns_within_20_px_of_an_edge_or_in_an_edge_lane() {
        for a in arrivals() {
            let (x, y) = a.path.pos(0);
            match side_entry(&a) {
                // a school from beyond a side: in the upper half, and only remoras
                Some(_) => assert!(a.kind == EnemyKind::Remora && y <= FIELD_H as f32 * 0.5, "{a:?}"),
                None => assert!(x >= SPAWN_EDGE && x <= FIELD_W as f32 - SPAWN_EDGE, "{a:?}"),
            }
        }
        assert_eq!(unfair(&arrivals()), None);
        let mut edge = arrivals();
        edge[0].path = Path::Spline(Spline::new(&[(12.0, -10.0), (12.0, 100.0)], 1.0));
        assert!(unfair(&edge).is_some(), "the check catches a spawn on the edge");
        let mut low = arrivals();
        low[0].path = Path::Spline(Spline::new(&[(-14.0, 300.0), (100.0, 300.0)], 1.0));
        assert!(unfair(&low).is_some(), "the check catches a side entry in the lower half");
        for (_, cue) in SCRIPT {
            let lane = match *cue {
                Drones { lane, .. } | Couriers { lane, .. } | Usher { lane, .. } | Gunship { lane, .. } | Buoy { lane, .. } | Heavy { lane, .. } | Lancer { lane, .. } | Wreck { lane, .. } => lane,
                Derelict | Remoras { .. } => 3,
            };
            assert!((1..LANES as u8 - 1).contains(&lane), "{cue:?}");
        }
    }

    /// Every school coming in from beyond a side is warned at that edge a full
    /// `SIDE_WARNING` frames before it arrives.
    #[test]
    fn schools_from_the_sides_are_warned_two_seconds_ahead() {
        let side: Vec<Arrival> = arrivals().into_iter().filter(|a| side_entry(a).is_some()).collect();
        assert!(side.len() >= 100, "{} remoras", side.len());
        assert!(SIDE_WARNING >= 120, "the guide's 2-3 seconds");
        let mut s = Stage::new();
        let mut foes = Foes::default();
        let mut ev = vec![];
        let mut warned = std::collections::HashSet::new();
        let last = side.last().unwrap().frame;
        while s.t < last {
            s.tick(&mut foes, &mut ev);
            foes.list.clear();
            for (side, y, left) in s.incoming(SIDE_WARNING) {
                assert!(left >= 1 && left <= SIDE_WARNING);
                if left == SIDE_WARNING {
                    warned.insert((s.t + left, side == Left, y.to_bits()));
                }
            }
        }
        for a in &side {
            let (sd, y) = side_entry(a).unwrap();
            assert!(warned.contains(&(a.frame, sd == Left, y.to_bits())), "{a:?} came in unwarned");
        }
    }

    /// The Swarm: lots of fodder, rising to an overrun. The last fighting section
    /// sends the most fodder a second, it rises through that section, and the most
    /// crowded 10 seconds of the stage end in its last 5; the gunships are fewer than
    /// they were.
    #[test]
    fn the_swarm_rises_to_an_overrun() {
        let all = arrivals();
        let fodder = |a: f32, b: f32| all.iter().filter(|x| x.kind.fodder() && (frames(a)..frames(b)).contains(&x.frame)).count();
        let rate = |s: usize| fodder(SECTIONS[s].0, SECTIONS[s + 1].0) as f32 / (SECTIONS[s + 1].0 - SECTIONS[s].0);
        let heavies = rate(5);
        for s in [0, 1, 2, 3] {
            assert!(heavies > rate(s) * 1.25, "{}: {} a second, the overrun {heavies}", SECTIONS[s].1, rate(s));
        }
        let (h0, h1) = (SECTIONS[5].0, SECTIONS[6].0);
        let third = (h1 - h0) / 3.0;
        let thirds: Vec<usize> = (0..3).map(|k| fodder(h0 + third * k as f32, h0 + third * (k + 1) as f32)).collect();
        assert!(thirds.windows(2).all(|p| p[1] > p[0]), "rising through the section: {thirds:?}");
        let busiest = (0..=(h1 - 10.0) as u32).max_by_key(|&k| (fodder(k as f32, k as f32 + 10.0), k)).unwrap();
        assert!(busiest as f32 >= h1 - 15.0, "the busiest 10 s start at {busiest} s, not in the overrun");
        assert!(all.iter().filter(|a| a.kind == EnemyKind::Gunship).count() <= 14, "fewer gunships (there were 31)");
        assert!(all.iter().filter(|a| a.kind == EnemyKind::Lancer).count() >= 30);
        assert!(all.iter().filter(|a| a.kind.scenery()).count() >= 6);
    }

    #[test]
    fn no_two_heavies_spawn_within_2_s() {
        let heavy: Vec<u32> = arrivals().iter().filter(|a| a.kind.heavy()).map(|a| a.frame).collect();
        assert!(heavy.len() >= 4);
        for w in heavy.windows(2) {
            assert!(w[1] - w[0] >= frames(HEAVY_GAP), "{} and {}", w[0], w[1]);
        }
        let mut close = arrivals();
        let i = close.iter().position(|a| a.kind == EnemyKind::Heavy).unwrap();
        let mut twin = close[i];
        twin.frame += frames(HEAVY_GAP) - 1;
        close.insert(i + 1, twin);
        assert!(unfair(&close).is_some(), "the check catches two heavies too close");
    }

    #[test]
    fn the_script_follows_the_timeline() {
        assert!(SCRIPT.windows(2).all(|w| w[0].0 <= w[1].0), "in time order");
        assert_eq!(SECTIONS.len(), 8);
        assert!(SECTIONS[WARNING_SECTION].0 < JAWS_AT && JAWS_AT < SECTIONS[7].0, "the boss rises during the WARNING");
        // nothing that shoots before the first-fire time, nothing at all in the last calm
        for &(at, cue) in SCRIPT {
            if at < FIRST_FIRE {
                assert!(matches!(cue, Drones { fire: false, .. } | Couriers { fire: false, .. }), "{at}: {cue:?}");
            }
            assert!(at < SECTIONS[WARNING_SECTION].0, "{at}: the WARNING section is calm");
        }
        // the breather fires nothing (scenery to shoot, quiet flights); the derelict has
        // its section to itself
        for &(at, cue) in SCRIPT.iter().filter(|(at, _)| (SECTIONS[3].0..SECTIONS[5].0).contains(at)) {
            assert!(matches!(cue, Drones { fire: false, .. } | Couriers { fire: false, .. } | Remoras { fire: false, .. } | Wreck { .. } | Derelict), "{at}: {cue:?}");
            assert!(at < SECTIONS[4].0 || cue == Derelict, "{at}: {cue:?} in the derelict's section");
        }
        assert_eq!(SCRIPT.iter().filter(|(_, c)| *c == Derelict).count(), 1);
        assert_eq!(SCRIPT.iter().find(|(_, c)| *c == Derelict).unwrap().0, SECTIONS[4].0);
        // every section after the fly-in brings someone new in its first two seconds
        for s in 1..6 {
            if s == 3 {
                continue; // the breather
            }
            assert!(SCRIPT.iter().any(|(at, _)| (SECTIONS[s].0..SECTIONS[s].0 + 2.0).contains(at)), "{}", SECTIONS[s].1);
        }
    }

    #[test]
    fn the_stage_runs_its_clock_and_spawns_on_cue() {
        let mut s = Stage::new();
        let mut foes = Foes::default();
        let mut ev = vec![];
        let total = arrivals().iter().filter(|a| !a.kind.scenery()).count() as u32;
        for _ in 0..frames(SECTIONS[7].0) {
            s.tick(&mut foes, &mut ev);
            foes.list.clear();
        }
        assert_eq!(s.spawned, total);
        assert!(ev.contains(&Event::Warning));
        assert!(!s.over, "the stage only ends with the boss");
        let starts: Vec<u32> = s.began.iter().map(|b| b.unwrap()).collect();
        let planned: Vec<u32> = SECTIONS.iter().map(|x| frames(x.0)).collect();
        assert_eq!(starts, planned);
    }

    #[test]
    fn an_early_kill_brings_the_next_section_forward_but_not_far() {
        let mut s = Stage::new();
        let mut foes = Foes::default();
        let mut ev = vec![];
        s.skip_to(frames(SECTIONS[4].0));
        s.tick(&mut foes, &mut ev);
        assert_eq!(s.section, 4);
        assert!(foes.list.iter().any(|e| e.kind == EnemyKind::Hull));
        for _ in 0..frames(6.0) {
            s.tick(&mut foes, &mut ev);
        }
        s.hurry();
        s.tick(&mut foes, &mut ev);
        assert_eq!(s.t, frames(SECTIONS[5].0 - SECTION_EARLY));
        while s.section == 4 {
            s.tick(&mut foes, &mut ev);
        }
        let early = frames(SECTIONS[5].0) - s.began[5].unwrap();
        assert!(early <= frames(5.0), "within 5 s of the timeline");
    }
}
