use bevy::prelude::*;
use crate::common::components::*;

pub fn compute_stats_system(
    mut query: Query<(&Stats, &mut ComputedStats)>,
) {
    for (stats, mut computed) in &mut query {
        computed.speed = stats.speed();
        computed.acceleration = stats.acceleration();
        computed.friction = stats.friction();
        computed.dash_speed = stats.dash_speed();
        computed.dash_time = stats.dash_time();
        computed.dash_friction = stats.dash_friction();
        computed.dash_stop_time = stats.dash_stop_time();
    }
}
