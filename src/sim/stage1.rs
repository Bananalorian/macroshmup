//! Stage 1, OUTER DISK: the script, as data. Every cue is timed in seconds from the
//! start, on 7 lanes, and the two edge lanes are never used. The Toaplan pattern:
//! gunships alternate sides, popcorn fills in around them, waves overlap, and the
//! high-HP enemies come one at a time.
//!
//!   0:00  fly-in: the title, the first drones (no bullets for 6 s)
//!   0:10  gunships alternate sides, popcorn around them: aimed fire, side-to-side flow
//!   0:40  turret buoys join: static plus aimed makes lanes
//!   1:10  a breather: the disk flares, debris drifts by, a tiny figure tumbles past
//!   1:25  the derelict hull rises out of the disk (the mid-boss)
//!   1:55  heavies one at a time through swarms: the peak
//!   2:30  calm, WARNING
//!   2:40  the end of the stage (the boss arrives in v0.7)

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
    /// A gunship flies in to hover at height `y`.
    Gunship { lane: u8, y: f32 },
    /// A turret buoy drifts down: rings, or a spiral.
    Buoy { lane: u8, spiral: bool },
    /// A heavy flies in to hover at height `y`.
    Heavy { lane: u8, y: f32 },
    /// The derelict hull rises out of the disk.
    Derelict,
}

use Cue::*;
use Flight::*;
use Side::*;

const fn pop(lane: u8, path: Flight, side: Side, n: u8, fire: bool) -> Cue {
    Drones { lane, path, side, n, gap: 9, fire }
}

