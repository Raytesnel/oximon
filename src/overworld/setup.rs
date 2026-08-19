use crate::common::components::GameLayer;
use crate::overworld::components::*;
use crate::overworld::player_movement;
use avian2d::prelude::*;
use bevy::asset::{AssetServer, Assets, Handle};
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::tiled::PropertyValue;
use bevy_ecs_tiled::prelude::*;

const ELEVATION_STEP: f32 = 10.0;
const FLOOR_BACK_OFFSET: f32 = 1.0;

pub fn sync_player_elevation_filter(
    mut player_q: Query<(&Elevation, &mut CollisionLayers), (Changed<Elevation>, With<OverworldPlayer>)>,
) {
    for (elevation, mut layers) in &mut player_q {
        let layer = elevation_layer(elevation.0);
        *layers = CollisionLayers::new(
            [GameLayer::Overworld, layer],
            [GameLayer::Overworld, layer],
        );
    }
}

pub fn apply_fixed_elevation(
    mut query: Query<(&FixedElevation, &mut Transform, Option<&ChildOf>)>,
    parent_transform_q: Query<&Transform, Without<FixedElevation>>,
) {
    for (fixed, mut transform, child_of) in &mut query {
        let parent_z = child_of
            .and_then(|c| parent_transform_q.get(c.parent()).ok())
            .map(|t| t.translation.z)
            .unwrap_or(0.0);
        transform.translation.z = -parent_z + fixed.0;
    }
}

pub fn stair_elevation_system(
    mut collision_started: MessageReader<CollisionStart>,
    stair_q: Query<&StairZone>,
    mut player_q: Query<(Entity, &mut Elevation), With<OverworldPlayer>>,
) {
    let Ok((player_entity, mut elevation)) = player_q.single_mut() else {
        return;
    };

    for event in collision_started.read() {
        let zone_entity = if event.collider1 == player_entity {
            event.collider2
        } else if event.collider2 == player_entity {
            event.collider1
        } else {
            continue;
        };
        if let Ok(zone) = stair_q.get(zone_entity) {
            elevation.0 = zone.target;
        }
    }
}

