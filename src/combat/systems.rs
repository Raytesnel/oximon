use super::components::*;
use super::events::*;
use crate::common::components::{Enemy, Player};
use bevy::ecs::error::info;
use bevy::prelude::*;
use crate::movement::components::Facing;
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
pub fn attack_hit_system(
    attacks: Query<(&Transform, &Attack)>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    mut writer: MessageWriter<DamageEvent>,
) {
    for (attack_transform, attack) in &attacks {
        for (enemy, enemy_transform) in &enemies {
            let distance = attack_transform
                .translation
                .distance(enemy_transform.translation);

            if distance < attack.range {
                writer.write(DamageEvent {
                    target: enemy,
                    amount: attack.damage,
                });
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
        println!(
            "attack lifetime: remaining {:?}",
            attack.lifetime.remaining()
        );
        if attack.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn spawn_attack_system(
    mut commands: Commands,
    mut events: MessageReader<AttackEvent>,
    query: Query<(&Transform, &Facing, &Stats)>,
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