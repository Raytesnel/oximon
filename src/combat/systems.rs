use super::components::*;
use super::events::*;
use crate::combat::attack_definition::{
    AttackDefinition, AttackEffect, EffectTrigger, ModifierTarget, StatusEffect,
};
use crate::combat::attacks::{
    AttackSpawn, HitBehavior, KnockbackDefinition, KnockbackDirection, KnockbackMode, quick_attack,
    simple_beam, slow_down, speedo,
};
use crate::common::components::{
    Enemy, ModifierLifetime, ModifierTrigger, Player, RuntimeModifier, Stats,
};
use crate::movement::components::{Facing, Movable, Velocity};
use bevy::ecs::error::info;
use bevy::prelude::*;

pub const JUMP_BUTTON: KeyCode = KeyCode::Space;
pub const QUICK_ATTACK: KeyCode = KeyCode::KeyQ;
pub const PEWPEW: KeyCode = KeyCode::KeyW;

pub struct AttackContext<'a> {
    pub cooldowns: &'a mut Cooldowns,
    pub combat_state: &'a mut CombatState,
    pub id_counter: &'a mut AttackIdCounter,
    pub owner: Entity,
}
fn get_attack_for_key(key: KeyCode) -> Option<AttackDefinition> {
    match key {
        QUICK_ATTACK => Some(quick_attack()),
        JUMP_BUTTON => Some(speedo()),
        PEWPEW => Some(slow_down()),
        _ => None,
    }
}
pub fn attack_input_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut id_counter: ResMut<AttackIdCounter>,
    mut query: Query<(Entity, &mut Cooldowns, &mut CombatState), (With<Movable>, Without<Hitstun>)>,
) {
    for (entity, mut cooldowns, mut combat_state) in &mut query {
        for key in keyboard.get_just_pressed() {
            let Some(def) = get_attack_for_key(*key) else {
                continue;
            };

            // cooldown check
            if let Some(timer) = cooldowns.timers.get(&def.name) {
                if !timer.is_finished() {
                    continue;
                }
            }

            // start cooldown
            cooldowns.timers.insert(
                def.name.clone(),
                Timer::from_seconds(def.cooldown, TimerMode::Once),
            );

            // id
            let id = AttackId(id_counter.next);
            id_counter.next += 1;

            let sprite = def.spawn.build_sprite();

            // 🔥 spawn attack
            let mut entity_commands = commands.spawn((
                Attack::from_definition(def.clone(), entity, id),
                Transform::default(),
                sprite,
            ));

            let size = match &def.spawn {
                AttackSpawn::Hitbox { size, .. } => *size,
            };

            entity_commands.insert(Hitbox { size });

            *combat_state = CombatState::Attacking;
        }
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

        for timed in &attack.definition.effects {
            if !matches!(timed.trigger, EffectTrigger::OnCast) {
                continue;
            }

            match &timed.effect {
                AttackEffect::StatModifier(stat) => {
                    let entity = match stat.target {
                        ModifierTarget::SelfEntity => attack.owner,
                        ModifierTarget::TargetEntity => continue, // meestal niet logisch bij cast
                    };

                    if let Ok(mut stats) = stats_query.get_mut(entity) {
                        stats.add_modifier(stat.modifier.to_runtime(attack.id));
                    }
                }

                _ => {} // andere effects doen niks bij cast
            }
        }

        attack.applied_start_modifiers = true;
    }
}

fn intersects(pos_a: Vec3, size_a: Vec2, pos_b: Vec3, size_b: Vec2) -> bool {
    let half_a = size_a / 2.0;
    let half_b = size_b / 2.0;

    let delta = pos_a - pos_b;

    delta.x.abs() <= (half_a.x + half_b.x) && delta.y.abs() <= (half_a.y + half_b.y)
}

