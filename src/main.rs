pub mod movement;

use crate::movement::MovementPlugin;
use bevy::prelude::*;
use movement::components::{Player, Velocity};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MovementPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(20.0, 20.0)), // your "block"
            ..default()
        },
        Transform::from_xyz(0., 0., 0.),
        Player,
        Velocity::default(),
    ));
}
