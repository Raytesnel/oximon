use bevy::prelude::*;

pub mod components;
pub mod systems;

use systems::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                apply_acceleration,
                apply_friction.after(apply_acceleration),
                apply_velocity.after(apply_friction),
            ),
        );
    }
}
