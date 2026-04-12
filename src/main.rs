pub mod movement;
mod combat;
mod common;

use crate::movement::MovementPlugin;
use bevy::prelude::*;
use common::components::Enemy;
use movement::components::{MovementState, Velocity};
use crate::combat::CombatPlugin;
use crate::combat::components::{CombatState, Health, Stats};
use crate::common::components::Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MovementPlugin)
        .add_plugins(CombatPlugin)
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
        Velocity::default(),
        MovementState::Idle,

        Health {
            current: 100.0,
            max: 100.0,
        },
        Stats {
            attack: 25.0,
        },
        CombatState::Idle,
    ));

    // Dummy enemy
    commands.spawn((
        Sprite {
            color: Color::srgb(0.,0.,1.0),
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(100.0, 0.0, 0.0),

        Enemy,

        Health {
            current: 100.0,
            max: 100.0,
        },
        Stats {
            attack: 25.0,
        },
        CombatState::Idle,
    ));

}
