use bevy::prelude::*;

pub mod components;
pub mod input;
pub mod systems;
pub mod types;

mod tests;

use crate::combat::systems::not_in_hitstop;
use systems::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        // INPUT (frame-based)
        app.add_systems(Update, (handle_dash_input, player_input_system));

        // SIMULATION (fixed)
        app.add_systems(
            FixedUpdate,
            (
                update_dash_timer,
                update_recover,
                update_movement_state,
                update_facing,
                apply_acceleration.after(update_facing),
                apply_friction.after(apply_acceleration),
                apply_velocity.after(apply_acceleration),
            )
                .run_if(not_in_hitstop),
        );
    }
}
