use crate::MainCamera;
use crate::movement::input::*;
use crate::overworld::components::{Facing, OverworldEntity, OverworldPlayer, YSort};
use avian2d::prelude::*;
use bevy::prelude::*;

pub fn spawn_player_overworld(commands: &mut Commands) {
    commands.spawn((
        OverworldPlayer,
        OverworldEntity,
        Sprite {
            color: Color::srgb(1.0, 0., 1.0),
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },
        YSort,
        Facing::Down,
        Transform::from_xyz(0., 0., 10.),
        RigidBody::Dynamic,
        Collider::rectangle(18.0, 18.0),
        LockedAxes::ROTATION_LOCKED,
        LinearDamping(8.0),
        Friction::new(0.0),
        Restitution::new(0.0),
        GravityScale(0.0),
        CollisionEventsEnabled,
    ));
}

pub fn y_sort(mut query: Query<&mut Transform, With<YSort>>) {
    for mut transform in &mut query {
        transform.translation.z = -transform.translation.y / 1000.0;
    }
}

pub fn camera_follow(
    mut cam_q: Query<&mut Transform, With<MainCamera>>,
    player_q: Query<&Transform, (With<OverworldPlayer>, Without<MainCamera>)>,
) {
    let player_transform = player_q.single().expect("Expected exactly one player");
    let mut cam_transform = cam_q.single_mut().expect("Expected exactly one camera");

    let deadzone = Vec2::new(200.0, 120.0);

    let delta = player_transform.translation - cam_transform.translation;
    let lerp_factor = 0.005;

    if delta.x.abs() > deadzone.x {
        cam_transform.translation.x = cam_transform
            .translation
            .x
            .lerp(player_transform.translation.x, lerp_factor);
    }

    if delta.y.abs() > deadzone.y {
        cam_transform.translation.y = cam_transform
            .translation
            .y
            .lerp(player_transform.translation.y, lerp_factor);
    }
}

pub fn update_facing(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Facing, With<OverworldPlayer>>,
) {
    for mut facing in &mut query {
        if keyboard.pressed(MOVE_UP_BUTTON) {
            *facing = Facing::Up;
        } else if keyboard.pressed(MOVE_DOWN_BUTTON) {
            *facing = Facing::Down;
        } else if keyboard.pressed(MOVE_LEFT_BUTTON) {
            *facing = Facing::Left;
        } else if keyboard.pressed(MOVE_RIGHT_BUTTON) {
            *facing = Facing::Right;
        }
    }
}
