//! Every tunable number in one place. Gameplay timings are in frames of the
//! fixed 60 Hz simulation, distances in low-res canvas pixels, speeds in px per
//! frame, angles in degrees (0 is right, 90 is straight down the screen).

use crate::sim::boss::{BossGun, Muzzle, bgun, enraged};
use crate::sim::bullets::{Behavior, Hue, Kind};
use crate::sim::enemy::{Arm, arm};
use crate::sim::pattern::{Gun, Mount, Preset, Shape};

// ------------------------------------------------------------------ the clock

/// Simulation ticks per second. Every gameplay timing counts these.
pub const TICK_HZ: u32 = 60;
/// After a stall (a dragged window, a background tab) catch up at most this many
/// ticks in one display frame, then drop the backlog instead of fast-forwarding.
pub const MAX_TICKS_PER_FRAME: u32 = 4;
/// A display frame longer than this (seconds) counts as this long.
pub const MAX_FRAME_DT: f64 = 0.25;
/// Display frames within this many seconds of a whole number of ticks count as
/// exactly that many, so vsync jitter on a 60 Hz screen never gives 0 or 2 ticks.
pub const VSYNC_SNAP: f64 = 0.0006;

// ------------------------------------------------------------------ the canvas

/// Landscape: the whole low-res canvas, with the playfield in the middle.
pub const CANVAS_W: u32 = 640;
pub const CANVAS_H: u32 = 360;
/// The playfield is the same size (square, full height) in every layout, so the game
/// plays the same everywhere. At 1080p it's 1080 x 1080 screen pixels.
pub const FIELD_W: u32 = 360;
pub const FIELD_H: u32 = 360;
/// Portrait (phones): a HUD strip above the playfield and a thumb zone below it.
pub const PORTRAIT_W: u32 = 360;
pub const PORTRAIT_H: u32 = 480;
pub const PORTRAIT_HUD_H: u32 = 24;

// ------------------------------------------------------------------ the Pilgrim
// Defaults for the live tuning panel (pause menu); players' changes are saved.

/// Full speed, px per frame (240 px/s: crosses the playfield in about 1.1 s).
pub const MOVE_SPEED: f32 = 4.0;
/// Focused speed as a share of full speed.
pub const FOCUS_RATIO: f32 = 0.55;
/// Frames to blend between full and focused speed.
pub const FOCUS_BLEND: u32 = 5;
/// The focus ratios the tuning panel steps through (Sprint 1 A/B test).
pub const FOCUS_PRESETS: [f32; 6] = [0.45, 0.50, 0.55, 0.60, 0.67, 0.75];
/// Tuning panel limits and steps.
pub const SPEED_RANGE: (f32, f32, f32) = (2.5, 6.0, 0.25);
pub const BLEND_MAX: u32 = 15;

/// The hitbox: a circle this many px in radius, centered on the cross.
pub const HITBOX_R: f32 = 2.0;
/// How close the hitbox center may get to each playfield edge.
pub const EDGE_X: f32 = 13.0;
pub const EDGE_TOP: f32 = 14.0;
pub const EDGE_BOTTOM: f32 = 22.0;
/// Where a new run starts, in playfield px.
pub const START_X: f32 = 180.0;
pub const START_Y: f32 = 300.0;

/// The pod streams leave from the antennas' tips: this far either side of the
/// center, closer as the blades close (v0.8.6: where the Pilgrim's antennas are
/// drawn), and this far above the hitbox.
pub const POD_X: f32 = 9.0;
pub const POD_X_FOCUS: f32 = 6.0;
pub const POD_GUN_Y: f32 = -20.0;
/// Frames between bank steps (the ship rolls -2..2 as it moves sideways).
pub const BANK_FRAMES: u32 = 3;

// ------------------------------------------------------------------ shots
// "Fast, long, dense and a little messy." Shots start low, at the ship, so an
// enemy you're sitting on still gets hit.

/// Frames between volleys of the wide shot.
pub const FIRE_INTERVAL: u32 = 3;
/// The twin center bolts.
pub const BOLT_SPEED: f32 = 14.0;
pub const BOLT_LEN: f32 = 18.0;
pub const BOLT_HIT_W: f32 = 8.0;
pub const BOLT_GAP: f32 = 3.0;
pub const BOLT_DMG: f32 = 3.0;
/// The pod streams, angled outward (alternating between two angles, in degrees;
/// from the antennas since v0.8.6, opened up so they cover what they did at range).
pub const POD_SPEED: f32 = 13.0;
pub const POD_LEN: f32 = 14.0;
pub const POD_HIT_W: f32 = 6.0;
pub const POD_ANGLES: [f32; 2] = [6.0, 11.5];
pub const POD_DMG: f32 = 1.5;
/// Where shots start, relative to the hitbox (positive is below it).
pub const MUZZLE_Y: f32 = 6.0;
/// The focus beam: width, and damage per frame (about 1.6x the wide shot's
/// single-target damage from the two center bolts).
pub const BEAM_W: f32 = 12.0;
pub const BEAM_DMG: f32 = 1.6 * 2.0 * BOLT_DMG / FIRE_INTERVAL as f32;
/// The beam's point-blank aura: extra damage per frame to anything this close.
pub const AURA_R: f32 = 28.0;
pub const AURA_DMG: f32 = 1.0;

// ------------------------------------------------------------------ target dummies

pub const DUMMY_RESPAWN: u32 = 90;
pub const DUMMY_HP: [f32; 3] = [120.0, 80.0, 1500.0]; // still, moving, big
/// The moving dummy swings this far either side, once per this many frames.
pub const DUMMY_SWING: (f32, u32) = (107.0, 300);
/// Hit flash: frames lit, and the minimum gap between flashes.
pub const HIT_FLASH: (u8, u8) = (2, 6);
/// Bumping a dummy flashes the hull (no damage yet); frames of flash and cooldown.
pub const BUMP_FLASH: u32 = 12;
pub const BUMP_COOLDOWN: u32 = 40;

// ------------------------------------------------------------------ enemy bullets

/// Pool capacity (never reallocated during play).
pub const BULLET_CAP: usize = 8192;
/// Sprite radius and, for rice, needles and diamonds, the half-length of the core
/// along the flight direction. Indexed by `Kind` (small, orb, large, rice, needle,
/// star, diamond). A star's arms reach past its radius: only its middle hurts.
pub const BULLET_SIZE: [(f32, f32); 7] = [(3.5, 0.0), (5.5, 0.0), (9.5, 0.0), (2.5, 2.0), (1.5, 5.5), (5.0, 0.0), (3.2, 3.0)];
/// Frames a star takes to turn one step (an eighth of half a turn) as it drifts.
pub const STAR_SPIN: u16 = 5;
/// The bullet's hitbox is this share of its sprite radius (leans in the player's favor).
pub const BULLET_HIT: f32 = 0.55;
/// New bullets bloom harmlessly for this many frames.
pub const BLOOM_FRAMES: u16 = 4;
/// No bullet may appear this close to the player.
pub const SAFE_SPAWN: f32 = 24.0;
/// Bullets this far outside the playfield are gone; none lives longer than this.
pub const BULLET_MARGIN: f32 = 16.0;
pub const BULLET_MAX_AGE: u16 = 900;
/// Grazing: a bullet's core passing within this many px of the hitbox (once per bullet).
pub const GRAZE_R: f32 = 14.0;
/// Red muzzle flash (telegraph) frames on an emitter after it fires.
pub const MUZZLE_FRAMES: u32 = 4;
/// At most 3 full-screen flashes a second.
pub const FLASH_GAP: u64 = 20;
pub const FLASH_FRAMES: u32 = 8;

