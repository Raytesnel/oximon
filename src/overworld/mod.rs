use crate::GameState;
use crate::overworld::overworld::*;
use bevy::prelude::*;
mod overworld;

pub struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Overworld), setup_overworld)
            .add_systems(
                Update,
                (
                    camera_follow,
                    y_sort,
                    tick_sign_popups,
                )
                    .run_if(in_state(GameState::Overworld)),
            )
            .add_systems(
                PostUpdate,
                (
                    update_facing,
                    interaction_input_system.after(update_facing),
                )
                    .run_if(in_state(GameState::Overworld)),
            )
            .add_systems(OnExit(GameState::Overworld), cleanup_overworld)
            .add_systems(
                FixedUpdate,
                overworld_movement.run_if(in_state(GameState::Overworld)),
            )
            .add_observer(on_interaction)
            .add_observer(on_sign_interaction);
    }
}