/// The whole stage. Sections start at `tune::SECTIONS`.
pub const SCRIPT: &[(f32, Cue)] = &[
    // FLY-IN: drones only; the first shots come after 6 s
    (1.5, pop(2, Swoop, Right, 5, false)),
    (3.5, pop(4, Swoop, Left, 5, false)),
    (5.5, pop(1, Arc, Right, 4, false)),
    (5.5, pop(5, Arc, Left, 4, false)),
    (7.0, pop(3, Dive, Right, 4, true)),
    (8.5, pop(2, Swoop, Right, 5, true)),
    // GUNSHIPS: alternating sides, popcorn around them
    (10.0, Gunship { lane: 1, y: 70.0 }),
    (11.0, pop(4, Swoop, Left, 5, true)),
    (12.5, Gunship { lane: 5, y: 64.0 }),
    (14.0, pop(2, Swoop, Right, 5, true)),
    (15.0, Gunship { lane: 2, y: 76.0 }),
    (16.5, pop(3, Dive, Left, 4, true)),
    (17.5, Gunship { lane: 4, y: 70.0 }),
    (19.0, pop(5, Arc, Left, 4, true)),
    (20.0, Gunship { lane: 1, y: 62.0 }),
    (21.0, pop(3, Swoop, Right, 6, false)),
    (22.0, Gunship { lane: 5, y: 72.0 }),
    (23.5, pop(1, Arc, Right, 4, true)),
    (24.0, Gunship { lane: 3, y: 58.0 }),
    (25.5, pop(4, Cross, Left, 5, true)),
    (26.5, Gunship { lane: 1, y: 66.0 }),
    (27.5, Gunship { lane: 5, y: 72.0 }),
    (29.0, pop(3, Dive, Right, 4, true)),
    (30.0, Gunship { lane: 2, y: 74.0 }),
    (31.0, pop(2, Cross, Right, 5, true)),
    (31.5, Gunship { lane: 4, y: 64.0 }),
    (33.0, pop(5, Swoop, Left, 5, true)),
    (33.5, Gunship { lane: 1, y: 70.0 }),
    (35.0, Gunship { lane: 5, y: 60.0 }),
    (35.5, pop(1, Swoop, Right, 5, true)),
    (36.5, Gunship { lane: 3, y: 68.0 }),
    (37.5, pop(4, Arc, Left, 4, true)),
    (38.0, Gunship { lane: 2, y: 62.0 }),
    (39.0, pop(3, Arc, Right, 4, true)),
    // BUOYS: static fire joins the aimed
    (40.0, Buoy { lane: 3, spiral: false }),
    (41.5, Gunship { lane: 5, y: 70.0 }),
    (42.0, pop(1, Swoop, Right, 5, true)),
    (43.0, Buoy { lane: 1, spiral: true }),
    (44.5, Gunship { lane: 1, y: 64.0 }),
    (45.0, pop(4, Arc, Left, 4, true)),
    (46.0, Buoy { lane: 5, spiral: false }),
    (47.5, Gunship { lane: 4, y: 72.0 }),
    (48.0, pop(2, Dive, Right, 4, true)),
    (49.0, Buoy { lane: 2, spiral: true }),
    (50.5, Gunship { lane: 2, y: 66.0 }),
    (51.0, pop(5, Swoop, Left, 5, true)),
    (52.0, Buoy { lane: 4, spiral: false }),
    (53.5, Gunship { lane: 5, y: 62.0 }),
    (54.0, pop(3, Cross, Left, 5, true)),
    (55.0, Buoy { lane: 3, spiral: true }),
    (56.5, Gunship { lane: 1, y: 70.0 }),
    (57.0, pop(2, Swoop, Right, 5, true)),
    (58.0, Buoy { lane: 1, spiral: false }),
    (59.5, Gunship { lane: 3, y: 60.0 }),
    (60.0, pop(4, Dive, Left, 4, true)),
    (61.0, Buoy { lane: 5, spiral: true }),
    (62.5, Gunship { lane: 4, y: 68.0 }),
    (63.5, pop(1, Arc, Right, 4, true)),
    (64.5, Gunship { lane: 2, y: 72.0 }),
    (65.5, pop(5, Arc, Left, 4, true)),
    (66.5, Gunship { lane: 5, y: 66.0 }),
    (67.5, pop(2, Swoop, Right, 5, false)),
    // BREATHER: nothing fires; a couple of harmless flights to shoot at
    (76.0, pop(2, Swoop, Right, 6, false)),
    (79.0, pop(4, Swoop, Left, 6, false)),
    // DERELICT
    (85.0, Derelict),
    // HEAVIES: one at a time, through swarms
    (115.0, Heavy { lane: 2, y: 80.0 }),
    (116.0, pop(4, Swoop, Left, 6, true)),
    (117.5, Gunship { lane: 5, y: 60.0 }),
    (118.5, pop(5, Dive, Left, 4, true)),
    (120.0, pop(1, Arc, Right, 5, true)),
    (121.5, Gunship { lane: 4, y: 56.0 }),
    (123.5, Heavy { lane: 4, y: 74.0 }),
    (124.5, pop(2, Cross, Right, 5, true)),
    (126.0, Gunship { lane: 1, y: 60.0 }),
    (127.0, pop(5, Swoop, Left, 6, true)),
    (128.5, Buoy { lane: 2, spiral: true }),
    (129.5, pop(3, Swoop, Left, 6, true)),
    (132.0, Heavy { lane: 3, y: 70.0 }),
    (133.0, pop(1, Swoop, Right, 5, true)),
    (133.0, pop(5, Swoop, Left, 5, true)),
    (135.0, Gunship { lane: 1, y: 58.0 }),
    (136.0, Buoy { lane: 5, spiral: false }),
    (137.5, pop(4, Dive, Right, 4, true)),
    (138.5, Gunship { lane: 4, y: 56.0 }),
    (140.5, Heavy { lane: 2, y: 78.0 }),
    (141.5, Gunship { lane: 5, y: 64.0 }),
    (142.5, pop(3, Arc, Left, 5, true)),
    (144.0, pop(4, Swoop, Left, 6, true)),
    (145.5, pop(2, Swoop, Right, 6, false)),
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
        Swoop => Spline::new(&[p(0.0, -12.0), p(0.0, 60.0), p(36.0, 118.0), p(96.0, 122.0), p(150.0, 70.0), p(176.0, -30.0)], DRONE_SPEED),
        Arc => Spline::new(&[p(0.0, -12.0), p(0.0, 90.0), p(30.0, 140.0), p(64.0, 110.0), p(70.0, -30.0)], DRONE_SPEED),
        Cross => Spline::new(&[p(0.0, -12.0), p(70.0, 140.0), p(160.0, 260.0), p(220.0, 400.0)], DRONE_SPEED),
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
        }
    }
    out.sort_by_key(|a| a.frame);
    out
}

