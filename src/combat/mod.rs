pub mod ai;
mod attack_definition;
mod attacks;
pub mod components;
pub mod events;
mod status;
pub mod systems;
mod tests;
mod setup;

use crate::combat::ai::*;
use crate::combat::components::{AttackEvent, AttackIdCounter};
use crate::combat::events::DamageEvent;
use crate::combat::status::StatusEffectsPlugin;
use bevy::prelude::*;
use systems::*;
use crate::GameState;
use crate::combat::setup::setup_combat_players;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Combat), setup_combat_players);
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
            ).run_if(in_state(GameState::Combat)),
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
                .run_if(in_state(GameState::Combat).and(not_in_hitstop))
        );
        app.add_systems(FixedUpdate, (ai_decision_system, ai_attack_system))
            .insert_resource(AttackIdCounter::default());
        app.add_systems(OnExit(GameState::Combat), cleanup_combat);
    }
}
