use crate::GameState;
use crate::common::components::CombatSpawnContext;
use crate::overworld::components::*;
use avian2d::prelude::*;
use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::prelude::*;

pub fn on_sign_interaction(
    trigger: On<InteractionEvent>,
    query: Query<(&InteractionType, &SignText)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let entity = trigger.event().entity;
    let Ok((InteractionType::Sign, sign_text)) = query.get(entity) else {
        return;
    };

    // Spawn popup as a child — inherits the sign's Transform
    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            SignPopup {
                timer: Timer::from_seconds(3.0, TimerMode::Once),
            },
            Text2d::new(sign_text.0.clone()),
            TextFont {
                font: asset_server.load("fonts/your_font.otf"),
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::WHITE),
            // Float 32px above the sign
            Transform::from_xyz(0.0, 32.0, 5.0),
        ));
    });
}

pub fn tick_sign_popups(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut SignPopup)>,
) {
    for (entity, mut popup) in &mut query {
        popup.timer.tick(time.delta());
        if popup.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn on_lamp_interaction(
    trigger: On<InteractionEvent>,
    mut query: Query<(
        &InteractionType,
        &mut InteractionState,
        Option<&SpriteSheetHandle>,
        Option<&SpriteSheetProps>,
    )>,
    children_q: Query<&Children>,
    name_q: Query<&Name>,
    mut sprite_q: Query<&mut Sprite>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let entity = trigger.event().entity;
    let Ok((InteractionType::Lamp, mut state, maybe_handle, maybe_props)) = query.get_mut(entity)
    else {
        return;
    };
    let handle = if let Some(h) = maybe_handle {
        h.clone()
    } else if let Some(props) = maybe_props {
        let layout = layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(props.width, props.height),
            props.columns,
            props.rows,
            None,
            None,
        ));
        let h = SpriteSheetHandle {
            image: asset_server.load(props.path.clone()),
            layout,
        };
        commands.entity(entity).insert(h.clone());
        h
    } else {
        warn!("lamp has no spritesheet props");
        return;
    };
    // Find the TiledObjectVisual child that holds the sprite
    let Ok(children) = children_q.get(entity) else {
        return;
    };

    let visual_child = children.iter().find(|child| {
        name_q
            .get(child.clone().clone())
            .map(|n| n.as_str() == "TiledObjectVisual")
            .unwrap_or(false)
    });

    let Some(visual_entity) = visual_child else {
        warn!("no TiledObjectVisual child found");
        return;
    };

    let Ok(mut sprite) = sprite_q.get_mut(visual_entity.clone()) else {
        warn!("TiledObjectVisual has no Sprite");
        return;
    };

    let frames = match *state {
        InteractionState::Off => {
            *state = InteractionState::On;
            info!("turning ON: lamp");
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        }
        _ => {
            *state = InteractionState::Off;
            info!("turning OFF: lamp");
            vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0]
        }
    };
    info!("inserting frames: {:?}", frames);

    sprite.image = handle.image.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: handle.layout.clone(),
        index: frames[0],
    });

    // Put the animation state on the visual child, not the parent
    // so tick_lamp_animation can find the Sprite easily
    commands
        .entity(visual_entity.clone())
        .insert(LampAnimationState {
            timer: Timer::from_seconds(0.08, TimerMode::Repeating),
            frames,
            current: 0,
            hold_on_last: true,
        });

    info!("lamp animation started on visual child {:?}", visual_entity);
}

pub fn tick_lamp_animation(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut LampAnimationState, &mut Sprite)>,
) {
    for (entity, mut anim, mut sprite) in &mut query {
        anim.timer.tick(time.delta());
        if !anim.timer.just_finished() {
            continue;
        }

        let next = anim.current + 1;

        if next >= anim.frames.len() {
            // Reached the last frame
            if anim.hold_on_last {
                // Remove the animator — sprite stays on last frame
                commands.entity(entity).remove::<LampAnimationState>();
            } else {
                anim.current = 0; // loop
            }
        } else {
            anim.current = next;
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = anim.frames[next];
            }
        }
    }
}

// Snap a world position to the nearest grid cell (32x32)
fn to_grid(pos: Vec2, grid_size: f32) -> IVec2 {
    IVec2::new(
        (pos.x / grid_size).round() as i32,
        (pos.y / grid_size).round() as i32,
    )
}

pub fn on_monster_interaction(
    trigger: On<InteractionEvent>,
    monster_q: Query<(&InteractionType)>,
    player_q: Query<&Transform, With<OverworldPlayer>>,
    mut world_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    let entity = trigger.event().entity;
    let Ok((InteractionType::Monster)) = monster_q.get(entity) else {
        return;
    };
    if let Ok(player_tf) = player_q.single() {
        commands.insert_resource(CombatSpawnContext {
            player_world_pos: player_tf.translation,
        });
    }
    world_state.set(GameState::Combat);
}

