//! Lives, death and the bomb (Sprint 4): the cancel ring, the deathbomb window,
//! exact invincibility windows, the respawn, game over.

use super::bullets::{Behavior, Kind, Spawn};
use super::dummy::{Dummy, DummyKind};
use super::player::Controls;
use super::{BombBlast, Event, Life, World};
use crate::tune::*;

fn idle() -> Controls {
    Controls::default()
}

fn bomb() -> Controls {
    Controls { bomb: true, ..Default::default() }
}

/// A quiet world: no guns, no dummies.
fn quiet() -> World {
    let mut w = World::new(1, Default::default());
    w.dummies.clear();
    w
}

/// A stationary, already-live bullet right on the Pilgrim.
fn park(w: &mut World) {
    let (x, y) = (w.player.x, w.player.y);
    w.hazards.bullets.push(Spawn { x, y, vx: 0.0, vy: 0.0, kind: Kind::Orb, behavior: Behavior::Straight });
    let i = w.hazards.bullets.len() - 1;
    w.hazards.bullets.age[i] = 30;
}

/// Tick until something is true, returning how many ticks it took (at most `max`).
fn ticks_until(w: &mut World, max: u32, c: Controls, mut done: impl FnMut(&World) -> bool) -> Option<u32> {
    for k in 1..=max {
        w.tick(c);
        if done(w) {
            return Some(k);
        }
        w.events.clear();
    }
    None
}

#[test]
fn the_bomb_cancels_every_enemy_bullet_inside_its_ring() {
    let mut w = quiet();
    // a field full of bullets that don't move
    for i in 0..400 {
        let (x, y) = ((i * 37 % 270) as f32, (i * 53 % 360) as f32);
        w.hazards.bullets.push(Spawn { x, y, vx: 0.0, vy: 0.0, kind: Kind::Small, behavior: Behavior::Straight });
    }
    w.player.x = 30.0; // off to one side, so the ring has a long way to go
    w.tick(bomb());
    let (bx, by) = (w.bomb.unwrap().x, w.bomb.unwrap().y);
    for t in 0..BOMB_GROW {
        let r = BombBlast::radius(t);
        let b = &w.hazards.bullets;
        assert!((0..b.len()).all(|i| (b.x[i] - bx).hypot(b.y[i] - by) > r), "frame {t}: a bullet survived inside the ring");
        if t == 0 {
            assert!(b.len() > 300, "the ring takes time to spread");
        }
        w.tick(idle());
    }
    assert_eq!(w.hazards.bullets.len(), 0, "it covers the whole screen in {BOMB_GROW} frames");
}

#[test]
fn the_bombs_shards_and_the_sweep_are_reported() {
    let mut w = quiet();
    park(&mut w);
    w.tick(bomb());
    assert!(w.cancel_shards && !w.cancelled.is_empty(), "cancelled bullets turn to shards");
    assert!(w.events.iter().any(|e| matches!(e, Event::Bomb { .. })));
    let swept = ticks_until(&mut w, 60, idle(), |w| w.events.contains(&Event::BombSwept));
    assert_eq!(swept, Some(BOMB_GROW - 1));
}

/// Struck on one tick, then bomb `k` ticks later: does the Pilgrim survive?
fn bomb_after_hit(k: u32) -> bool {
    let mut w = quiet();
    park(&mut w);
    w.tick(idle());
    assert!(matches!(w.life, Life::Struck { .. }), "struck");
    for _ in 1..k {
        w.tick(idle());
    }
    w.tick(bomb());
    for _ in 0..20 {
        w.tick(idle());
    }
    w.deaths == 0
}

#[test]
fn a_bomb_8_frames_after_a_hit_saves_you_and_9_does_not() {
    assert_eq!(DEATHBOMB, 8);
    for k in 1..=8 {
        assert!(bomb_after_hit(k), "a bomb {k} frames after the hit should save you");
    }
    assert!(!bomb_after_hit(9), "9 frames is too late");
    assert!(!bomb_after_hit(12));
}

#[test]
fn a_saving_bomb_spends_a_bomb_and_counts_the_hit() {
    let mut w = quiet();
    park(&mut w);
    w.tick(idle());
    w.tick(bomb());
    assert_eq!((w.life, w.bombs, w.hits, w.deaths), (Life::Alive, BOMBS - 1, 1, 0));
}

#[test]
fn bomb_invincibility_lasts_exactly_180_frames() {
    let mut w = quiet();
    w.tick(bomb()); // bomb on tick 1
    // once the ring stops cancelling, park bullets on the ship every frame
    for _ in 1..BOMB_ACTIVE + 1 {
        w.tick(idle());
    }
    let mut struck_at = None;
    for tick in BOMB_ACTIVE as u64 + 2..=400 {
        park(&mut w);
        w.tick(idle());
        if matches!(w.life, Life::Struck { .. }) {
            struck_at = Some(tick);
            break;
        }
    }
    assert_eq!(struck_at, Some(1 + BOMB_INVULN as u64), "safe on ticks 1..=180, hit on 181");
}

/// Die (no bomb), returning the world on the tick the next ship appears.
fn die_and_respawn(w: &mut World) -> u64 {
    park(w);
    w.tick(idle());
    let hit = w.frame;
    ticks_until(w, 200, idle(), |w| w.events.contains(&Event::Respawn)).expect("a respawn");
    assert_eq!(w.frame - hit, RESPAWN_AT as u64, "the next ship comes {RESPAWN_AT} frames after the hit");
    w.frame
}

