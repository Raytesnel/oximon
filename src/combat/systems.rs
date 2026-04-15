use super::components::*;
use super::events::*;
use crate::common::components::{Enemy, ModifierTrigger, Player, StatModifier, StatType, Stats};
use crate::movement::components::{Dash, Facing};
use bevy::prelude::*;
const ATTACK_RANGE: f32 = 100.0; // ~5 blocks

pub fn attack_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<Entity, With<Player>>,
    mut writer: MessageWriter<AttackEvent>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    for entity in &player_query {
        writer.write(AttackEvent { entity });
    }
}

pub fn quick_attack_input_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &AttackStats), With<Player>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyQ) {
        return;
    }

    for (entity, stats) in &query {
        commands.spawn((
            Attack {
                damage: stats.attack,
                range: 30.0,
                lifetime: Timer::from_seconds(2.0, TimerMode::Once),
                hit_timer: Timer::from_seconds(2.1, TimerMode::Repeating),
                follow_entity: Some(entity),
                active: false,
                applied_start_modifiers: false,
                stat_modifiers: vec![
                    StatModifier {
                        stat_type: StatType::Speed,
                        flat: 0.0,
                        multiplier: 2.0,
                        timer: Some(Timer::from_seconds(2.1, TimerMode::Once)),
                        trigger: ModifierTrigger::Cast,
                    },
                    StatModifier {
                        stat_type: StatType::Acceleration,
                        flat: 0.0,
                        multiplier: 10.0,
                        timer: Some(Timer::from_seconds(2.1, TimerMode::Once)),
                        trigger: ModifierTrigger::Cast,
                    },
                ],
            },
            Transform::default(),
            Sprite {
                color: Color::srgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(10.0, 10.0)),
                ..default()
            },
        ));
    }
}

pub fn apply_damage_system(
    mut events: MessageReader<DamageEvent>,
    mut query: Query<(&mut Health, &mut CombatState)>,
) {
    for event in events.read() {
        info!("event received!");
        match query.get_mut(event.target) {
            Ok((mut health, mut state)) => {
                health.current -= event.amount;
                info!(
                    "{:?} is hit, new health: {:?}",
                    event.target, health.current
                );

                if health.current <= 0.0 {
                    health.current = 0.0;
                    *state = CombatState::Dead;
                    info!("{:?} is Dead", event.target);
                }
            }
            Err(e) => {
                info!("FAILED to get entity {:?}: {:?}", event.target, e);
            }
        }
    }
}

pub fn attack_start_system(mut attacks: Query<&mut Attack>, mut stats_query: Query<&mut Stats>) {
    for mut attack in &mut attacks {
        if attack.applied_start_modifiers {
            continue;
        }

        if let Some(source) = attack.follow_entity {
            if let Ok(mut stats) = stats_query.get_mut(source) {
                for modifier in &attack.stat_modifiers {
                    if matches!(modifier.trigger, ModifierTrigger::Cast) {
                        stats.add_modifier(modifier.clone());
                    }
                }
            }
        }

        attack.applied_start_modifiers = true;
    }
}

pub fn attack_hit_system(
    mut attacks: Query<(&Transform, &mut Attack)>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    mut writer: MessageWriter<DamageEvent>,
    time: Res<Time>,
) {
    for (attack_transform, mut attack) in &mut attacks {
        let tick_ready = attack.hit_timer.tick(time.delta()).just_finished();
        if !attack.active {
            for (enemy, enemy_transform) in &enemies {
                let distance = attack_transform
                    .translation
                    .distance(enemy_transform.translation);

                if distance < attack.range {
                    attack.active = true;
                    writer.write(DamageEvent {
                        target: enemy,
                        amount: attack.damage,
                    });

                    break;
                }
            }

            continue;
        }

        if tick_ready && !attack.lifetime.is_finished() {
            for (enemy, enemy_transform) in &enemies {
                let distance = attack_transform
                    .translation
                    .distance(enemy_transform.translation);

                if distance < attack.range {
                    writer.write(DamageEvent {
                        target: enemy,
                        amount: attack.damage,
                    });

                    break;
                }
            }
        }
    }
}

pub fn attack_lifetime_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Attack)>,
) {
    for (entity, mut attack) in &mut query {
        attack.lifetime.tick(time.delta());
        if attack.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn spawn_attack_system(
    mut commands: Commands,
    mut events: MessageReader<AttackEvent>,
    query: Query<(&Transform, &Facing, &AttackStats)>,
) {
    for event in events.read() {
        if let Ok((transform, facing, stats)) = query.get(event.entity) {
            let origin = transform.translation;
            let direction = facing.0.normalize();

            let offset = (direction * (ATTACK_RANGE * 0.5)).extend(0.0);

            commands.spawn((
                Attack {
                    damage: stats.attack,
                    range: ATTACK_RANGE,
                    lifetime: Timer::from_seconds(0.1, TimerMode::Once),
                    hit_timer: Timer::from_seconds(0.05, TimerMode::Repeating),
                    follow_entity: None,
                    active: false,
                    stat_modifiers: vec![],
                    applied_start_modifiers: true,
                },
                Transform::from_translation(origin + offset),
                Sprite {
                    color: Color::srgb(1.0, 0.0, 0.0),
                    custom_size: Some(Vec2::new(ATTACK_RANGE, 20.0)),
                    ..default()
                },
            ));
        }
    }
}

pub fn attack_follow_system(
    mut attacks: Query<(&mut Transform, &Attack)>,
    targets: Query<&Transform, Without<Attack>>,
) {
    for (mut transform, attack) in &mut attacks {
        if let Some(entity) = attack.follow_entity {
            if let Ok(target_transform) = targets.get(entity) {
                transform.translation = target_transform.translation;
            }
        }
    }
}

pub fn despawn_dead_system(mut commands: Commands, query: Query<(Entity, &CombatState)>) {
    for (entity, state) in &query {
        if *state == CombatState::Dead {
            commands.entity(entity).despawn();
        }
    }
}
