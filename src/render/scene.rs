//! The playfield, in the art bible's draw order: the background (stars and the
//! black hole), enemies, the Pilgrim's shots, the Pilgrim, explosions and
//! particles, then enemy bullets above everything but the HUD.

use super::Renderer;
use super::art::Atlas;
use super::layout::Layout;
use super::palette::{ASH, CREAM, DUSK, SIGNAL, SLATE, TEAL, TEAL_HOT};
use super::ship::{self, Pose};
use crate::sim::{BombBlast, Life, World};
use crate::sim::geom::Rect as SimRect;
use crate::sim::shots::ShotKind;
use crate::sim::stage1::frames;
use crate::tune::{AFTERIMAGE_MIN, AURA_R, BEAM_W, BG_DIM_AT, BG_DIM_MIN, BG_MAX_VALUE, DEBRIS_AT, FIGURE_AT, FLARE, GRAIN, HITBOX_R, HOLE_R, HOLE_X, HOLE_Y, SECTIONS, STAGE_FRAMES};
use macroquad::color::Color;

/// Max opacity of the Pilgrim's shots, so they never read as enemy bullets.
const SHOT_ALPHA: f32 = 0.85;

fn snap(v: f32) -> i32 {
    v.round() as i32
}

/// How far the background dims for this many bullets (1 is not at all).
pub fn dim_for(bullets: usize) -> f32 {
    1.0 - (1.0 - BG_DIM_MIN) * (bullets as f32 / BG_DIM_AT).min(1.0)
}

/// The black hole's brightness ceiling: the background's budget, less room for the
/// film grain, scaled down as bullets crowd in.
pub fn hole_bright(dim: f32) -> f32 {
    (BG_MAX_VALUE - GRAIN * 0.5) * dim
}

/// The black hole's shadow radius `frame`s into the stage: it grows as you approach.
pub fn hole_radius(frame: u64) -> f32 {
    let k = (frame as f32 / STAGE_FRAMES as f32).min(1.0);
    HOLE_R.0 + (HOLE_R.1 - HOLE_R.0) * k
}

/// How hard the disk flares `t` frames into the stage: it swells at the breather,
/// holds, and dies away.
pub fn flare(t: u64) -> f32 {
    let (peak, rise, hold, fall) = FLARE;
    let s = t as f32 / 60.0 - SECTIONS[3].0;
    if s < 0.0 {
        0.0
    } else if s < rise {
        peak * s / rise
    } else if s < rise + hold {
        peak * (1.0 + 0.08 * (s * 5.0).sin())
    } else {
        peak * (1.0 - (s - rise - hold) / fall).max(0.0)
    }
}

/// How hard the disk flares as the hole feeds: through the boss's last phase, and
/// blazing as it swallows the boss.
pub fn feeding(w: &World) -> f32 {
    use crate::sim::boss::Act;
    match w.boss.as_ref().map(|b| (b.act, b.phase, b.t)) {
        Some((Act::Fight | Act::Break, 4, t)) => 0.7 + 0.2 * (t as f32 * 0.05).sin(),
        Some((Act::Falling, _, t)) => 0.9 + 1.4 * (t as f32 / crate::tune::JAWS_FALL as f32),
        _ => 0.0,
    }
}

/// The breather's background extras: debris drifting by, the tiny figure falling.
fn breather(a: &Atlas, ox: i32, oy: i32, t: u32) {
    let (d0, d1) = (frames(DEBRIS_AT.0), frames(DEBRIS_AT.1));
    if (d0..d1).contains(&t) {
        let edge = ((t - d0) as f32 / 90.0).min((d1 - t) as f32 / 90.0).min(1.0);
        super::foes::draw_debris(a, ox, oy, t - d0, edge);
    }
    let (f0, len) = (frames(FIGURE_AT.0), frames(FIGURE_AT.1));
    if (f0..f0 + len).contains(&t) {
        super::foes::draw_figure(a, ox, oy, (t - f0) as f32 / len as f32, (HOLE_X, HOLE_Y));
    }
}

