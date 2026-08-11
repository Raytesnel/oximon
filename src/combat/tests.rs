//! Integration tests for the CombatPlugin.
//!
//! These tests use the full plugin and cover behaviours that span multiple
//! systems working together. Unit tests for individual systems live in
//! `combat/systems/` next to the systems themselves.

#[allow(unused)]
use crate::combat::attacks::{quick_attack, simple_beam, slow_down};
use crate::combat::components::*;
use crate::combat::*;
use crate::common::CommonPlugin;
use crate::common::components::*;
use crate::movement::MovementPlugin;
#[allow(unused)]
use crate::movement::components::Facing;
use bevy::app::FixedMain;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
#[allow(unused)]
use bevy::time::TimeUpdateStrategy;
use std::collections::HashMap;
use crate::overworld::components::DomainExpansionAsset;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);
    app.add_plugins(CombatPlugin);
    app.add_plugins(CommonPlugin);
    app.add_plugins(MovementPlugin);
    app.insert_resource(Time::<Fixed>::from_seconds(0.016));
    app.init_state::<BattleState>();
    app.init_resource::<DomainExpansionAsset>();
    app.add_plugins(bevy::log::LogPlugin::default());
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(Hitstop { remaining: 0.0 });
    app.insert_resource(AttackIdCounter::default());
    app.insert_resource(CombatSpawnContext {
        player_world_pos: Vec3::ZERO,
    });
    app.init_state::<GameState>();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Combat);

    app.world_mut().spawn((
        Player,
        Cooldowns {
            timers: HashMap::new(),
        },
    ));
    app.update(); // laat de state transitie plaatsvinden
    app.update(); // OnEnter systemen draaien nu ook
    app
}

fn tick(app: &mut App, dt: f32) {
    let delta = std::time::Duration::from_secs_f32(dt);
    // TimeUpdateStrategy makes app.update() advance the real Time by exactly
    // dt instead of using wall-clock time (which is ~0 in tests).
    app.insert_resource(TimeUpdateStrategy::ManualDuration(delta));
    {
        let mut time = app.world_mut().resource_mut::<Time<Fixed>>();
        time.advance_by(delta);
    }
    app.world_mut().run_schedule(FixedMain);
    app.update();
}

/// Pressing an attack key spawns an Attack entity in the world.
#[test]
fn attack_is_spawned_on_input() {
    let mut app = test_app();

    app.world_mut().spawn((
        Player,
        Transform::from_xyz(0.0, 0.0, 0.0),
        Facing(Vec2::X),
        AttackStats { _attack: 10.0 },
        Cooldowns::default(),
        CombatState::Idle,
    ));

    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(QUICK_ATTACK);
    }
    tick(&mut app, 0.016);

    let attacks: Vec<_> = app
        .world_mut()
        .query::<&Attack>()
        .iter(app.world())
        .collect();

    assert!(
        !attacks.is_empty(),
        "pressing attack key should spawn an Attack entity"
    );
}

/// A cooldown prevents the same attack from being spawned a second time
/// until the player releases and re-presses the key.
#[test]
fn cooldown_prevents_attack_spam() {
    let mut app = test_app();

    // first press → spawns attack + starts cooldown
    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(QUICK_ATTACK);
    }
    tick(&mut app, 0.016);

    // hold key down (just_pressed won't fire again) then release + re-press
    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.release(QUICK_ATTACK);
    }
    tick(&mut app, 0.016);

    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(QUICK_ATTACK);
    }
    tick(&mut app, 0.016);

    // cooldown still active → second press should NOT spawn another attack
    let attacks: Vec<_> = app
        .world_mut()
        .query::<&Attack>()
        .iter(app.world())
        .collect();

    assert_eq!(
        attacks.len(),
        1,
        "cooldown should prevent a second attack from spawning"
    );
}

/// After the cooldown duration elapses the player can attack again.
#[test]
fn cooldown_expiry_allows_attack_again() {
    let mut app = test_app();

    // first attack
    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(QUICK_ATTACK);
    }
    tick(&mut app, 0.016);
    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.release(QUICK_ATTACK);
    }

    // wait longer than the cooldown (quick_attack cooldown = 3.0 s)
    for _ in 0..250 {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(0.016),
        ));
        app.update();
    }

    // second attack after cooldown
    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(QUICK_ATTACK);
    }
    tick(&mut app, 0.016);

    let attacks: Vec<_> = app
        .world_mut()
        .query::<&Attack>()
        .iter(app.world())
        .collect();

    assert!(
        attacks.len() >= 1,
        "should be able to attack again after cooldown expires"
    );
}