pub fn on_block_interaction(
    trigger: On<InteractionEvent>,
    block_q: Query<(
        &InteractionType,
        &PushableBlock,
        &Transform,
        Option<&SpriteSheetHandle>,
        Option<&SpriteSheetProps>,
    )>,
    player_q: Query<(&Facing, &Transform), With<OverworldPlayer>>,
    obstacle_q: Query<&Transform, Or<(With<RigidBody>, With<PushableBlock>)>>,
    sliding_q: Query<&BlockSliding>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let entity = trigger.event().entity;
    let Ok((InteractionType::Block, block, block_tf, maybe_handle, maybe_props)) =
        block_q.get(entity)
    else {
        return;
    };
    if sliding_q.get(entity).is_ok() {
        return;
    }
    let Ok((facing, _player_tf)) = player_q.single() else {
        return;
    };

    let push_dir: Vec2 = match facing {
        Facing::Up => Vec2::Y,
        Facing::Down => -Vec2::Y,
        Facing::Left => -Vec2::X,
        Facing::Right => Vec2::X,
    };
    let current_pos = block_tf.translation.truncate();
    let target_pos = current_pos + push_dir * block.grid_size;
    let target_grid = to_grid(target_pos, block.grid_size);

    let is_blocked = obstacle_q.iter().any(|obs_tf| {
        let obs_pos = obs_tf.translation.truncate();
        if to_grid(obs_pos, block.grid_size) == to_grid(current_pos, block.grid_size) {
            return false;
        }
        to_grid(obs_pos, block.grid_size) == target_grid
    });
    if is_blocked {
        return;
    }

    // Build spritesheet handle lazily on first push
    let handle = if let Some(h) = maybe_handle {
        Some(h.clone())
    } else if let Some(props) = maybe_props {
        let layout = layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(props.width, props.height),
            props.columns,
            props.rows,
            None,
            None,
        ));
        let h = SpriteSheetHandle {
            image: asset_server.load(props.path.clone()),
            layout,
        };
        commands.entity(entity).insert(h.clone());
        Some(h)
    } else {
        None
    };

    commands.entity(entity).insert(BlockSliding {
        from: current_pos,
        to: target_pos,
        timer: Timer::from_seconds(1.0, TimerMode::Once),
    });
}

pub fn tick_block_sliding(
    mut commands: Commands,
    time: Res<Time>,

    mut query: Query<(
        Entity,
        &mut BlockSliding,
        &mut Transform,
        Option<&Children>,
        &SpriteSheetHandle,
    )>,
    name_q: Query<&Name>,
    mut sprite_q: Query<&mut Sprite>,
) {
    for (entity, mut sliding, mut tf, children, spritesheet) in &mut query {
        sliding.timer.tick(time.delta());
        let t = sliding.timer.fraction(); // 0.0 -> 1.0

        // Smooth step for nicer feel
        let smoothed = t * t * (3.0 - 2.0 * t);
        let new_pos = sliding.from.lerp(sliding.to, smoothed);
        tf.translation.x = new_pos.x;
        tf.translation.y = new_pos.y;

        // Animate the visual child if spritesheet is loaded
        if let (sheet, Some(children)) = (spritesheet.clone(), children) {
            let visual = children.iter().find(|&c| {
                name_q
                    .get(c)
                    .map(|n| n.as_str() == "TiledObjectVisual")
                    .unwrap_or(false)
            });

            if let Some(visual_entity) = visual {
                if let Ok(mut sprite) = sprite_q.get_mut(visual_entity) {
                    let frame_count = 9usize;
                    let frame = ((t * frame_count as f32) as usize).min(frame_count - 1);

                    // Always set both — don't branch on whether atlas exists
                    sprite.image = sheet.image.clone();
                    sprite.rect = None; // clear any Tiled rect
                    sprite.custom_size = Some(Vec2::new(32.0, 46.0));

                    if let Some(atlas) = &mut sprite.texture_atlas {
                        atlas.layout = sheet.layout.clone();
                        atlas.index = frame;
                    } else {
                        sprite.texture_atlas = Some(TextureAtlas {
                            layout: sheet.layout.clone(),
                            index: frame,
                        });
                    }
                }
            }
        }

        if sliding.timer.is_finished() {
            // Snap exactly to target to avoid float drift
            tf.translation.x = sliding.to.x;
            tf.translation.y = sliding.to.y;
            commands.entity(entity).remove::<BlockSliding>();
            info!("block slide complete");
        }
    }
}