// ------------------------------------------------------------------ lives, death and the bomb

/// Ships per credit (including the one you're flying) and bombs per life.
pub const LIVES: u8 = 3;
pub const BOMBS: u8 = 3;
/// The deathbomb: bombing within this many frames of a hit still saves you.
pub const DEATHBOMB: u32 = 8;
/// After a hit: the screen's bullets cancel this many frames later, and the
/// next ship starts flying in this many frames later.
pub const DEATH_CANCEL: u32 = 30;
pub const RESPAWN_AT: u32 = 40;
/// The new ship flies up from below the screen for this many frames (no control
/// yet), and can't be hit for this many frames from the moment it appears.
pub const RESPAWN_FLY: u32 = 36;
pub const RESPAWN_INVULN: u32 = 210;
/// The bomb: frames of invincibility; the cancel ring reaches `BOMB_REACH` px in
/// `BOMB_GROW` frames (enough to cover the playfield from anywhere) and keeps
/// cancelling for `BOMB_ACTIVE` frames.
pub const BOMB_INVULN: u32 = 180;
pub const BOMB_GROW: u32 = 24;
pub const BOMB_REACH: f32 = 460.0;
pub const BOMB_ACTIVE: u32 = 70;
/// Damage to everything the ring passes over; big targets (bosses) take at most
/// this share of their health from one bomb, so it's never a one-shot.
pub const BOMB_DAMAGE: f32 = 600.0;
pub const BOMB_BOSS_CAP: f32 = 0.4;
/// The brief slow-motion dip: this many ticks run at half speed.
pub const BOMB_DIP: u32 = 24;
/// The RGB surge: aberration held at this many px for this many frames, then fading.
pub const BOMB_SURGE: (f32, u32) = (4.5, 20);
/// The death's RGB split: gentler than the bomb's, so the shattering hull reads.
pub const DEATH_SURGE: (f32, u32) = (2.0, 6);
/// Shake when the bomb's ring has swept the screen clean.
pub const SHAKE_BOMB: f32 = 1.0;

// ------------------------------------------------------------------ light and particles

/// The particle pool (cosmetic only, never allocates during play).
pub const PARTICLE_CAP: usize = 8000;
/// Chromatic aberration rests at zero, spikes with explosions, and decays by this
/// factor each frame (to under 5% in 18 frames, about 0.3 s).
pub const CA_DECAY: f32 = 0.84;
/// Explosions this far from the Pilgrim (px) add no aberration; closer ones add more.
pub const CA_RANGE: f32 = 320.0;
/// "Reduce flashing" scales aberration spikes by this (and turns off screen flashes).
pub const REDUCED_CA: f32 = 0.35;
/// Shock rings: at most this many at once (Shinjuku's ring distortion).
pub const SHOCK_MAX: usize = 4;
/// Screen shake: px at full strength, decay per frame, and the "low" setting's share.
pub const SHAKE_PX: f32 = 4.0;
pub const SHAKE_DECAY: f32 = 0.85;
pub const SHAKE_LOW: f32 = 0.5;
/// Shake for the placeholder death (after the screen's been cancelled).
pub const SHAKE_DEATH: f32 = 0.9;
/// macroquad batches draws into buffers of this many vertices and indices (4096
/// quads), and creates a new GPU buffer whenever a frame needs more batches than
/// ever before, which stalls about a frame. So batches are big, and the loading
/// frame draws enough to create this many up front.
pub const DRAW_CALL_VERTICES: usize = 16384;
pub const DRAW_CALL_INDICES: usize = 24576;
pub const PREWARM_DRAW_CALLS: usize = 40;
/// The post pass: bloom strength, vignette, film grain.
pub const BLOOM: f32 = 0.14;
pub const VIGNETTE: f32 = 0.24;
pub const GRAIN: f32 = 0.032;

/// One explosion size. Each is much bigger than what died.
#[derive(Clone, Copy, Debug)]
pub struct Blast {
    pub sparks: u16,
    pub embers: u16,
    pub shards: u16,
    pub streaks: u16,
    pub dust: u16,
    pub rings: u8,
    /// How fast the debris flies (px per frame).
    pub speed: f32,
    /// The shock ring: how far it spreads (px), how hard it pushes (px), how long (frames).
    pub shock: (f32, f32, u32),
    /// Chromatic aberration spike (px) when it goes off right beside the Pilgrim.
    pub ca: f32,
    /// A full-screen flash (limited to 3 a second, off with Reduce flashing).
    pub flash: bool,
}

/// Small, medium, large, boss.
pub const BLASTS: [Blast; 4] = [
    Blast { sparks: 14, embers: 6, shards: 10, streaks: 2, dust: 6, rings: 1, speed: 2.6, shock: (34.0, 1.5, 16), ca: 0.8, flash: false },
    Blast { sparks: 30, embers: 14, shards: 22, streaks: 6, dust: 14, rings: 1, speed: 3.4, shock: (64.0, 2.5, 22), ca: 1.6, flash: false },
    Blast { sparks: 60, embers: 26, shards: 40, streaks: 14, dust: 30, rings: 2, speed: 4.4, shock: (110.0, 3.5, 28), ca: 2.6, flash: true },
    Blast { sparks: 120, embers: 50, shards: 80, streaks: 30, dust: 60, rings: 3, speed: 5.6, shock: (190.0, 5.0, 38), ca: 3.6, flash: true },
];

/// Hit-stop: frames the game holds still after a kill of each blast size (fodder
/// never stops the flow), and after a boss phase breaks.
pub const HITSTOP: [u32; 4] = [0, 2, 4, 6];
pub const HITSTOP_BOSS: u32 = 6;
/// A kill's impact frame: what died lit bone-bright for this many frames (by blast size).
pub const IMPACT: [u8; 4] = [2, 3, 4, 4];

/// The stress presets also keep this many particles alive, with a blast every so often.
pub const FX_STRESS_PARTICLES: usize = 6000;
pub const FX_STRESS_EVERY: u64 = 40;

// ------------------------------------------------------------------ the black hole
// Drawn by a shader after Jason's falling-figure image. It grows as you approach.

