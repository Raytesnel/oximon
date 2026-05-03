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
