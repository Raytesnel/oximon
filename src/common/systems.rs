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

pub fn tick_domain_anim(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DomainExpansionAnim, &mut Sprite)>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_battle_state: ResMut<NextState<BattleState>>,
    battle_state: Res<State<BattleState>>,
    mut domain_asset: ResMut<DomainExpansionAsset>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    player_q: Query<&Transform, With<OverworldPlayer>>,
    time: Res<Time>,
) {
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
            Transform::from_xyz(pos.x, pos.y, 100.0),
            RigidBody::Static,
            Collider::polyline(points, None),
            CollisionLayers::new(GameLayer::Combat, [GameLayer::Combat]),
            DomainExpansionAnim {
                timer: Timer::from_seconds(1.0 / 12.0, TimerMode::Repeating),
                total_frames: 28,
                current_frame: 0,
                border_frame: 24,
                swap_frame: 19,
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
        }
        BattleState::Inactive => {}
    }

    if let Some(atlas) = &mut sprite.texture_atlas {
        atlas.index = anim.current_frame;
    }
}
