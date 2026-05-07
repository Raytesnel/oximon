use bevy::prelude::{Commands, Entity, GlobalTransform, Name, On, Query, Res, Transform, With};
use bevy::asset::{AssetServer, Assets, Handle};
use bevy_ecs_tiled::prelude::{ColliderCreated, ObjectCreated, TiledEvent, TiledMap, TiledMapAsset, TiledPhysicsAvianBackend, TiledPhysicsSettings};
use avian2d::prelude::{Collider, CollidingEntities, CollisionEventsEnabled, RigidBody, Sensor};
use bevy_ecs_tiled::prelude::tiled::PropertyValue;
use crate::overworld::components::{Interactable, InteractionField, InteractionFieldMarker, InteractionState, InteractionType, OverworldEntity, SignText};
use crate::overworld::overworld;
use crate::overworld::components::YSort;

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
                let obj_type = object.user_type.as_str();

                match obj_type {
                    "chest" => {
                        let interactable_entity = entity;

                        commands.entity(entity).insert((
                            Interactable,
                            InteractionType::Chest,
                            InteractionState::Closed,
                            Name::new(object.name.clone()),
                        ));
                        commands.entity(entity).with_children(|parent| {
                            parent.spawn((
                                InteractionFieldMarker,
                                InteractionField {
                                    owner: interactable_entity,
                                },
                                Collider::rectangle(40.0, 40.0),
                                Sensor,
                                CollidingEntities::default(),
                                CollisionEventsEnabled,
                                Transform::default(),
                                GlobalTransform::default(),
                            ));
                        });
                    }
                    "lamp" => {
                        let interactable_entity = entity;

                        commands.entity(entity).insert((
                            Interactable,
                            InteractionType::Lamp,
                            InteractionState::Off,
                        ));
                        commands.entity(entity).with_children(|parent| {
                            parent.spawn((
                                InteractionFieldMarker,
                                InteractionField {
                                    owner: interactable_entity,
                                },
                                Collider::rectangle(40.0, 40.0),
                                Sensor,
                                CollidingEntities::default(),
                                CollisionEventsEnabled,
                                Transform::default(),
                                GlobalTransform::default(),
                            ));
                        });
                    }
                    "sign" => {
                        let text = object
                            .properties
                            .get("text")
                            .and_then(|v| match v {
                                PropertyValue::StringValue(s) => Some(s.clone()),
                                _ => None,
                            })
                            .unwrap_or("...".to_string());

                        let interactable_entity = entity;

                        commands.entity(entity).insert((
                            YSort,
                            Interactable,
                            InteractionType::Sign,
                            SignText(text),
                            Name::new(object.name.clone()),
                        ));

                        commands.entity(entity).with_children(|parent| {
                            parent.spawn((
                                InteractionFieldMarker,
                                InteractionField {
                                    owner: interactable_entity,
                                },
                                Collider::rectangle(40.0, 40.0),
                                Sensor,
                                CollidingEntities::default(),
                                CollisionEventsEnabled,
                                Transform::default(),
                                GlobalTransform::default(),
                            ));
                        });
                    }
                    _ => {
                        commands.entity(entity).insert(YSort);
                    }
                }
            },
        );

    overworld::spawn_player_overworld(&mut commands);
}

pub fn cleanup_overworld(mut commands: Commands, query: Query<Entity, With<OverworldEntity>>) {
    for e in &query {
        commands.entity(e).despawn();
    }
}