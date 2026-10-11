use crate::combat::components::{AttackId, KnockbackEffect, Poison, Slow, Stun};
use crate::combat::events::DamageEvent;
use crate::common::components::{ModifierLifetime, RuntimeModifier, StatType, Stats};
use bevy::prelude::*;

pub fn poison_system(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Poison)>,
    mut writer: MessageWriter<DamageEvent>,
    mut commands: Commands,
) {
    for (entity, mut poison) in &mut query {
        poison.tick_timer.tick(time.delta());
        poison.duration.tick(time.delta());

        if poison.tick_timer.just_finished() {
            writer.write(DamageEvent {
                target: entity,
                amount: poison.damage,
            });
            commands.entity(entity).insert(KnockbackEffect {
                velocity: Vec3::ZERO, // tweak this value
                timer: Timer::from_seconds(0.1, TimerMode::Once),
            });
        }

        if poison.duration.is_finished() {
            commands.entity(entity).remove::<Poison>();
        }
    }
}

pub fn slow_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Slow, &mut Stats)>,
) {
    for (entity, mut slow, mut stats) in &mut query {
        // apply ONCE
        if !slow.applied {
            stats.add_modifier(RuntimeModifier {
                source: AttackId(9999),
                stat_type: StatType::Speed,
                flat: 0.0,
                multiplier: slow.multiplier,
                lifetime: ModifierLifetime::Duration,
                timer: Some(slow.duration.clone()),
            });

            slow.applied = true;
        }

        slow.duration.tick(time.delta());

        if slow.duration.is_finished() {
            commands.entity(entity).remove::<Slow>();
        }
    }
}
pub fn stun_system(time: Res<Time>, mut commands: Commands, mut query: Query<(Entity, &mut Stun)>) {
    for (entity, mut stun) in &mut query {
        stun.duration.tick(time.delta());

        if stun.duration.is_finished() {
            commands.entity(entity).remove::<Stun>();
        }
    }
}

pub struct StatusEffectsPlugin;

