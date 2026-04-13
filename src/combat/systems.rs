use super::components::*;
use super::events::*;
use crate::common::components::{Enemy, Player};
use bevy::ecs::error::info;
use bevy::prelude::*;
use crate::movement::components::Facing;
const ATTACK_RANGE: f32 = 100.0; // ~5 blocks

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
pub fn player_attack_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<(&Transform, &Facing, &Stats), With<Player>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    mut writer: MessageWriter<DamageEvent>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    for (player_transform, facing, stats) in &player_query {
        let origin = player_transform.translation.truncate();
        let forward = facing.0.normalize();
        for (enemy, enemy_transform) in &enemies {
            let to_enemy = enemy_transform.translation.truncate() - origin;
            let distance = to_enemy.length();
            if distance > ATTACK_RANGE {
                info!("attack missed, {:?}, but character is {:?}",ATTACK_RANGE,to_enemy);
                continue;
            }

            let dir = to_enemy.normalize_or_zero();

            // ✅ must be in front (cone check)
            let alignment = dir.dot(forward);

            if alignment > 0.6 {
                writer.write(DamageEvent {
                    target: enemy,
                    amount: stats.attack,
                });
            }
        }
    }
}