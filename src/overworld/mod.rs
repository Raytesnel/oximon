use crate::GameState;
use crate::overworld::overworld::*;
use bevy::prelude::*;
use input_systems::{interaction_input_system, overworld_movement};
use interactables::{on_sign_interaction, tick_sign_popups};
use setup::{cleanup_overworld, setup_overworld};
use crate::overworld::interactables::*;

mod overworld;
mod interactables;
mod setup;
mod components;
mod input_systems;

pub struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Overworld), (setup_overworld,load_lamp_spritesheet))
            .add_systems(
                Update,
                (
                    camera_follow,
                    y_sort,
                    tick_sign_popups,
                    tick_lamp_animation
                )
                    .run_if(in_state(GameState::Overworld)),
            )
            .add_systems(
                PostUpdate,
                (
                    interaction_input_system,
                )
                    .run_if(in_state(GameState::Overworld)),
            )
            .add_systems(OnExit(GameState::Overworld), cleanup_overworld)
            .add_systems(
                FixedUpdate,
                overworld_movement.run_if(in_state(GameState::Overworld)),
            )
            // .add_observer(on_interaction)
            .add_observer(on_sign_interaction)
            .add_observer(on_lamp_interaction);
    }
}
