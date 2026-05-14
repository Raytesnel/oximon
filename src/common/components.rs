use crate::combat::components::AttackId;
use bevy::prelude::*;

#[derive(Resource)]
pub struct CombatSpawnContext {
    pub player_world_pos: Vec3,
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Enemy;

#[derive(Clone, Copy, Debug)]
pub enum StatType {
    Speed,
    Acceleration,
    Friction,
    DashSpeed,
    DashTime,
    DashFriction,
    DashStopTime,
}

#[derive(Component)]
pub struct StatsModifiers {
    pub speed: Vec<StatModifier>,
    pub acceleration: Vec<StatModifier>,
    pub friction: Vec<StatModifier>,
    pub dash_speed: Vec<StatModifier>,
    pub dash_time: Vec<StatModifier>,
    pub dash_friction: Vec<StatModifier>,
    pub dash_stop_time: Vec<StatModifier>,
}

#[derive(Component, Debug)]
pub struct Stats {
    pub speed: Vec<RuntimeModifier>,
    pub acceleration: Vec<RuntimeModifier>,
    pub friction: Vec<RuntimeModifier>,
    pub dash_speed: Vec<RuntimeModifier>,
    pub dash_time: Vec<RuntimeModifier>,
    pub dash_friction: Vec<RuntimeModifier>,
    pub dash_stop_time: Vec<RuntimeModifier>,
}
impl Default for Stats {
    fn default() -> Self {
        Self {
            speed: vec![],
            acceleration: vec![],
            friction: vec![],
            dash_speed: vec![],
            dash_time: vec![],
            dash_friction: vec![],
            dash_stop_time: vec![],
        }
    }
}

#[derive(Component, Default, Debug, Clone)]
pub struct ComputedStats {
    pub speed: f32,
    pub acceleration: f32,
    pub friction: f32,
    pub dash_speed: f32,
    pub dash_time: f32,
    pub dash_friction: f32,
    pub dash_stop_time: f32,
}
#[derive(Clone, Copy)]
pub enum ModifierTrigger {
    Cast,
    OnHit,
    After,
}
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ModifierLifetime {
    Duration, // ignores all, always waiting till defined lifetime.
    WhileAttacking,
    OnAttackEnd,
    Permanent,
}
#[derive(Clone)]
pub struct StatModifier {
    pub stat_type: StatType,
    pub flat: f32,
    pub multiplier: f32,
    pub duration: Option<f32>,
    pub trigger: ModifierTrigger,
    pub lifetime: ModifierLifetime,
}
impl StatModifier {
    pub fn to_runtime(&self, source: AttackId) -> RuntimeModifier {
        RuntimeModifier {
            source,
            stat_type: self.stat_type,
            flat: self.flat,
            multiplier: self.multiplier,
            lifetime: self.lifetime.clone(),
            timer: self
                .duration
                .map(|d| Timer::from_seconds(d, TimerMode::Once)),
        }
    }
}
#[derive(Clone, Debug)]
pub struct RuntimeModifier {
    pub source: AttackId,
    pub stat_type: StatType,
    pub flat: f32,
    pub multiplier: f32,
    pub timer: Option<Timer>,
    pub lifetime: ModifierLifetime,
}
impl Stats {
    fn compute(modifiers: &Vec<RuntimeModifier>) -> f32 {
        let mut base = 0.0;

        for m in modifiers {
            base += m.flat;
            base *= m.multiplier;
        }
        base
    }
}

impl Stats {
    pub fn add_modifier(&mut self, modifier: RuntimeModifier) {
        match modifier.stat_type {
            StatType::Speed => self.speed.push(modifier),
            StatType::Acceleration => self.acceleration.push(modifier),
            StatType::Friction => self.friction.push(modifier),
            StatType::DashSpeed => self.dash_speed.push(modifier),
            StatType::DashTime => self.dash_time.push(modifier),
            StatType::DashFriction => self.dash_friction.push(modifier),
            StatType::DashStopTime => self.dash_stop_time.push(modifier),
        }
    }
}

impl Stats {
    pub fn speed(&self) -> f32 {
        Self::compute(&self.speed)
    }
    pub fn acceleration(&self) -> f32 {
        Self::compute(&self.acceleration)
    }
    pub fn friction(&self) -> f32 {
        Self::compute(&self.friction)
    }
    pub fn dash_speed(&self) -> f32 {
        Self::compute(&self.dash_speed)
    }
    pub fn dash_time(&self) -> f32 {
        Self::compute(&self.dash_time)
    }
    pub fn dash_friction(&self) -> f32 {
        Self::compute(&self.dash_friction)
    }
    pub fn dash_stop_time(&self) -> f32 {
        Self::compute(&self.dash_stop_time)
    }
}
