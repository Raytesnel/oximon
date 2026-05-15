use avian2d::prelude::*;
use bevy::prelude::*;

use crate::combat::components::*;
use crate::movement::components::*;

pub type AllowedMovable = (With<Movable>, Without<Hitstun>);
pub type NoneOverWorldMovable = (With<Movable>, Without<RigidBody>);
