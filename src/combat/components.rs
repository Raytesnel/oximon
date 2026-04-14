use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Debug)]
pub struct AttackStats {
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
    pub hit_timer: Timer, // how many seconds per hit
    pub follow_entity: Option<Entity>,
    pub active: bool, //
}

#[derive(Message, Debug)]
pub struct AttackEvent {
    pub entity: Entity,
}
