use crate::combat::components::*;
use crate::combat::events::*;
use crate::combat::systems::*;
use crate::combat::*;
use bevy::prelude::*;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<DamageEvent>()
        .add_systems(Update, apply_damage_system);
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
fn entity_dies_when_health_zero() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Health {
                current: 20.0,
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

    let state = app.world_mut().get::<CombatState>(entity).unwrap();
    assert_eq!(*state, CombatState::Dead);
}
