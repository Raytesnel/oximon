use crate::combat::components::*;
use crate::combat::events::*;
use crate::combat::systems::*;
use crate::combat::*;
use crate::common::components::{Enemy, Player};
use crate::movement::components::Facing;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<DamageEvent>()
        .add_plugins(bevy::input::InputPlugin)
        .add_plugins(CombatPlugin);
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
                max: 100.0,
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
            AttackStats { attack: 10.0 },
        ))
        .id();
    // manually run spawn system logic path
    app.world_mut()
        .write_message(AttackEvent { entity: player });

    app.update();

    let mut query = app.world_mut().query::<(&Attack, &Transform)>();
    let attacks: Vec<_> = query.iter(&app.world()).collect();

    assert!(!attacks.is_empty());
}

#[test]
fn attack_despawns_after_time() {
    let mut app = test_app();
    let duration = 0.1;
    let attack = app
        .world_mut()
        .spawn((
            Attack {
                damage: 10.0,
                range: 100.0,
                lifetime: Timer::from_seconds(duration, TimerMode::Once),
                hit_timer: Timer::from_seconds(duration, TimerMode::Repeating),
                follow_entity: None,
                active: false,
                stat_modifiers: vec![],
                applied_start_modifiers: false,
            },
            Transform::default(),
        ))
        .id();

    for i in 0..(duration / 0.016).round() as i32 + 2 {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(0.016),
        ));
        app.update();
        let time = app.world().resource::<Time>();
        println!("frame {i}, elapsed: {:?}", time.elapsed());
    }
    let exists = app.world().get_entity(attack).is_ok();
    assert!(!exists);
}
