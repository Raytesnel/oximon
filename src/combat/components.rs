use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Debug)]
pub struct Stats {
    pub attack: f32,
}

#[derive(Component, Debug, PartialEq, Eq)]
pub enum CombatState {
    Idle,
    Attacking,
    Dead,
}

#[derive(Component)]
pub struct Attack {
    pub damage: f32,
    pub range: f32,
    pub lifetime: Timer,
}

#[derive(Message, Debug)]
pub struct AttackEvent {
    pub entity: Entity,
}