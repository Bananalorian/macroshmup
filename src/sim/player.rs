//! The Pilgrim. No inertia, ever: input moves the ship on the frame it's read,
//! diagonals are normalized, and focus slows it over a few frames.

use crate::tune::*;

/// What the player is asking for on one frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Controls {
    /// Direction from keys or a stick; anything longer than 1 is normalized.
    pub mv: (f32, f32),
    /// Touch steering: px of finger travel this frame. Moves the ship 1:1, capped at its speed.
    pub drag: (f32, f32),
    pub fire: bool,
    pub focus: bool,
    /// Pressed this frame.
    pub bomb: bool,
}

/// The numbers the tuning panel can change while you play.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tuning {
    pub speed: f32,
    pub focus_ratio: f32,
    pub focus_blend: u32,
}

impl Default for Tuning {
    fn default() -> Self {
        Tuning { speed: MOVE_SPEED, focus_ratio: FOCUS_RATIO, focus_blend: FOCUS_BLEND }
    }
}

/// Positions kept for afterimages, newest first.
pub const HISTORY: usize = 8;

pub struct Player {
    /// The hitbox center (the cross), sub-pixel; drawn snapped to the grid.
    pub x: f32,
    pub y: f32,
    /// How far the ship moved this frame.
    pub dx: f32,
    pub dy: f32,
    /// Frames into the focus blend, 0 (full speed) to the tuning's blend frames.
    pub focus_t: u32,
    /// Roll, -2 (hard left) to 2 (hard right).
    pub bank: i8,
    bank_t: u32,
    /// The wing pods' offsets from the hitbox, left then right. They lag a little.
    pub pods: [(f32, f32); 2],
    pub fire_cd: u32,
    /// Shooting the wide shot / the beam this frame.
    pub firing: bool,
    pub beaming: bool,
    /// Frames spent shooting in a row (drives the fire animation).
    pub fire_t: u32,
    pub volley: u32,
    /// Hull flash frames left, and frames until it can flash again.
    pub hit_t: u32,
    pub bump_cd: u32,
    pub history: [(f32, f32); HISTORY],
}

impl Player {
    pub fn new() -> Player {
        Player {
            x: START_X,
            y: START_Y,
            dx: 0.0,
            dy: 0.0,
            focus_t: 0,
            bank: 0,
            bank_t: 0,
            pods: [(-POD_X, 0.0), (POD_X, 0.0)],
            fire_cd: 0,
            firing: false,
            beaming: false,
            fire_t: 0,
            volley: 0,
            hit_t: 0,
            bump_cd: 0,
            history: [(START_X, START_Y); HISTORY],
        }
    }

    /// How far into focus the ship is, 0..1.
    pub fn blend(&self, t: &Tuning) -> f32 {
        if t.focus_blend == 0 {
            if self.focus_t > 0 { 1.0 } else { 0.0 }
        } else {
            (self.focus_t as f32 / t.focus_blend as f32).min(1.0)
        }
    }

    /// Speed this frame, px per frame.
    pub fn speed(&self, t: &Tuning) -> f32 {
        t.speed * (1.0 - self.blend(t) * (1.0 - t.focus_ratio))
    }

    pub fn focused(&self) -> bool {
        self.focus_t > 0
    }

    /// Thrusting: moving up the screen this frame.
    pub fn thrust(&self) -> bool {
        self.dy < -0.01
    }

