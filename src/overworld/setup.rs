use bevy::prelude::*;
use bevy::asset::{AssetServer, Assets, Handle};
use bevy_ecs_tiled::prelude::*;
use avian2d::prelude::*;
use bevy_ecs_tiled::prelude::tiled::PropertyValue;
use crate::overworld::components::*;
use crate::overworld::overworld;

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
                            YSort,
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
                                Collider::rectangle(32.0, 32.0),
                                Sensor,
                                CollidingEntities::default(),
                                CollisionEventsEnabled,
                                Transform::from_xyz(16.0, 16.0, 0.0),
                                GlobalTransform::default(),
                            ));
                        });
                    }
                    "lamp" => {
                        let interactable_entity = entity;

                        commands.entity(entity).insert((
                            Interactable,
                            YSort,
                            InteractionType::Lamp,
                            InteractionState::Off,
                        ));
                        commands.entity(entity).with_children(|parent| {
                            parent.spawn((
                                InteractionFieldMarker,
                                InteractionField {
                                    owner: interactable_entity,
                                },
                                Collider::rectangle(32.0, 32.0),
                                Sensor,
                                CollidingEntities::default(),
                                CollisionEventsEnabled,
                                Transform::from_xyz(16.0, 16.0, 0.0),
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
                            Interactable,
                            YSort,
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
                                Collider::rectangle(32.0, 32.0),
                                Sensor,
                                CollidingEntities::default(),
                                CollisionEventsEnabled,
                                Transform::from_xyz(16.0, 16.0, 0.0),
                                GlobalTransform::default(),
                            ));
                        });
                    }
                    "block" => {
                        commands.entity(entity).insert((
                            Interactable,
                            YSort,
                            InteractionType::Block,
                            PushableBlock { grid_size: 32.0 },
                            Name::new(object.name.clone()),
                            RigidBody::Dynamic,
                            LockedAxes::ROTATION_LOCKED,
                            LinearDamping(100.0),  // high damping so physics doesn't interfere
                            GravityScale(0.0),
                        ));

                        // Interaction field child
                        commands.entity(entity).with_children(|parent| {
                            parent.spawn((
                                InteractionFieldMarker,
                                InteractionField { owner: entity },
                                Collider::rectangle(32.0, 32.0),
                                Sensor,
                                CollidingEntities::default(),
                                CollisionEventsEnabled,
                                Transform::from_xyz(16.0, 16.0, 0.0),
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

pub fn load_block_spritesheet(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(32, 46),
        9, 1,
        None, None,
    ));
    commands.insert_resource(BlockSpriteSheet {
        image: asset_server.load("sprites/objects/block_push.png"),
        layout,
    });
}
pub fn debug_ysort(
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &Transform, Option<&Name>), With<YSort>>,
) {
    if !keyboard.just_pressed(KeyCode::F1) {
        return;
    }
    for (entity, tf, name) in &query {
        info!("YSort entity {:?} name={:?} z={:.3} y={:.3}",
            entity, name, tf.translation.z, tf.translation.y);
    }
}