use crate::combat::attacks::AttackDefinition;
use crate::common::components::StatModifier;
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
    pub definition: AttackDefinition,
    pub lifetime_timer: Timer,
    pub hit_timer: Timer,
    pub active: bool,
    pub follow_entity: Option<Entity>,
    pub applied_start_modifiers: bool,
}
impl Attack {
    pub fn from_definition(def: AttackDefinition, owner: Entity) -> Self {
        Self {
            lifetime_timer: Timer::from_seconds(def.lifetime, TimerMode::Once),
            hit_timer: Timer::from_seconds(def.hit_interval, TimerMode::Repeating),
            definition: def,
            active: false,
            follow_entity: Some(owner),
            applied_start_modifiers: false,
        }
    }
}

#[derive(Message, Debug)]
pub struct AttackEvent {
    pub entity: Entity,
}
