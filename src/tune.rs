//! Every tunable number in one place. Gameplay timings are in frames of the
//! fixed 60 Hz simulation, distances in low-res canvas pixels, speeds in px per
//! frame, angles in degrees (0 is right, 90 is straight down the screen).

use crate::sim::bullets::{Behavior, Kind};
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
/// The playfield is the same size (3:4) in every layout, so the game plays the same everywhere.
pub const FIELD_W: u32 = 270;
pub const FIELD_H: u32 = 360;
/// Portrait (phones): a HUD strip above the playfield and a thumb zone below it.
pub const PORTRAIT_W: u32 = 270;
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
pub const START_X: f32 = 135.0;
pub const START_Y: f32 = 300.0;

/// Wing pods sit this far either side of the center, closer while focused.
pub const POD_X: f32 = 11.0;
pub const POD_X_FOCUS: f32 = 7.0;
/// Pods close this share of the gap to where they should be each frame (the lag).
pub const POD_FOLLOW: f32 = 0.6;
/// They never trail by more than this many px.
pub const POD_MAX_LAG: f32 = 3.0;
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
pub const BOLT_GAP: f32 = 4.0;
pub const BOLT_DMG: f32 = 3.0;
/// The pod streams, angled outward (alternating between two angles, in degrees).
pub const POD_SPEED: f32 = 13.0;
pub const POD_LEN: f32 = 14.0;
pub const POD_HIT_W: f32 = 6.0;
pub const POD_ANGLES: [f32; 2] = [4.0, 9.0];
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
pub const DUMMY_SWING: (f32, u32) = (80.0, 300);
/// Hit flash: frames lit, and the minimum gap between flashes.
pub const HIT_FLASH: (u8, u8) = (2, 6);
/// Bumping a dummy flashes the hull (no damage yet); frames of flash and cooldown.
pub const BUMP_FLASH: u32 = 12;
pub const BUMP_COOLDOWN: u32 = 40;

// ------------------------------------------------------------------ enemy bullets

/// Pool capacity (never reallocated during play).
pub const BULLET_CAP: usize = 8192;
/// Sprite radius and, for rice and needles, the half-length of the core along
/// the flight direction. Indexed by `Kind` (small, orb, large, rice, needle).
pub const BULLET_SIZE: [(f32, f32); 5] = [(3.5, 0.0), (5.5, 0.0), (9.5, 0.0), (2.5, 2.0), (1.5, 5.5)];
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
/// The placeholder death: frames the patterns hold off after a hit.
pub const DEATH_PAUSE: u32 = 60;
/// Red muzzle flash (telegraph) frames on an emitter after it fires.
pub const MUZZLE_FRAMES: u32 = 4;
/// At most 3 full-screen flashes a second.
pub const FLASH_GAP: u64 = 20;
pub const FLASH_FRAMES: u32 = 8;

// ------------------------------------------------------------------ the pattern sandbox
// Each preset is one or more guns. The sandbox cycles through them (F7, [ ], LB,
// the NEXT button or the pause menu). Speeds are px per frame.

const fn gun(mount: Mount, shape: Shape, kind: Kind, speed: f32, every: u32, delay: u32, behavior: Behavior) -> Gun {
    Gun { mount, shape, kind, speed, every, delay, behavior }
}
const STRAIGHT: Behavior = Behavior::Straight;

