use crate::combat::components::Hitstop;
use crate::common::components::*;
use crate::overworld::components::{DomainExpansionAsset, OverworldPlayer, SpriteSheetHandle};
use crate::{GameState, overworld::components::DomainExpansionAnim};
use avian2d::prelude::*;
use bevy::prelude::*;

pub fn compute_stats_system(mut query: Query<(&Stats, &mut ComputedStats)>) {
    for (stats, mut computed) in &mut query {
        computed.speed = stats.speed();
        computed.acceleration = stats.acceleration();
        computed.friction = stats.friction();
        computed.dash_speed = stats.dash_speed();
        computed.dash_time = stats.dash_time();
        computed.dash_friction = stats.dash_friction();
        computed.dash_stop_time = stats.dash_stop_time();
    }
}

pub fn update_stat_timers(time: Res<Time>, mut query: Query<&mut Stats>) {
    for mut stats in &mut query {
        update_list(&mut stats.speed, &time);
        update_list(&mut stats.acceleration, &time);
        update_list(&mut stats.friction, &time);
        update_list(&mut stats.dash_speed, &time);
        update_list(&mut stats.dash_time, &time);
        update_list(&mut stats.dash_friction, &time);
        update_list(&mut stats.dash_stop_time, &time);
    }
}

pub fn update_list(list: &mut Vec<RuntimeModifier>, time: &Time) {
    list.retain_mut(|modifier| {
        if let Some(timer) = &mut modifier.timer {
            timer.tick(time.delta());
            !timer.is_finished()
        } else {
            true
        }
    });
}

