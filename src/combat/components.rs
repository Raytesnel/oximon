use crate::combat::attack_definition::AttackDefinition;
use bevy::prelude::*;
use std::collections::HashMap;
#[derive(Component)]

pub struct CombatEntity;

#[derive(Component, Debug)]
pub struct Health {
    pub current: f32,
    pub _max: f32,
}

#[derive(Component, Debug)]
pub struct AttackStats {
    pub _attack: f32,
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
    pub id: AttackId,
    pub owner: Entity,
    pub definition: AttackDefinition,
    pub lifetime_timer: Timer,
    pub hit_timer: Timer,
    pub follow_entity: Option<Entity>,
    pub applied_start_modifiers: bool,
    pub hits_done: u32,
    pub has_hit: bool,
}
impl Attack {
    pub fn from_definition(def: AttackDefinition, owner: Entity, id: AttackId) -> Self {
        Self {
            id,
            owner,
            lifetime_timer: Timer::from_seconds(def.lifetime, TimerMode::Once),
            hit_timer: Timer::from_seconds(def.hit_interval, TimerMode::Repeating),
            definition: def,
            hits_done: 0,
            has_hit: false,
            follow_entity: Some(owner),
            applied_start_modifiers: false,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AttackId(pub u32);

#[derive(Resource, Default)]
pub struct AttackIdCounter {
    pub next: u32,
}

#[derive(Message, Debug)]
pub struct AttackEvent {
    pub _entity: Entity,
}

#[derive(Component, Debug)]
pub struct Hitstun {
    pub remaining: f32,
}

#[derive(Resource)]
pub struct Hitstop {
    pub remaining: f32,
}
#[derive(Component)]
pub struct KnockbackEffect {
    pub velocity: Vec3,
    pub timer: Timer,
}

#[derive(Component)]
pub struct Hitbox {
    pub size: Vec2,
}
#[derive(Component)]
pub struct Hurtbox {
    pub size: Vec2,
}

#[derive(Component)]
pub struct Poison {
    pub damage: f32,
    pub tick_timer: Timer,
    pub duration: Timer,
}

#[derive(Component)]
pub struct Slow {
    pub multiplier: f32,
    pub duration: Timer,
    pub applied: bool,
}

#[derive(Component)]
pub struct Stun {
    pub duration: Timer,
}
