use crate::combat::attacks::AttackDefinition;
use crate::common::components::StatModifier;
use bevy::prelude::*;
use std::collections::HashMap;

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
pub struct Cooldowns {
    pub timers: HashMap<String, Timer>,
}
impl Default for Cooldowns {
    fn default() -> Self {
        Self {
            timers: HashMap::new(),
        }
    }
}

#[derive(Component)]
pub struct Attack {
    pub owner: Entity,
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
            owner,
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

#[derive(Component, Debug)]
pub struct Hitstun {
    pub remaining: f32,
}