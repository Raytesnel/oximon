mod attacks;
pub mod components;
pub mod events;
pub mod systems;
pub mod ai;
mod tests;

use crate::combat::components::AttackEvent;
use crate::combat::ai::*;
use crate::combat::events::DamageEvent;
use bevy::prelude::*;
use systems::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DamageEvent>();
        app.add_message::<AttackEvent>();
        app.add_systems(Update,(
            apply_damage_system,
            despawn_dead_system,
            tick_hitstun,
        ));
        app.add_systems(
            Update,
            (
                attack_input_system,
                quick_attack_input_system,
                spawn_attack_system,
                attack_start_system,
                attack_lifetime_system,
                attack_hit_system,
                attack_follow_system,
                cooldown_tick_system,
            ).run_if(not_in_hitstop)
        );
        app.add_systems(Update, (
            ai_decision_system,
            ai_movement_system,
            ai_attack_system,
        ).chain());
    }
}