/// `wrecks`: the derelict's wreck during its death, (x, y, 0..1 of the way to the end).
pub fn draw(r: &Renderer, lay: &Layout, w: &World, hitboxes: bool, show_dot: bool, wrecks: &[(f32, f32, f32)]) {
    let a = &r.atlas;
    let (ox, oy) = (lay.field.x, lay.field.y);
    let dim = dim_for(w.hazards.bullets.len());
    let clock = w.clock();
    r.stars.draw(a, lay.field, w.frame, dim);
    r.hole(lay.field, HOLE_X, HOLE_Y, hole_radius(clock), hole_bright(dim), w.frame as f32 / 60.0, flare(clock).max(feeding(w)));
    if let Some(s) = &w.stage {
        breather(a, ox, oy, s.t);
    }
    for &(x, y, k) in wrecks {
        super::foes::draw_wreck(a, ox, oy, x, y, k, w.frame);
    }
    if let Some(b) = &w.boss {
        super::jawsthic::draw(r, b, ox, oy, w.frame);
    }
    super::foes::draw(a, w, ox, oy);

    // enemies
    for (i, d) in w.dummies.iter().enumerate() {
        if !d.alive() {
            continue;
        }
        let k = d.kind as usize;
        let src = a.r.dummy[k];
        let (x, y) = (ox + snap(d.x - src.w * 0.5), oy + snap(d.y - src.h * 0.5));
        a.blit(src, x, y, 1, Color::new(1.0, 1.0, 1.0, 1.0));
        if d.flash > 0 {
            a.blit(a.r.dummy_sil[k], x, y, 1, Color { a: 0.8, ..CREAM });
        }
        // health and damage per second, for tuning
        let bw = src.w as i32;
        let hp = ((d.hp / d.max_hp) * bw as f32).round() as i32;
        let by = y + src.h as i32 + 2;
        a.fill(x, by, bw, 1, DUSK);
        a.fill(x, by, hp, 1, ASH);
        let dps = d.dps();
        if dps > 0.0 || i == 2 {
            let label = format!("DPS {}", dps.round() as i32);
            super::hud::text(a, &label, x, by + 3, 1, 1, SLATE);
        }
    }

    super::bullets::draw_guns(a, w, ox, oy);

    // the Pilgrim's shots: chunky arcade bolts, a little see-through, no outline
    for s in &w.shots {
        let (tx, ty) = s.tip();
        match s.kind {
            ShotKind::Bolt => {
                let src = a.r.ship.bolt[(s.age / 2 % 4) as usize];
                a.blit(src, ox + snap(s.x) - 2, oy + snap(ty), 1, Color { a: SHOT_ALPHA, ..Color::new(1.0, 1.0, 1.0, 1.0) });
            }
            ShotKind::Pod => {
                let src = a.r.ship.pod_shot[(s.age / 2 % 2) as usize];
                let ang = s.vx.atan2(-s.vy);
                let (cx, cy) = ((s.x + tx) * 0.5, (s.y + ty) * 0.5);
                a.blit_rot(src, ox as f32 + cx, oy as f32 + cy, ang, Color::new(1.0, 1.0, 1.0, 0.7));
            }
        }
    }

    let p = &w.player;
    let (px, py) = (ox + snap(p.x), oy + snap(p.y));
    let shown = matches!(w.life, Life::Alive | Life::Struck { .. } | Life::Respawning { .. });
    if let Some(b) = w.beam {
        draw_beam(a, ox, oy, b.x, b.top, p.y + ship::TIP as f32, w.frame);
        draw_aura(a, px, py, w.frame);
    }

    // aiming a slam's next link: brackets lock onto it, a line runs out to it
    if let Some((_, r)) = w.reticle {
        draw_reticle(a, ox, oy, (p.x, p.y), r, w.frame);
    }

    // afterimages at full speed; slamming, the ship's the sword
    let moving = (p.dx * p.dx + p.dy * p.dy).sqrt();
    let dashing = matches!(w.slam, Some(crate::sim::Slam::Dash { .. }));
    let form = if dashing { 1.0 } else { p.blend(&w.tuning) };
    if shown && dashing {
        let at = [1, 2, 4].map(|k| (ox + snap(p.history[k].0), oy + snap(p.history[k].1)));
        ship::draw_afterimages(a, form, 0, &at);
    }
    if shown && !dashing && !p.focused() && moving >= w.tuning.speed * AFTERIMAGE_MIN {
        let at = [2, 4, 6].map(|k| (ox + snap(p.history[k].0), oy + snap(p.history[k].1)));
        ship::draw_afterimages(a, form, p.bank, &at);
    }

    let pose = Pose {
        bank: p.bank,
        form,
        thrust: p.thrust(),
        fire_t: p.fire_t,
        beaming: p.beaming,
        hit_t: p.hit_t,
        locked: if dashing { p.locked.max(1) } else { p.locked },
        frame: w.frame,
    };
    if shown {
        ship::draw(a, px, py, &pose);
        draw_shields(a, w, px, py);
    }
    if let Some(b) = w.bomb {
        draw_bomb_ring(a, ox, oy, b.x, b.y, b.t.saturating_sub(1), w.frame);
    }

    r.parts.draw(a, ox, oy, false);
    r.additive(true);
    r.parts.draw(a, ox, oy, true);
    r.additive(false);
    // a full-screen flash washes over everything but the bullets, so they stay readable
    let flash = r.fx.flash.level();
    if flash > 0.0 {
        let f = lay.field;
        a.fill(f.x, f.y, f.w, f.h, Color { a: 0.35 * flash, ..CREAM });
    }
    // and the boss's death whites the screen out (every bullet's already gone)
    let white = r.fx.white();
    if white > 0.0 {
        let f = lay.field;
        a.fill(f.x, f.y, f.w, f.h, Color { a: white, ..CREAM });
    }
    // the score's popups and grades, under the bullets so they never hide one
    r.popups.draw(a, ox, oy);
    // enemy bullets above everything but the HUD (and the hitbox dot, which is HUD),
    // the boss's laser among them
    if let Some(l) = w.boss.as_ref().and_then(|b| b.laser) {
        super::jawsthic::draw_laser(a, &l, ox, oy, w.frame);
    }
    super::bullets::draw_bullets(a, w, ox, oy);
    if w.alive() {
        let blend = if show_dot { 1.0 } else { p.blend(&w.tuning) };
        ship::draw_hitbox(a, px, py, blend, w.frame);
    }

    if hitboxes {
        let red = Color::new(1.0, 0.2, 0.2, 0.7);
        let box_ = |rc: SimRect, c: Color| {
            let (x0, y0) = (ox + snap(rc.x0), oy + snap(rc.y0));
            let (x1, y1) = (ox + snap(rc.x1), oy + snap(rc.y1));
            a.fill(x0, y0, x1 - x0, 1, c);
            a.fill(x0, y1 - 1, x1 - x0, 1, c);
            a.fill(x0, y0, 1, y1 - y0, c);
            a.fill(x1 - 1, y0, 1, y1 - y0, c);
        };
        for d in w.dummies.iter().filter(|d| d.alive()) {
            box_(d.rect(), Color::new(1.0, 1.0, 0.3, 0.7));
        }
        for e in w.foes.list.iter().filter(|e| e.alive()) {
            let c = if e.hittable() { Color::new(1.0, 1.0, 0.3, 0.7) } else { Color::new(0.5, 0.5, 0.5, 0.5) };
            for p in (0..e.parts.len()).filter(|&p| e.parts[p].alive()) {
                box_(e.rect(p), c);
            }
            box_(e.body(), Color::new(1.0, 0.3, 0.3, 0.6));
        }
        for s in &w.shots {
            box_(s.rect(), Color::new(0.3, 1.0, 0.6, 0.5));
        }
        if let Some(b) = w.beam {
            box_(b.rect(), Color::new(0.3, 1.0, 0.6, 0.5));
        }
        let bl = &w.hazards.bullets;
        for i in 0..bl.len() {
            let k = bl.kind[i];
            let (r, h) = (k.hit_r(), k.half_len());
            let ext = SimRect { x0: bl.x[i] - r - (bl.dx[i] * h).abs(), y0: bl.y[i] - r - (bl.dy[i] * h).abs(), x1: bl.x[i] + r + (bl.dx[i] * h).abs(), y1: bl.y[i] + r + (bl.dy[i] * h).abs() };
            box_(ext, Color::new(0.4, 0.8, 1.0, 0.8));
        }
        box_(w.body(), Color::new(1.0, 1.0, 1.0, 0.35));
        let r = HITBOX_R as i32;
        a.fill(px - r, py - r, 2 * r + 1, 2 * r + 1, red);
    }
}