    /// One frame of movement. Everything here happens on the frame the input arrives.
    pub fn step(&mut self, c: &Controls, t: &Tuning) {
        // focus first, so this frame's speed already reflects it
        let top = t.focus_blend.max(1);
        self.focus_t = if c.focus { (self.focus_t + 1).min(top) } else { self.focus_t.saturating_sub(1).min(top) };
        let speed = self.speed(t);

        let (mut dx, mut dy) = if c.drag != (0.0, 0.0) {
            // touch: 1:1 with the finger, never faster than the ship can fly
            let len = (c.drag.0 * c.drag.0 + c.drag.1 * c.drag.1).sqrt();
            let k = if len > speed { speed / len } else { 1.0 };
            (c.drag.0 * k, c.drag.1 * k)
        } else {
            let (mx, my) = c.mv;
            let len = (mx * mx + my * my).sqrt();
            let k = if len > 1.0 { 1.0 / len } else { 1.0 };
            (mx * k * speed, my * k * speed)
        };
        let (ox, oy) = (self.x, self.y);
        self.x = (self.x + dx).clamp(EDGE_X, FIELD_W as f32 - EDGE_X);
        self.y = (self.y + dy).clamp(EDGE_TOP, FIELD_H as f32 - EDGE_BOTTOM);
        dx = self.x - ox;
        dy = self.y - oy;
        self.dx = dx;
        self.dy = dy;

        // the pods chase where they should be, so they trail slightly
        let k = self.blend(t);
        let px = POD_X + (POD_X_FOCUS - POD_X) * k;
        for (i, side) in [-1.0f32, 1.0].iter().enumerate() {
            let (want_x, want_y) = (self.x + side * px, self.y);
            let (mut ax, mut ay) = (ox + self.pods[i].0, oy + self.pods[i].1);
            ax += (want_x - ax) * POD_FOLLOW;
            ay += (want_y - ay) * POD_FOLLOW;
            let (lx, ly) = (ax - want_x, ay - want_y);
            let lag = (lx * lx + ly * ly).sqrt();
            if lag > POD_MAX_LAG {
                ax = want_x + lx / lag * POD_MAX_LAG;
                ay = want_y + ly / lag * POD_MAX_LAG;
            }
            self.pods[i] = (ax - self.x, ay - self.y);
        }

        // roll toward the sideways speed, one step every few frames
        let want = ((dx / t.speed.max(0.1)) * 2.0).round().clamp(-2.0, 2.0) as i8;
        self.bank_t += 1;
        if want != self.bank && self.bank_t >= BANK_FRAMES {
            self.bank += (want - self.bank).signum();
            self.bank_t = 0;
        }

        self.history.rotate_right(1);
        self.history[0] = (self.x, self.y);
        self.hit_t = self.hit_t.saturating_sub(1);
        self.bump_cd = self.bump_cd.saturating_sub(1);
    }

