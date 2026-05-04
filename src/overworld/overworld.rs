use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use crate::movement::components::{Facing, Movable, MoveIntent, MovementState, Velocity};
use crate::GameState;

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
    ));
}
pub fn overworld_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut MoveIntent, With<OverworldPlayer>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for mut intent in &mut query {
        let mut dir = Vec3::ZERO;

        if keyboard.pressed(KeyCode::KeyW) {
            dir.y += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            dir.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            dir.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
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