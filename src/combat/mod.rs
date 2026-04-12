pub mod components;
pub mod events;
pub mod systems;
mod tests;

use bevy::prelude::*;
use systems::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<events::DamageEvent>().add_systems(
            Update,
            (
                player_attack_input_system,
                apply_damage_system.after(player_attack_input_system),
            ),
        );
    }
}