/// Where it hangs in the playfield (px), and its shadow's radius at the start and end of the stage.
pub const HOLE_X: f32 = 180.0;
pub const HOLE_Y: f32 = 92.0;
pub const HOLE_R: (f32, f32) = (11.0, 30.0);
/// How long the approach takes (the length of Stage 1, frames).
pub const STAGE_FRAMES: u64 = 150 * 60;
/// How open the accretion disk looks (0 is edge-on).
pub const HOLE_TILT: f32 = 0.18;

// ------------------------------------------------------------------ the pattern sandbox
// Each preset is one or more guns. The sandbox cycles through them (F7, [ ], LB,
// the NEXT button or the pause menu). Speeds are px per frame.

const fn gun(mount: Mount, shape: Shape, kind: Kind, speed: f32, every: u32, delay: u32, behavior: Behavior) -> Gun {
    Gun { mount, shape, kind, speed, every, delay, behavior, hue: Hue::Red }
}
const STRAIGHT: Behavior = Behavior::Straight;

pub const SANDBOX: &[Preset] = &[
    Preset { name: "OFF", guns: &[], fill: 0 },
    Preset {
        name: "AIMED FANS",
        guns: &[
            gun(Mount::Dummy(1), Shape::Fan { n: 5, spread: 48.0, aimed: true }, Kind::Orb, 2.6, 50, 30, STRAIGHT).hue(Hue::Red),
            gun(Mount::At(80.0, 40.0), Shape::Fan { n: 3, spread: 24.0, aimed: true }, Kind::Small, 3.2, 70, 60, STRAIGHT).hue(Hue::Amber),
            gun(Mount::At(280.0, 40.0), Shape::Fan { n: 3, spread: 24.0, aimed: true }, Kind::Small, 3.2, 70, 95, STRAIGHT).hue(Hue::Amber),
        ],
        fill: 0,
    },
    Preset {
        name: "RINGS",
        guns: &[
            gun(Mount::At(180.0, 56.0), Shape::Ring { n: 20, turn: 9.0 }, Kind::Small, 1.8, 36, 20, STRAIGHT).hue(Hue::Teal),
            gun(Mount::At(88.0, 70.0), Shape::Ring { n: 14, turn: -6.0 }, Kind::Orb, 1.5, 60, 38, STRAIGHT).hue(Hue::Blue),
            gun(Mount::At(272.0, 70.0), Shape::Ring { n: 14, turn: 6.0 }, Kind::Orb, 1.5, 60, 68, STRAIGHT).hue(Hue::Blue),
        ],
        fill: 0,
    },
    Preset {
        name: "SPIRAL",
        guns: &[gun(Mount::At(180.0, 70.0), Shape::Spiral { arms: 3, spin: 11.0, flip: 40 }, Kind::Rice, 2.2, 5, 20, STRAIGHT).hue(Hue::Violet)],
        fill: 0,
    },
    Preset {
        name: "STREAMS",
        guns: &[
            gun(Mount::At(107.0, 46.0), Shape::Stream { burst: 8, gap: 4 }, Kind::Needle, 4.5, 75, 20, STRAIGHT).hue(Hue::Red),
            gun(Mount::At(253.0, 46.0), Shape::Stream { burst: 8, gap: 4 }, Kind::Needle, 4.5, 75, 57, STRAIGHT).hue(Hue::Red),
        ],
        fill: 0,
    },
    Preset {
        name: "RANDOM CONE",
        guns: &[gun(Mount::Sweep { y: 40.0, x0: 93.0, x1: 267.0, period: 360 }, Shape::Cone { n: 3, spread: 70.0, jitter: 0.25 }, Kind::Small, 2.4, 6, 20, STRAIGHT).hue(Hue::Amber)],
        fill: 0,
    },
    Preset {
        name: "ACCELERATE",
        guns: &[gun(Mount::At(180.0, 70.0), Shape::Ring { n: 16, turn: 7.0 }, Kind::Orb, 0.6, 45, 20, Behavior::Accel { accel: 0.05, max: 3.6 }).hue(Hue::Blue)],
        fill: 0,
    },
    Preset {
        name: "SLOW, PAUSE, AIM",
        guns: &[gun(Mount::At(180.0, 80.0), Shape::Ring { n: 12, turn: 15.0 }, Kind::Orb, 3.0, 70, 20, Behavior::SlowAim { slow: 30, pause: 20, speed: 3.2 }).hue(Hue::Violet)],
        fill: 0,
    },
    Preset {
        name: "CURVE",
        guns: &[
            gun(Mount::At(180.0, 56.0), Shape::Fan { n: 5, spread: 100.0, aimed: false }, Kind::Rice, 2.0, 24, 20, Behavior::Curve { turn: 0.7, frames: 120 }).hue(Hue::Teal),
            gun(Mount::At(180.0, 56.0), Shape::Fan { n: 5, spread: 100.0, aimed: false }, Kind::Rice, 2.0, 24, 32, Behavior::Curve { turn: -0.7, frames: 120 }).hue(Hue::Teal),
        ],
        fill: 0,
    },
    Preset {
        name: "SPLIT",
        guns: &[gun(
            Mount::At(180.0, 50.0),
            Shape::Fan { n: 3, spread: 60.0, aimed: true },
            Kind::Large,
            1.6,
            80,
            20,
            Behavior::Split { at: 70, n: 10, kind: Kind::Small, speed: 2.0 },
        )
        .hue(Hue::Amber)],
        fill: 0,
    },
    Preset {
        name: "PULL",
        guns: &[
            gun(Mount::At(53.0, 60.0), Shape::Ring { n: 10, turn: 13.0 }, Kind::Small, 2.2, 30, 20, Behavior::Pull { x: 180.0, y: 120.0, pull: 0.035, max: 3.0 }).hue(Hue::Violet),
            gun(Mount::At(307.0, 60.0), Shape::Ring { n: 10, turn: -13.0 }, Kind::Small, 2.2, 30, 35, Behavior::Pull { x: 180.0, y: 120.0, pull: 0.035, max: 3.0 }).hue(Hue::Violet),
        ],
        fill: 0,
    },
    Preset {
        name: "LAYERED",
        guns: &[
            gun(Mount::At(180.0, 56.0), Shape::Ring { n: 18, turn: 10.0 }, Kind::Small, 1.6, 40, 20, STRAIGHT).hue(Hue::Teal),
            gun(Mount::Dummy(1), Shape::Fan { n: 3, spread: 20.0, aimed: true }, Kind::Orb, 3.0, 30, 40, STRAIGHT).hue(Hue::Red),
        ],
        fill: 0,
    },
    Preset { name: "STRESS 2000", guns: &[], fill: 2000 },
    Preset { name: "STRESS 4000", guns: &[], fill: 4000 },
];

// ------------------------------------------------------------------ Stage 1: OUTER DISK