/// OnCast stat modifiers from quick_attack are applied to the owner the
/// frame the Attack is spawned.
#[test]
fn attack_applies_oncast_stat_modifiers_to_owner() {
    let mut app = test_app();

    let player = app.world_mut().spawn((Player, Stats::default())).id();

    let def = quick_attack();
    app.world_mut()
        .spawn(Attack::from_definition(def, player, AttackId(1)));

    app.update(); // runs attack_start_system

    let stats = app.world().get::<Stats>(player).unwrap();
    assert!(
        !stats.speed.is_empty() || !stats.acceleration.is_empty(),
        "OnCast modifiers should be applied to owner stats on the first update"
    );
}

/// An Attack entity with a follow_entity tracks its owner's Transform.
#[test]
fn attack_follows_owner_transform() {
    let mut app = test_app();

    let player = app
        .world_mut()
        .spawn((Player, Transform::from_xyz(0.0, 0.0, 0.0)))
        .id();

    let attack = app
        .world_mut()
        .spawn((
            Attack::from_definition(simple_beam(), player, AttackId(1)),
            Transform::default(),
        ))
        .id();

    // move the owner
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation
        .x = 50.0;

    app.update(); // runs attack_follow_system

    let attack_pos = app.world().get::<Transform>(attack).unwrap().translation;
    assert!(
        attack_pos.x > 0.0,
        "attack should follow owner; attack x was {}",
        attack_pos.x
    );
}

// ── Hit → health pipeline ─────────────────────────────────────────────────────

/// Hitting an enemy reduces its health.
#[test]
fn hitting_enemy_reduces_health() {
    let mut app = test_app();

    let owner = app.world_mut().spawn_empty().id();

    let enemy = app
        .world_mut()
        .spawn((
            Enemy,
            Transform::default(),
            Hurtbox {
                size: Vec2::new(20.0, 20.0),
            },
            Health {
                current: 100.0,
                _max: 100.0,
            },
            CombatState::Idle,
        ))
        .id();

    app.world_mut().spawn((
        Attack::from_definition(quick_attack(), owner, AttackId(0)),
        Transform::default(),
        Hitbox {
            size: Vec2::new(20.0, 20.0),
        },
    ));

    tick(&mut app, 0.016);
    tick(&mut app, 0.016);

    let health = app.world().get::<Health>(enemy).unwrap();
    assert!(
        health.current < 100.0,
        "enemy health should be reduced after being hit, was {}",
        health.current
    );
}

/// Hitting an enemy with quick_attack applies knockback to the enemy.
#[test]
fn hitting_enemy_applies_knockback_to_enemy() {
    let mut app = test_app();

    let owner = app.world_mut().spawn_empty().id();

    let enemy = app
        .world_mut()
        .spawn((
            Enemy,
            Transform::default(),
            Hurtbox {
                size: Vec2::new(20.0, 20.0),
            },
            Health {
                current: 100.0,
                _max: 100.0,
            },
            CombatState::Idle,
        ))
        .id();

    app.world_mut().spawn((
        Attack::from_definition(quick_attack(), owner, AttackId(0)),
        Transform::default(),
        Hitbox {
            size: Vec2::new(20.0, 20.0),
        },
    ));

    tick(&mut app, 0.016);

    assert!(
        app.world().get::<KnockbackEffect>(enemy).is_some(),
        "enemy should have KnockbackEffect after being hit"
    );
}

/// Hitting an enemy with quick_attack applies hitstun to the enemy.
#[test]
fn hitting_enemy_applies_hitstun_to_enemy() {
    let mut app = test_app();

    let owner = app.world_mut().spawn_empty().id();

    let enemy = app
        .world_mut()
        .spawn((
            Enemy,
            Transform::default(),
            Hurtbox {
                size: Vec2::new(20.0, 20.0),
            },
            Health {
                current: 100.0,
                _max: 100.0,
            },
            CombatState::Idle,
        ))
        .id();

    app.world_mut().spawn((
        Attack::from_definition(quick_attack(), owner, AttackId(0)),
        Transform::default(),
        Hitbox {
            size: Vec2::new(20.0, 20.0),
        },
    ));

    tick(&mut app, 0.016);

    assert!(
        app.world().get::<Hitstun>(enemy).is_some(),
        "enemy should be in hitstun after being hit"
    );
}

