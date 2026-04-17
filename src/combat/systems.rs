use super::components::*;
use super::events::*;
use crate::combat::attacks::{quick_attack, simple_beam};
use crate::common::components::{Enemy, ModifierTrigger, Player, RuntimeModifier, Stats};
use crate::movement::components::Facing;
use bevy::prelude::*;

pub const JUMP_BUTTON: KeyCode = KeyCode::Space;
pub const QUICK_ATTACK: KeyCode = KeyCode::KeyQ;

pub fn attack_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut Cooldowns), With<Player>>,
    mut writer: MessageWriter<AttackEvent>,
) {
    if !keyboard.just_pressed(JUMP_BUTTON) {
        return;
    }

    for (entity, mut cooldowns) in &mut query {
        let def = simple_beam();

        // 👇 check cooldown
        if let Some(timer) = cooldowns.timers.get(&def.name) {
            if !timer.is_finished() {
                continue; // still on cooldown
            }
        }

        // 👇 start cooldown
        cooldowns.timers.insert(
            def.name.clone(),
            Timer::from_seconds(def.cooldown, TimerMode::Once),
        );
        writer.write(AttackEvent { entity });
    }
}

pub fn quick_attack_input_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut Cooldowns), With<Player>>,
) {
    if !keyboard.just_pressed(QUICK_ATTACK) {
        return;
    }
    println!("q pressed");
    for (entity, mut cooldowns) in &mut query {
        let def = quick_attack();
        // 👇 check cooldown
        if let Some(timer) = cooldowns.timers.get(&def.name) {
            if !timer.is_finished() {
                continue; // still on cooldown
            }
        }

        // 👇 start cooldown
        cooldowns.timers.insert(
            def.name.clone(),
            Timer::from_seconds(def.cooldown, TimerMode::Once),
        );
        let sprite = def.spawn.build_sprite();
        commands.spawn((
            Attack::from_definition(def, entity),
            Transform::default(),
            sprite,
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
                for modifier in &attack.definition.stat_modifiers {
                    if matches!(modifier.trigger, ModifierTrigger::Cast) {
                        stats.add_modifier(RuntimeModifier {
                            stat_type: modifier.stat_type,
                            flat: modifier.flat,
                            multiplier: modifier.multiplier,
                            timer: modifier
                                .duration
                                .map(|d| Timer::from_seconds(d, TimerMode::Once)),
                        });
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

                if distance < attack.definition.range {
                    attack.active = true;
                    writer.write(DamageEvent {
                        target: enemy,
                        amount: attack.definition.damage,
                    });

                    break;
                }
            }

            continue;
        }

        if tick_ready && !attack.lifetime_timer.is_finished() {
            for (enemy, enemy_transform) in &enemies {
                let distance = attack_transform
                    .translation
                    .distance(enemy_transform.translation);

                if distance < attack.definition.range {
                    writer.write(DamageEvent {
                        target: enemy,
                        amount: attack.definition.damage,
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
        attack.lifetime_timer.tick(time.delta());
        if attack.lifetime_timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn spawn_attack_system(
    mut commands: Commands,
    mut events: MessageReader<AttackEvent>,
    query: Query<(&Transform, &Facing)>,
    query_player: Query<Entity, With<Player>>,
) {
    for event in events.read() {
        if let Ok((transform, facing)) = query.get(event.entity) {
            let origin = transform.translation;
            let direction = facing.0.normalize();

            let offset = (direction * (100.0 * 0.5)).extend(0.0);
            let def = simple_beam();
            let sprite = def.spawn.build_sprite();
            let mut attack_command = Attack::from_definition(def, event.entity);
            attack_command.definition.offset = offset;
            commands.spawn((
                attack_command,
                Transform::from_translation(origin + offset),
                sprite,
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
                transform.translation = target_transform.translation + attack.definition.offset;
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

pub fn cooldown_tick_system(time: Res<Time>, mut query: Query<&mut Cooldowns>) {
    for mut cooldowns in &mut query {
        cooldowns.timers.retain(|_, timer| {
            timer.tick(time.delta());
            !timer.just_finished()
        });
    }
}