pub const STAGE_NAME: &str = "OUTER DISK";
/// The timeline, in seconds: (start, name). Each section's enemies arrive on cue.
pub const SECTIONS: [(f32, &str); 8] = [
    (0.0, "FLY-IN"),
    (10.0, "GUNSHIPS"),
    (40.0, "BUOYS"),
    (70.0, "BREATHER"),
    (85.0, "DERELICT"),
    (115.0, "HEAVIES"),
    (150.0, "WARNING"),
    (157.5, "JAWSTHIC"),
];
/// The section that sounds the WARNING (the boss's section follows it). The stage
/// ends when the boss is gone.
pub const WARNING_SECTION: usize = 6;
/// Kill the derelict early and the next section comes forward, landing at most this
/// many seconds ahead of the timeline.
pub const SECTION_EARLY: f32 = 4.0;
/// No enemy fires for this many seconds after the fly-in.
pub const FIRST_FIRE: f32 = 6.0;
/// The stage title shows for this many frames.
pub const TITLE_CARD: u32 = 210;

/// The breather's set dressing. The disk flares (peak boost of its light; seconds to
/// rise, hold and fall), a debris field drifts by (start, end in seconds), and a tiny
/// figure tumbles into the hole (start, seconds to fall).
pub const FLARE: (f32, f32, f32, f32) = (1.6, 2.0, 5.0, 6.0);
pub const DEBRIS_AT: (f32, f32) = (71.0, 84.0);
pub const FIGURE_AT: (f32, f32) = (72.5, 11.0);

/// The playfield's 7 lanes; the two edge lanes never get spawns.
pub const LANES: usize = 7;
/// Nothing spawns closer than this to a side edge.
pub const SPAWN_EDGE: f32 = 20.0;
/// Enemies this close to the top can't be hurt (no killing what you can't see),
/// and enemies this close to the bottom don't fire (no shots from point-blank range).
pub const DEAD_ZONE: f32 = 12.0;
pub const CEASEFIRE: f32 = 60.0;
/// High-HP enemies arrive one at a time: at least this many seconds apart.
pub const HEAVY_GAP: f32 = 2.0;
/// Enemies flash on a hit at most this often (frames lit, gap).
pub const ENEMY_FLASH: (u8, u8) = (2, 5);
/// Crashing into an enemy: its body is this share of its hurtbox (against your hitbox).
pub const BODY_SHRINK: f32 = 0.7;
/// No enemy stays longer than this (frames), whatever its path.
pub const ENEMY_MAX_AGE: u32 = 60 * 60;
/// A red charge glow on a gun this many frames before it fires (the telegraph).
pub const CHARGE_FRAMES: u32 = 14;

/// Each enemy: (hit points, hurtbox w, h). Big hurtboxes, just enough HP for the job
/// (the wide shot does about 120 a second to one target, the beam about 190).
pub const DRONE: (f32, f32, f32) = (4.0, 16.0, 15.0);
/// The two lighter fodder: the courier (a fast three-spired dart) and the usher (a
/// horseshoe that stops, fires a pair, and leaves).
pub const COURIER: (f32, f32, f32) = (6.0, 16.0, 22.0);
pub const USHER: (f32, f32, f32) = (16.0, 22.0, 18.0);
pub const GUNSHIP: (f32, f32, f32) = (140.0, 30.0, 24.0);
pub const BUOY: (f32, f32, f32) = (220.0, 24.0, 24.0);
pub const HEAVY: (f32, f32, f32) = (1100.0, 48.0, 34.0);
/// The derelict: the hull (the spire, the shoulders and the iris eye at its heart),
/// and its three weak points.
pub const HULL: (f32, f32, f32) = (2400.0, 112.0, 70.0);
pub const HULL_TURRET_HP: f32 = 300.0;
/// The weak points (offset from the hull's center, and hurtbox w, h): the left gun pod,
/// the nose gun, the right gun pod. They hang lower than the hull, so shots reach them first.
pub const HULL_TURRETS: [(f32, f32, f32, f32); 3] = [(-44.0, 22.0, 16.0, 34.0), (0.0, 38.0, 12.0, 16.0), (44.0, 22.0, 16.0, 34.0)];
/// The derelict rises out of the disk (it can't be hurt meanwhile) to hover here.
pub const HULL_EMERGE: u32 = 150;
pub const HULL_Y: f32 = 78.0;

/// How long things stay if you don't kill them (frames), and how fast they move.
pub const DRONE_SPEED: f32 = 2.2;
pub const COURIER_SPEED: f32 = 3.4;
/// Ushers fly in this fast, wait this long (firing), then leave.
pub const USHER_ENTER: u32 = 35;
pub const USHER_STAY: u32 = 70;
pub const GUNSHIP_STAY: u32 = 240;
pub const BUOY_DRIFT: f32 = 0.38;
/// Buoys stop firing below this share of the playfield.
pub const BUOY_QUIET: f32 = 0.6;
pub const HEAVY_STAY: u32 = 450;
/// The derelict leaves this long after it starts to rise (frames).
pub const HULL_STAY: u32 = 25 * 60;
/// The derelict cracks open (its second pattern) when its turrets are gone or this long after it arrives.
pub const HULL_PHASE2: u32 = 12 * 60;
/// Frames for enemies to fly in to where they hover.
pub const ENTER_FRAMES: u32 = 50;
/// Leaving: accelerating up the screen at this many px a frame, every frame.
pub const LEAVE_ACCEL: f32 = 0.05;
/// Hovering enemies sway side to side by these many px, once per SWAY_FRAMES.
pub const SWAY_FRAMES: u32 = 240;
pub const GUNSHIP_SWAY: f32 = 6.0;
pub const BUOY_SWAY: f32 = 6.0;
pub const HEAVY_SWAY: f32 = 10.0;
pub const HULL_SWAY: f32 = 12.0;

