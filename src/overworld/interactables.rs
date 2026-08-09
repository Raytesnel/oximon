use crate::common::components::{BattleState, CombatSpawnContext};
use crate::overworld::components::*;
use avian2d::prelude::*;
use bevy::asset::AssetServer;
use bevy::color::Color;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

type PlayerQueryItem = (&'static Facing, &'static Transform);
type ObstacleQueryItem = &'static Transform;
type ObstacleFilter = (With<RigidBody>, With<PushableBlock>);
type BlockQueryItem = (
    &'static InteractionType,
    &'static PushableBlock,
    &'static Transform,
    Option<&'static SpriteSheetHandle>,
    Option<&'static SpriteSheetProps>,
);
type LampQueryItem = (
    &'static InteractionType,
    &'static mut InteractionState,
    Option<&'static SpriteSheetHandle>,
    Option<&'static SpriteSheetProps>,
);

#[derive(SystemParam)]
pub struct BlockInteractionContext<'w, 's> {
    pub block_q: Query<'w, 's, BlockQueryItem>,
    pub player_q: Query<'w, 's, PlayerQueryItem, With<OverworldPlayer>>,
    pub obstacle_q: Query<'w, 's, ObstacleQueryItem, Or<ObstacleFilter>>,
    pub sliding_q: Query<'w, 's, &'static BlockSliding>,
}