pub fn attack_hit_system(
    mut commands: Commands,
    mut attacks: Query<(&Transform, &Hitbox, &mut Attack)>,
    enemies: Query<(Entity, &Transform, &Hurtbox), With<Enemy>>,
    mut hitstop: ResMut<Hitstop>,
    mut writer: MessageWriter<DamageEvent>,
    mut stats_query: Query<&mut Stats>,

    time: Res<Time>,
) {
    for (attack_transform, hitbox, mut attack) in &mut attacks {
        let attack_pos = attack_transform.translation;

        let tick_ready = attack.hit_timer.tick(time.delta()).just_finished();

        for (enemy, enemy_transform, hurtbox) in &enemies {
            let enemy_pos = enemy_transform.translation;
            if !intersects(attack_pos, hitbox.size, enemy_pos, hurtbox.size) {
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
                        &mut hitstop,
                        &mut writer,
                        &mut stats_query,
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
                            &mut hitstop,
                            &mut writer,
                            &mut stats_query,
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
                            &mut hitstop,
                            &mut writer,
                            &mut stats_query,
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
    attack_pos: Vec3,
    hitstop: &mut Hitstop,
    writer: &mut MessageWriter<DamageEvent>,
    stats_query: &mut Query<&mut Stats>,
) {
    for timed_effect in &attack.definition.effects {
        match &timed_effect.effect {
            AttackEffect::Damage(dmg) => {
                let entity = match dmg.target {
                    ModifierTarget::SelfEntity => attack.owner,
                    ModifierTarget::TargetEntity => target,
                };

                writer.write(DamageEvent {
                    target: entity,
                    amount: dmg.amount,
                });

                hitstop.remaining = hitstop.remaining.max(0.05);
            }

            AttackEffect::Knockback(kb) => {
                let entity = match kb.target {
                    ModifierTarget::SelfEntity => attack.owner,
                    ModifierTarget::TargetEntity => target,
                };

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

                commands.entity(entity).insert(Hitstun {
                    remaining: kb.hitstun,
                });
            }

            AttackEffect::StatModifier(stat) => {
                let entity = match stat.target {
                    ModifierTarget::SelfEntity => attack.owner,
                    ModifierTarget::TargetEntity => target,
                };

                if let Ok(mut stats) = stats_query.get_mut(entity) {
                    stats.add_modifier(stat.modifier.to_runtime(attack.id));
                }
            }
            AttackEffect::ApplyStatus(status) => match status {
                StatusEffect::Poison {
                    dps,
                    duration,
                    tick_rate,
                } => {
                    commands.entity(target).insert(Poison {
                        damage: *dps,
                        tick_timer: Timer::from_seconds(*tick_rate, TimerMode::Repeating),
                        duration: Timer::from_seconds(*duration, TimerMode::Once),
                    });
                }

                StatusEffect::Slow {
                    multiplier,
                    duration,
                } => {
                    commands.entity(target).insert(Slow {
                        multiplier: *multiplier,
                        applied: false,
                        duration: Timer::from_seconds(*duration, TimerMode::Once),
                    });
                }

                StatusEffect::Stun { duration } => {
                    commands.entity(target).insert(Stun {
                        duration: Timer::from_seconds(*duration, TimerMode::Once),
                    });
                }
            },
        }
    }
}

fn remove_attack_modifiers(stats: &mut Stats, attack_id: AttackId) {
    let should_remove = |m: &RuntimeModifier| {
        m.source == attack_id
            && matches!(
                m.lifetime,
                ModifierLifetime::WhileAttacking | ModifierLifetime::OnAttackEnd
            )
    };

    stats.speed.retain(|m| !should_remove(m));
    stats.acceleration.retain(|m| !should_remove(m));
    stats.friction.retain(|m| !should_remove(m));
    stats.dash_speed.retain(|m| !should_remove(m));
    stats.dash_time.retain(|m| !should_remove(m));
    stats.dash_friction.retain(|m| !should_remove(m));
    stats.dash_stop_time.retain(|m| !should_remove(m));
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
