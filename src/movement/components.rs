use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component, Default, Debug)]
pub struct Velocity(pub Vec3);
#[derive(Component)]
pub struct Dash {
    pub direction: Vec3,
    pub timer: Timer,
}
