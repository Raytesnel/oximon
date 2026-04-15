use bevy::app::{App, Plugin, Update};
use crate::common::systems::compute_stats_system;

pub mod components;
mod systems;
pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                compute_stats_system
            ),
        );
    }
}