#[test]
fn respawn_invincibility_lasts_exactly_210_frames_and_you_cant_die_in_it() {
    let mut w = quiet();
    let r = die_and_respawn(&mut w);
    assert_eq!(w.invuln, RESPAWN_INVULN - 1);
    let mut struck_at = None;
    for _ in 0..400 {
        park(&mut w);
        w.tick(idle());
        if matches!(w.life, Life::Struck { .. }) {
            struck_at = Some(w.frame);
            break;
        }
        assert_eq!(w.deaths, 1, "no dying while invincible");
    }
    assert_eq!(struck_at, Some(r + RESPAWN_INVULN as u64));
}

#[test]
fn bullets_cancel_30_frames_after_the_hit() {
    let mut w = quiet();
    for i in 0..50 {
        w.hazards.bullets.push(Spawn { x: 20.0 + i as f32 * 4.0, y: 40.0, vx: 0.0, vy: 0.0, kind: Kind::Small, behavior: Behavior::Straight });
    }
    park(&mut w);
    w.tick(idle());
    let hit = w.frame;
    let k = ticks_until(&mut w, 100, idle(), |w| w.events.contains(&Event::Cancel)).unwrap();
    assert_eq!(w.frame - hit, DEATH_CANCEL as u64);
    assert_eq!(w.hazards.bullets.len(), 0);
    assert!(w.cancelled.len() >= 50 && !w.cancel_shards, "they pop (not bomb shards)");
    assert!(k > 0);
}

#[test]
fn the_next_ship_flies_in_from_the_bottom_with_bombs_refilled() {
    let mut w = quiet();
    w.tick(bomb());
    for _ in 0..BOMB_INVULN + 5 {
        w.tick(idle());
    }
    assert_eq!(w.bombs, BOMBS - 1);
    die_and_respawn(&mut w);
    assert_eq!(w.bombs, BOMBS, "refilled");
    assert!(w.player.y > FIELD_H as f32, "starts below the screen");
    let before = w.player.y;
    for _ in 0..RESPAWN_FLY - 1 {
        w.tick(Controls { mv: (1.0, 0.0), ..Default::default() });
        assert!(!w.alive(), "no control while flying in");
    }
    assert!(w.player.y < before && w.player.x == START_X);
    w.tick(idle());
    assert!(w.alive());
    assert!((w.player.y - START_Y).abs() < 0.01, "home at the start position");
    w.tick(Controls { mv: (1.0, 0.0), ..Default::default() });
    assert_eq!(w.player.x, START_X + MOVE_SPEED, "and in control");
}

#[test]
fn three_deaths_and_its_game_over() {
    let mut w = quiet();
    assert_eq!(w.lives, LIVES);
    for life in 1..=LIVES {
        park(&mut w);
        w.tick(idle());
        let over = ticks_until(&mut w, 120, idle(), |w| w.events.contains(&Event::GameOver) || w.events.contains(&Event::Respawn)).unwrap();
        assert!(over > 0);
        assert_eq!(w.lives, LIVES - life);
        w.events.clear();
        if life < LIVES {
            // wait out the respawn invincibility
            for _ in 0..RESPAWN_INVULN + 1 {
                w.tick(idle());
                w.events.clear();
            }
        }
    }
    assert_eq!(w.life, Life::GameOver);
    for _ in 0..300 {
        w.tick(bomb());
        assert_eq!(w.life, Life::GameOver, "no bombing your way out");
    }
}

#[test]
fn bombs_run_out() {
    let mut w = quiet();
    for _ in 0..BOMBS {
        w.tick(bomb());
        for _ in 0..BOMB_ACTIVE {
            w.tick(idle());
        }
    }
    assert_eq!(w.bombs, 0);
    w.events.clear();
    w.tick(bomb());
    assert!(w.events.contains(&Event::NoBomb));
    assert!(w.bomb.is_none());
}

#[test]
fn the_bomb_hurts_everything_once_but_never_one_shots_a_big_target() {
    let mut w = World::new(1, Default::default());
    w.dummies = vec![Dummy::new(DummyKind::Still, 60.0, 80.0), Dummy::new(DummyKind::Big, 200.0, 80.0)];
    w.tick(bomb());
    for _ in 0..BOMB_ACTIVE {
        w.tick(idle());
    }
    assert!(!w.dummies[0].alive(), "small things break");
    let big = &w.dummies[1];
    assert!(big.alive(), "the big one survives a bomb");
    assert!((big.max_hp - big.hp - big.max_hp * BOMB_BOSS_CAP).abs() < 1.0, "and takes its share, once");
}

#[test]
fn the_bomb_slows_time_for_a_moment() {
    let mut w = quiet();
    w.tick(bomb());
    assert_eq!(w.dip, BOMB_DIP - 1);
    for _ in 0..BOMB_DIP {
        w.tick(idle());
    }
    assert_eq!(w.dip, 0);
}

#[test]
fn no_shooting_while_struck_or_dead() {
    let mut w = quiet();
    park(&mut w);
    w.tick(idle());
    for _ in 0..30 {
        w.tick(Controls { fire: true, ..Default::default() });
        assert!(!w.player.firing);
    }
}
