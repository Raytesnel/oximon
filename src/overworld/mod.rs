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
                )
                    .run_if(in_state(GameState::Overworld)),
            )
            .add_systems(OnExit(GameState::Overworld), cleanup_overworld);
        app.add_systems(
            FixedUpdate,
            overworld_movement.run_if(in_state(GameState::Overworld)),
        );
    }
}
