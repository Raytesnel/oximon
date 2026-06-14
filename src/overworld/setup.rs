use crate::overworld::components::*;
use crate::overworld::player_movement;
use avian2d::prelude::*;
use bevy::asset::{AssetServer, Assets, Handle};
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::tiled::PropertyValue;
use bevy_ecs_tiled::prelude::*;

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
            move |object_created: On<TiledEvent<ObjectCreated>>,
                  assets: Res<Assets<TiledMapAsset>>,
                  mut commands: Commands| {
                let event = object_created.event();
                let object = event.get_object(&assets).expect("object must exist");
                let entity = event.origin;
                let obj_type = if !object.user_type.is_empty() {
                    object.user_type.clone()
                } else {
                    object
                        .get_tile()
                        .and_then(|ot| ot.get_tile())
                        .and_then(|t| t.user_type.clone())
                        .unwrap_or_default()
                };
                match obj_type.as_str() {
                    "chest" => {
                        let interactable_entity = entity;

                        let mut entity_cmd = commands.entity(entity);

                        entity_cmd.insert((
                            Interactable,
                            YSort,
                            InteractionType::Chest,
                            InteractionState::Closed,
                            Name::new(object.name.clone()),
                        ));
                        if let Some(props) = parse_spritesheet_props(&object) {
                            entity_cmd.insert(props);
                        }
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
                    "monster" => {
                        let interactable_entity = entity;

                        let mut entity_cmd = commands.entity(entity);

                        entity_cmd.insert((Interactable, YSort, InteractionType::Monster));
                        if let Some(props) = parse_spritesheet_props(&object) {
                            entity_cmd.insert(props);
                        }
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

                        let mut entity_cmd = commands.entity(entity);

                        entity_cmd.insert((
                            Interactable,
                            YSort,
                            InteractionType::Lamp,
                            InteractionState::Off,
                        ));
                        if let Some(props) = parse_spritesheet_props(&object) {
                            entity_cmd.insert(props);
                        }
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
                        let mut entity_cmd = commands.entity(entity);

                        entity_cmd.insert((
                            Interactable,
                            YSort,
                            InteractionType::Block,
                            PushableBlock { grid_size: 32.0 },
                            Name::new(object.name.clone()),
                            RigidBody::Kinematic,
                            LockedAxes::ROTATION_LOCKED,
                            LinearDamping(100.0), // high damping so physics doesn't interfere
                            GravityScale(0.0),
                        ));
                        if let Some(props) = parse_spritesheet_props(&object) {
                            entity_cmd.insert(props);
                        }
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

    player_movement::spawn_player_overworld(&mut commands);
}

pub fn hide_overworld(mut commands: Commands, query: Query<Entity, With<OverworldEntity>>) {
    for e in &query {
        commands.entity(e).insert(Visibility::Hidden);
    }
}

pub fn show_overworld(mut commands: Commands, query: Query<Entity, With<OverworldEntity>>) {
    for e in &query {
        commands.entity(e).insert(Visibility::Visible);
    }
}

fn parse_spritesheet_props(object: &tiled::Object<'_>) -> Option<SpriteSheetProps> {
    fn get_string(props: &tiled::Properties, key: &str) -> Option<String> {
        if let Some(PropertyValue::StringValue(s)) = props.get(key) {
            Some(s.clone())
        } else {
            None
        }
    }
    fn get_int(props: &tiled::Properties, key: &str) -> Option<u32> {
        if let Some(PropertyValue::IntValue(n)) = props.get(key) {
            Some(*n as u32)
        } else {
            None
        }
    }

    let instance_props = &object.properties;
    // Tile derefs to TileData which has a .properties field directly
    let tile_props = object
        .get_tile()
        .and_then(|ot| ot.get_tile())
        .map(|t| t.properties.clone());

    let get_s = |key| {
        get_string(instance_props, key)
            .or_else(|| tile_props.as_ref().and_then(|p| get_string(p, key)))
    };
    let get_i = |key| {
        get_int(instance_props, key).or_else(|| tile_props.as_ref().and_then(|p| get_int(p, key)))
    };

    let path = get_s("sprite_sheet")?;
    let width = get_i("sprite_width").unwrap_or(32);
    let height = get_i("sprite_height").unwrap_or(32);
    let columns = get_i("sprite_columns").unwrap_or(1);
    let rows = get_i("sprite_rows").unwrap_or(1);

    Some(SpriteSheetProps {
        path,
        width,
        height,
        columns,
        rows,
    })
}
pub fn debug_ysort(
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &Transform, Option<&Name>), With<YSort>>,
) {
    if !keyboard.just_pressed(KeyCode::F1) {
        return;
    }
    for (entity, tf, name) in &query {
        info!(
            "YSort entity {:?} name={:?} z={:.3} y={:.3}",
            entity, name, tf.translation.z, tf.translation.y
        );
    }
}
