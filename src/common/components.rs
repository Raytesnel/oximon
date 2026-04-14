use bevy::prelude::Component;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Stats {
    pub max_speed: f32,
    pub acceleration: f32,
    pub friction: f32,
    pub dash_speed: f32,
    pub dash_time: f32,
    pub dash_friction: f32,
    pub dash_stop_time: f32,
}