pub fn tick_hitstop(mut hitstop: ResMut<Hitstop>, time: Res<Time>) {
    if hitstop.remaining > 0.0 {
        hitstop.remaining -= time.delta_secs();
    }
}
#[allow(clippy::too_many_arguments)]
pub fn tick_domain_anim(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DomainExpansionAnim, &mut Sprite)>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_battle_state: ResMut<NextState<BattleState>>,
    battle_state: Res<State<BattleState>>,
    mut domain_asset: ResMut<DomainExpansionAsset>,
    asset_server: Option<Res<AssetServer>>,
    mut layouts: Option<ResMut<Assets<TextureAtlasLayout>>>,
    player_q: Query<&Transform, With<OverworldPlayer>>,
    time: Res<Time>,
) {
    let (Some(asset_server), Some(layouts)) = (asset_server, layouts.as_mut()) else {
        return;
    };
    // Spawn the entity when entering
    if *battle_state.get() == BattleState::Entering && query.is_empty() {
        let handle = domain_asset.handle.get_or_insert_with(|| {
            let layout = layouts.add(TextureAtlasLayout::from_grid(
                UVec2::new(960, 960),
                28,
                1,
                None,
                None,
            ));
            SpriteSheetHandle {
                image: asset_server.load("tiles/dungeon/transition_bol_prototype.png"),
                layout,
            }
        });

        let pos = player_q
            .single()
            .map(|tf| tf.translation)
            .unwrap_or(Vec3::ZERO);

        let points: Vec<Vec2> = (0..64)
            .map(|i| {
                let angle = (i as f32 / 64.0) * std::f32::consts::TAU;
                Vec2::new(angle.cos() * 475.0, angle.sin() * 475.0)
            })
            .collect();

        commands.spawn((
            Sprite {
                image: handle.image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: handle.layout.clone(),
                    index: 0,
                }),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 10.0),
            RigidBody::Static,
            Collider::polyline(points, None),
            CollisionLayers::new(GameLayer::Combat, [GameLayer::Combat]),
            DomainExpansionAnim {
                timer: Timer::from_seconds(1.0 / 12.0, TimerMode::Repeating),
                total_frames: 28,
                current_frame: 0,
                border_frame: 24,
                swap_frame: 19,
                center: Vec2::new(pos.x, pos.y),
                current_radius: 0.0,
                max_radius: 960.0,
            },
        ));
        return;
    }

    // Tick the animation
    let Ok((entity, mut anim, mut sprite)) = query.single_mut() else {
        return;
    };

    anim.timer.tick(time.delta());
    if !anim.timer.just_finished() {
        return;
    }

    match battle_state.get() {
        BattleState::Entering => {
            anim.current_frame += 1;
            anim.current_radius =
                (anim.current_frame as f32 / anim.swap_frame as f32) * anim.max_radius;
            if anim.current_frame == anim.swap_frame {
                next_game_state.set(GameState::Combat);
            }
            if anim.current_frame >= anim.total_frames - 1 {
                anim.current_frame = anim.border_frame;
                next_battle_state.set(BattleState::Active);
            }
        }
        BattleState::Active => {
            anim.current_frame += 1;
            anim.current_radius = anim.max_radius;
            if anim.current_frame >= anim.total_frames {
                anim.current_frame = anim.border_frame;
            }
        }
        BattleState::Ending => {
            if anim.current_frame == 0 {
                commands.entity(entity).despawn();
                next_battle_state.set(BattleState::Inactive);
                return;
            }
            if anim.current_frame == anim.swap_frame {
                next_game_state.set(GameState::Overworld);
            }
            anim.current_frame -= 1;
            anim.current_radius =
                (anim.current_frame as f32 / anim.swap_frame as f32) * anim.max_radius;
        }
        BattleState::Inactive => {}
    }

    if let Some(atlas) = &mut sprite.texture_atlas {
        atlas.index = anim.current_frame;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;
    use bevy::state::app::StatesPlugin;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    fn anim_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.add_plugins(AssetPlugin::default());
        app.init_asset::<TextureAtlasLayout>();
        app.init_state::<GameState>();
        app.init_state::<BattleState>();
        app.init_resource::<DomainExpansionAsset>();
        app.add_systems(Update, tick_domain_anim);
        app
    }

    /// Tick with a given dt, driving the anim system once.
    fn tick(app: &mut App, dt: f32) {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            dt,
        )));
        app.update();
    }

    /// Flush any queued NextState without advancing the animation timer.
    fn flush_state(app: &mut App) {
        tick(app, 0.0);
    }

    fn set_battle_state(app: &mut App, state: BattleState) {
        app.world_mut()
            .resource_mut::<NextState<BattleState>>()
            .set(state);
        flush_state(app);
    }

    fn spawn_anim(app: &mut App, current_frame: usize, current_radius: f32) -> Entity {
        app.world_mut()
            .spawn((
                DomainExpansionAnim {
                    timer: Timer::from_seconds(1.0 / 12.0, TimerMode::Repeating),
                    total_frames: 28,
                    current_frame,
                    border_frame: 24,
                    swap_frame: 19,
                    center: Vec2::ZERO,
                    current_radius,
                    max_radius: 960.0,
                },
                Sprite {
                    texture_atlas: Some(TextureAtlas {
                        layout: Handle::default(),
                        index: current_frame,
                    }),
                    ..default()
                },
            ))
            .id()
    }

    // one full frame period plus a hair, so the repeating timer fires exactly once
    const ONE_FRAME: f32 = 1.0 / 12.0 + 0.001;

    fn anim_of<'a>(app: &'a mut App) -> DomainExpansionAnim {
        app.world_mut()
            .query::<&DomainExpansionAnim>()
            .single(app.world())
            .expect("anim entity should exist")
            .clone() // requires DomainExpansionAnim: Clone; swap for field reads if not
    }

    // ---------- basic frame ticking ----------

    #[test]
    fn frame_advances_after_timer_finishes() {
        let mut app = anim_test_app();
        set_battle_state(&mut app, BattleState::Active);
        spawn_anim(&mut app, 24, 960.0);

        tick(&mut app, ONE_FRAME);

        assert_eq!(anim_of(&mut app).current_frame, 25);
    }

    #[test]
    fn frame_does_not_advance_before_timer_finishes() {
        let mut app = anim_test_app();
        set_battle_state(&mut app, BattleState::Active);
        spawn_anim(&mut app, 24, 960.0);

        tick(&mut app, 0.01); // well under 1/12s

        assert_eq!(
            anim_of(&mut app).current_frame,
            24,
            "frame shouldn't advance before the timer fires"
        );
    }

    // ---------- Entering phase ----------

    #[test]
    fn entering_reaching_swap_frame_triggers_combat_state() {
        let mut app = anim_test_app();
        spawn_anim(&mut app, 18, 0.0);
        set_battle_state(&mut app, BattleState::Entering);

        tick(&mut app, ONE_FRAME);
        flush_state(&mut app);

        assert_eq!(
            *app.world().resource::<State<GameState>>().get(),
            GameState::Combat
        );
    }

    #[test]
    fn entering_reaching_last_frame_resets_and_transitions_to_active() {
        let mut app = anim_test_app();
        spawn_anim(&mut app, 26, 0.0);
        set_battle_state(&mut app, BattleState::Entering);

        tick(&mut app, ONE_FRAME);
        flush_state(&mut app);

        let anim = anim_of(&mut app);
        assert_eq!(anim.current_frame, 24, "should reset to border_frame");
        assert_eq!(
            *app.world().resource::<State<BattleState>>().get(),
            BattleState::Active
        );
    }

    #[test]
    fn entering_radius_scales_with_frame() {
        let mut app = anim_test_app();
        spawn_anim(&mut app, 8, 0.0);
        set_battle_state(&mut app, BattleState::Entering);

        tick(&mut app, ONE_FRAME);

        let anim = anim_of(&mut app);
        let expected = (9.0 / anim.swap_frame as f32) * anim.max_radius;
        assert!((anim.current_radius - expected).abs() < 0.01);
    }

    // ---------- Active phase ----------

    #[test]
    fn active_radius_stays_at_max() {
        let mut app = anim_test_app();
        set_battle_state(&mut app, BattleState::Active);
        spawn_anim(&mut app, 24, 500.0); // radius deliberately wrong beforehand

        tick(&mut app, ONE_FRAME);

        assert_eq!(anim_of(&mut app).current_radius, 960.0);
    }

    #[test]
    fn active_frame_wraps_at_total_frames() {
        let mut app = anim_test_app();
        set_battle_state(&mut app, BattleState::Active);
        spawn_anim(&mut app, 27, 960.0); // one tick from total_frames (28)

        tick(&mut app, ONE_FRAME);

        let anim = anim_of(&mut app);
        assert_eq!(anim.current_frame, 24, "should wrap back to border_frame");
        assert_eq!(
            *app.world().resource::<State<BattleState>>().get(),
            BattleState::Active
        );
    }

    // ---------- Ending phase ----------

    #[test]
    fn ending_reaching_swap_frame_triggers_overworld_state() {
        let mut app = anim_test_app();
        set_battle_state(&mut app, BattleState::Ending);
        spawn_anim(&mut app, 20, 0.0); // one tick from swap_frame (19), counting down

        tick(&mut app, ONE_FRAME);
        flush_state(&mut app);

        assert_eq!(
            *app.world().resource::<State<GameState>>().get(),
            GameState::Overworld
        );
    }

    #[test]
    fn ending_at_zero_despawns_and_sets_inactive() {
        let mut app = anim_test_app();
        set_battle_state(&mut app, BattleState::Ending);
        let entity = spawn_anim(&mut app, 0, 0.0);

        tick(&mut app, ONE_FRAME);
        flush_state(&mut app);

        assert!(
            app.world().get_entity(entity).is_err(),
            "anim entity should despawn"
        );
        assert_eq!(
            *app.world().resource::<State<BattleState>>().get(),
            BattleState::Inactive
        );
    }

    #[test]
    fn ending_radius_scales_down_with_frame() {
        let mut app = anim_test_app();
        set_battle_state(&mut app, BattleState::Ending);
        spawn_anim(&mut app, 10, 0.0);

        tick(&mut app, ONE_FRAME); // frame becomes 9

        let anim = anim_of(&mut app);
        let expected = (9.0 / anim.swap_frame as f32) * anim.max_radius;
        assert!((anim.current_radius - expected).abs() < 0.01);
    }

    // ---------- Inactive phase ----------

    #[test]
    fn inactive_state_is_a_no_op() {
        let mut app = anim_test_app();
        set_battle_state(&mut app, BattleState::Inactive);
        spawn_anim(&mut app, 10, 300.0);

        tick(&mut app, ONE_FRAME);

        let anim = anim_of(&mut app);
        assert_eq!(anim.current_frame, 10, "Inactive should not tick the frame");
        assert_eq!(
            anim.current_radius, 300.0,
            "Inactive should not touch radius"
        );
    }

    // ---------- sprite index sync ----------

    #[test]
    fn sprite_atlas_index_follows_current_frame() {
        let mut app = anim_test_app();
        set_battle_state(&mut app, BattleState::Active);
        spawn_anim(&mut app, 24, 960.0);

        tick(&mut app, ONE_FRAME);

        let mut query = app.world_mut().query::<(&DomainExpansionAnim, &Sprite)>();
        let (anim, sprite) = query.single(app.world()).unwrap();
        assert_eq!(
            sprite.texture_atlas.as_ref().unwrap().index,
            anim.current_frame
        );
    }

    // ---------- regression: missing assets shouldn't panic ----------

    #[test]
    fn system_no_ops_without_asset_server() {
        // Deliberately skip AssetPlugin/init_asset — mirrors the combat test harness.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.init_state::<GameState>();
        app.init_state::<BattleState>();
        app.init_resource::<DomainExpansionAsset>();
        app.add_systems(Update, tick_domain_anim);

        // should not panic
        app.update();
    }
}
