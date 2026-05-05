use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use crate::combat::components::AttackId;
use crate::movement::input::*;
use crate::common::components::{ComputedStats, ModifierLifetime, RuntimeModifier, StatType, Stats};
use crate::movement::components::{Facing, Movable, MoveIntent, MovementState, Velocity};
use crate::{GameState, MainCamera};

#[derive(Component)]
pub struct OverworldEntity;


pub fn setup_overworld(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load your tiled map

    let map_handle: Handle<TiledMapAsset> = asset_server.load("map/main.tmx");

    // Spawn a new entity with the TiledMap component
    commands.spawn((TiledMap(map_handle),OverworldEntity));

    spawn_player_overworld(&mut commands);
}
#[derive(Component)]
pub struct OverworldPlayer;

pub fn spawn_player_overworld(commands: &mut Commands) {
    commands.spawn((
        OverworldPlayer,
        OverworldEntity,
        Sprite {
            color: Color::srgb(1.0, 0., 1.0),
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(0., 0., 10.),
        Movable,
        Velocity::default(),
        MovementState::Idle,
        MoveIntent { direction: Vec3::ZERO },
        Facing(Vec2::X),
        ComputedStats {
            speed: 150.0,
            acceleration: 800.0,
            friction: 500.0,
            dash_speed: 0.0,
            dash_time: 0.0,
            dash_friction: 0.0,
            dash_stop_time: 0.0,
        },
    ));
}
pub fn overworld_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut MoveIntent, With<OverworldPlayer>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for mut intent in &mut query {
        let mut dir = Vec3::ZERO;

        if keyboard.pressed(MOVE_UP_BUTTON) {
            dir.y += 1.0;
        }
        if keyboard.pressed(MOVE_DOWN_BUTTON) {
            dir.y -= 1.0;
        }
        if keyboard.pressed(MOVE_LEFT_BUTTON) {
            dir.x -= 1.0;
        }
        if keyboard.pressed(MOVE_RIGHT_BUTTON) {
            dir.x += 1.0;
        }

        intent.direction = dir.normalize_or_zero();
    }
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Combat);
    }
}

pub fn cleanup_overworld(
    mut commands: Commands,
    query: Query<Entity, With<OverworldEntity>>,
) {
    for e in &query {
        commands.entity(e).despawn();
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
    let lerp_factor = 0.01;

    if delta.x.abs() > deadzone.x {
        cam_transform.translation.x = cam_transform
            .translation.x
            .lerp(player_transform.translation.x, lerp_factor);
    }

    if delta.y.abs() > deadzone.y {
        cam_transform.translation.y = cam_transform
            .translation.y
            .lerp(player_transform.translation.y, lerp_factor);
    }
}