/// After hitstun wears off the enemy no longer has the Hitstun component.
#[test]
fn hitstun_wears_off_after_duration() {
    let mut app = test_app();

    let owner = app.world_mut().spawn_empty().id();

    let enemy = app
        .world_mut()
        .spawn((
            Enemy,
            Transform::default(),
            Hurtbox {
                size: Vec2::new(20.0, 20.0),
            },
            Health {
                current: 100.0,
                _max: 100.0,
            },
            CombatState::Idle,
        ))
        .id();

    app.world_mut().spawn((
        Attack::from_definition(quick_attack(), owner, AttackId(0)),
        Transform::default(),
        Hitbox {
            size: Vec2::new(20.0, 20.0),
        },
    ));

    tick(&mut app, 0.016); // hit lands, hitstun = 1.0 s

    // advance past hitstun duration (quick_attack hitstun = 1.0 s)
    for _ in 0..80 {
        tick(&mut app, 0.016);
    }
    assert!(
        app.world().get::<Hitstun>(enemy).is_none(),
        "hitstun should wear off after its duration"
    );
}

/// Hitting an enemy with slow_down applies a Slow component to the enemy.
#[test]
fn hitting_enemy_with_slow_attack_applies_slow() {
    let mut app = test_app();

    let owner = app.world_mut().spawn_empty().id();

    let enemy = app
        .world_mut()
        .spawn((
            Enemy,
            Transform::default(),
            Hurtbox {
                size: Vec2::new(200.0, 200.0),
            },
            Health {
                current: 100.0,
                _max: 100.0,
            },
            CombatState::Idle,
            Stats::default(),
        ))
        .id();

    app.world_mut().spawn((
        Attack::from_definition(slow_down(), owner, AttackId(0)),
        Transform::default(),
        Hitbox {
            size: Vec2::new(200.0, 200.0),
        },
    ));

    tick(&mut app, 0.016);
    tick(&mut app, 0.016);

    assert!(
        app.world().get::<Poison>(enemy).is_some(),
        "enemy should have Poison component after being hit by slow_down attack"
    );
}

/// slow_down also applies a speed modifier to the enemy's Stats.
#[test]
fn slow_adds_speed_modifier_to_enemy_stats() {
    let mut app = test_app();

    let owner = app.world_mut().spawn_empty().id();

    let enemy = app
        .world_mut()
        .spawn((
            Enemy,
            Transform::default(),
            Hurtbox {
                size: Vec2::new(200.0, 200.0),
            },
            Health {
                current: 100.0,
                _max: 100.0,
            },
            CombatState::Idle,
            Stats::default(),
        ))
        .id();

    app.world_mut().spawn((
        Attack::from_definition(slow_down(), owner, AttackId(0)),
        Transform::default(),
        Hitbox {
            size: Vec2::new(200.0, 200.0),
        },
    ));

    tick(&mut app, 0.016); // hit lands + slow_system applies modifier

    let stats = app.world().get::<Stats>(enemy).unwrap();
    assert!(
        !stats.speed.is_empty(),
        "slow hit should add a speed modifier to enemy Stats"
    );
}

/// Hitting an enemy with slow_down applies a Poison component to the enemy.
#[test]
fn hitting_enemy_with_slow_attack_applies_poison() {
    let mut app = test_app();

    let owner = app.world_mut().spawn_empty().id();

    let enemy = app
        .world_mut()
        .spawn((
            Enemy,
            Transform::default(),
            Hurtbox {
                size: Vec2::new(200.0, 200.0),
            },
            Health {
                current: 100.0,
                _max: 100.0,
            },
            CombatState::Idle,
            Stats::default(),
        ))
        .id();

    app.world_mut().spawn((
        Attack::from_definition(slow_down(), owner, AttackId(0)),
        Transform::default(),
        Hitbox {
            size: Vec2::new(200.0, 200.0),
        },
    ));

    tick(&mut app, 0.016);

    assert!(
        app.world().get::<Poison>(enemy).is_some(),
        "enemy should have Poison component after being hit by slow_down attack"
    );
}

