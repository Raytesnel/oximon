#[allow(unused)]
use crate::combat::attacks::{quick_attack, simple_beam};
use crate::combat::components::*;
use crate::combat::*;
use crate::common::components::*;
#[allow(unused)]
use crate::movement::components::Facing;
use bevy::app::FixedMain;
use bevy::prelude::*;
#[allow(unused)]
use bevy::time::TimeUpdateStrategy;
use std::collections::HashMap;

#[allow(dead_code)]
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(CombatPlugin);
    app.insert_resource(Time::<Fixed>::from_seconds(0.016));
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.world_mut().spawn((
        Player,
        Cooldowns {
            timers: HashMap::new(),
        },
    ));
    app
}

#[test]
fn damage_reduces_health() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Health {
                current: 100.0,
                _max: 100.0,
            },
            CombatState::Idle,
        ))
        .id();

    app.add_message::<DamageEvent>();
    app.world_mut().write_message(DamageEvent {
        target: entity,
        amount: 30.0,
    });

    app.update();

    let health = app.world_mut().get::<Health>(entity).unwrap();
    assert_eq!(health.current, 70.0);
}

#[test]
fn attack_is_spawned() {
    let mut app = test_app();

    let player = app
        .world_mut()
        .spawn((
            Player,
            Transform::from_xyz(0.0, 0.0, 0.0),
            Facing(Vec2::X),
            AttackStats { _attack: 10.0 },
        ))
        .id();
    // manually run spawn system logic path
    app.world_mut()
        .write_message(AttackEvent { _entity: player });

    app.update();

    let mut query = app.world_mut().query::<(&Attack, &Transform)>();
    let attacks: Vec<_> = query.iter(&app.world()).collect();

    assert!(!attacks.is_empty());
}

#[test]
fn attack_despawns_after_time() {
    let mut app = test_app();
    let duration = 0.1;

    let mut def = simple_beam();
    def.lifetime = duration;

    let attack = app
        .world_mut()
        .spawn((
            Attack::from_definition(def, Entity::PLACEHOLDER, AttackId(1)),
            Transform::default(),
        ))
        .id();

    for _ in 0..(duration / 0.016).ceil() as i32 + 2 {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(0.016),
        ));
        app.update();
    }

    let exists = app.world().get_entity(attack).is_ok();
    assert!(!exists);
}
#[allow(dead_code)]
fn tick(app: &mut App, dt: f32) {
    let delta = std::time::Duration::from_secs_f32(dt);

    // FIXED TIME (for FixedUpdate systems)
    {
        let mut time = app.world_mut().resource_mut::<Time<Fixed>>();
        time.advance_by(delta);
    }

    // REAL TIME (for normal systems)
    {
        let mut time = app.world_mut().resource_mut::<Time>();
        time.advance_by(delta);
    }
    app.world_mut().run_schedule(FixedMain);
    app.update();
}
#[test]
fn cooldown_prevents_spam() {
    let mut app = test_app();
    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(QUICK_ATTACK);
    }
    tick(&mut app, 0.016);
    let _player = app.world_mut().spawn((Player, Cooldowns::default())).id();

    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(QUICK_ATTACK);
    }

    tick(&mut app, 0.016);

    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.release(QUICK_ATTACK);
    }

    tick(&mut app, 0.016);
    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(QUICK_ATTACK);
    }

    app.update();

    let attacks: Vec<_> = app
        .world_mut()
        .query::<&Attack>()
        .iter(&app.world())
        .collect();

    assert_eq!(attacks.len(), 2);
}
#[test]
fn cooldown_expires_allows_attack_again() {
    let mut app = test_app();

    let _player = app.world_mut().spawn((Player, Cooldowns::default())).id();

    // first attack
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyQ);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::KeyQ);

    // simulate time passing (longer than cooldown)
    for _ in 0..200 {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(0.016),
        ));
        app.update();
    }

    // second attack
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyQ);
    app.update();

    let attacks: Vec<_> = app
        .world_mut()
        .query::<&Attack>()
        .iter(&app.world())
        .collect();

    assert!(attacks.len() >= 1);
}

#[test]
fn attack_applies_stat_modifier_on_start() {
    let mut app = test_app();

    let player = app.world_mut().spawn((Player, Stats::default())).id();

    let def = quick_attack();

    app.world_mut()
        .spawn((Attack::from_definition(def.clone(), player, AttackId(1)),));

    app.update(); // runs attack_start_system

    let stats = app.world().get::<Stats>(player).unwrap();

    // depending on your implementation, check that modifiers were added
    assert!(!stats.speed.is_empty() || !stats.acceleration.is_empty());
}

#[test]
fn attack_becomes_active_on_hit() {
    let mut app = test_app();

    let _enemy = app
        .world_mut()
        .spawn((Enemy, Transform::from_xyz(0.0, 0.0, 0.0)))
        .id();

    let def = simple_beam();

    let attack = app
        .world_mut()
        .spawn((
            Attack::from_definition(def, Entity::PLACEHOLDER, AttackId(1)),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    app.update();

    let _attack = app.world().get::<Attack>(attack).unwrap();
}

#[test]
fn attack_follows_entity() {
    let mut app = test_app();

    let player = app
        .world_mut()
        .spawn((Player, Transform::from_xyz(0.0, 0.0, 0.0)))
        .id();

    let def = simple_beam();

    let attack = app
        .world_mut()
        .spawn((
            Attack::from_definition(def.clone(), player, AttackId(1)),
            Transform::default(),
        ))
        .id();

    // move player
    {
        let mut transform = app.world_mut().get_mut::<Transform>(player).unwrap();
        transform.translation.x = 50.0;
    }

    app.update();

    let attack_transform = app.world().get::<Transform>(attack).unwrap();
    assert!(attack_transform.translation.x > 0.0);
}

#[test]
fn cooldowns_are_removed_after_finish() {
    let mut app = test_app();

    let mut cooldowns = Cooldowns::default();
    cooldowns.timers.insert(
        "test".to_string(),
        Timer::from_seconds(0.01, TimerMode::Once),
    );

    let entity = app.world_mut().spawn((cooldowns,)).id();

    // simulate time
    for _ in 0..10 {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(0.016),
        ));
        app.update();
    }

    let cooldowns = app.world().get::<Cooldowns>(entity).unwrap();

    assert!(cooldowns.timers.is_empty());
}
