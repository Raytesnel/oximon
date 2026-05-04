use bevy::prelude::*;
use crate::GameState;
use crate::overworld::overworld::*;

mod overworld;

pub struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Overworld), setup_overworld)
            .add_systems(
                Update,
                overworld_input.run_if(in_state(GameState::Overworld)),
            )
            .add_systems(OnExit(GameState::Overworld), cleanup_overworld);
    }
}