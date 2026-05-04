pub mod ai;
mod attack_definition;
mod attacks;
pub mod components;
pub mod events;
mod status;
pub mod systems;
mod tests;

use crate::combat::ai::*;
use crate::combat::components::AttackEvent;
use crate::combat::events::DamageEvent;
use crate::combat::status::StatusEffectsPlugin;
use bevy::prelude::*;
use systems::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DamageEvent>();
        app.add_message::<AttackEvent>();
        app.add_plugins(StatusEffectsPlugin);
        app.add_systems(
            Update,
            (
                apply_damage_system,
                despawn_dead_system,
                tick_hitstun,
                apply_knockback_system,
            ),
        );
        app.add_systems(
            Update,
            (
                attack_input_system,
                attack_start_system,
                attack_lifetime_system,
                attack_hit_system,
                attack_follow_system,
                cooldown_tick_system,
            )
                .run_if(not_in_hitstop),
        );
        app.add_systems(FixedUpdate, (ai_decision_system, ai_attack_system));
    }
}
