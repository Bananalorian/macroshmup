//! The playfield: target dummies, the Pilgrim's shots and beam, the Pilgrim, and
//! the particles, in the art bible's draw order.

use super::Renderer;
use super::art::Atlas;
use super::layout::Layout;
use super::palette::{ASH, CREAM, DUSK, SIGNAL, SLATE, TEAL};
use super::ship::{self, Pose};
use crate::sim::World;
use crate::sim::geom::Rect as SimRect;
use crate::sim::shots::ShotKind;
use crate::tune::{AFTERIMAGE_MIN, AURA_R, BEAM_W, HITBOX_R};
use macroquad::color::Color;

/// Max opacity of the Pilgrim's shots, so they never read as enemy bullets.
const SHOT_ALPHA: f32 = 0.6;

fn snap(v: f32) -> i32 {
    v.round() as i32
}

pub fn draw(r: &Renderer, lay: &Layout, w: &World, hitboxes: bool) {
    let a = &r.atlas;
    let (ox, oy) = (lay.field.x, lay.field.y);
    r.stars.draw(a, lay.field, w.frame);

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

    // the Pilgrim's shots: translucent, streaked, no outline
    for s in &w.shots {
        let (tx, ty) = s.tip();
        match s.kind {
            ShotKind::Bolt => {
                let src = a.r.ship.bolt[(s.age / 2 % 4) as usize];
                a.blit(src, ox + snap(s.x) - 1, oy + snap(ty), 1, Color { a: SHOT_ALPHA, ..Color::new(1.0, 1.0, 1.0, 1.0) });
            }
            ShotKind::Pod => {
                let src = a.r.ship.pod_shot[(s.age / 2 % 2) as usize];
                let ang = s.vx.atan2(-s.vy);
                let (cx, cy) = ((s.x + tx) * 0.5, (s.y + ty) * 0.5);
                a.blit_rot(src, ox as f32 + cx, oy as f32 + cy, ang, Color::new(1.0, 1.0, 1.0, 0.45));
            }
        }
    }

    let p = &w.player;
    let (px, py) = (ox + snap(p.x), oy + snap(p.y));
    if let Some(b) = w.beam {
        draw_beam(a, ox, oy, b.x, b.top, p.y - 11.0, w.frame);
        draw_aura(a, px, py, w.frame);
    }

    // afterimages at full speed
    let moving = (p.dx * p.dx + p.dy * p.dy).sqrt();
    if !p.focused() && moving >= w.tuning.speed * AFTERIMAGE_MIN {
        let at = [2, 4, 6].map(|k| (ox + snap(p.history[k].0), oy + snap(p.history[k].1)));
        ship::draw_afterimages(a, p.bank, &at);
    }

    let pose = Pose {
        bank: p.bank,
        pods: p.pods,
        thrust: p.thrust(),
        fire_t: p.fire_t,
        beaming: p.beaming,
        hit_t: p.hit_t,
        frame: w.frame,
    };
    ship::draw(a, px, py, &pose);

    r.fx.draw(a, ox, oy);
    ship::draw_hitbox(a, px, py, p.blend(&w.tuning), w.frame);

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
        for s in &w.shots {
            box_(s.rect(), Color::new(0.3, 1.0, 0.6, 0.5));
        }
        if let Some(b) = w.beam {
            box_(b.rect(), Color::new(0.3, 1.0, 0.6, 0.5));
        }
        box_(w.body(), Color::new(1.0, 1.0, 1.0, 0.35));
        let r = HITBOX_R as i32;
        a.fill(px - r, py - r, 2 * r + 1, 2 * r + 1, red);
    }
}

/// The focus beam: a translucent red column with a cream core, crawling upward.
fn draw_beam(a: &Atlas, ox: i32, oy: i32, x: f32, top: f32, nose: f32, frame: u64) {
    let (y0, y1) = (top.max(0.0), nose);
    if y1 <= y0 {
        return;
    }
    let cx = ox + snap(x);
    let (top_px, h) = (oy + snap(y0), snap(y1 - y0));
    let wob = ((frame / 2) % 3) as i32 - 1; // the width breathes a pixel
    let half = (BEAM_W * 0.5) as i32;
    a.fill(cx - half - wob.max(0), top_px, 2 * (half + wob.max(0)), h, Color { a: 0.18, ..SIGNAL });
    a.fill(cx - half + 2, top_px, 2 * half - 4, h, Color { a: 0.34, ..SIGNAL });
    a.fill(cx - 1 - (wob == 1) as i32, top_px, 2 + 2 * (wob == 1) as i32, h, Color { a: 0.55, ..CREAM });
    // bright dashes running up the beam
    let step = 14;
    let off = (frame as i32 * 7) % step;
    let mut y = top_px + h - off;
    while y > top_px {
        a.fill(cx - 3, y, 6, 2, Color { a: 0.3, ..CREAM });
        y -= step;
    }
    // where it lands
    if top > 0.0 {
        let flare = if frame % 4 < 2 { 0.6 } else { 0.4 };
        a.blit(a.r.sparkle, cx - 3, top_px - 3, 1, Color { a: flare, ..CREAM });
        a.fill(cx - half, top_px - 1, 2 * half, 2, Color { a: 0.4, ..SIGNAL });
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
