use super::components::*;
use super::events::*;
use crate::common::components::{Enemy, Player};
use bevy::ecs::error::info;
use bevy::prelude::*;

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

pub fn simple_attack_system(
    mut damage_writer: MessageWriter<DamageEvent>,
    query: Query<(Entity, &Transform, &Stats)>,
) {
    let entities: Vec<_> = query.iter().collect();

    for (attacker, transform, stats) in &entities {
        for (target, target_transform, _) in &entities {
            if attacker == target {
                continue;
            }

            let distance = transform.translation.distance(target_transform.translation);

            if distance < 50.0 {
                damage_writer.write(DamageEvent {
                    target: *target,
                    amount: stats.attack,
                });
            }
        }
    }
}

pub fn player_attack_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &Stats), With<Player>>,
    targets: Query<Entity, (With<Enemy>, With<Health>)>,
    mut writer: MessageWriter<DamageEvent>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }
    info!("Attack!");
    // for now: hit first enemy found (prototype)
    info!("player count: {}", query.iter().count());
    if let Some(target) = targets.iter().next() {
        info!("attacking {:?}", target);
        for (_player, stats) in query.iter() {
            writer.write(DamageEvent {
                target,
                amount: stats.attack,
            });
            info!("message send to attack :{:?}", target);
        }
    } else {
        info!("no target found, ");
    }
}
