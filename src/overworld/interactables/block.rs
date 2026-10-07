use crate::overworld::components::*;
use avian2d::prelude::*;
use bevy::asset::AssetServer;
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

#[derive(SystemParam)]
pub struct BlockInteractionContext<'w, 's> {
    pub block_q: Query<'w, 's, BlockQueryItem>,
    pub player_q: Query<'w, 's, PlayerQueryItem, With<OverworldPlayer>>,
    pub obstacle_q: Query<'w, 's, ObstacleQueryItem, Or<ObstacleFilter>>,
    pub sliding_q: Query<'w, 's, &'static BlockSliding>,
}

// Snap a world position to the nearest grid cell (32x32)
fn to_grid(pos: Vec2, grid_size: f32) -> IVec2 {
    IVec2::new(
        (pos.x / grid_size).round() as i32,
        (pos.y / grid_size).round() as i32,
    )
}

fn facing_to_grid_dir(facing: &Facing) -> IVec2 {
    match facing {
        Facing::Up => IVec2::Y,
        Facing::Down => -IVec2::Y,
        Facing::Left => -IVec2::X,
        Facing::Right => IVec2::X,
    }
}

/// Returns the push direction (grid unit vector, away from the player) if the
/// player is close enough AND facing the block. `block_origin` is the entity
/// origin (bottom-left corner for Tiled objects), `player_pos` is the player's
/// centre.
pub(crate) fn push_direction(
    block_origin: Vec2,
    player_pos: Vec2,
    facing: &Facing,
    grid_size: f32,
) -> Option<IVec2> {
    // Entity-origin is the bottom-left corner; compare the block's centre.
    let block_center = block_origin + Vec2::splat(grid_size * 0.5);
    let diff = block_center - player_pos;

    // Too far away, or standing exactly on the block centre (no direction)
    if diff.length() > grid_size * 1.5 || diff.length_squared() < f32::EPSILON {
        return None;
    }

    // Dominant axis decides the direction (always away from the player)
    let delta = if diff.x.abs() > diff.y.abs() {
        IVec2::new(diff.x.signum() as i32, 0)
    } else {
        IVec2::new(0, diff.y.signum() as i32)
    };

    // Player must be facing the block
    (facing_to_grid_dir(facing) == delta).then_some(delta)
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
    let Ok((facing, player_tf)) = block_ctx.player_q.single() else {
        return;
    };

    let grid_size = block.grid_size;
    let current_pos = block_tf.translation.truncate();
    let block_grid = to_grid(current_pos, grid_size);

    let Some(delta) = push_direction(
        current_pos,
        player_tf.translation.truncate(),
        facing,
        grid_size,
    ) else {
        return;
    };

    // Duwrichting is altijd weg van de speler
    let push_dir = delta.as_vec2();
    let target_pos = current_pos + push_dir * grid_size;
    let target_grid = block_grid + delta;

    let is_blocked = block_ctx.obstacle_q.iter().any(|obs_tf| {
        let obs_grid = to_grid(obs_tf.translation.truncate(), grid_size);
        obs_grid != block_grid && obs_grid == target_grid
    });
    if is_blocked {
        return;
    }

    // Spritesheet lazily opbouwen bij eerste push
    if maybe_handle.is_none() {
        if let Some(props) = maybe_props {
            let layout = layouts.add(TextureAtlasLayout::from_grid(
                UVec2::new(props.width, props.height),
                props.columns,
                props.rows,
                None,
                None,
            ));
            commands.entity(entity).insert(SpriteSheetHandle {
                image: asset_server.load(props.path.clone()),
                layout,
            });
        }
    }

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

    use crate::overworld::components::{BlockSliding, InteractionState, SpriteSheetHandle};
    use crate::overworld::interactables::*;

    fn make_app_with_time(step_seconds: f32) -> App {
        let mut app = App::new();
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(step_seconds),
        ));
        app.add_plugins(MinimalPlugins);
        app
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

// ── push_direction ────────────────────────────────────────────────────────

use crate::overworld::components::Facing;

const GRID: f32 = 32.0;
// Block origin = bottom-left corner, so its centre is at (80, 80).
const BLOCK_ORIGIN: Vec2 = Vec2::new(64.0, 64.0);
const BLOCK_CENTER: Vec2 = Vec2::new(80.0, 80.0);

fn all_facings() -> [Facing; 4] {
    [Facing::Up, Facing::Down, Facing::Left, Facing::Right]
}