/// The script's fairness rules, broken: nothing may arrive within `SPAWN_EDGE` px of a
/// side, and high-HP enemies arrive at least `HEAVY_GAP` seconds apart.
pub fn unfair(queue: &[Arrival]) -> Option<String> {
    let mut last_heavy: Option<u32> = None;
    for a in queue {
        let (x, _) = a.path.pos(0);
        if x < SPAWN_EDGE || x > FIELD_W as f32 - SPAWN_EDGE {
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

/// Where the stage is: its clock, who's still to come, and the tally.
#[derive(Clone)]
pub struct Stage {
    /// Frames since the stage began (it can jump ahead when the derelict dies early).
    pub t: u32,
    queue: Vec<Arrival>,
    next: usize,
    pub section: usize,
    /// When each section actually began, and deaths in each.
    pub began: [Option<u32>; 7],
    pub deaths: [u32; 7],
    pub spawned: u32,
    pub killed: u32,
    /// Reached the end.
    pub over: bool,
}

impl Stage {
    pub fn new() -> Stage {
        let mut began = [None; 7];
        began[0] = Some(0);
        let queue = arrivals();
        debug_assert_eq!(unfair(&queue), None);
        Stage { t: 0, queue, next: 0, section: 0, began, deaths: [0; 7], spawned: 0, killed: 0, over: false }
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

    /// One frame of the timeline: arrivals due now, a new section, the warning, the end.
    pub fn tick(&mut self, foes: &mut Foes, events: &mut Vec<Event>) {
        self.t += 1;
        while let Some(a) = self.queue.get(self.next).filter(|a| a.frame <= self.t) {
            // in a firing flight, only some of the popcorn fire (which ones changes every run)
            let quiet = a.kind == EnemyKind::Drone && a.variant == 1 && !foes.rng.chance(DRONE_FIRE_SHARE);
            foes.add(a.kind, if quiet { 0 } else { a.variant }, a.path);
            self.spawned += 1;
            self.next += 1;
        }
        let s = section_at(self.t);
        if s != self.section {
            self.section = s;
            self.began[s] = Some(self.t);
            events.push(Event::Section(s as u8));
            if s == SECTIONS.len() - 1 {
                events.push(Event::Warning);
            }
        }
        if !self.over && self.t >= frames(STAGE_END) {
            self.over = true;
            events.push(Event::StageEnd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_spawns_within_20_px_of_an_edge_or_in_an_edge_lane() {
        for a in arrivals() {
            let (x, _) = a.path.pos(0);
            assert!(x >= SPAWN_EDGE && x <= FIELD_W as f32 - SPAWN_EDGE, "{a:?}");
        }
        assert_eq!(unfair(&arrivals()), None);
        let mut edge = arrivals();
        edge[0].path = Path::Spline(Spline::new(&[(12.0, -10.0), (12.0, 100.0)], 1.0));
        assert!(unfair(&edge).is_some(), "the check catches a spawn on the edge");
        for (_, cue) in SCRIPT {
            let lane = match *cue {
                Drones { lane, .. } | Gunship { lane, .. } | Buoy { lane, .. } | Heavy { lane, .. } => lane,
                Derelict => 3,
            };
            assert!((1..LANES as u8 - 1).contains(&lane), "{cue:?}");
        }
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
        assert_eq!(SECTIONS.len(), 7);
        // nothing that shoots before the first-fire time, nothing at all in the last calm
        for &(at, cue) in SCRIPT {
            if at < FIRST_FIRE {
                assert!(matches!(cue, Drones { fire: false, .. }), "{at}: {cue:?}");
            }
            assert!(at < SECTIONS[6].0, "{at}: the WARNING section is calm");
        }
        // the breather fires nothing; the derelict has its section to itself
        for &(at, cue) in SCRIPT.iter().filter(|(at, _)| (SECTIONS[3].0..SECTIONS[5].0).contains(at)) {
            assert!(matches!(cue, Drones { fire: false, .. } | Derelict), "{at}: {cue:?}");
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
    fn the_stage_runs_its_clock_spawns_on_cue_and_ends() {
        let mut s = Stage::new();
        let mut foes = Foes::default();
        let mut ev = vec![];
        let total = arrivals().len() as u32;
        for _ in 0..frames(STAGE_END) {
            s.tick(&mut foes, &mut ev);
            foes.list.clear();
        }
        assert_eq!(s.spawned, total);
        assert!(s.over && ev.contains(&Event::StageEnd) && ev.contains(&Event::Warning));
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
