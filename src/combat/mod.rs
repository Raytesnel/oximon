pub mod ai;
mod attack_definition;
mod attacks;
pub mod components;
pub mod events;
mod setup;
mod status;
pub mod systems;
#[cfg(test)]
mod tests;

use crate::GameState;
use crate::combat::ai::*;
use crate::combat::components::{AttackEvent, AttackIdCounter};
use crate::combat::events::DamageEvent;
use crate::combat::setup::{hide_combat, setup_combat_players, setup_combat_world, show_combat};
use crate::combat::status::StatusEffectsPlugin;
use bevy::prelude::*;
use systems::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Combat),
            (setup_combat_players, show_combat),
        );
        app.add_systems(
            Startup,
            (setup_combat_world, hide_combat.after(setup_combat_world)),
        );
        app.add_message::<DamageEvent>();
        app.add_message::<AttackEvent>();
        app.add_plugins(StatusEffectsPlugin);
        app.add_systems(
            Update,
            (
                debug_collisions,
                apply_damage_system,
                despawn_dead_system,
                tick_hitstun,
                apply_knockback_system,
            )
                .run_if(in_state(GameState::Combat)),
        );
        app.add_systems(
            Update,
            (
                attack_input_system,
                attack_start_system,
                attack_lifetime_system,
                attack_hit_system,
                projectile_movement_system,
                attack_follow_system,
                cooldown_tick_system,
            )
                .run_if(in_state(GameState::Combat).and(not_in_hitstop)),
        );
        app.add_systems(FixedUpdate, (ai_decision_system, ai_attack_system))
            .insert_resource(AttackIdCounter::default());
        app.add_systems(OnExit(GameState::Combat), (hide_combat, cleanup_combat));
    }
}
