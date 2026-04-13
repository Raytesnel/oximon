use bevy::prelude::*;

pub mod components;
pub mod systems;

use systems::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        // INPUT (frame-based)
        app.add_systems(Update, handle_dash_input);

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
            ),
        );
    }
}