#[derive(SystemParam)]
pub struct LampInteractionContext<'w, 's> {
    pub lamp_q: Query<'w, 's, LampQueryItem>,
    pub children_q: Query<'w, 's, &'static Children>,
    pub name_q: Query<'w, 's, &'static Name>,
    pub sprite_q: Query<'w, 's, &'static mut Sprite>,
}

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
    mut lamp_ctx: LampInteractionContext,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let entity = trigger.event().entity;
    let Ok((InteractionType::Lamp, mut state, maybe_handle, maybe_props)) =
        lamp_ctx.lamp_q.get_mut(entity)
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
    let Ok(children) = lamp_ctx.children_q.get(entity) else {
        return;
    };

    let visual_child = children.iter().find(|child| {
        lamp_ctx
            .name_q
            .get(*child)
            .map(|n| n.as_str() == "TiledObjectVisual")
            .unwrap_or(false)
    });

    let Some(visual_entity) = visual_child else {
        warn!("no TiledObjectVisual child found");
        return;
    };

    let Ok(mut sprite) = lamp_ctx.sprite_q.get_mut(visual_entity) else {
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
    commands.entity(visual_entity).insert(LampAnimationState {
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
    monster_q: Query<&InteractionType>,
    player_q: Query<&Transform, With<OverworldPlayer>>,
    mut next_battle_state: ResMut<NextState<BattleState>>,
    mut commands: Commands,
) {
    let entity = trigger.event().entity;
    let Ok(InteractionType::Monster) = monster_q.get(entity) else {
        return;
    };

    if let Ok(player_tf) = player_q.single() {
        commands.insert_resource(CombatSpawnContext {
            player_world_pos: player_tf.translation,
        });
    }

    next_battle_state.set(BattleState::Entering);
}

pub fn on_block_interaction(
    trigger: On<InteractionEvent>,
    block_ctx: BlockInteractionContext,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let entity = trigger.event().entity;
    let Ok((InteractionType::Block, block, block_tf, maybe_handle, maybe_props)) =
        block_ctx.block_q.get(entity)
    else {
        return;
    };
    if block_ctx.sliding_q.get(entity).is_ok() {
        return;
    }
    let Ok((facing, _player_tf)) = block_ctx.player_q.single() else {
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

    let is_blocked = block_ctx.obstacle_q.iter().any(|obs_tf| {
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
    let _handle = if let Some(h) = maybe_handle {
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

            if let Some(visual_entity) = visual
                && let Ok(mut sprite) = sprite_q.get_mut(visual_entity)
            {
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

        if sliding.timer.is_finished() {
            // Snap exactly to target to avoid float drift
            tf.translation.x = sliding.to.x;
            tf.translation.y = sliding.to.y;
            commands.entity(entity).remove::<BlockSliding>();
            info!("block slide complete");
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    use crate::overworld::interactables::*;

    fn make_app_with_time(step_seconds: f32) -> App {
        let mut app = App::new();
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(step_seconds),
        ));
        app.add_plugins(MinimalPlugins);
        app
    }

    // ── to_grid ───────────────────────────────────────────────────────────────

    fn to_grid(pos: Vec2, grid_size: f32) -> IVec2 {
        IVec2::new(
            (pos.x / grid_size).round() as i32,
            (pos.y / grid_size).round() as i32,
        )
    }
    #[test]
    fn to_grid_exact_cell_centre() {
        assert_eq!(to_grid(Vec2::new(64.0, 96.0), 32.0), IVec2::new(2, 3));
    }

    #[test]
    fn to_grid_rounds_to_nearest() {
        assert_eq!(to_grid(Vec2::new(47.0, 0.0), 32.0), IVec2::new(1, 0));
        assert_eq!(to_grid(Vec2::new(49.0, 0.0), 32.0), IVec2::new(2, 0));
    }

    #[test]
    fn to_grid_negative_coords() {
        assert_eq!(to_grid(Vec2::new(-32.0, -64.0), 32.0), IVec2::new(-1, -2));
    }

    // ── tick_sign_popups ──────────────────────────────────────────────────────

    #[test]
    fn sign_popup_despawns_after_timer() {
        // Each update() advances time by 2 seconds
        let mut app = make_app_with_time(2.0);
        app.add_systems(Update, tick_sign_popups);

        let popup = app
            .world_mut()
            .spawn(SignPopup {
                timer: Timer::from_seconds(3.0, TimerMode::Once),
            })
            .id();
        app.update();
        app.update();
        assert!(app.world().get_entity(popup).is_ok(), "popup alive at 2 s");
        for _ in 0..15 {
            app.update();
        }
        assert!(
            app.world().get_entity(popup).is_err(),
            "popup must be despawned after timer expires"
        );
    }

    #[test]
    fn sign_popup_survives_before_timer() {
        let mut app = make_app_with_time(1.0);
        app.add_systems(Update, tick_sign_popups);

        let popup = app
            .world_mut()
            .spawn(SignPopup {
                timer: Timer::from_seconds(3.0, TimerMode::Once),
            })
            .id();

        app.update(); // +1 s
        app.update(); // +2 s
        assert!(app.world().get_entity(popup).is_ok());
    }

    // ── tick_lamp_animation ───────────────────────────────────────────────────

    fn spawn_lamp_visual(app: &mut App, frames: Vec<usize>) -> Entity {
        app.world_mut()
            .spawn((
                Sprite::default(),
                LampAnimationState {
                    // Use a 1-second frame interval so our 2-second step fires it
                    timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                    frames,
                    current: 0,
                    hold_on_last: true,
                },
            ))
            .id()
    }

    #[test]
    fn lamp_animation_advances_frame() {
        let mut app = make_app_with_time(1.1); // advances past the 1 s frame timer
        app.add_systems(Update, tick_lamp_animation);

        let entity = spawn_lamp_visual(&mut app, vec![0, 1, 2, 3]);
        for _ in 0..5 {
            app.update();
        }
        let anim = app.world().get::<LampAnimationState>(entity).unwrap();
        assert_eq!(anim.current, 1, "frame should have advanced to index 1");
    }

    #[test]
    fn lamp_animation_hold_on_last_removes_component() {
        let mut app = make_app_with_time(1.1);
        app.add_systems(Update, tick_lamp_animation);

        let entity = spawn_lamp_visual(&mut app, vec![0, 1]);
        for _ in 0..15 {
            app.update();
        }

        assert!(
            app.world().get::<LampAnimationState>(entity).is_none(),
            "LampAnimationState should be removed on last frame with hold_on_last"
        );
    }

    // ── tick_block_sliding ────────────────────────────────────────────────────

    fn make_dummy_handle() -> SpriteSheetHandle {
        SpriteSheetHandle {
            image: Handle::default(),
            layout: Handle::default(),
        }
    }

    #[test]
    fn block_slide_moves_transform_over_time() {
        // 0.5 s step → timer fraction = 0.5 → block should be mid-slide
        let mut app = make_app_with_time(0.5);
        app.add_systems(Update, tick_block_sliding);

        let from = Vec2::new(0.0, 0.0);
        let to = Vec2::new(32.0, 0.0);

        let entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(from.x, from.y, 0.0),
                GlobalTransform::default(),
                BlockSliding {
                    from,
                    to,
                    timer: Timer::from_seconds(1.0, TimerMode::Once),
                },
                make_dummy_handle(),
            ))
            .id();

        for _ in 0..2 {
            app.update();
        }

        let x = app.world().get::<Transform>(entity).unwrap().translation.x;
        assert!(x > 0.0 && x < 32.0, "block should be mid-slide, got x={x}");
    }

    #[test]
    fn block_slide_snaps_to_target_and_removes_component() {
        // 1.1 s step → timer finishes in first update
        let mut app = make_app_with_time(1.1);
        app.add_systems(Update, tick_block_sliding);

        let from = Vec2::new(0.0, 0.0);
        let to = Vec2::new(32.0, 0.0);

        let entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(from.x, from.y, 0.0),
                GlobalTransform::default(),
                BlockSliding {
                    from,
                    to,
                    timer: Timer::from_seconds(1.0, TimerMode::Once),
                },
                make_dummy_handle(),
            ))
            .id();

        for _ in 0..15 {
            app.update();
        }

        let tf = app.world().get::<Transform>(entity).unwrap().translation;
        assert_eq!(tf.x, to.x, "block must snap exactly to target x");
        assert_eq!(tf.y, to.y, "block must snap exactly to target y");
        assert!(
            app.world().get::<BlockSliding>(entity).is_none(),
            "BlockSliding must be removed after slide completes"
        );
    }

    // ── InteractionState toggle ───────────────────────────────────────────────

    #[test]
    fn interaction_state_off_to_on() {
        let state = InteractionState::Off;
        let next = match state {
            InteractionState::Off => InteractionState::On,
            _ => InteractionState::Off,
        };
        assert_eq!(next, InteractionState::On);
    }

    #[test]
    fn interaction_state_on_to_off() {
        let state = InteractionState::On;
        let next = match state {
            InteractionState::Off => InteractionState::On,
            _ => InteractionState::Off,
        };
        assert_eq!(next, InteractionState::Off);
    }
}
