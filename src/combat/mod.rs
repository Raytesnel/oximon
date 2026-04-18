mod attacks;
pub mod components;
pub mod events;
pub mod systems;
mod tests;

use crate::combat::components::AttackEvent;
use crate::combat::events::DamageEvent;
use bevy::prelude::*;
use systems::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DamageEvent>();
        app.add_message::<AttackEvent>();
        app.add_systems(
            Update,
            (
                attack_input_system,
                quick_attack_input_system,
                spawn_attack_system,
                attack_start_system,
                attack_lifetime_system,
                attack_hit_system,
                apply_damage_system,
                despawn_dead_system,
                attack_follow_system,
                cooldown_tick_system,
                tick_hitstun,
            ),
        );
    }
}