/// The guns: which part carries each (0 the body, 1-3 the derelict's turrets), the
/// phase it fires in (0 always, 1 before the derelict cracks, 2 after) and the gun.
/// Delays count from the enemy's arrival; hovering enemies only fire once they've settled.
/// In a firing flight of popcorn, this share fire (the rest are just obstacles).
pub const DRONE_FIRE_SHARE: f32 = 0.8;
/// Popcorn: one aimed shot, about half a second in.
pub const DRONE_GUNS: &[Arm] = &[arm(0, 0, gun(Mount::Owner { dx: 0.0, dy: 6.0 }, Shape::Fan { n: 1, spread: 0.0, aimed: true }, Kind::Small, 2.9, 10_000, 34, STRAIGHT).hue(Hue::Red))];
/// Couriers: one fast aimed needle, early.
pub const COURIER_GUNS: &[Arm] = &[arm(0, 0, gun(Mount::Owner { dx: 0.0, dy: 10.0 }, Shape::Fan { n: 1, spread: 0.0, aimed: true }, Kind::Needle, 3.5, 10_000, 24, STRAIGHT).hue(Hue::Red))];
/// Ushers: an aimed shot from each prong, twice, while they wait.
pub const USHER_GUNS: &[Arm] = &[
    arm(0, 0, gun(Mount::Owner { dx: -10.0, dy: 10.0 }, Shape::Fan { n: 1, spread: 0.0, aimed: true }, Kind::Small, 2.7, 34, 40, STRAIGHT).hue(Hue::Violet)),
    arm(0, 0, gun(Mount::Owner { dx: 10.0, dy: 10.0 }, Shape::Fan { n: 1, spread: 0.0, aimed: true }, Kind::Small, 2.7, 34, 40, STRAIGHT).hue(Hue::Violet)),
];
/// Gunships: aimed 3-way fans, then 5-way, while they hover.
pub const GUNSHIP_GUNS: &[Arm] = &[
    arm(0, 0, gun(Mount::Owner { dx: 0.0, dy: 11.0 }, Shape::Fan { n: 3, spread: 24.0, aimed: true }, Kind::Rice, 2.9, 90, 56, STRAIGHT).hue(Hue::Amber)),
    arm(0, 0, gun(Mount::Owner { dx: 0.0, dy: 11.0 }, Shape::Fan { n: 5, spread: 48.0, aimed: true }, Kind::Small, 2.5, 90, 101, STRAIGHT).hue(Hue::Red)),
];
/// Buoys: a static ring, or a three-armed spiral.
pub const BUOY_RING: &[Arm] = &[arm(0, 0, gun(Mount::Owner { dx: 0.0, dy: 0.0 }, Shape::Ring { n: 11, turn: 5.6 }, Kind::Orb, 1.55, 50, 30, STRAIGHT).hue(Hue::Teal))];
pub const BUOY_SPIRAL: &[Arm] = &[arm(0, 0, gun(Mount::Owner { dx: 0.0, dy: 0.0 }, Shape::Spiral { arms: 3, spin: 13.0, flip: 0 }, Kind::Rice, 1.85, 10, 30, STRAIGHT).hue(Hue::Violet))];
/// Heavies: slow rings of big orbs (static) layered with needle bursts from both flanks
/// (aimed) and a sprinkle of small ones (random).
pub const HEAVY_GUNS: &[Arm] = &[
    arm(0, 0, gun(Mount::Owner { dx: 0.0, dy: 6.0 }, Shape::Ring { n: 12, turn: 10.0 }, Kind::Large, 1.35, 64, 56, STRAIGHT).hue(Hue::Blue)),
    arm(0, 0, gun(Mount::Owner { dx: -16.0, dy: 15.0 }, Shape::Stream { burst: 4, gap: 5 }, Kind::Needle, 3.3, 70, 90, STRAIGHT).hue(Hue::Red)),
    arm(0, 0, gun(Mount::Owner { dx: 16.0, dy: 15.0 }, Shape::Stream { burst: 4, gap: 5 }, Kind::Needle, 3.3, 70, 90, STRAIGHT).hue(Hue::Red)),
    arm(0, 0, gun(Mount::Owner { dx: 0.0, dy: 10.0 }, Shape::Cone { n: 3, spread: 90.0, jitter: 0.25 }, Kind::Small, 2.2, 26, 70, STRAIGHT).hue(Hue::Amber)),
];
/// The derelict. First: aimed fans from the hanging gun pods and slow rings from the
/// nose gun. Cracked: a reversing spiral from the iris and aimed needle bursts.
pub const HULL_GUNS: &[Arm] = &[
    arm(1, 0, gun(Mount::Owner { dx: -44.0, dy: 41.0 }, Shape::Fan { n: 5, spread: 36.0, aimed: true }, Kind::Rice, 2.7, 60, 20, STRAIGHT).hue(Hue::Amber)),
    arm(3, 0, gun(Mount::Owner { dx: 44.0, dy: 41.0 }, Shape::Fan { n: 5, spread: 36.0, aimed: true }, Kind::Rice, 2.7, 60, 50, STRAIGHT).hue(Hue::Amber)),
    arm(2, 0, gun(Mount::Owner { dx: 0.0, dy: 46.0 }, Shape::Ring { n: 14, turn: 4.5 }, Kind::Orb, 1.55, 64, 40, STRAIGHT).hue(Hue::Teal)),
    arm(0, 2, gun(Mount::Owner { dx: 0.0, dy: -2.0 }, Shape::Spiral { arms: 5, spin: 8.5, flip: 30 }, Kind::Rice, 1.95, 10, 0, STRAIGHT).hue(Hue::Violet)),
    arm(0, 2, gun(Mount::Owner { dx: 0.0, dy: 26.0 }, Shape::Stream { burst: 5, gap: 4 }, Kind::Needle, 3.4, 90, 50, STRAIGHT).hue(Hue::Red)),
];

// ------------------------------------------------------------------ the boss: JAWSTHIC
// A great white carved from moon rock (Jason's concept, Sprint 6). Distances are px
// from its body's pivot as if it faced right (it mirrors to face left).