/// Player centre one cell away from the block centre, plus the facing that
/// looks at the block and the direction the block must move.
fn sides() -> [(Vec2, Facing, IVec2); 4] {
    [
        (
            BLOCK_CENTER + Vec2::new(-GRID, 0.0),
            Facing::Right,
            IVec2::X,
        ), // player left  -> push right
        (BLOCK_CENTER + Vec2::new(GRID, 0.0), Facing::Left, -IVec2::X), // player right -> push left
        (BLOCK_CENTER + Vec2::new(0.0, -GRID), Facing::Up, IVec2::Y),   // player below -> push up
        (BLOCK_CENTER + Vec2::new(0.0, GRID), Facing::Down, -IVec2::Y), // player above -> push down
    ]
}

/// Regression test for the offset bug: the block origin is its bottom-left
/// corner, so comparing origin-vs-player made "push right" resolve to
/// "down". Standing left of the block and facing it must push right.
#[test]
fn push_right_works_when_player_is_left_of_block() {
    let player = Vec2::new(48.0, 80.0); // centre-to-centre diff = (32, 0)
    assert_eq!(
        push_direction(BLOCK_ORIGIN, player, &Facing::Right, GRID),
        Some(IVec2::X)
    );
}

#[test]
fn push_left_works_when_player_is_right_of_block() {
    let player = Vec2::new(112.0, 80.0);
    assert_eq!(
        push_direction(BLOCK_ORIGIN, player, &Facing::Left, GRID),
        Some(-IVec2::X)
    );
}

/// The "stand in the bottom corner" workaround must no longer be needed:
/// pushing right works at any height along the block's left side.
#[test]
fn push_right_works_at_any_height_along_the_side() {
    for dy in [-12.0, -6.0, 0.0, 6.0, 12.0] {
        let player = Vec2::new(48.0, 80.0 + dy);
        assert_eq!(
            push_direction(BLOCK_ORIGIN, player, &Facing::Right, GRID),
            Some(IVec2::X),
            "dy = {dy}"
        );
    }
}

#[test]
fn push_direction_is_always_away_from_player_on_all_sides() {
    for (player, facing, expected) in sides() {
        assert_eq!(
            push_direction(BLOCK_ORIGIN, player, &facing, GRID),
            Some(expected),
            "player at {player:?}"
        );
    }
}

/// The core "only one direction" guarantee: for a given player position,
/// at most one facing may push, and it must be the one away from the player.
#[test]
fn only_one_facing_can_push_from_each_side() {
    for (player, _, expected) in sides() {
        let results: Vec<IVec2> = all_facings()
            .iter()
            .filter_map(|f| push_direction(BLOCK_ORIGIN, player, f, GRID))
            .collect();
        assert_eq!(
            results,
            vec![expected],
            "exactly one facing should push from {player:?}"
        );
    }
}

/// Quickly turning while pressing interact must not push the block sideways.
#[test]
fn turning_away_from_block_does_not_push() {
    let player = Vec2::new(48.0, 80.0); // left of block
    for facing in [Facing::Up, Facing::Down, Facing::Left] {
        assert_eq!(push_direction(BLOCK_ORIGIN, player, &facing, GRID), None);
    }
}

#[test]
fn too_far_away_does_not_push() {
    let player = Vec2::new(-100.0, 80.0);
    assert_eq!(
        push_direction(BLOCK_ORIGIN, player, &Facing::Right, GRID),
        None
    );
}

#[test]
fn player_exactly_on_block_centre_does_not_push() {
    for facing in all_facings() {
        assert_eq!(
            push_direction(BLOCK_ORIGIN, BLOCK_CENTER, &facing, GRID),
            None
        );
    }
}

/// With several blocks around the player, only the one in the facing
/// direction is selected.
#[test]
fn only_the_faced_block_is_selected_among_neighbours() {
    let player = Vec2::new(80.0, 80.0);
    // Block origins (bottom-left) one cell left, right, above, below the player.
    let blocks = [
        Vec2::new(32.0, 64.0), // left
        Vec2::new(96.0, 64.0), // right
        Vec2::new(64.0, 96.0), // above
        Vec2::new(64.0, 32.0), // below
    ];

    let selected: Vec<usize> = blocks
        .iter()
        .enumerate()
        .filter(|(_, origin)| push_direction(**origin, player, &Facing::Right, GRID).is_some())
        .map(|(i, _)| i)
        .collect();

    assert_eq!(
        selected,
        vec![1],
        "only the right-hand block should be pushable"
    );
}