/// What the Pilgrim's state looks like: a red warning ring while struck (bomb now!),
/// a teal blink while respawn-invincible, a teal shimmer while bomb-invincible.
fn draw_shields(a: &Atlas, w: &World, x: i32, y: i32) {
    let f = w.frame;
    let ring = |r: f32, n: i32, c: Color, spin: f32| {
        for k in 0..n {
            let ang = k as f32 / n as f32 * std::f32::consts::TAU + f as f32 * spin;
            a.fill(x + (ang.cos() * r).round() as i32, y + (ang.sin() * r).round() as i32, 1, 1, c);
        }
    };
    match w.life {
        Life::Struck { left } => {
            let r = 10.0 + (8 - left.min(8)) as f32 * 2.0;
            ring(r, 32, Color { a: if f % 4 < 2 { 0.9 } else { 0.4 }, ..SIGNAL }, 0.0);
        }
        _ if w.invuln > 0 && w.invuln_blink => {
            if (f / 3) % 2 == 0 {
                ship::draw_silhouette(a, w.player.blend(&w.tuning), 0, x, y, Color { a: 0.55, ..TEAL });
            }
        }
        _ if w.invuln > 0 => ring(20.0 + (f % 8) as f32 * 0.25, 24, Color { a: 0.3, ..TEAL }, 0.05),
        _ => {}
    }
}