pub const BOSS_NAME: &str = "JAWSTHIC";
/// It arrives this many seconds into the stage, after the WARNING. The intro (the fin
/// cutting through the disk, the breach, the roar) runs to these frames: the fin's
/// pass, under the surface, the breach's peak, the fight. It can't be hurt till then.
pub const JAWS_AT: f32 = 152.0;
pub const JAWS_INTRO: [u32; 4] = [170, 215, 268, 330];
/// The fight's health. Each phase: (its share of the bar, seconds before it times
/// out, damage taken per point of shot damage, the turn period in frames, 0 never).
/// The shares make the bar; the damage keeps each phase 20-35 s with full power.
pub const JAWS_HP: f32 = 12000.0;
pub const JAWS_PHASES: [(f32, u32, f32, u32); 4] = [(0.25, 30, 0.65, 840), (0.30, 40, 0.8, 0), (0.30, 40, 0.8, 600), (0.15, 35, 0.42, 420)];
pub const JAWS_PHASE_NAMES: [&str; 4] = ["TEETH", "FINS", "GLITCH", "EVENT HORIZON"];
/// Between phases: frames of calm while it shakes the break off (no fire, no damage).
pub const JAWS_BREAK: u32 = 120;
/// Through that calm (shares of it), the health bar refills outward from its gem,
/// the gem already the next phase's color (it flashes as it turns).
pub const JAWS_REFILL: (f32, f32) = (0.2, 0.75);
/// Phase 2's weak points, the pectoral fin and the tail fin turrets: health each, and
/// the share of the phase's health breaking one tears away.
pub const JAWS_FIN_HP: f32 = 900.0;
pub const JAWS_FIN_BLOW: f32 = 0.25;
/// Where it swims: its height, and its home across the screen facing right and left
/// (the whole fish stays on the screen); how far it sways and bobs (px, period).
pub const JAWS_Y: f32 = 82.0;
pub const JAWS_HOME: (f32, f32) = (224.0, 136.0);
pub const JAWS_SWAY: (f32, u32) = (16.0, 480);
pub const JAWS_BOB: (f32, u32) = (5.0, 300);
/// How quickly it closes on where it wants to be (share of the gap per frame).
pub const JAWS_FOLLOW: f32 = 0.05;
/// A turn (it glitches round to face the other way): frames, and how far its pivot
/// shifts so the whole fish stays about where it was.
pub const JAWS_TURN: u32 = 28;
pub const JAWS_TURN_SHIFT: f32 = 88.0;
/// The jaw opens this many frames before a mouth gun fires (the telegraph), and how
/// fast it swings (share of the way per frame).
pub const JAWS_JAW_LEAD: u32 = 22;
pub const JAWS_JAW_SPEED: f32 = 0.2;
/// The tail: its swim (wave radians per frame, idle and while turning), and how far
/// each joint bends, root to the tail fin (radians).
pub const JAWS_SWIM: (f32, f32) = (0.075, 0.24);
pub const JAWS_BEND: [f32; 6] = [0.05, 0.07, 0.09, 0.11, 0.13, 0.17];
/// Its body swims too, on the tail's beat: a wave rolls from its snout, swaying it
/// a little (px), back to where the tail joins, swinging more (px); how far along
/// the wave each px of body is (radians).
pub const JAWS_BODY_WAVE: (f32, f32, f32) = (1.2, 2.6, 0.018);
/// Between attacks it chomps: every so often (frames, plus up to as many again),
/// two quick snaps, this far open (share of wide), each this long (frames).
pub const JAWS_CHOMP: (u32, u32, f32, u32) = (160, 120, 0.5, 10);
/// Now and then it lunges: every so often (frames, plus up to as many again) it
/// draws back (px, frames), then strikes forward and down (px, px, frames) and
/// bites, and eases home.
pub const JAWS_LUNGE_EVERY: (u32, u32) = (300, 180);
pub const JAWS_LUNGE: (f32, u32, f32, f32, u32) = (6.0, 18, 16.0, 6.0, 7);
/// Hit hard, it flinches: when the damage it's taken lately (keeping this share of
/// it each frame) passes this, or a fin breaks, it recoils (px back, px up) and
/// lashes its tail; then it won't again for a while (frames).
pub const JAWS_FLINCH: (f32, f32, f32, f32, u32) = (0.95, 120.0, 8.0, 4.0, 60);
/// The tail sweeps up off the body (radians), the way Jason drew it, on top of the
/// shape of its own plates (each plate's slope as drawn, root to the fin).
pub const JAWS_TAIL_TILT: f32 = 0.25;
pub const JAWS_TAIL_ART: [f32; 6] = [-0.124, 0.124, 0.322, 0.322, 0.709, 0.0];
/// The rig, facing right.
pub const JAWS_HINGE: (f32, f32) = (-13.0, 9.0);
pub const JAWS_TAIL_ROOT: (f32, f32) = (-76.0, -9.0);
/// The tail's plates, front to back: (length, height).
pub const JAWS_TAIL: [(f32, f32); 5] = [(8.06, 20.0), (8.06, 22.0), (9.49, 20.0), (9.49, 18.0), (9.22, 18.0)];
pub const JAWS_DORSAL_AT: (f32, f32) = (-14.0, -14.0);
pub const JAWS_PECTORAL_AT: (f32, f32) = (-17.0, 18.0);
pub const JAWS_WINDOWS: [f32; 4] = [-24.0, -13.0, -2.0, 8.0];
pub const JAWS_EYE: (f32, f32) = (26.0, -22.0);
/// Its guns' muzzles: the jaws, the windows (at this height), the pectoral fin's joint
/// (a turret's red core), the belly; and the dorsal fin's window (just a light).
pub const JAWS_MOUTH: (f32, f32) = (54.0, -3.0);
/// How far the mouth's center moves as the jaw opens wide (px): back and down, to the
/// crosshair in its throat.
pub const JAWS_MOUTH_OPEN: (f32, f32) = (-16.0, 17.0);
pub const JAWS_WINDOW_Y: f32 = 0.0;
pub const JAWS_DORSAL_CORE: (f32, f32) = (-17.0, -26.0);
pub const JAWS_BELLY: (f32, f32) = (1.0, 29.0);
/// Hurtboxes (center x, center y, w, h): the body and the pectoral turret; the tail
/// fin's (how far back along its plate its middle is, w, h) swings with the tail.
pub const JAWS_BODY: (f32, f32, f32, f32) = (4.0, -2.0, 128.0, 52.0);
pub const JAWS_PECTORAL: (f32, f32, f32, f32) = (-30.0, 28.0, 30.0, 26.0);
pub const JAWS_TAIL_FIN: (f32, f32, f32) = (16.0, 26.0, 64.0);
/// Phase 3's laser: frames of warning, frames of beam, how far it sweeps (degrees),
/// how wide it's drawn (px; it hits on a little over half of that), how often it
/// comes and when the first one does (frames into the phase).
pub const JAWS_LASER: (u32, u32, f32, f32) = (45, 60, 55.0, 9.0);
pub const JAWS_LASER_EVERY: (u32, u32) = (330, 150);
/// The death: frames from the kill to the tally (the first `JAWS_SLOWMO` run at quarter
/// speed), and the frames the blasts go off on, walking up from the tail fin, the
/// last the big one and its white-out.
pub const JAWS_DEATH: u32 = 235;
pub const JAWS_SLOWMO: u32 = 48;
pub const JAWS_BLASTS: [u32; 11] = [0, 8, 15, 22, 30, 40, 52, 66, 82, 100, 140];
/// Glitch smears tearing off its tail (Jason's picture): how many at once in each
/// phase, and through a turn or its end; how long each lives (frames).
pub const JAWS_SMEARS: ([u32; 4], u32) = ([4, 6, 9, 14], 24);
pub const JAWS_SMEAR_LIFE: u32 = 24;
/// The black hole stripping specks off its outline, like the shards a bomb sends to
/// you: how many at once in each phase, and through a turn or its end; how long each
/// lives (frames); the share of its life it spends lifting off before the hole takes
/// it; how far it swirls round on its way in (px, either way).
pub const JAWS_STRIP: ([u32; 4], u32) = ([90, 110, 140, 180], 280);
pub const JAWS_STRIP_LIFE: u32 = 64;
pub const JAWS_STRIP_LIFT: f32 = 0.4;
pub const JAWS_STRIP_SWIRL: f32 = 26.0;
/// Timed out in phase 4: frames for the hole to swallow it.
pub const JAWS_FALL: u32 = 150;

const O: Mount = Mount::Owner { dx: 0.0, dy: 0.0 };
/// Phase 4's bullets bend toward the black hole (and it swallows any that reach it).
const HOLE_PULL: Behavior = Behavior::Pull { x: HOLE_X, y: HOLE_Y, pull: 0.006, max: 2.4 };

