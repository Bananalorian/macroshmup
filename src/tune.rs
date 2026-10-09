//! Every tunable number in one place. Gameplay timings are in frames of the
//! fixed 60 Hz simulation, distances in low-res canvas pixels.

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

// ------------------------------------------------------------------ type and screens

/// Spacing between letters, in font pixels, for the spaced capitals of the sheet.
pub const TRACK: i32 = 2;
/// The big PILGRIM on the title: pixel scale and letter spacing.
pub const TITLE_SCALE: i32 = 3;
pub const TITLE_TRACK: i32 = 6;
/// PRESS START blinks on and off every this many frames.
pub const BLINK_FRAMES: u64 = 40;

// ------------------------------------------------------------------ debug

/// The performance overlay averages over this many display frames.
pub const PERF_WINDOW: usize = 60;