/// The bomb's cancel ring: a teal band sweeping out across the screen, a cream edge
/// inside it, fading once it's covered everything.
fn draw_bomb_ring(a: &Atlas, ox: i32, oy: i32, bx: f32, by: f32, t: u32, frame: u64) {
    use crate::tune::{BOMB_ACTIVE, BOMB_GROW};
    let r = BombBlast::radius(t);
    let fade = if t < BOMB_GROW { 1.0 } else { 1.0 - (t - BOMB_GROW) as f32 / (BOMB_ACTIVE - BOMB_GROW) as f32 };
    let n = ((r * std::f32::consts::TAU) / 1.5).clamp(24.0, 1400.0) as i32;
    for k in 0..n {
        let ang = k as f32 / n as f32 * std::f32::consts::TAU;
        let (c, s) = (ang.cos(), ang.sin());
        for (dr, col, alpha) in [(0.0, TEAL, 0.85), (-2.0, TEAL, 0.45), (-4.0, CREAM, 0.3)] {
            if dr < -1.0 && (k + frame as i32) % 2 != 0 {
                continue; // dithered inner edge
            }
            let rr = r + dr;
            let (px, py) = (bx + c * rr, by + s * rr);
            if px < 0.0 || py < 0.0 || px >= crate::tune::FIELD_W as f32 || py >= crate::tune::FIELD_H as f32 {
                continue; // off the playfield
            }
            a.fill(ox + px.round() as i32, oy + py.round() as i32, 1, 1, Color { a: alpha * fade, ..col });
        }
    }
}

/// The focus beam, the laser from the sword's tip: a translucent teal column with
/// a hot core, bright dashes crawling up it.
pub fn draw_beam(a: &Atlas, ox: i32, oy: i32, x: f32, top: f32, nose: f32, frame: u64) {
    let (y0, y1) = (top.max(0.0), nose);
    if y1 <= y0 {
        return;
    }
    let cx = ox + snap(x);
    let (top_px, h) = (oy + snap(y0), snap(y1 - y0));
    let wob = ((frame / 2) % 3) as i32 - 1; // the width breathes a pixel
    let half = (BEAM_W * 0.5) as i32;
    a.fill(cx - half - wob.max(0), top_px, 2 * (half + wob.max(0)), h, Color { a: 0.16, ..TEAL });
    a.fill(cx - half + 2, top_px, 2 * half - 4, h, Color { a: 0.3, ..TEAL });
    a.fill(cx - 1 - (wob == 1) as i32, top_px, 2 + 2 * (wob == 1) as i32, h, Color { a: 0.6, ..TEAL_HOT });
    // bright dashes running up the beam
    let step = 14;
    let off = (frame as i32 * 7) % step;
    let mut y = top_px + h - off;
    while y > top_px {
        a.fill(cx - 3, y, 6, 2, Color { a: 0.3, ..TEAL_HOT });
        y -= step;
    }
    // where it lands it hits: it flattens against what it hit in a splash wider than
    // the beam, flickering, and a flare swells and shrinks round a hot core
    if top > 0.0 {
        let spread = half + 3 + (frame % 3) as i32;
        a.fill(cx - spread, top_px - 1, 2 * spread, 1, Color { a: 0.5, ..TEAL });
        a.fill(cx - spread + 2, top_px, 2 * spread - 4, 2, Color { a: 0.75, ..TEAL_HOT });
        a.fill(cx - spread + 4, top_px + 2, 2 * spread - 8, 1, Color { a: 0.4, ..TEAL });
        let big = frame % 6 < 3;
        let s = 1 + big as i32;
        let flare = if big { 0.55 } else { 0.7 };
        a.blit(a.r.sparkle, cx - 3 * s - (s - 1), top_px - 3 * s - (s - 1), s, Color { a: flare, ..TEAL_HOT });
        let r = 2 + (frame % 3 == 0) as usize;
        a.blit(a.r.discs[r - 1], cx - r as i32 - 1, top_px - r as i32 - 1, 1, Color { a: 0.75, ..TEAL_HOT });
        a.fill(cx - half, top_px - 1, 2 * half, 2, Color { a: 0.4, ..TEAL });
    }
}