/// Phase 1, TEETH: aimed 5-way fans of crystal shards from the jaws over slow static
/// rings from the windows, one window after another: orbs, then slower stars.
pub const JAWS_P1: &[BossGun] = &[
    bgun(Muzzle::Mouth, gun(O, Shape::Fan { n: 5, spread: 48.0, aimed: true }, Kind::Diamond, 2.8, 40, 50, STRAIGHT).hue(Hue::Red)),
    bgun(Muzzle::Window(0), gun(O, Shape::Ring { n: 12, turn: 7.0 }, Kind::Orb, 1.3, 168, 30, STRAIGHT).hue(Hue::Teal)),
    bgun(Muzzle::Window(1), gun(O, Shape::Ring { n: 12, turn: -7.0 }, Kind::Star, 1.0, 168, 72, STRAIGHT).hue(Hue::Amber)),
    bgun(Muzzle::Window(2), gun(O, Shape::Ring { n: 12, turn: 7.0 }, Kind::Orb, 1.3, 168, 114, STRAIGHT).hue(Hue::Teal)),
    bgun(Muzzle::Window(3), gun(O, Shape::Ring { n: 12, turn: -7.0 }, Kind::Star, 1.0, 168, 156, STRAIGHT).hue(Hue::Blue)),
];
/// Phase 2, FINS: the pectoral fin and the tail fin spin opposing spirals while the
/// jaws fire aimed needle bursts. Breaking a fin silences its spiral.
pub const JAWS_P2: &[BossGun] = &[
    bgun(Muzzle::Pectoral, gun(O, Shape::Spiral { arms: 3, spin: 7.0, flip: 0 }, Kind::Rice, 1.85, 8, 40, STRAIGHT).hue(Hue::Violet)),
    bgun(Muzzle::Tail, gun(O, Shape::Spiral { arms: 3, spin: -7.0, flip: 0 }, Kind::Rice, 1.85, 8, 43, STRAIGHT).hue(Hue::Blue)),
    bgun(Muzzle::Mouth, gun(O, Shape::Stream { burst: 5, gap: 4 }, Kind::Needle, 3.3, 84, 70, STRAIGHT).hue(Hue::Red)),
    // its fins gone, its windows open up: slow rings of stars
    enraged(Muzzle::Window(0), gun(O, Shape::Ring { n: 14, turn: 9.0 }, Kind::Star, 1.15, 120, 0, STRAIGHT).hue(Hue::Amber)),
    enraged(Muzzle::Window(3), gun(O, Shape::Ring { n: 14, turn: -9.0 }, Kind::Star, 1.15, 120, 60, STRAIGHT).hue(Hue::Amber)),
];
/// Phase 3, GLITCH: a curtain of rice like a corrupted screen, each row slipping
/// sideways, a lane drifting through it; slow stars sprinkled off the tail; and the
/// laser sweeps (they pause the curtain).
pub const JAWS_P3: &[BossGun] = &[
    bgun(Muzzle::Belly, gun(O, Shape::Curtain { n: 23, spacing: 15.0, gap: 3, period: 22, jitter: 6.0 }, Kind::Rice, 1.65, 24, 30, STRAIGHT).hue(Hue::Teal)),
    bgun(Muzzle::Tail, gun(O, Shape::Cone { n: 2, spread: 140.0, jitter: 0.3 }, Kind::Star, 1.05, 26, 40, STRAIGHT).hue(Hue::Violet)),
];
/// Phase 4, EVENT HORIZON: everything it throws bends toward the black hole: a turning
/// flower of stars from its jaws, spiral arms off its tail; and aimed needle bursts,
/// straight.
pub const JAWS_P4: &[BossGun] = &[
    bgun(Muzzle::Mouth, gun(O, Shape::Flower { n: 18, petals: 5, turn: 9.0 }, Kind::Star, 2.1, 56, 40, HOLE_PULL).hue(Hue::Amber)),
    bgun(Muzzle::Tail, gun(O, Shape::Spiral { arms: 2, spin: 11.0, flip: 0 }, Kind::Rice, 1.85, 11, 60, HOLE_PULL).hue(Hue::Violet)),
    bgun(Muzzle::Mouth, gun(O, Shape::Stream { burst: 4, gap: 5 }, Kind::Needle, 3.5, 72, 60, STRAIGHT).hue(Hue::Red)),
];
pub const JAWS_GUNS: [&[BossGun]; 4] = [JAWS_P1, JAWS_P2, JAWS_P3, JAWS_P4];

// ------------------------------------------------------------------ scoring
// Clear before deep: kill things (closer is worth double), break the boss's phases
// clean and fast, don't die, don't bomb. Arcade grades: S A B C (D for the stage).

/// Points for destroying each kind of enemy: drone, courier, usher, gunship, buoy,
/// heavy, the derelict.
pub const KILL_POINTS: [u32; 7] = [500, 750, 1_500, 7_500, 10_000, 40_000, 200_000];
/// A kill this close to the Pilgrim (px) scores this many times over (a teal ring flashes).
pub const POINT_BLANK: (f32, u32) = (48.0, 2);
/// Each of the derelict's gun pods and its nose gun.
pub const PART_POINTS: u32 = 15_000;
/// Each bullet cancelled when a heavy or the derelict dies, or a boss phase breaks:
/// a shard that flies to you. (A bomb's cancels give nothing.)
pub const SHARD_POINTS: u32 = 50;
/// Each graze.
pub const GRAZE_POINTS: u32 = 100;
/// JAWSTHIC: each fin turret broken, and the kill.
pub const FIN_POINTS: u32 = 25_000;
pub const BOSS_KILL_POINTS: u32 = 300_000;
/// Each phase's capture bonus: grade A, broken without a death or a bomb. Grade S
/// (captured inside this share of its time) doubles it; B (broken, not clean) gets
/// a fifth; C (timed out) nothing.
pub const CAPTURE: [u32; 4] = [150_000, 200_000, 200_000, 300_000];
pub const GRADE_S_TIME: f32 = 0.7;
/// The stage's end: no deaths, no bombs.
pub const NO_MISS_BONUS: u32 = 500_000;
pub const NO_BOMB_BONUS: u32 = 300_000;
/// The stage's rating out of 100: kill rate (up to `RATE_KILLS`), JAWSTHIC destroyed,
/// each phase's grade (S A B C), no misses, no bombs (each death or bomb takes a share).
pub const RATE_KILLS: u32 = 30;
pub const RATE_DESTROYED: u32 = 10;
pub const RATE_GRADES: [u32; 4] = [10, 7, 4, 0];
pub const RATE_NO_MISS: (u32, u32) = (15, 5);
pub const RATE_NO_BOMB: (u32, u32) = (5, 2);
/// The rank for a rating: S, A, B, C at these or more, else D.
pub const RANKS: [u32; 4] = [90, 75, 55, 35];
/// An extra ship at each of these scores, up to this many ships. (The stage-end
/// bonuses come too late to count.)
pub const EXTENDS: [u64; 2] = [800_000, 1_800_000];
pub const MAX_LIVES: u8 = 6;
/// Points worth a big flashing number.
pub const BIG_POINTS: u32 = 20_000;
/// The stage-clear tally (frames from the stage's end): each line comes this long
/// after the last and counts up over `TALLY_COUNT`; then the score, the rank's slam,
/// the last line, and its menu.
pub const TALLY_STEP: u32 = 18;
pub const TALLY_COUNT: u32 = 16;
pub const TALLY_SCORE: u32 = 6 * TALLY_STEP;
pub const TALLY_RANK: u32 = TALLY_SCORE + 40;
pub const TALLY_LINE: u32 = TALLY_RANK + 40;
pub const TALLY_DONE: u32 = TALLY_LINE + 40;
/// The title waits this long untouched (frames) before the bot plays a demo, which
/// runs this long at most.
pub const ATTRACT_AFTER: u64 = 20 * 60;
pub const DEMO_LEN: u64 = 60 * 60;
/// The HOLD TO BEAM fire scheme: fire held this many frames becomes the beam.
pub const HOLD_BEAM: u32 = 18;

