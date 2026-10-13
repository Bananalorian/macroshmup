//! The kit in play: style earns it a piece at a time, the augments fire with you and
//! hurt things, the shield takes a hit instead of you and comes back, and a life lost
//! loses it all.

use super::bullets::{Behavior, Kind, Spawn};
use super::dummy::DummyKind;
use super::player::Controls;
use super::shots::ShotKind;
use super::style::Move;
use super::{Event, Life, World};
use crate::tune::*;

/// A stage world with nothing scheduled (so style counts) and nothing about.
fn empty() -> World {
    let mut w = World::stage(1, Default::default());
    let s = w.stage.as_mut().unwrap();
    s.skip_to(super::stage1::frames(SECTIONS[WARNING_SECTION].0));
    s.boss_called = true;
    w
}

/// A live bullet sitting on the Pilgrim.
fn park(w: &mut World) {
    let (x, y) = (w.player.x, w.player.y);
    w.hazards.bullets.push(Spawn { x, y, vx: 0.0, vy: 0.0, kind: Kind::Orb, behavior: Behavior::Straight, owner: 0, hue: Default::default() });
    let i = w.hazards.bullets.len() - 1;
    w.hazards.bullets.age[i] = 30;
}

fn kit_events(w: &World) -> Vec<u8> {
    w.events.iter().filter_map(|e| if let Event::Kit { level, .. } = e { Some(*level) } else { None }).collect()
}

#[test]
fn style_earns_the_kit_in_play_b_a_then_s() {
    let mut w = empty();
    let mut got = vec![];
    for _ in 0..5 {
        w.stylish(Move::Break, 200.0);
        got.extend(kit_events(&w));
        w.events.clear();
    }
    assert!(w.style.rank() >= 4, "S");
    assert_eq!(got, vec![1, 2, 3], "B the left augment, A the right, S the shield");
    assert_eq!((w.kit.augments(), w.kit.shield_up()), (2, true));
    // falling back and rising again earns nothing more: it's this life's to keep
    let r = w.style.drop(3);
    w.restyled(r);
    w.stylish(Move::Break, 300.0);
    assert!(kit_events(&w).is_empty());
    assert_eq!(w.kit.level, 3);
}

#[test]
fn the_sandbox_earns_no_kit() {
    let mut w = World::new(1, Default::default());
    for _ in 0..10 {
        w.stylish(Move::Break, 200.0);
    }
    assert_eq!(w.kit.level, 0);
}

/// The damage a second of the wide shot does to a dummy above the right augment's
/// post, with the kit at `level`, and the most augment bolts in the air at once.
fn second_of_fire(level: u8) -> (f32, usize, World) {
    let mut w = World::new(1, Default::default());
    if level > 0 {
        w.give_kit(level);
    }
    let (px, py) = (w.player.x, w.player.y);
    w.dummies = vec![super::dummy::Dummy::new(DummyKind::Still, px + AUG_WIDE.0, py - 80.0)];
    let hp = w.dummies[0].hp;
    let mut aug_shots = 0;
    for _ in 0..60 {
        w.tick(Controls { fire: true, ..Default::default() });
        aug_shots = aug_shots.max(w.shots.iter().filter(|s| s.kind == ShotKind::Aug).count());
        w.events.clear();
    }
    (hp - w.dummies[0].hp, aug_shots, w)
}

#[test]
fn augments_fire_with_you_and_hurt_what_they_hit() {
    let (bare, none, _) = second_of_fire(0);
    let (armed, aug_shots, mut w) = second_of_fire(2);
    assert_eq!(none, 0);
    assert!(aug_shots >= 2, "the augments fire");
    assert!(armed > bare, "and hit: {armed} damage with them, {bare} without");
    // the beam too: tucked in, they keep firing
    let before = w.shots.iter().filter(|s| s.kind == ShotKind::Aug).count();
    w.shots.clear();
    for _ in 0..30 {
        w.tick(Controls { fire: true, focus: true, ..Default::default() });
    }
    assert!(w.shots.iter().any(|s| s.kind == ShotKind::Aug), "still firing beside the beam ({before} before)");
    let (dx, _) = super::kit::Kit::post(1, 1.0);
    assert!((w.kit.aug[1].0 - (w.player.x + dx)).abs() < 1.0, "tucked in beside the sword");
}

#[test]
fn the_shield_takes_a_hit_instead_of_you_then_recharges() {
    let mut w = empty();
    w.give_kit(3);
    w.style.gain(Move::Break, 450.0);
    let rank = w.style.rank();
    // a ring of bullets close round the ship, and one far off
    let (x, y) = (w.player.x, w.player.y);
    for k in 0..8 {
        let a = k as f32 * 0.785;
        w.hazards.bullets.push(Spawn { x: x + a.cos() * 30.0, y: y + a.sin() * 30.0, vx: 0.0, vy: 0.0, kind: Kind::Small, behavior: Behavior::Straight, owner: 0, hue: Default::default() });
    }
    w.hazards.bullets.push(Spawn { x: 20.0, y: 20.0, vx: 0.0, vy: 0.0, kind: Kind::Small, behavior: Behavior::Straight, owner: 0, hue: Default::default() });
    park(&mut w);
    w.tick(Controls::default());
    assert!(w.events.iter().any(|e| matches!(e, Event::ShieldBreak { .. })));
    assert_eq!((w.life, w.hits), (Life::Alive, 0), "not struck, not a miss");
    assert!(!w.kit.shield_up());
    assert_eq!(w.hazards.bullets.len(), 1, "the bullets round you popped, the far one stayed");
    assert_eq!(w.style.rank(), rank - 1, "it costs one rank, not two");
    assert!(w.invuln >= SHIELD_INVULN - 1);
    // untouchable for a moment, then exposed until it's back
    park(&mut w);
    for _ in 0..SHIELD_INVULN - 2 {
        w.tick(Controls::default());
    }
    assert_eq!(w.life, Life::Alive, "blinking: untouchable");
    w.hazards.bullets.clear();
    w.events.clear();
    let mut up = None;
    for t in 0..SHIELD_RECHARGE {
        w.tick(Controls::default());
        if w.events.iter().any(|e| matches!(e, Event::ShieldUp { .. })) {
            up = Some(t);
        }
        w.events.clear();
    }
    assert!(up.is_some() && w.kit.shield_up(), "recharged in {SHIELD_RECHARGE} frames");
}

#[test]
fn without_the_shield_a_hit_is_a_hit() {
    let mut w = empty();
    w.give_kit(2);
    park(&mut w);
    w.tick(Controls::default());
    assert!(matches!(w.life, Life::Struck { .. }), "two augments, no shield: struck");
}

#[test]
fn a_life_lost_loses_the_kit_and_a_saving_bomb_keeps_it() {
    // bombed in the deathbomb window: you live, and keep it
    let mut w = empty();
    w.give_kit(2);
    park(&mut w);
    w.tick(Controls::default());
    w.tick(Controls { bomb: true, ..Default::default() });
    assert_eq!((w.life, w.kit.level), (Life::Alive, 2));
    // no bomb: the ship and its augments go down
    let mut w = empty();
    w.give_kit(3);
    w.kit.break_shield();
    park(&mut w);
    let mut lost = 0;
    for _ in 0..30 {
        w.tick(Controls::default());
        lost += w.events.iter().filter(|e| matches!(e, Event::AugmentLost { .. })).count();
        w.events.clear();
    }
    assert_eq!(w.deaths, 1);
    assert_eq!(lost, 2, "both augments shatter with it");
    assert_eq!(w.kit.level, 0, "the next life starts bare");
    assert_eq!(w.style.rank(), 0);
}
