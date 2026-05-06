use std::ops::Deref;
use crate::combat::components::AttackId;
use crate::common::components::{
    ComputedStats, ModifierLifetime, RuntimeModifier, StatType, Stats,
};
use crate::movement::components::{Movable, MoveIntent, MovementState, Velocity};
use crate::movement::input::*;
use crate::{GameState, MainCamera};
use avian2d::prelude::*;
use bevy::ecs::event::Trigger;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tiled::prelude::tiled::PropertyValue;

#[derive(Component)]
pub struct OverworldEntity;

#[derive(Component)]
pub struct Interactable;

#[derive(Component)]
pub enum InteractionType {
    Chest,
    Lamp,
    Sign,
}

#[derive(Component, PartialEq)]
pub enum InteractionState {
    Closed,
    Open,
    Off,
    On,
}

#[derive(Component)]
pub struct SignText(pub String);

#[derive(Event)]
pub struct InteractionEvent {
    pub entity: Entity,
}
#[derive(Component, Clone, Copy)]
pub enum Facing {
    Up,
    Down,
    Left,
    Right,
}
pub fn setup_overworld(mut commands: Commands, asset_server: Res<AssetServer>) {
    let map_handle: Handle<TiledMapAsset> = asset_server.load("map/main.tmx");

    commands
        .spawn((
            TiledMap(map_handle),
            OverworldEntity,
            TiledPhysicsSettings::<TiledPhysicsAvianBackend>::default(),
        ))
        .observe(
            |collider_created: On<TiledEvent<ColliderCreated>>, mut commands: Commands| {
                commands
                    .entity(collider_created.event().origin)
                    .insert(RigidBody::Static);
            },
        )
        .observe(
            |object_created: On<TiledEvent<ObjectCreated>>,
             assets: Res<Assets<TiledMapAsset>>,
             mut commands: Commands| {
                let event = object_created.event();

                let object = event.get_object(&assets).expect("object must exist");

                let entity = event.origin;

                commands.entity(entity).insert(YSort);
                let obj_type = object.user_type.as_str();

                match obj_type {
                    "chest" => {
                        commands.entity(entity).insert((
                            Interactable,
                            InteractionType::Chest,
                            InteractionState::Closed,
                            Name::new(object.name.clone()),
                        ));
                    }
                    "lamp" => {
                        commands.entity(entity).insert((
                            Interactable,
                            InteractionType::Lamp,
                            InteractionState::Off,
                            Name::new(object.name.clone()),
                        ));
                    }
                    "sign" => {
                        let text = object
                            .properties
                            .get("text")
                            .and_then(|v| {
                                match v {
                                    PropertyValue::StringValue(s) => Some(s.clone()),
                                    _ => None,
                                }
                            })
                            .unwrap_or("...".to_string());

                        commands.entity(entity).insert((
                            Interactable,
                            InteractionType::Sign,
                            SignText(text),
                            Name::new(object.name.clone()),
                        ));
                    }
                    _ => {}
                }
            },
        )
        .observe(
            |object_created: On<TiledEvent<ObjectCreated>>, mut commands: Commands| {
                commands.entity(object_created.event().origin).insert(YSort);
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
pub fn overworld_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut LinearVelocity, With<OverworldPlayer>>,
) {
    for mut lin_vel in &mut query {
        let mut dir = Vec2::ZERO;
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

        lin_vel.0 = dir.normalize_or_zero() * 150.0;
    }
}

pub fn y_sort(mut query: Query<&mut Transform, With<YSort>>) {
    for mut transform in &mut query {
        transform.translation.z = -transform.translation.y / 1000.0;
    }
}

#[derive(Component)]
pub struct YSort;

pub fn cleanup_overworld(mut commands: Commands, query: Query<Entity, With<OverworldEntity>>) {
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
pub fn interaction_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_q: Query<(&Transform, &Facing), With<OverworldPlayer>>,
    interactables: Query<(Entity, &Transform,Option<&Name>), With<Interactable>>,
    mut commands: Commands,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }
    let (player_tf, facing) = player_q.single().expect("Expected exactly one player");

    let forward = match facing {
        Facing::Up => Vec3::Y,
        Facing::Down => -Vec3::Y,
        Facing::Left => -Vec3::X,
        Facing::Right => Vec3::X,
    };

    let check_pos = player_tf.translation + forward * 32.0;
    info!("player forward check: {:?}", check_pos);
    for (entity, tf,name) in &interactables {
        let name = name.map(|n| n.as_str()).unwrap_or("Unnamed");
        info!("interactable at: {:?} with name:{:?}", tf.translation, name);
        if tf.translation.distance(check_pos) < 20.0 {
            commands.trigger(InteractionEvent { entity });
            break;
        } else {
            info!("nothing is standing in front of us.")
        }
    }
}
pub fn on_interaction(
    mut trigger: On<InteractionEvent>,
    mut query: Query<(&InteractionType, &mut InteractionState)>,
) {
    let entity = trigger.event().entity;

    if let Ok((kind, mut state)) = query.get_mut(entity) {
        match kind {
            InteractionType::Chest => {
                *state = InteractionState::Open;
                info!("we found an chest");
            }
            InteractionType::Lamp => {
                *state = match *state {
                    InteractionState::Off => InteractionState::On,
                    _ => InteractionState::Off,

                };
                info!("we found a lamp");
            }
            InteractionType::Sign => {
                info!("Sign interacted");
            }
        }
    }
}