pub const SANDBOX: &[Preset] = &[
    Preset { name: "OFF", guns: &[], fill: 0 },
    Preset {
        name: "AIMED FANS",
        guns: &[
            gun(Mount::Dummy(1), Shape::Fan { n: 5, spread: 48.0, aimed: true }, Kind::Orb, 2.6, 50, 30, STRAIGHT),
            gun(Mount::At(60.0, 40.0), Shape::Fan { n: 3, spread: 24.0, aimed: true }, Kind::Small, 3.2, 70, 60, STRAIGHT),
            gun(Mount::At(210.0, 40.0), Shape::Fan { n: 3, spread: 24.0, aimed: true }, Kind::Small, 3.2, 70, 95, STRAIGHT),
        ],
        fill: 0,
    },
    Preset {
        name: "RINGS",
        guns: &[
            gun(Mount::At(135.0, 56.0), Shape::Ring { n: 20, turn: 9.0 }, Kind::Small, 1.8, 36, 20, STRAIGHT),
            gun(Mount::At(66.0, 70.0), Shape::Ring { n: 14, turn: -6.0 }, Kind::Orb, 1.5, 60, 38, STRAIGHT),
            gun(Mount::At(204.0, 70.0), Shape::Ring { n: 14, turn: 6.0 }, Kind::Orb, 1.5, 60, 68, STRAIGHT),
        ],
        fill: 0,
    },
    Preset {
        name: "SPIRAL",
        guns: &[gun(Mount::At(135.0, 70.0), Shape::Spiral { arms: 3, spin: 11.0, flip: 40 }, Kind::Rice, 2.2, 5, 20, STRAIGHT)],
        fill: 0,
    },
    Preset {
        name: "STREAMS",
        guns: &[
            gun(Mount::At(80.0, 46.0), Shape::Stream { burst: 8, gap: 4 }, Kind::Needle, 4.5, 75, 20, STRAIGHT),
            gun(Mount::At(190.0, 46.0), Shape::Stream { burst: 8, gap: 4 }, Kind::Needle, 4.5, 75, 57, STRAIGHT),
        ],
        fill: 0,
    },
    Preset {
        name: "RANDOM CONE",
        guns: &[gun(Mount::Sweep { y: 40.0, x0: 70.0, x1: 200.0, period: 360 }, Shape::Cone { n: 3, spread: 70.0, jitter: 0.25 }, Kind::Small, 2.4, 6, 20, STRAIGHT)],
        fill: 0,
    },
    Preset {
        name: "ACCELERATE",
        guns: &[gun(Mount::At(135.0, 70.0), Shape::Ring { n: 16, turn: 7.0 }, Kind::Orb, 0.6, 45, 20, Behavior::Accel { accel: 0.05, max: 3.6 })],
        fill: 0,
    },
    Preset {
        name: "SLOW, PAUSE, AIM",
        guns: &[gun(Mount::At(135.0, 80.0), Shape::Ring { n: 12, turn: 15.0 }, Kind::Orb, 3.0, 70, 20, Behavior::SlowAim { slow: 30, pause: 20, speed: 3.2 })],
        fill: 0,
    },
    Preset {
        name: "CURVE",
        guns: &[
            gun(Mount::At(135.0, 56.0), Shape::Fan { n: 5, spread: 100.0, aimed: false }, Kind::Rice, 2.0, 24, 20, Behavior::Curve { turn: 0.7, frames: 120 }),
            gun(Mount::At(135.0, 56.0), Shape::Fan { n: 5, spread: 100.0, aimed: false }, Kind::Rice, 2.0, 24, 32, Behavior::Curve { turn: -0.7, frames: 120 }),
        ],
        fill: 0,
    },
    Preset {
        name: "SPLIT",
        guns: &[gun(
            Mount::At(135.0, 50.0),
            Shape::Fan { n: 3, spread: 60.0, aimed: true },
            Kind::Large,
            1.6,
            80,
            20,
            Behavior::Split { at: 70, n: 10, kind: Kind::Small, speed: 2.0 },
        )],
        fill: 0,
    },
    Preset {
        name: "PULL",
        guns: &[
            gun(Mount::At(40.0, 60.0), Shape::Ring { n: 10, turn: 13.0 }, Kind::Small, 2.2, 30, 20, Behavior::Pull { x: 135.0, y: 120.0, pull: 0.035, max: 3.0 }),
            gun(Mount::At(230.0, 60.0), Shape::Ring { n: 10, turn: -13.0 }, Kind::Small, 2.2, 30, 35, Behavior::Pull { x: 135.0, y: 120.0, pull: 0.035, max: 3.0 }),
        ],
        fill: 0,
    },
    Preset {
        name: "LAYERED",
        guns: &[
            gun(Mount::At(135.0, 56.0), Shape::Ring { n: 18, turn: 10.0 }, Kind::Small, 1.6, 40, 20, STRAIGHT),
            gun(Mount::Dummy(1), Shape::Fan { n: 3, spread: 20.0, aimed: true }, Kind::Orb, 3.0, 30, 40, STRAIGHT),
        ],
        fill: 0,
    },
    Preset { name: "STRESS 2000", guns: &[], fill: 2000 },
    Preset { name: "STRESS 4000", guns: &[], fill: 4000 },
];

// ------------------------------------------------------------------ the bot
// A dodger: tries 9 directions at two speeds, looking this many frames ahead.

pub const BOT_LOOKAHEAD: usize = 12;
/// Where it likes to sit, and how much it cares.
pub const BOT_HOME: (f32, f32) = (135.0, 285.0);
pub const BOT_HOME_PULL: f32 = 0.1;
/// A small bonus for keeping the same choice (so it doesn't dither).
pub const BOT_STICKY: f32 = 0.3;
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

/// Rumble: (strong motor, weak motor, ms). Continuous ones repeat while held.
pub const RUMBLE_FIRE: (f32, f32, u32) = (0.0, 0.07, 90);
pub const RUMBLE_BEAM: (f32, f32, u32) = (0.10, 0.18, 90);
pub const RUMBLE_BREAK: (f32, f32, u32) = (0.35, 0.45, 140);
pub const RUMBLE_BIG_BREAK: (f32, f32, u32) = (0.8, 0.7, 380);
pub const RUMBLE_BUMP: (f32, f32, u32) = (0.25, 0.4, 90);
pub const RUMBLE_BOMB: (f32, f32, u32) = (1.0, 0.8, 450);
pub const RUMBLE_CONNECT: (f32, f32, u32) = (0.3, 0.3, 160);
pub const RUMBLE_DEATH: (f32, f32, u32) = (0.9, 0.8, 350);
pub const RUMBLE_GRAZE: (f32, f32, u32) = (0.0, 0.12, 35);
/// Continuous rumble is re-sent every this many frames.
pub const RUMBLE_REPEAT: u64 = 4;
/// "Low" rumble scales everything by this.
pub const RUMBLE_LOW: f32 = 0.5;

// ------------------------------------------------------------------ sound

/// Seconds between plays of each sound, so rapid fire never machine-guns.
pub const SHOT_SOUND_GAP: f64 = 0.095;
pub const HIT_SOUND_GAP: f64 = 0.06;
/// Mix levels.
pub const VOL_SHOT: f32 = 0.16;
pub const VOL_HIT: f32 = 0.10;
pub const VOL_BEAM: f32 = 0.22;
pub const VOL_BREAK: f32 = 0.40;
pub const VOL_BIG_BREAK: f32 = 0.65;
pub const VOL_UI: f32 = 0.35;
pub const VOL_GRAZE: f32 = 0.07;
pub const GRAZE_SOUND_GAP: f64 = 0.05;
pub const VOL_DEATH: f32 = 0.55;
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
