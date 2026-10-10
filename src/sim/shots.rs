//! The Pilgrim's shots. Long, with generous hitboxes, and fired from low on the
//! ship so a target you're sitting right on still takes the hit.

use super::geom::Rect;
use super::player::Player;
use crate::tune::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShotKind {
    /// The twin center bolts.
    Bolt,
    /// The angled streams from the wing pods.
    Pod,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shot {
    pub kind: ShotKind,
    /// The tail of the shot; the tip is `len` px ahead along its velocity.
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub len: f32,
    pub hit_w: f32,
    pub dmg: f32,
    pub age: u32,
}

impl Shot {
    pub fn tip(&self) -> (f32, f32) {
        let v = (self.vx * self.vx + self.vy * self.vy).sqrt().max(1e-6);
        (self.x + self.vx / v * self.len, self.y + self.vy / v * self.len)
    }

    /// The box the shot covers this frame, tail to tip, `hit_w` wide.
    pub fn rect(&self) -> Rect {
        let (tx, ty) = self.tip();
        let h = self.hit_w * 0.5;
        Rect { x0: self.x.min(tx) - h, y0: self.y.min(ty), x1: self.x.max(tx) + h, y1: self.y.max(ty) }
    }

    /// Off the playfield for good.
    pub fn gone(&self) -> bool {
        let (tx, ty) = self.tip();
        self.y < 0.0 && ty < 0.0 || tx < -20.0 || tx > FIELD_W as f32 + 20.0 || self.y > FIELD_H as f32 + 20.0
    }
}

/// One volley of the wide shot: two bolts down the middle, one shot from each pod.
pub fn volley(p: &Player, out: &mut Vec<Shot>) {
    let mess = if p.volley % 2 == 0 { -0.5 } else { 0.5 }; // "a little messy"
    let y = p.y + MUZZLE_Y;
    for side in [-1.0, 1.0] {
        out.push(Shot {
            kind: ShotKind::Bolt,
            x: p.x + side * BOLT_GAP + mess,
            y,
            vx: 0.0,
            vy: -BOLT_SPEED,
            len: BOLT_LEN,
            hit_w: BOLT_HIT_W,
            dmg: BOLT_DMG,
            age: 0,
        });
    }
    let a = POD_ANGLES[(p.volley % 2) as usize].to_radians();
    for (i, (gx, gy)) in p.pod_guns().into_iter().enumerate() {
        let side = if i == 0 { -1.0 } else { 1.0 };
        out.push(Shot {
            kind: ShotKind::Pod,
            x: gx,
            y: gy + MUZZLE_Y,
            vx: side * a.sin() * POD_SPEED,
            vy: -a.cos() * POD_SPEED,
            len: POD_LEN,
            hit_w: POD_HIT_W,
            dmg: POD_DMG,
            age: 0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_volley_starts_at_the_ship_and_reaches_above_it() {
        let p = Player::new();
        let mut v = vec![];
        volley(&p, &mut v);
        assert_eq!(v.len(), 4);
        for s in &v {
            let r = s.rect();
            assert!(r.y1 >= p.y, "starts at or below the hitbox (no dead zone at the muzzle)");
            assert!(r.y0 < p.y - 6.0, "and reaches up the ship");
        }
        assert!(v.iter().filter(|s| s.kind == ShotKind::Bolt).all(|s| s.rect().y0 < p.y - 10.0), "bolts reach the nose");
        assert_eq!(v.iter().filter(|s| s.kind == ShotKind::Pod).count(), 2);
        assert!(v.iter().filter(|s| s.kind == ShotKind::Pod).all(|s| s.vx.signum() == (s.x - p.x).signum()), "pods angle outward");
    }

    #[test]
    fn shots_are_long_enough_to_never_skip_a_target() {
        assert!(BOLT_LEN >= BOLT_SPEED);
        assert!(POD_LEN >= POD_SPEED);
    }
}
