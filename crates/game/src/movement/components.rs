use bevy::prelude::*;

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
    Recovering,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Facing(pub Vec2);

#[derive(Component, Debug)]
pub struct Movable;

#[derive(Component, Debug, Default)]
pub struct MoveIntent {
    pub direction: Vec3,
}
