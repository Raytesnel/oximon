use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use avian2d::prelude::*;
use crate::combat::components::AttackId;
use crate::movement::input::*;
use crate::common::components::{ComputedStats, ModifierLifetime, RuntimeModifier, StatType, Stats};
use crate::movement::components::{Facing, Movable, MoveIntent, MovementState, Velocity};
use crate::{GameState, MainCamera};

#[derive(Component)]
pub struct OverworldEntity;


pub fn setup_overworld(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map_handle: Handle<TiledMapAsset> = asset_server.load("map/main.tmx");

    commands
        .spawn((
            TiledMap(map_handle),
            OverworldEntity,
            TiledPhysicsSettings::<TiledPhysicsAvianBackend>::default(),
        ))
        .observe(
            |collider_created: On<TiledEvent<ColliderCreated>>,
             mut commands: Commands| {
                commands
                    .entity(collider_created.event().origin)
                    .insert(RigidBody::Static);
                info!("Collider entity: {:?}", collider_created.event().origin);
                // log the source info too
                info!("Collider source: {:?}", collider_created.event().event.source);
            },
        );

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
        RigidBody::Dynamic,
        Collider::rectangle(18.0, 18.0),
        LockedAxes::ROTATION_LOCKED,
        LinearDamping(8.0),
        Friction::new(0.0),
        Restitution::new(0.0),
        GravityScale(0.0),
        CollisionEventsEnabled

    ));
}
pub fn overworld_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut LinearVelocity, With<OverworldPlayer>>,
    time: Res<Time>,
) {
    let max_speed = 200.0;
    let acceleration = 15.0;

    for mut lin_vel in &mut query {
        let mut dir = Vec2::ZERO;
        if keyboard.pressed(MOVE_UP_BUTTON)    { dir.y += 1.0; }
        if keyboard.pressed(MOVE_DOWN_BUTTON)  { dir.y -= 1.0; }
        if keyboard.pressed(MOVE_LEFT_BUTTON)  { dir.x -= 1.0; }
        if keyboard.pressed(MOVE_RIGHT_BUTTON) { dir.x += 1.0; }

        let target = dir.normalize_or_zero() * max_speed;
        lin_vel.0 = lin_vel.0.lerp(target, acceleration * time.delta_secs());
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