fn get_elevation(object: &tiled::Object<'_>) -> Option<i32> {
    fn get_int(props: &tiled::Properties, key: &str) -> Option<i32> {
        match props.get(key) {
            Some(PropertyValue::IntValue(n)) => Some(*n),
            _ => None,
        }
    }
    get_int(&object.properties, "elevation").or_else(|| {
        object
            .get_tile()
            .and_then(|ot| ot.get_tile())
            .and_then(|t| get_int(&t.properties, "elevation"))
    })
}
pub fn elevation_layer(level: i32) -> GameLayer {
    match level {
        -1 => GameLayer::Elevation0,
        0 => GameLayer::Elevation1,
        1 => GameLayer::Elevation2,
        _ => GameLayer::Elevation3,
    }
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
            |collider_created: On<TiledEvent<ColliderCreated>>,
             assets: Res<Assets<TiledMapAsset>>,
             layer_elevations: Res<LayerElevations>,
             parent_q: Query<&ChildOf>,
             mut commands: Commands| {
                let event = collider_created.event();
                let is_stair_zone = event
                    .get_object(&assets)
                    .map(|object| object.properties.contains_key("target_elevation"))
                    .unwrap_or(false);

                if is_stair_zone {
                    commands.entity(event.origin).insert((Sensor, CollisionEventsEnabled));
                    return;
                }

                // Walk up the hierarchy until we find an ancestor layer with a known elevation.
                let mut level = 0;
                let mut current = event.origin;
                while let Ok(child_of) = parent_q.get(current) {
                    let parent = child_of.parent();
                    if let Some(l) = layer_elevations.0.get(&parent) {
                        level = *l;
                        break;
                    }
                    current = parent;
                }

                let layer = elevation_layer(level);
                commands.entity(event.origin).insert((
                    RigidBody::Static,
                    CollisionLayers::new(layer, [layer]),
                ));
            },
        )
        .observe(
            |layer_created: On<TiledEvent<LayerCreated>>,
             assets: Res<Assets<TiledMapAsset>>,
             mut layer_elevations: ResMut<LayerElevations>| {
                let event = layer_created.event();
                let Some(layer) = event.get_layer(&assets) else {
                    return;
                };
                if let Some(PropertyValue::IntValue(n)) = layer.properties.get("elevation") {
                    layer_elevations.0.insert(event.origin, *n);
                }
            },
        )
        .observe(
            move |object_created: On<TiledEvent<ObjectCreated>>,
                  assets: Res<Assets<TiledMapAsset>>,
                  layer_elevations: Res<LayerElevations>,
                  parent_q: Query<&ChildOf>,
                  mut commands: Commands| {
                let event = object_created.event();
                let object = event.get_object(&assets).expect("object must exist");
                let entity = event.origin;

                let from_layer = parent_q
                    .get(entity)
                    .ok()
                    .and_then(|child_of| layer_elevations.0.get(&child_of.parent()).copied());

                let level = get_elevation(&object).or(from_layer).unwrap_or(0);
                commands.entity(entity).insert(Elevation(level));
                let debug_location =
                    object
                        .properties
                        .get("debug_location")
                        .and_then(|v| match v {
                            PropertyValue::StringValue(s) => Some(s.clone()),
                            _ => None,
                        });

                if let Some(debug_location) = debug_location {
                    commands
                        .entity(entity)
                        .insert(DebugLocation(debug_location));
                }
                let target_elevation =
                    object
                        .properties
                        .get("target_elevation")
                        .and_then(|v| match v {
                            PropertyValue::IntValue(n) => Some(*n),
                            _ => None,
                        });

                if let Some(target) = target_elevation {
                    let (width, height) = match object.shape {
                        tiled::ObjectShape::Rect { width, height } => (width, height),
                        _ => (32.0, 32.0), // fallback for non-rect shapes
                    };
                    commands.entity(entity).insert((
                        StairZone { target },
                        Sensor,
                        Collider::rectangle(width, height),
                        CollidingEntities::default(),
                        CollisionEventsEnabled,
                    ));
                    return; // stair zones are invisible triggers, skip the obj_type match entirely
                }

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
                            CollisionLayers::new(GameLayer::Overworld, [GameLayer::Overworld]),
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
                                CollisionLayers::new(GameLayer::Overworld, [GameLayer::Overworld]),
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
                            CollisionLayers::new(GameLayer::Overworld, [GameLayer::Overworld]),
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
                            CollisionLayers::new(GameLayer::Overworld, [GameLayer::Overworld]),
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
                            CollisionLayers::new(GameLayer::Overworld, [GameLayer::Overworld]),
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
                    "overhang" => {
                        commands.entity(entity).insert(FixedElevation(
                            level as f32 * ELEVATION_STEP - FLOOR_BACK_OFFSET,
                        ));
                    }

                    "floor" => {
                        commands.entity(entity).insert(FixedElevation(
                            level as f32 * ELEVATION_STEP + FLOOR_BACK_OFFSET,
                        ));
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
    if !keyboard.just_pressed(KeyCode::F2) {
        return;
    }
    for (entity, tf, name) in &query {
        info!(
            "YSort entity {:?} name={:?} z={:.3} y={:.3}",
            entity, name, tf.translation.z, tf.translation.y
        );
    }
}

pub fn debug_collision_layers(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_q: Query<(&Elevation, &Transform, &CollisionLayers), With<OverworldPlayer>>,
    collider_q: Query<Entity, With<Collider>>,
    elevation_collider_q: Query<&Elevation, With<Collider>>,
) {
    if !keyboard.just_pressed(KeyCode::F3) {
        return;
    }
    if let Ok((elevation, transform, layers)) = player_q.single() {
        info!(
            "PLAYER: elevation={} z={:.3} membership={:?} filter={:?}",
            elevation.0, transform.translation.z, layers.memberships, layers.filters,
        );
    }
    info!("Total colliders: {}", collider_q.iter().count());
    for level in [-1, 0, 1, 2] {
        let count = elevation_collider_q.iter().filter(|e| e.0 == level).count();
        info!("  elevation {}: {} colliders", level, count);
    }
}