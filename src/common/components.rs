use bevy::prelude::*;
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Enemy;

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

#[derive(Component)]
pub struct Stats {
    pub speed: Vec<StatModifier>,
    pub acceleration: Vec<StatModifier>,
    pub friction: Vec<StatModifier>,
    pub dash_speed: Vec<StatModifier>,
    pub dash_time: Vec<StatModifier>,
    pub dash_friction: Vec<StatModifier>,
    pub dash_stop_time: Vec<StatModifier>,
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

#[derive(Clone)]
pub struct StatModifier {
    pub flat: f32,
    pub multiplier: f32,
    pub timer: Option<Timer>,
}
impl Stats {
    fn compute(modifiers: &Vec<StatModifier>) -> f32 {
        let mut base = 0.0;

        for m in modifiers {
            base  += m.flat;
            base *=m.multiplier;
        }
        base
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