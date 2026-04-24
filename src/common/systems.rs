use crate::combat::components::Hitstop;
use crate::common::components::*;
use bevy::prelude::*;

pub fn compute_stats_system(mut query: Query<(&Stats, &mut ComputedStats)>) {
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

pub fn update_stat_timers(time: Res<Time>, mut query: Query<&mut Stats>) {
    for mut stats in &mut query {
        update_list(&mut stats.speed, &time);
        update_list(&mut stats.acceleration, &time);
        update_list(&mut stats.friction, &time);
        update_list(&mut stats.dash_speed, &time);
        update_list(&mut stats.dash_time, &time);
        update_list(&mut stats.dash_friction, &time);
        update_list(&mut stats.dash_stop_time, &time);
    }
}

pub fn update_list(list: &mut Vec<RuntimeModifier>, time: &Time) {
    list.retain_mut(|modifier| {
        if let Some(timer) = &mut modifier.timer {
            timer.tick(time.delta());
            !timer.is_finished()
        } else {
            true
        }
    });
}

pub fn tick_hitstop(mut hitstop: ResMut<Hitstop>, time: Res<Time>) {
    if hitstop.remaining > 0.0 {
        hitstop.remaining -= time.delta_secs();
    }
}
