mod combat;
mod common;
pub mod movement;

use crate::combat::CombatPlugin;
use crate::combat::components::{AttackStats, CombatState, Health};
use crate::common::CommonPlugin;
use crate::common::components::{
    ComputedStats, ModifierTrigger, Player, StatModifier, StatType, Stats,
};
use crate::movement::MovementPlugin;
use crate::movement::components::Facing;
use bevy::prelude::*;
use common::components::Enemy;
use movement::components::{MovementState, Velocity};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MovementPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(CommonPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Player
    commands.spawn((
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(0., 0., 0.),
        Player,
        ComputedStats {
            speed: 250.0,
            acceleration: 1250.0,
            friction: 625.0,
            dash_speed: 600.0,
            dash_time: 0.01,
            dash_friction: 50.0,
            dash_stop_time: 0.2,
        },
        Stats {
            speed: vec![StatModifier {
                flat: 250.0,
                multiplier: 1.0,
                timer: None,
                stat_type: StatType::Speed,
                trigger: ModifierTrigger::Cast,
            }],
            acceleration: vec![StatModifier {
                flat: 1250.0,
                multiplier: 1.0,
                timer: None,
                stat_type: StatType::Acceleration,
                trigger: ModifierTrigger::Cast,
            }],
            friction: vec![StatModifier {
                flat: 625.0,
                multiplier: 1.0,
                timer: None,
                stat_type: StatType::Friction,
                trigger: ModifierTrigger::Cast,
            }],
            dash_speed: vec![StatModifier {
                flat: 600.0,
                multiplier: 1.0,
                timer: None,
                stat_type: StatType::DashSpeed,
                trigger: ModifierTrigger::Cast,
            }],
            dash_time: vec![StatModifier {
                flat: 0.01,
                multiplier: 1.0,
                timer: None,
                stat_type: StatType::DashTime,
                trigger: ModifierTrigger::Cast,
            }],
            dash_friction: vec![StatModifier {
                flat: 500.0,
                multiplier: 1.0,
                timer: None,
                stat_type: StatType::DashFriction,
                trigger: ModifierTrigger::Cast,
            }],
            dash_stop_time: vec![StatModifier {
                flat: 0.01,
                multiplier: 1.0,
                timer: None,
                stat_type: StatType::DashStopTime,
                trigger: ModifierTrigger::Cast,
            }],
        },
        Facing(Vec2::X),
        Velocity::default(),
        MovementState::Idle,
        Health {
            current: 100.0,
            max: 100.0,
        },
        AttackStats { attack: 25.0 },
        CombatState::Idle,
    ));

    // Dummy enemy
    commands.spawn((
        Sprite {
            color: Color::srgb(0., 0., 1.0),
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(100.0, 0.0, 0.0),
        Enemy,
        Health {
            current: 100.0,
            max: 100.0,
        },
        AttackStats { attack: 25.0 },
        CombatState::Idle,
    ));
}