/// The slam's reticle on the target its next link would strike: teal corner brackets
/// closing in and pulsing, and a dashed line from the ship out to it.
fn draw_reticle(a: &Atlas, ox: i32, oy: i32, from: (f32, f32), r: SimRect, frame: u64) {
    let pulse = (frame / 4 % 2) as i32;
    let (x0, y0, x1, y1) = (ox + snap(r.x0) - 3 - pulse, oy + snap(r.y0) - 3 - pulse, ox + snap(r.x1) + 2 + pulse, oy + snap(r.y1) + 2 + pulse);
    let c = Color { a: 0.9, ..TEAL_HOT };
    let arm = 5;
    for (x, y, dx, dy) in [(x0, y0, 1, 1), (x1, y0, -1, 1), (x0, y1, 1, -1), (x1, y1, -1, -1)] {
        a.fill(x.min(x + dx * arm), y, arm, 1, c);
        a.fill(x, y.min(y + dy * arm), 1, arm, c);
    }
    let (tx, ty) = ((r.x0 + r.x1) * 0.5, (r.y0 + r.y1) * 0.5);
    let d = (tx - from.0).hypot(ty - from.1);
    let steps = (d / 2.0) as i32;
    for k in 2..steps {
        if (k + (frame as i32 / 2)) % 4 < 2 {
            let t = k as f32 / steps as f32;
            let (x, y) = (from.0 + (tx - from.0) * t, from.1 + (ty - from.1) * t);
            a.fill(ox + snap(x), oy + snap(y), 1, 1, Color { a: 0.55, ..TEAL });
        }
    }
}

/// The point-blank aura: a teal ring of dots turning around the ship.
fn draw_aura(a: &Atlas, x: i32, y: i32, frame: u64) {
    let n = 28;
    for k in 0..n {
        if (k + (frame / 3) as i32) % 4 == 0 {
            continue;
        }
        let ang = k as f32 / n as f32 * std::f32::consts::TAU - frame as f32 * 0.05;
        let (dx, dy) = ((ang.cos() * AURA_R).round() as i32, (ang.sin() * AURA_R).round() as i32);
        a.fill(x + dx, y + dy, 1, 1, Color { a: 0.35, ..TEAL });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hole_grows_across_the_stage_and_the_sky_dims_with_bullets() {
        assert_eq!(hole_radius(0), HOLE_R.0);
        assert!(hole_radius(STAGE_FRAMES / 2) > HOLE_R.0);
        assert_eq!(hole_radius(STAGE_FRAMES * 3), HOLE_R.1);
        assert_eq!(dim_for(0), 1.0);
        assert!((dim_for(100_000) - BG_DIM_MIN).abs() < 1e-6, "down to half: 30% becomes 15%");
        assert!(hole_bright(1.0) + GRAIN * 0.5 <= BG_MAX_VALUE + 1e-6, "grain and all, the hole stays in budget");
    }

    #[test]
    fn the_disk_flares_through_the_breather_and_settles() {
        let at = |s: f32| flare(frames(s) as u64);
        assert_eq!(at(SECTIONS[3].0 - 1.0), 0.0);
        assert!(at(SECTIONS[3].0 + FLARE.1 + 1.0) > FLARE.0 * 0.9);
        assert_eq!(at(SECTIONS[4].0 + 10.0), 0.0, "calm again for the derelict... after it rises");
    }
}