// ------------------------------------------------------------------ the bot
// A dodger: tries 9 directions at two speeds, looking this many frames ahead.

pub const BOT_LOOKAHEAD: usize = 12;
/// Where it likes to sit, and how much it cares.
pub const BOT_HOME: (f32, f32) = (180.0, 285.0);
pub const BOT_HOME_PULL: f32 = 0.1;
/// A small bonus for keeping the same choice (so it doesn't dither).
pub const BOT_STICKY: f32 = 0.3;
/// How strongly it lines up under its target (enemies above it).
pub const BOT_AIM: f32 = 0.06;
/// Bullets inside this many px of clearance count as danger.
pub const BOT_DANGER: f32 = 12.0;
/// Extra clearance it wants before calling a path safe.
pub const BOT_MARGIN: f32 = 1.0;

// ------------------------------------------------------------------ input and feel

/// Touch steering: ship px per canvas px of finger travel.
pub const TOUCH_SENS: f32 = 1.0;
pub const TOUCH_SENS_RANGE: (f32, f32, f32) = (0.5, 2.5, 0.1);
/// Left stick deadzone.
pub const STICK_DEADZONE: f32 = 0.25;
/// Afterimages show when moving at least this share of full speed, unfocused.
pub const AFTERIMAGE_MIN: f32 = 0.9;

// ------------------------------------------------------------------ sound

/// The options' volume steps (0-10) start here.
pub const VOLUME_DEFAULT: u8 = 8;

/// Seconds between plays of each sound, so rapid fire never machine-guns.
pub const SHOT_SOUND_GAP: f64 = 0.095;
pub const HIT_SOUND_GAP: f64 = 0.06;
/// Mix levels.
pub const VOL_SHOT: f32 = 0.16;
pub const VOL_HIT: f32 = 0.10;
pub const VOL_BEAM: f32 = 0.22;
/// Blasts: small, medium, large, boss.
pub const VOL_BLAST: [f32; 4] = [0.30, 0.40, 0.60, 0.85];
pub const VOL_UI: f32 = 0.35;
pub const VOL_GRAZE: f32 = 0.07;
pub const GRAZE_SOUND_GAP: f64 = 0.05;
pub const VOL_DEATH: f32 = 0.55;
pub const VOL_BOMB: f32 = 0.9;
pub const VOL_STRUCK: f32 = 0.5;
pub const VOL_RESPAWN: f32 = 0.35;
pub const VOL_GAME_OVER: f32 = 0.55;
pub const VOL_WARNING: f32 = 0.5;
/// The boss: its theme (music), its approach, its roar, the laser, glitches, the hole
/// swallowing it; and how fast the music fades in and out (volume per second).
pub const VOL_MUSIC: f32 = 0.32;
pub const VOL_APPROACH: f32 = 0.55;
pub const VOL_ROAR: f32 = 0.7;
pub const VOL_LASER: f32 = 0.38;
pub const VOL_GLITCH: f32 = 0.22;
pub const VOL_CHOMP: f32 = 0.5;
/// The crunch under every kill (scaled by the blast's size: small to boss).
pub const VOL_CRUNCH: f32 = 0.45;
pub const CRUNCH_BY_SIZE: [f32; 4] = [0.7, 0.85, 1.0, 1.0];
pub const VOL_SWALLOWED: f32 = 0.7;
/// The score: the tally's ticks, a grade's or the rank's slam, an extend.
pub const VOL_TICK: f32 = 0.3;
pub const VOL_SLAM: f32 = 0.6;
pub const VOL_EXTEND: f32 = 0.55;
pub const MUSIC_FADE: f32 = 0.8;
/// How fast the beam hum fades in and out (volume per second).
pub const BEAM_FADE: f32 = 6.0;

// ------------------------------------------------------------------ the starfield

pub struct StarLayer {
    /// Stars per 10,000 px² of the area they fill.
    pub density: f32,
    /// Fall speed in px per frame.
    pub speed: f32,
    /// Brightness as a fraction of white (the value budget caps the background at 0.30).
    pub value: f32,
    /// Share of the layer drawn as a small plus instead of a single pixel.
    pub plus_share: f32,
}

/// Far to near: each nearer layer is sparser, faster and brighter.
pub const STAR_LAYERS: [StarLayer; 3] = [
    StarLayer { density: 14.0, speed: 0.10, value: 0.11, plus_share: 0.0 },
    StarLayer { density: 6.0, speed: 0.28, value: 0.19, plus_share: 0.0 },
    StarLayer { density: 2.0, speed: 0.70, value: 0.29, plus_share: 0.25 },
];
/// How far a star's brightness dips when it twinkles (fraction of its value).
pub const STAR_TWINKLE: f32 = 0.35;
/// The odd red or teal star, among the two nearer layers.
pub const STAR_TINT_SHARE: f32 = 0.04;
/// The background's brightness ceiling (art bible: about 30%).
pub const BG_MAX_VALUE: f32 = 0.30;
/// The background dims toward half that (15%) as the bullet count climbs to this.
pub const BG_DIM_AT: f32 = 1500.0;
pub const BG_DIM_MIN: f32 = 0.5;

// ------------------------------------------------------------------ type and screens

/// Spacing between letters, in font pixels, for the spaced capitals of the sheet.
pub const TRACK: i32 = 2;
/// The big PILGRIM on the title: pixel scale and letter spacing.
pub const TITLE_SCALE: i32 = 3;
pub const TITLE_TRACK: i32 = 6;
/// PRESS START blinks on and off every this many frames.
pub const BLINK_FRAMES: u64 = 40;
/// On-screen messages (CONTROLLER CONNECTED...) last this many frames.
pub const TOAST_FRAMES: u32 = 150;

// ------------------------------------------------------------------ debug

/// The performance overlay averages over this many display frames.
pub const PERF_WINDOW: usize = 60;
