use crate::combat::components::Hitstop;
use crate::common::systems::*;
use bevy::app::{App, Plugin, Update};

pub mod components;
mod systems;
pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (compute_stats_system, update_stat_timers, tick_hitstop),
        )
        .insert_resource(Hitstop { remaining: 0.0 });
    }
}
