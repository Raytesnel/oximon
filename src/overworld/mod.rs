use crate::GameState;
use crate::overworld::interactables::*;
use crate::overworld::player_movement::*;
use bevy::prelude::*;
use input_systems::{interaction_input_system, overworld_movement};
use interactables::{on_sign_interaction, tick_sign_popups};
use setup::*;

mod components;
mod input_systems;
mod interactables;
mod player_movement;
mod setup;
#[cfg(test)]
mod test;

pub struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Overworld), (setup_overworld,))
            .add_systems(
                Update,
                (
                    camera_follow,
                    y_sort,
                    tick_sign_popups,
                    tick_lamp_animation,
                    tick_block_sliding,
                    update_facing,
                    interaction_input_system,
                    debug_ysort,
                )
                    .run_if(in_state(GameState::Overworld)),
            )
            .add_systems(OnExit(GameState::Overworld), cleanup_overworld)
            .add_systems(
                FixedUpdate,
                overworld_movement.run_if(in_state(GameState::Overworld)),
            )
            .add_observer(on_sign_interaction)
            .add_observer(on_block_interaction)
            .add_observer(on_monster_interaction)
            .add_observer(on_lamp_interaction);
    }
}