    /// Where the pods' guns are this frame.
    pub fn pod_guns(&self) -> [(f32, f32); 2] {
        [(self.x + self.pods[0].0, self.y + self.pods[0].1), (self.x + self.pods[1].0, self.y + self.pods[1].1)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn held(mv: (f32, f32)) -> Controls {
        Controls { mv, ..Default::default() }
    }

    fn moved(p: &Player) -> f32 {
        (p.dx * p.dx + p.dy * p.dy).sqrt()
    }

    #[test]
    fn input_moves_the_ship_on_the_tick_it_is_read() {
        let t = Tuning::default();
        let mut p = Player::new();
        p.step(&held((1.0, 0.0)), &t);
        assert_eq!(p.x, START_X + MOVE_SPEED, "moved the full step on the first frame");
        p.step(&held((0.0, 0.0)), &t);
        assert_eq!(p.x, START_X + MOVE_SPEED, "and stopped on the frame it was let go");
    }

    #[test]
    fn diagonals_are_the_same_speed_as_straight_lines() {
        let t = Tuning::default();
        for mv in [(1.0, 1.0), (-1.0, 1.0), (1.0, -1.0), (-1.0, -1.0), (1.0, 0.0), (0.0, -1.0)] {
            let mut p = Player::new();
            p.step(&held(mv), &t);
            assert!((moved(&p) - MOVE_SPEED).abs() < 1e-5, "{mv:?} moved {}", moved(&p));
        }
    }

    #[test]
    fn there_is_no_acceleration_or_drift() {
        let t = Tuning::default();
        let mut p = Player::new();
        p.x = 30.0; // room to fly right for a while
        for i in 0..40 {
            p.step(&held((1.0, 0.0)), &t);
            assert_eq!(p.dx, MOVE_SPEED, "frame {i}");
        }
        p.step(&held((0.0, 0.0)), &t);
        assert_eq!((p.dx, p.dy), (0.0, 0.0));
    }

    #[test]
    fn the_focus_blend_finishes_in_five_frames() {
        let t = Tuning::default();
        assert_eq!(t.focus_blend, 5);
        let focus = MOVE_SPEED * FOCUS_RATIO;
        let mut p = Player::new();
        let focused = Controls { focus: true, ..Default::default() };
        let mut speeds = vec![];
        for _ in 0..6 {
            p.step(&focused, &t);
            speeds.push(p.speed(&t));
        }
        assert!(speeds.windows(2).take(4).all(|w| w[1] < w[0]), "{speeds:?}");
        assert!(speeds[3] > focus, "not there after 4 frames");
        assert!((speeds[4] - focus).abs() < 1e-6, "there after 5: {speeds:?}");
        assert!((speeds[5] - focus).abs() < 1e-6);
        for _ in 0..5 {
            p.step(&Controls::default(), &t);
        }
        assert_eq!(p.speed(&t), MOVE_SPEED, "back to full speed in 5 frames");
    }

    #[test]
    fn blending_applies_to_the_frame_focus_is_pressed() {
        let t = Tuning::default();
        let mut p = Player::new();
        p.step(&Controls { mv: (1.0, 0.0), focus: true, ..Default::default() }, &t);
        let want = MOVE_SPEED * (1.0 - 0.2 * (1.0 - FOCUS_RATIO));
        assert!((p.dx - want).abs() < 1e-6);
    }

    #[test]
    fn a_zero_frame_blend_is_instant() {
        let t = Tuning { focus_blend: 0, ..Default::default() };
        let mut p = Player::new();
        p.step(&Controls { mv: (1.0, 0.0), focus: true, ..Default::default() }, &t);
        assert!((p.dx - MOVE_SPEED * FOCUS_RATIO).abs() < 1e-5, "{} vs {}", p.dx, MOVE_SPEED * FOCUS_RATIO);
    }

    #[test]
    fn half_a_stick_is_half_speed() {
        let t = Tuning::default();
        let mut p = Player::new();
        p.step(&held((0.5, 0.0)), &t);
        assert_eq!(p.dx, MOVE_SPEED * 0.5);
    }

    #[test]
    fn touch_drags_one_to_one_up_to_full_speed() {
        let t = Tuning::default();
        let mut p = Player::new();
        p.step(&Controls { drag: (1.5, -1.0), ..Default::default() }, &t);
        assert_eq!((p.dx, p.dy), (1.5, -1.0));
        p.step(&Controls { drag: (30.0, 40.0), ..Default::default() }, &t);
        assert!((moved(&p) - MOVE_SPEED).abs() < 1e-5, "capped at the ship's speed");
    }

    #[test]
    fn the_ship_stays_on_the_playfield() {
        let t = Tuning::default();
        let mut p = Player::new();
        for _ in 0..200 {
            p.step(&held((-1.0, -1.0)), &t);
        }
        assert_eq!((p.x, p.y), (EDGE_X, EDGE_TOP));
        for _ in 0..200 {
            p.step(&held((1.0, 1.0)), &t);
        }
        assert_eq!((p.x, p.y), (FIELD_W as f32 - EDGE_X, FIELD_H as f32 - EDGE_BOTTOM));
        assert_eq!(p.dx, 0.0, "no movement reported against a wall");
    }

    #[test]
    fn pods_lag_a_little_then_settle_and_tuck_in_for_focus() {
        let t = Tuning::default();
        let mut p = Player::new();
        p.x = 40.0;
        for _ in 0..20 {
            p.step(&held((1.0, 0.0)), &t);
            let lag = p.pods[1].0 - POD_X;
            assert!(lag < 0.0 && lag >= -POD_MAX_LAG - 1e-4, "the right pod trails: {lag}");
        }
        for _ in 0..30 {
            p.step(&held((0.0, 0.0)), &t);
        }
        assert!((p.pods[0].0 + POD_X).abs() < 0.01 && (p.pods[1].0 - POD_X).abs() < 0.01);
        for _ in 0..30 {
            p.step(&Controls { focus: true, ..Default::default() }, &t);
        }
        assert!((p.pods[1].0 - POD_X_FOCUS).abs() < 0.01, "tucked in: {:?}", p.pods);
    }

    #[test]
    fn the_ship_banks_with_its_sideways_speed() {
        let t = Tuning::default();
        let mut p = Player::new();
        p.x = 30.0;
        for _ in 0..20 {
            p.step(&held((1.0, 0.0)), &t);
        }
        assert_eq!(p.bank, 2);
        for _ in 0..20 {
            p.step(&Controls { mv: (-1.0, 0.0), focus: true, ..Default::default() }, &t);
        }
        assert_eq!(p.bank, -1, "focused, it only rolls halfway");
        for _ in 0..20 {
            p.step(&held((0.0, 0.0)), &t);
        }
        assert_eq!(p.bank, 0);
    }

    #[test]
    fn afterimage_history_is_newest_first() {
        let t = Tuning::default();
        let mut p = Player::new();
        p.step(&held((1.0, 0.0)), &t);
        p.step(&held((1.0, 0.0)), &t);
        assert_eq!(p.history[0].0, p.x);
        assert_eq!(p.history[1].0, p.x - MOVE_SPEED);
    }
}
