use bevy::prelude::*;

#[derive(Component, Default, Debug)]
pub struct Velocity(pub Vec3);
#[derive(Component, Debug)]
pub struct Dash {
    pub direction: Vec3,
    pub timer: Timer,
}
#[derive(Component, Debug)]
pub struct Recover {
    pub timer: Timer,
}

#[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
pub enum MovementState {
    Idle,
    Moving,
    Dashing,
    Recovering,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Facing(pub Vec2);
