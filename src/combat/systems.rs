use super::components::*;
use super::events::*;
use crate::combat::attacks::{
    AttackDefinition, HitBehavior, KnockbackDefinition, KnockbackDirection, KnockbackMode,
    quick_attack, simple_beam,
};
use crate::common::components::{
    Enemy, ModifierLifetime, ModifierTrigger, Player, RuntimeModifier, Stats,
};
use crate::movement::components::{Facing, Movable, Velocity};
use bevy::ecs::error::info;
use bevy::prelude::*;

pub const JUMP_BUTTON: KeyCode = KeyCode::Space;
pub const QUICK_ATTACK: KeyCode = KeyCode::KeyQ;

pub fn attack_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut Cooldowns, &mut CombatState), (With<Movable>, Without<Hitstun>)>,
    mut writer: MessageWriter<AttackEvent>,
) {
    if !keyboard.just_pressed(JUMP_BUTTON) {
        return;
    }

    for (entity, mut cooldowns, mut combat_state) in &mut query {
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
        *combat_state = CombatState::Attacking;
    }
}

pub fn quick_attack_input_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut id_counter: ResMut<AttackIdCounter>,
    mut query: Query<(Entity, &mut Cooldowns, &mut CombatState), (With<Movable>, Without<Hitstun>)>,
) {
    if !keyboard.just_pressed(QUICK_ATTACK) {
        return;
    }
    for (entity, mut cooldowns, mut combat_state) in &mut query {
        let def = quick_attack();
        // 👇 check cooldown
        if let Some(timer) = cooldowns.timers.get(&def.name) {
            if !timer.is_finished() {
                continue; // still on cooldown
            }
        }

        cooldowns.timers.insert(
            def.name.clone(),
            Timer::from_seconds(def.cooldown, TimerMode::Once),
        );
        let sprite = def.spawn.build_sprite();
        let id = AttackId(id_counter.next);
        id_counter.next += 1;
        commands.spawn((
            Attack::from_definition(def, entity, id),
            Transform::default(),
            sprite,
        ));
        *combat_state = CombatState::Attacking;
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
                            source: attack.id,
                            stat_type: modifier.stat_type,
                            flat: modifier.flat,
                            multiplier: modifier.multiplier,
                            lifetime: modifier.lifetime.clone(),
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
    mut commands: Commands,
    mut attacks: Query<(&Transform, &mut Attack)>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    player: Query<&Transform, With<Player>>,
    mut hitstop: ResMut<Hitstop>,
    mut writer: MessageWriter<DamageEvent>,
    time: Res<Time>,
) {
    for (attack_transform, mut attack) in &mut attacks {
        let attack_pos = attack_transform.translation;

        let tick_ready = attack.hit_timer.tick(time.delta()).just_finished();

        for (enemy, enemy_transform) in &enemies {
            let enemy_pos = enemy_transform.translation;
            let Ok(player_transform) = player.single() else {
                continue;
            };
            let player_pos = player_transform.translation;
            if attack_pos.distance(enemy_pos) >= attack.definition.range {
                continue;
            }

            match attack.definition.hit_behavior {
                HitBehavior::Single => {
                    if attack.has_hit {
                        continue;
                    }
                    apply_hit_effects(
                        &mut commands,
                        &attack,
                        enemy,
                        enemy_pos,
                        attack_pos,
                        player_pos,
                        &mut hitstop,
                        &mut writer,
                    );

                    attack.has_hit = true;
                    attack.hit_timer.finish();
                    attack.lifetime_timer.finish();
                }

                HitBehavior::MultiHit => {
                    if tick_ready {
                        apply_hit_effects(
                            &mut commands,
                            &attack,
                            enemy,
                            enemy_pos,
                            attack_pos,
                            player_pos,
                            &mut hitstop,
                            &mut writer,
                        );
                    }
                }

                HitBehavior::Limited(max_hits) => {
                    if attack.hits_done >= max_hits {
                        continue;
                    }

                    if tick_ready {
                        apply_hit_effects(
                            &mut commands,
                            &attack,
                            enemy,
                            enemy_pos,
                            attack_pos,
                            player_pos,
                            &mut hitstop,
                            &mut writer,
                        );

                        attack.hits_done += 1;

                        if attack.hits_done >= max_hits {
                            attack.hit_timer.finish();
                            attack.lifetime_timer.finish();
                        }
                    }
                }
            }
        }
    }
}

fn apply_hit_effects(
    commands: &mut Commands,
    attack: &Attack,
    target: Entity,
    target_position: Vec3,
    position_attacker: Vec3,
    attack_pos: Vec3,
    hitstop: &mut Hitstop,
    writer: &mut MessageWriter<DamageEvent>,
) {
    // DAMAGE
    writer.write(DamageEvent {
        target,
        amount: attack.definition.damage,
    });
    hitstop.remaining = hitstop.remaining.max(0.05);

    // KNOCKBACK + HITSTUN
    let knockbacks = [
        (&attack.definition.knockback_target, target),
        (&attack.definition.knockback_self, attack.owner),
    ];

    for (kb_opt, entity) in knockbacks {
        if let Some(kb) = kb_opt {
            let dir = match kb.direction {
                KnockbackDirection::SourceToTarget => {
                    (target_position - attack_pos).normalize_or_zero()
                }
                KnockbackDirection::TargetToSource => {
                    (attack_pos - target_position).normalize_or_zero()
                }
                KnockbackDirection::Fixed(v) => v.normalize_or_zero(),
            };

            let velocity = dir * kb.force;

            commands.entity(entity).insert(KnockbackEffect {
                velocity,
                timer: Timer::from_seconds(0.2, TimerMode::Once),
            });

            commands.entity(target).insert(Hitstun {
                remaining: kb.hitstun,
            });
        }
    }
}

fn remove_attack_modifiers(stats: &mut Stats, attack_id: AttackId) {
    stats
        .speed
        .retain(|m| m.source != attack_id || m.lifetime == ModifierLifetime::Permanent);
    stats
        .acceleration
        .retain(|m| m.source != attack_id || m.lifetime == ModifierLifetime::Permanent);
    stats
        .friction
        .retain(|m| m.source != attack_id || m.lifetime == ModifierLifetime::Permanent);
    stats
        .dash_speed
        .retain(|m| m.source != attack_id || m.lifetime == ModifierLifetime::Permanent);
    stats
        .dash_time
        .retain(|m| m.source != attack_id || m.lifetime == ModifierLifetime::Permanent);
    stats
        .dash_friction
        .retain(|m| m.source != attack_id || m.lifetime == ModifierLifetime::Permanent);
    stats
        .dash_stop_time
        .retain(|m| m.source != attack_id || m.lifetime == ModifierLifetime::Permanent);
}

fn cleanup_attack(
    commands: &mut Commands,
    stats_query: &mut Query<&mut Stats>,
    combat_query: &mut Query<&mut CombatState>,
    attack_entity: Entity,
    attack: &Attack,
) {
    // reset state
    if let Ok(mut combat_state) = combat_query.get_mut(attack.owner) {
        *combat_state = CombatState::Idle;
    }

    // remove modifiers
    if let Ok(mut stats) = stats_query.get_mut(attack.owner) {
        remove_attack_modifiers(&mut stats, attack.id);
        info!("new stats are: {:?}", stats)
    }

    // despawn attack
    commands.entity(attack_entity).despawn();
}

pub fn attack_lifetime_system(
    mut commands: Commands,
    time: Res<Time>,
    mut attack_query: Query<(Entity, &mut Attack)>,
    mut combat_query: Query<&mut CombatState>,
    mut stats_query: Query<&mut Stats>,
) {
    for (attack_entity, mut attack) in &mut attack_query {
        attack.lifetime_timer.tick(time.delta());

        if attack.lifetime_timer.is_finished() {
            cleanup_attack(
                &mut commands,
                &mut stats_query,
                &mut combat_query,
                attack_entity,
                &attack,
            );
        }
    }
}

pub fn spawn_attack_system(
    mut commands: Commands,
    mut events: MessageReader<AttackEvent>,
    mut id_counter: ResMut<AttackIdCounter>,
    query: Query<(&Transform, &Facing)>,
) {
    for event in events.read() {
        if let Ok((transform, facing)) = query.get(event.entity) {
            let origin = transform.translation;
            let direction = facing.0.normalize();

            let offset = (direction * (100.0 * 0.5)).extend(0.0);
            let def = simple_beam();
            let sprite = def.spawn.build_sprite();
            let id = AttackId(id_counter.next);
            id_counter.next += 1;
            let mut attack_command = Attack::from_definition(def, event.entity, id);
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

pub fn tick_hitstun(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Hitstun)>,
) {
    for (entity, mut hitstun) in &mut query {
        hitstun.remaining -= time.delta_secs();

        if hitstun.remaining <= 0.0 {
            commands.entity(entity).remove::<Hitstun>();
            info!("hitstun is gone");
        }
    }
}

pub fn not_in_hitstop(hitstop: Res<Hitstop>) -> bool {
    hitstop.remaining <= 0.0
}

pub fn apply_knockback_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Velocity, &mut KnockbackEffect)>,
) {
    for (entity, mut velocity, mut knockback) in &mut query {
        // Apply knockback velocity
        velocity.value = knockback.velocity;
        knockback.velocity *= 0.9;

        knockback.timer.tick(time.delta());

        if knockback.timer.is_finished() {
            commands.entity(entity).remove::<KnockbackEffect>();
        }
    }
}