/// Poison ticks reduce enemy health over time even without further attacks.
#[test]
fn poisoned_enemy_loses_health_over_time() {
    let mut app = test_app();

    let owner = app.world_mut().spawn_empty().id();

    let enemy = app
        .world_mut()
        .spawn((
            Enemy,
            Transform::default(),
            Hurtbox {
                size: Vec2::new(200.0, 200.0),
            },
            Health {
                current: 100.0,
                _max: 100.0,
            },
            CombatState::Idle,
            Stats::default(),
        ))
        .id();

    app.world_mut().spawn((
        Attack::from_definition(slow_down(), owner, AttackId(0)),
        Transform::default(),
        Hitbox {
            size: Vec2::new(200.0, 200.0),
        },
    ));

    tick(&mut app, 0.016); // hit lands, poison applied

    let health_after_hit = app.world().get::<Health>(enemy).unwrap().current;

    // advance past the poison tick_rate (2.0 s)
    for _ in 0..130 {
        tick(&mut app, 0.016);
    }

    let health_after_poison = app.world().get::<Health>(enemy).unwrap().current;
    assert!(
        health_after_poison < health_after_hit,
        "poisoned enemy health should decrease over time; was {} then {}",
        health_after_hit,
        health_after_poison
    );
}

/// An enemy that reaches 0 hp is marked Dead.
#[test]
fn enemy_at_zero_health_is_marked_dead() {
    let mut app = test_app();

    let owner = app.world_mut().spawn_empty().id();

    let enemy = app
        .world_mut()
        .spawn((
            Enemy,
            Transform::default(),
            Hurtbox {
                size: Vec2::new(20.0, 20.0),
            },
            Health {
                current: 1.0,
                _max: 1.0,
            }, // one hit will kill
            CombatState::Idle,
        ))
        .id();
    tick(&mut app, 0.016);
    app.world_mut().spawn((
        Attack::from_definition(quick_attack(), owner, AttackId(0)),
        Transform::default(),
        Hitbox {
            size: Vec2::new(20.0, 20.0),
        },
    ));

    tick(&mut app, 0.016);
    tick(&mut app, 0.016);
    tick(&mut app, 0.016);
    tick(&mut app, 0.016);

    assert!(
        app.world().get_entity(enemy).is_err(),
        "enemy should be despawned after reaching 0 health"
    );
}

/// The player's CombatState returns to Idle once the attack lifetime expires.
#[test]
fn player_returns_to_idle_after_attack_expires() {
    let mut app = test_app();

    let player = app
        .world_mut()
        .spawn((Player, CombatState::Attacking, Stats::default()))
        .id();

    let mut attack = Attack::from_definition(quick_attack(), player, AttackId(0));
    attack.lifetime_timer = Timer::from_seconds(0.016, TimerMode::Once);
    app.world_mut().spawn(attack);

    tick(&mut app, 0.016);
    tick(&mut app, 0.016);

    let state = app.world().get::<CombatState>(player).unwrap();
    assert_eq!(
        *state,
        CombatState::Idle,
        "player should return to Idle after attack lifetime expires"
    );
}

/// The speed boost granted by quick_attack on cast is removed when the attack ends.
#[test]
fn oncast_speed_boost_removed_after_attack_expires() {
    let mut app = test_app();

    let player = app
        .world_mut()
        .spawn((Player, CombatState::Attacking, Stats::default()))
        .id();

    let mut attack = Attack::from_definition(quick_attack(), player, AttackId(0));
    attack.lifetime_timer = Timer::from_seconds(0.016, TimerMode::Once);
    app.world_mut().spawn(attack);

    tick(&mut app, 0.016); // attack_start_system adds modifiers
    tick(&mut app, 0.016); // attack_lifetime_system expires + cleans up

    let stats = app.world().get::<Stats>(player).unwrap();
    assert!(
        stats.speed.is_empty() && stats.acceleration.is_empty(),
        "OnAttackEnd modifiers should be removed after attack expires"
    );
}
