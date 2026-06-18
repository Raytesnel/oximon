use super::components::*;
use super::events::*;
use crate::GameState;
use crate::combat::attack_definition::{
    AttackDefinition, AttackEffect, EffectTrigger, ModifierTarget, StatusEffect,
};
use crate::combat::attacks::{
    AttackSpawn, HitBehavior, KnockbackDirection, quick_attack, simple_beam, slow_down, speedo,
};
use crate::common::components::BattleState;
use crate::common::components::{Enemy, ModifierLifetime, RuntimeModifier, Stats};
use crate::movement::types::AllowedMovable;
use avian2d::collision::collider::CollidingEntities;
use avian2d::dynamics::rigid_body::LinearVelocity;
use bevy::prelude::*;

pub const JUMP_BUTTON: KeyCode = KeyCode::Space;
pub const QUICK_ATTACK: KeyCode = KeyCode::KeyQ;
pub const PEWPEW: KeyCode = KeyCode::KeyW;
pub const POWPOW: KeyCode = KeyCode::KeyE;

fn get_attack_for_key(key: KeyCode) -> Option<AttackDefinition> {
    match key {
        QUICK_ATTACK => Some(quick_attack()),
        JUMP_BUTTON => Some(speedo()),
        PEWPEW => Some(slow_down()),
        POWPOW => Some(simple_beam()),

        _ => None,
    }
}
pub fn attack_input_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut id_counter: ResMut<AttackIdCounter>,
    mut query: Query<(Entity, &mut Cooldowns, &mut CombatState), AllowedMovable>,
) {
    for (entity, mut cooldowns, mut combat_state) in &mut query {
        for key in keyboard.get_just_pressed() {
            let Some(def) = get_attack_for_key(*key) else {
                continue;
            };

            // cooldown check
            if let Some(timer) = cooldowns.timers.get(&def.name)
                && !timer.is_finished()
            {
                continue;
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
                CombatEntity,
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

            if let AttackEffect::StatModifier(stat) = &timed.effect {
                let entity = match stat.target {
                    ModifierTarget::SelfEntity => attack.owner,
                    ModifierTarget::TargetEntity => continue,
                };

                if let Ok(mut stats) = stats_query.get_mut(entity) {
                    stats.add_modifier(stat.modifier.to_runtime(attack.id));
                }
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
                        HitEffectArgs {
                            attack: &attack,
                            target: enemy,
                            target_position: enemy_pos,
                            attack_pos,
                            hitstop: &mut hitstop,
                        },
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
                            HitEffectArgs {
                                attack: &attack,
                                target: enemy,
                                target_position: enemy_pos,
                                attack_pos,
                                hitstop: &mut hitstop,
                            },
                            &mut writer,
                            &mut stats_query,
                        );
                    }
                }

                HitBehavior::_Limited(max_hits) => {
                    if attack.hits_done >= max_hits {
                        continue;
                    }

                    if tick_ready {
                        apply_hit_effects(
                            &mut commands,
                            HitEffectArgs {
                                attack: &attack,
                                target: enemy,
                                target_position: enemy_pos,
                                attack_pos,
                                hitstop: &mut hitstop,
                            },
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

pub struct HitEffectArgs<'a> {
    pub attack: &'a Attack,
    pub target: Entity,
    pub target_position: Vec3,
    pub attack_pos: Vec3,
    pub hitstop: &'a mut Hitstop,
}

fn apply_hit_effects(
    commands: &mut Commands,
    attack_hit_connection: HitEffectArgs,
    writer: &mut MessageWriter<DamageEvent>,
    stats_query: &mut Query<&mut Stats>,
) {
    for timed_effect in &attack_hit_connection.attack.definition.effects {
        match &timed_effect.effect {
            AttackEffect::Damage(dmg) => {
                let entity = match dmg.target {
                    ModifierTarget::SelfEntity => attack_hit_connection.attack.owner,
                    ModifierTarget::TargetEntity => attack_hit_connection.target,
                };

                writer.write(DamageEvent {
                    target: entity,
                    amount: dmg.amount,
                });

                attack_hit_connection.hitstop.remaining =
                    attack_hit_connection.hitstop.remaining.max(0.05);
            }

            AttackEffect::Knockback(kb) => {
                let entity = match kb.target {
                    ModifierTarget::SelfEntity => attack_hit_connection.attack.owner,
                    ModifierTarget::TargetEntity => attack_hit_connection.target,
                };

                let dir = match kb.direction {
                    KnockbackDirection::SourceToTarget => (attack_hit_connection.target_position
                        - attack_hit_connection.attack_pos)
                        .normalize_or_zero(),
                    KnockbackDirection::_TargetToSource => (attack_hit_connection.attack_pos
                        - attack_hit_connection.target_position)
                        .normalize_or_zero(),
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
                    ModifierTarget::SelfEntity => attack_hit_connection.attack.owner,
                    ModifierTarget::TargetEntity => attack_hit_connection.target,
                };

                if let Ok(mut stats) = stats_query.get_mut(entity) {
                    stats.add_modifier(stat.modifier.to_runtime(attack_hit_connection.attack.id));
                }
            }
            AttackEffect::ApplyStatus(status) => match status {
                StatusEffect::Poison {
                    dps,
                    duration,
                    tick_rate,
                } => {
                    commands
                        .entity(attack_hit_connection.target)
                        .insert(Poison {
                            damage: *dps,
                            tick_timer: Timer::from_seconds(*tick_rate, TimerMode::Repeating),
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
                ModifierLifetime::_WhileAttacking | ModifierLifetime::OnAttackEnd
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
        if let Some(entity) = attack.follow_entity
            && let Ok(target_transform) = targets.get(entity)
        {
            transform.translation = target_transform.translation + attack.definition.offset;
        }
    }
}

pub fn despawn_dead_system(
    mut commands: Commands,
    query: Query<(Entity, &CombatState)>,
    mut next_state: ResMut<NextState<BattleState>>,
) {
    for (entity, state) in &query {
        if *state == CombatState::Dead {
            commands.entity(entity).despawn();
            info!("monster:{:?} is dead, ending battle...", entity);
            next_state.set(BattleState::Ending);
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
    mut query: Query<(Entity, &mut LinearVelocity, &mut KnockbackEffect)>,
) {
    for (entity, mut lv, mut knockback) in &mut query {
        lv.0 = knockback.velocity.truncate();
        knockback.velocity *= 0.9;
        knockback.timer.tick(time.delta());
        if knockback.timer.is_finished() {
            commands.entity(entity).remove::<KnockbackEffect>();
        }
    }
}

pub fn cleanup_combat(
    mut commands: Commands,
    query: Query<Entity, (With<CombatEntity>, Without<CombatSceneEntity>)>,
) {
    for e in &query {
        commands.entity(e).despawn();
    }
}

pub fn debug_collisions(query: Query<(Entity, &CollidingEntities), With<CombatEntity>>) {
    for (e, colliding) in &query {
        if !colliding.is_empty() {
            info!("{:?} is colliding with {:?}", e, colliding);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::attack_definition::*;
    use crate::combat::attacks::*;
    use crate::common::components::*;
    use bevy::app::FixedMain;

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn tick(app: &mut App, dt: f32) {
        let delta = std::time::Duration::from_secs_f32(dt);
        {
            let mut time = app.world_mut().resource_mut::<Time<Fixed>>();
            time.advance_by(delta);
        }
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(delta);
        }
        app.world_mut().run_schedule(FixedMain);
        app.update();
    }

    fn setup_app() -> App {
        let mut app = App::new();
        app.init_resource::<Time>();
        app.insert_resource(Time::<Fixed>::from_seconds(0.016));
        app.add_message::<DamageEvent>();
        app
    }

    fn make_health(current: f32) -> Health {
        Health {
            current,
            _max: current,
        }
    }

    /// A minimal single-hit attack with one damage effect targeting the enemy.
    fn damage_only_attack(owner: Entity, amount: f32) -> Attack {
        let def = AttackDefinition {
            name: "test_attack".to_string(),
            effects: vec![TimedEffect {
                trigger: EffectTrigger::OnHit,
                effect: AttackEffect::Damage(DamageEffect {
                    amount,
                    target: ModifierTarget::TargetEntity,
                }),
            }],
            lifetime: 5.0,
            hit_interval: 0.1,
            cooldown: 1.0,
            hit_behavior: HitBehavior::Single,
            offset: Vec3::ZERO,
            spawn: AttackSpawn::Hitbox {
                color: Color::WHITE,
                size: Vec2::new(10.0, 10.0),
            },
        };
        Attack::from_definition(def, owner, AttackId(0))
    }

    /// A minimal multi-hit attack (no damage, just to test tick behavior).
    fn multihit_attack(owner: Entity, amount: f32) -> Attack {
        let def = AttackDefinition {
            name: "test_multihit".to_string(),
            effects: vec![TimedEffect {
                trigger: EffectTrigger::OnHit,
                effect: AttackEffect::Damage(DamageEffect {
                    amount,
                    target: ModifierTarget::TargetEntity,
                }),
            }],
            lifetime: 5.0,
            hit_interval: 0.016,
            cooldown: 1.0,
            hit_behavior: HitBehavior::MultiHit,
            offset: Vec3::ZERO,
            spawn: AttackSpawn::Hitbox {
                color: Color::WHITE,
                size: Vec2::new(10.0, 10.0),
            },
        };
        Attack::from_definition(def, owner, AttackId(1))
    }

    // ── apply_damage_system ───────────────────────────────────────────────────

    fn setup_damage_app() -> App {
        let mut app = setup_app();
        app.add_systems(Update, super::apply_damage_system);
        app
    }

    #[test]
    fn test_damage_event_reduces_health() {
        let mut app = setup_damage_app();

        let entity = app
            .world_mut()
            .spawn((make_health(100.0), CombatState::Idle))
            .id();

        app.world_mut().write_message(DamageEvent {
            target: entity,
            amount: 30.0,
        });
        tick(&mut app, 0.016);

        let health = app.world().get::<Health>(entity).unwrap();
        assert!(
            (health.current - 70.0).abs() < 0.01,
            "health should be 70 after 30 damage, was {}",
            health.current
        );
    }

    #[test]
    fn test_damage_event_sets_dead_state_at_zero_health() {
        let mut app = setup_damage_app();

        let entity = app
            .world_mut()
            .spawn((make_health(10.0), CombatState::Idle))
            .id();

        app.world_mut().write_message(DamageEvent {
            target: entity,
            amount: 10.0,
        });
        tick(&mut app, 0.016);

        let state = app.world().get::<CombatState>(entity).unwrap();
        assert_eq!(*state, CombatState::Dead, "entity should be Dead at 0 hp");
    }

    #[test]
    fn test_damage_event_clamps_health_to_zero() {
        let mut app = setup_damage_app();

        let entity = app
            .world_mut()
            .spawn((make_health(5.0), CombatState::Idle))
            .id();

        app.world_mut().write_message(DamageEvent {
            target: entity,
            amount: 999.0,
        });
        tick(&mut app, 0.016);

        let health = app.world().get::<Health>(entity).unwrap();
        assert!(
            health.current >= 0.0,
            "health should never go below 0, was {}",
            health.current
        );
    }

    #[test]
    fn test_multiple_damage_events_stack() {
        let mut app = setup_damage_app();

        let entity = app
            .world_mut()
            .spawn((make_health(100.0), CombatState::Idle))
            .id();

        app.world_mut().write_message(DamageEvent {
            target: entity,
            amount: 10.0,
        });
        app.world_mut().write_message(DamageEvent {
            target: entity,
            amount: 20.0,
        });
        tick(&mut app, 0.016);

        let health = app.world().get::<Health>(entity).unwrap();
        assert!(
            (health.current - 70.0).abs() < 0.01,
            "health should be 70 after 10+20 damage, was {}",
            health.current
        );
    }

    // ── tick_hitstun ──────────────────────────────────────────────────────────

    fn setup_hitstun_app() -> App {
        let mut app = setup_app();
        app.add_systems(Update, super::tick_hitstun);
        app
    }

    #[test]
    fn test_hitstun_removed_when_expired() {
        let mut app = setup_hitstun_app();

        let entity = app.world_mut().spawn(Hitstun { remaining: 0.016 }).id();

        tick(&mut app, 0.016);

        assert!(
            app.world().get::<Hitstun>(entity).is_none(),
            "Hitstun should be removed after remaining reaches 0"
        );
    }

    #[test]
    fn test_hitstun_still_present_when_not_expired() {
        let mut app = setup_hitstun_app();

        let entity = app.world_mut().spawn(Hitstun { remaining: 1.0 }).id();

        tick(&mut app, 0.016);

        assert!(
            app.world().get::<Hitstun>(entity).is_some(),
            "Hitstun should still be present before remaining reaches 0"
        );
    }

    #[test]
    fn test_hitstun_decrements_remaining_each_tick() {
        let mut app = setup_hitstun_app();

        let entity = app.world_mut().spawn(Hitstun { remaining: 1.0 }).id();

        tick(&mut app, 0.016);

        let hitstun = app.world().get::<Hitstun>(entity).unwrap();
        assert!(
            hitstun.remaining < 1.0,
            "remaining should decrease each tick, was {}",
            hitstun.remaining
        );
    }

    // ── apply_knockback_system ────────────────────────────────────────────────

    fn setup_knockback_app() -> App {
        let mut app = setup_app();
        app.add_systems(Update, super::apply_knockback_system);
        app
    }

    #[test]
    fn test_knockback_sets_velocity() {
        let mut app = setup_knockback_app();
        let entity = app
            .world_mut()
            .spawn((
                LinearVelocity::default(),
                KnockbackEffect {
                    velocity: Vec3::new(100.0, 0.0, 0.0),
                    timer: Timer::from_seconds(0.2, TimerMode::Once),
                },
            ))
            .id();
        tick(&mut app, 0.016);
        let lv = app.world().get::<LinearVelocity>(entity).unwrap();
        assert!(
            lv.0.x > 0.0,
            "velocity.x should be set by knockback, was {}",
            lv.0.x
        );
    }

    #[test]
    fn test_knockback_removed_after_timer_expires() {
        let mut app = setup_knockback_app();
        let entity = app
            .world_mut()
            .spawn((
                LinearVelocity::default(),
                KnockbackEffect {
                    velocity: Vec3::new(100.0, 0.0, 0.0),
                    timer: Timer::from_seconds(0.016, TimerMode::Once),
                },
            ))
            .id();
        tick(&mut app, 0.016);
        assert!(
            app.world().get::<KnockbackEffect>(entity).is_none(),
            "KnockbackEffect should be removed after timer expires"
        );
    }

    #[test]
    fn test_knockback_not_removed_before_timer_expires() {
        let mut app = setup_knockback_app();
        let entity = app
            .world_mut()
            .spawn((
                LinearVelocity::default(),
                KnockbackEffect {
                    velocity: Vec3::new(100.0, 0.0, 0.0),
                    timer: Timer::from_seconds(1.0, TimerMode::Once),
                },
            ))
            .id();
        tick(&mut app, 0.016);
        assert!(app.world().get::<KnockbackEffect>(entity).is_some());
    }

    #[test]
    fn test_knockback_velocity_decays_each_tick() {
        let mut app = setup_knockback_app();
        let initial = 100.0;
        let entity = app
            .world_mut()
            .spawn((
                LinearVelocity::default(),
                KnockbackEffect {
                    velocity: Vec3::new(initial, 0.0, 0.0),
                    timer: Timer::from_seconds(1.0, TimerMode::Once),
                },
            ))
            .id();
        tick(&mut app, 0.016);
        tick(&mut app, 0.016);
        let lv = app.world().get::<LinearVelocity>(entity).unwrap();
        assert!(
            lv.0.x < initial,
            "knockback velocity should decay over ticks, was {}",
            lv.0.x
        );
    }

    // ── cooldown_tick_system ──────────────────────────────────────────────────

    fn setup_cooldown_app() -> App {
        let mut app = setup_app();
        app.add_systems(Update, super::cooldown_tick_system);
        app
    }

    #[test]
    fn test_cooldown_timer_ticks_down() {
        let mut app = setup_cooldown_app();

        let mut cooldowns = Cooldowns::default();
        cooldowns.timers.insert(
            "test".to_string(),
            Timer::from_seconds(1.0, TimerMode::Once),
        );
        let entity = app.world_mut().spawn(cooldowns).id();

        tick(&mut app, 0.016);

        // timer should still be in the map (not yet finished)
        let cooldowns = app.world().get::<Cooldowns>(entity).unwrap();
        assert!(
            cooldowns.timers.contains_key("test"),
            "cooldown timer should still exist before it finishes"
        );
    }

    #[test]
    fn test_cooldown_timer_removed_when_finished() {
        let mut app = setup_cooldown_app();

        let mut cooldowns = Cooldowns::default();
        // Very short timer — finishes on first tick
        cooldowns.timers.insert(
            "test".to_string(),
            Timer::from_seconds(0.001, TimerMode::Once),
        );
        let entity = app.world_mut().spawn(cooldowns).id();

        tick(&mut app, 0.016);

        let cooldowns = app.world().get::<Cooldowns>(entity).unwrap();
        assert!(
            !cooldowns.timers.contains_key("test"),
            "finished cooldown timer should be removed from the map"
        );
    }

    #[test]
    fn test_multiple_cooldowns_tick_independently() {
        let mut app = setup_cooldown_app();

        let mut cooldowns = Cooldowns::default();
        cooldowns.timers.insert(
            "short".to_string(),
            Timer::from_seconds(0.001, TimerMode::Once),
        );
        cooldowns.timers.insert(
            "long".to_string(),
            Timer::from_seconds(5.0, TimerMode::Once),
        );
        let entity = app.world_mut().spawn(cooldowns).id();

        tick(&mut app, 0.016);

        let cooldowns = app.world().get::<Cooldowns>(entity).unwrap();
        assert!(
            !cooldowns.timers.contains_key("short"),
            "short cooldown should be removed"
        );
        assert!(
            cooldowns.timers.contains_key("long"),
            "long cooldown should still be present"
        );
    }

    // ── attack_lifetime_system ────────────────────────────────────────────────

    fn setup_lifetime_app() -> App {
        let mut app = setup_app();
        app.add_systems(Update, super::attack_lifetime_system);
        app
    }

    #[test]
    fn test_attack_despawned_after_lifetime_expires() {
        let mut app = setup_lifetime_app();

        let owner = app
            .world_mut()
            .spawn((CombatState::Attacking, Stats::default()))
            .id();

        let mut attack = damage_only_attack(owner, 10.0);
        // Force lifetime to expire immediately
        attack.lifetime_timer = Timer::from_seconds(0.001, TimerMode::Once);

        let attack_entity = app.world_mut().spawn(attack).id();

        tick(&mut app, 0.016);

        assert!(
            app.world().get_entity(attack_entity).is_err(),
            "attack entity should be despawned after lifetime expires"
        );
    }

    #[test]
    fn test_attack_not_despawned_before_lifetime_expires() {
        let mut app = setup_lifetime_app();

        let owner = app
            .world_mut()
            .spawn((CombatState::Attacking, Stats::default()))
            .id();

        let attack_entity = app.world_mut().spawn(damage_only_attack(owner, 10.0)).id();

        tick(&mut app, 0.016); // lifetime is 5.0 s, so still alive

        assert!(
            app.world().get_entity(attack_entity).is_ok(),
            "attack entity should still exist before lifetime expires"
        );
    }

    #[test]
    fn test_attack_expiry_resets_owner_combat_state_to_idle() {
        let mut app = setup_lifetime_app();

        let owner = app
            .world_mut()
            .spawn((CombatState::Attacking, Stats::default()))
            .id();

        let mut attack = damage_only_attack(owner, 10.0);
        attack.lifetime_timer = Timer::from_seconds(0.001, TimerMode::Once);
        app.world_mut().spawn(attack);

        tick(&mut app, 0.016);

        let state = app.world().get::<CombatState>(owner).unwrap();
        assert_eq!(
            *state,
            CombatState::Idle,
            "owner CombatState should be reset to Idle when attack expires"
        );
    }

    #[test]
    fn test_attack_expiry_removes_onattackend_modifiers_from_owner() {
        let mut app = setup_lifetime_app();

        let mut stats = Stats::default();
        // Pre-insert a modifier that should be cleaned up on attack end
        stats.speed.push(RuntimeModifier {
            source: AttackId(0),
            stat_type: StatType::Speed,
            flat: 0.0,
            multiplier: 3.0,
            lifetime: ModifierLifetime::OnAttackEnd,
            timer: None,
        });

        let owner = app.world_mut().spawn((CombatState::Attacking, stats)).id();

        let mut attack = damage_only_attack(owner, 10.0);
        attack.lifetime_timer = Timer::from_seconds(0.001, TimerMode::Once);
        app.world_mut().spawn(attack);

        tick(&mut app, 0.016);

        let stats = app.world().get::<Stats>(owner).unwrap();
        assert!(
            stats.speed.is_empty(),
            "OnAttackEnd modifier should be removed when attack expires"
        );
    }

    // ── attack_hit_system ─────────────────────────────────────────────────────

    fn setup_hit_app() -> App {
        let mut app = setup_app();
        app.insert_resource(Hitstop { remaining: 0.0 });
        app.add_systems(
            Update,
            (super::attack_hit_system, super::apply_damage_system).chain(),
        );
        app
    }

    #[test]
    fn test_single_hit_attack_deals_damage_on_overlap() {
        let mut app = setup_hit_app();

        let owner = app.world_mut().spawn_empty().id();

        let enemy = app
            .world_mut()
            .spawn((
                Enemy,
                Transform::default(),
                Hurtbox {
                    size: Vec2::new(20.0, 20.0),
                },
                make_health(100.0),
                CombatState::Idle,
            ))
            .id();

        app.world_mut().spawn((
            damage_only_attack(owner, 10.0),
            Transform::default(), // same position → overlapping
            Hitbox {
                size: Vec2::new(20.0, 20.0),
            },
        ));

        tick(&mut app, 0.016);

        let health = app.world().get::<Health>(enemy).unwrap();
        assert!(
            health.current < 100.0,
            "enemy health should decrease on hit, was {}",
            health.current
        );
    }

    #[test]
    fn test_single_hit_attack_only_hits_once() {
        let mut app = setup_hit_app();

        let owner = app.world_mut().spawn_empty().id();

        let enemy = app
            .world_mut()
            .spawn((
                Enemy,
                Transform::default(),
                Hurtbox {
                    size: Vec2::new(20.0, 20.0),
                },
                make_health(100.0),
                CombatState::Idle,
            ))
            .id();

        app.world_mut().spawn((
            damage_only_attack(owner, 10.0),
            Transform::default(),
            Hitbox {
                size: Vec2::new(20.0, 20.0),
            },
        ));

        tick(&mut app, 0.016);
        tick(&mut app, 0.016);
        tick(&mut app, 0.016);

        let health = app.world().get::<Health>(enemy).unwrap();
        assert!(
            (health.current - 90.0).abs() < 0.01,
            "Single hit attack should only deal damage once, health was {}",
            health.current
        );
    }

    #[test]
    fn test_single_hit_attack_does_not_hit_when_not_overlapping() {
        let mut app = setup_hit_app();

        let owner = app.world_mut().spawn_empty().id();

        let enemy = app
            .world_mut()
            .spawn((
                Enemy,
                Transform::from_translation(Vec3::new(1000.0, 1000.0, 0.0)), // far away
                Hurtbox {
                    size: Vec2::new(20.0, 20.0),
                },
                make_health(100.0),
                CombatState::Idle,
            ))
            .id();

        app.world_mut().spawn((
            damage_only_attack(owner, 10.0),
            Transform::default(),
            Hitbox {
                size: Vec2::new(20.0, 20.0),
            },
        ));

        tick(&mut app, 0.016);

        let health = app.world().get::<Health>(enemy).unwrap();
        assert!(
            (health.current - 100.0).abs() < 0.01,
            "enemy should not take damage when not overlapping, health was {}",
            health.current
        );
    }

    #[test]
    fn test_multihit_attack_hits_multiple_times() {
        let mut app = setup_hit_app();

        let owner = app.world_mut().spawn_empty().id();

        let enemy = app
            .world_mut()
            .spawn((
                Enemy,
                Transform::default(),
                Hurtbox {
                    size: Vec2::new(20.0, 20.0),
                },
                make_health(100.0),
                CombatState::Idle,
            ))
            .id();

        app.world_mut().spawn((
            multihit_attack(owner, 5.0),
            Transform::default(),
            Hitbox {
                size: Vec2::new(20.0, 20.0),
            },
        ));

        // tick enough times to get multiple hits
        tick(&mut app, 0.016);
        tick(&mut app, 0.016);
        tick(&mut app, 0.016);

        let health = app.world().get::<Health>(enemy).unwrap();
        assert!(
            health.current < 90.0, // more than one hit's worth
            "MultiHit attack should deal damage more than once, health was {}",
            health.current
        );
    }

    #[test]
    fn test_hit_sets_hitstop() {
        let mut app = setup_hit_app();

        let owner = app.world_mut().spawn_empty().id();

        app.world_mut().spawn((
            Enemy,
            Transform::default(),
            Hurtbox {
                size: Vec2::new(20.0, 20.0),
            },
            make_health(100.0),
            CombatState::Idle,
        ));

        app.world_mut().spawn((
            damage_only_attack(owner, 10.0),
            Transform::default(),
            Hitbox {
                size: Vec2::new(20.0, 20.0),
            },
        ));

        tick(&mut app, 0.016);

        let hitstop = app.world().resource::<Hitstop>();
        assert!(hitstop.remaining > 0.0, "Hitstop should be set after a hit");
    }
}
