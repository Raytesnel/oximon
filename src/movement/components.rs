use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component, Default, Debug)]
pub struct Velocity(pub Vec3);