impl Plugin for StatusEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, poison_system)
            .add_systems(Update, slow_system)
            .add_systems(Update, stun_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::components::*;
    use bevy::app::FixedMain;

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
        app.add_message::<DamageEvent>();
        app.add_message::<AttackEvent>();
        app.insert_resource(Time::<Fixed>::from_seconds(0.016));
        app
    }

    // ── Poison(later also fire etc, damage over time) ────────────────────────────────────────────────────────────────

    fn make_poison(tick_secs: f32, duration_secs: f32, damage: f32) -> Poison {
        Poison {
            tick_timer: Timer::from_seconds(tick_secs, TimerMode::Repeating),
            duration: Timer::from_seconds(duration_secs, TimerMode::Once),
            damage,
        }
    }

    fn setup_poison_app() -> App {
        let mut app = setup_app();
        // Both systems run together so we can observe health as the
        // end result of the message being written and then consumed.
        app.add_systems(
            Update,
            (super::poison_system, crate::combat::apply_damage_system).chain(),
        );
        app
    }

    #[test]
    fn test_poison_tick_reduces_health() {
        let mut app = setup_poison_app();

        let entity = app
            .world_mut()
            .spawn((
                make_poison(0.016, 10.0, 5.0),
                Health {
                    current: 20.0,
                    _max: 20.0,
                },
                CombatState::Idle,
            ))
            .id();

        tick(&mut app, 0.016);

        let health = app.world().get::<Health>(entity).unwrap();
        assert!(
            (health.current - 15.0).abs() < 0.01,
            "health should be 15.0 after one poison tick, was {}",
            health.current
        );
    }

    #[test]
    fn test_poison_tick_inserts_knockback() {
        let mut app = setup_poison_app();

        let entity = app
            .world_mut()
            .spawn((
                make_poison(0.016, 10.0, 5.0),
                Health {
                    current: 20.0,
                    _max: 20.0,
                },
                CombatState::Idle,
            ))
            .id();

        tick(&mut app, 0.016);

        assert!(
            app.world().get::<KnockbackEffect>(entity).is_some(),
            "KnockbackEffect should be inserted on poison tick"
        );
    }

    #[test]
    fn test_poison_before_tick_does_not_reduce_health() {
        let mut app = setup_poison_app();

        let entity = app
            .world_mut()
            .spawn((
                make_poison(1.0, 10.0, 5.0), // tick every 1 s
                Health {
                    current: 20.0,
                    _max: 20.0,
                },
                CombatState::Idle,
            ))
            .id();

        tick(&mut app, 0.016); // too short to tick

        let health = app.world().get::<Health>(entity).unwrap();
        assert!(
            (health.current - 20.0).abs() < 0.01,
            "health should be unchanged before poison tick, was {}",
            health.current
        );
    }

    #[test]
    fn test_poison_multiple_ticks_reduce_health_each_tick() {
        let mut app = setup_poison_app();

        let entity = app
            .world_mut()
            .spawn((
                make_poison(0.016, 10.0, 3.0),
                Health {
                    current: 20.0,
                    _max: 20.0,
                },
                CombatState::Idle,
            ))
            .id();

        tick(&mut app, 0.016);
        tick(&mut app, 0.016);

        let health = app.world().get::<Health>(entity).unwrap();
        assert!(
            (health.current - 14.0).abs() < 0.01,
            "health should be 14.0 after two poison ticks, was {}",
            health.current
        );
    }

    #[test]
    fn test_poison_removed_after_duration_expires() {
        let mut app = setup_poison_app();

        let entity = app
            .world_mut()
            .spawn((
                make_poison(10.0, 0.016, 5.0),
                Health {
                    current: 20.0,
                    _max: 20.0,
                },
                CombatState::Idle,
            ))
            .id();

        tick(&mut app, 0.016);

        assert!(
            app.world().get::<Poison>(entity).is_none(),
            "Poison should be removed once duration finishes"
        );
    }

    #[test]
    fn test_poison_not_removed_before_duration_expires() {
        let mut app = setup_poison_app();

        let entity = app
            .world_mut()
            .spawn((
                make_poison(10.0, 1.0, 5.0),
                Health {
                    current: 20.0,
                    _max: 20.0,
                },
                CombatState::Idle,
            ))
            .id();

        tick(&mut app, 0.016);

        assert!(
            app.world().get::<Poison>(entity).is_some(),
            "Poison should still be present before duration finishes"
        );
    }

    // ── Slow(or later status modifiers) ──────────────────────────────────────────────────────────────────

    fn make_slow(duration_secs: f32, multiplier: f32) -> Slow {
        Slow {
            duration: Timer::from_seconds(duration_secs, TimerMode::Once),
            multiplier,
            applied: false,
        }
    }

    #[test]
    fn test_slow_adds_speed_modifier_on_first_tick() {
        let mut app = setup_app();
        app.add_systems(Update, super::slow_system);

        let entity = app
            .world_mut()
            .spawn((make_slow(1.0, 0.5), Stats::default()))
            .id();

        tick(&mut app, 0.016);

        let stats = app.world().get::<Stats>(entity).unwrap();
        assert!(
            stats
                .speed
                .iter()
                .any(|m| matches!(m.stat_type, StatType::Speed)),
            "slow should add a Speed modifier"
        );
    }

    #[test]
    fn test_slow_applied_flag_set_after_first_tick() {
        let mut app = setup_app();
        app.add_systems(Update, super::slow_system);

        let entity = app
            .world_mut()
            .spawn((make_slow(1.0, 0.5), Stats::default()))
            .id();

        tick(&mut app, 0.016);

        let slow = app.world().get::<Slow>(entity).unwrap();
        assert!(slow.applied, "applied flag should be true after first tick");
    }

    #[test]
    fn test_slow_does_not_add_duplicate_modifier_on_second_tick() {
        let mut app = setup_app();
        app.add_systems(Update, super::slow_system);

        let entity = app
            .world_mut()
            .spawn((make_slow(1.0, 0.5), Stats::default()))
            .id();

        tick(&mut app, 0.016);
        tick(&mut app, 0.016);

        let stats = app.world().get::<Stats>(entity).unwrap();
        let count = stats
            .speed
            .iter()
            .filter(|m| matches!(m.stat_type, StatType::Speed))
            .count();

        assert_eq!(count, 1, "modifier should only be applied once");
    }

    #[test]
    fn test_slow_removed_after_duration_expires() {
        let mut app = setup_app();
        app.add_systems(Update, super::slow_system);

        let entity = app
            .world_mut()
            .spawn((make_slow(0.016, 0.5), Stats::default()))
            .id();

        tick(&mut app, 0.016);

        assert!(
            app.world().get::<Slow>(entity).is_none(),
            "Slow should be removed after duration expires"
        );
    }

    #[test]
    fn test_slow_not_removed_before_duration_expires() {
        let mut app = setup_app();
        app.add_systems(Update, super::slow_system);

        let entity = app
            .world_mut()
            .spawn((make_slow(1.0, 0.5), Stats::default()))
            .id();

        tick(&mut app, 0.016);

        assert!(
            app.world().get::<Slow>(entity).is_some(),
            "Slow should still be present before duration expires"
        );
    }

    // ── Stun ──────────────────────────────────────────────────────────────────

    fn make_stun(duration_secs: f32) -> Stun {
        Stun {
            duration: Timer::from_seconds(duration_secs, TimerMode::Once),
        }
    }

    #[test]
    fn test_stun_removed_after_duration_expires() {
        let mut app = setup_app();
        app.add_systems(Update, super::stun_system);

        let entity = app.world_mut().spawn(make_stun(0.016)).id();

        tick(&mut app, 0.016);

        assert!(
            app.world().get::<Stun>(entity).is_none(),
            "Stun should be removed after duration expires"
        );
    }

    #[test]
    fn test_stun_not_removed_before_duration_expires() {
        let mut app = setup_app();
        app.add_systems(Update, super::stun_system);

        let entity = app.world_mut().spawn(make_stun(1.0)).id();

        tick(&mut app, 0.016);

        assert!(
            app.world().get::<Stun>(entity).is_some(),
            "Stun should still be present before duration expires"
        );
    }

    #[test]
    fn test_stun_still_present_mid_duration() {
        let mut app = setup_app();
        app.add_systems(Update, super::stun_system);

        let entity = app.world_mut().spawn(make_stun(0.1)).id();

        tick(&mut app, 0.016);
        tick(&mut app, 0.016);

        assert!(
            app.world().get::<Stun>(entity).is_some(),
            "Stun should still be present mid-duration"
        );
    }

    #[test]
    fn test_stun_removed_after_accumulated_ticks_exceed_duration() {
        let mut app = setup_app();
        app.add_systems(Update, super::stun_system);

        let entity = app.world_mut().spawn(make_stun(0.1)).id();

        tick(&mut app, 0.016);
        tick(&mut app, 0.016);
        tick(&mut app, 0.1);

        assert!(
            app.world().get::<Stun>(entity).is_none(),
            "Stun should be removed after accumulated time exceeds duration"
        );
    }
}
