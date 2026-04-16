use crate::common::components::{ModifierTrigger, StatModifier, StatType};
use bevy::prelude::*;

#[derive(Clone)]
pub enum AttackSpawn {
    Hitbox { size: Vec2, color: Color },
    // later:
    // Sprite { texture: Handle<Image> },
    // Animation { atlas: Handle<TextureAtlas>, ... }
}
impl AttackSpawn {
    pub fn build_sprite(&self) -> Sprite {
        match self {
            AttackSpawn::Hitbox { size, color } => Sprite {
                color: *color,
                custom_size: Some(*size),
                ..default()
            },
        }
    }
}

#[derive(Component, Clone)]
pub struct AttackDefinition {
    pub damage: f32,
    pub range: f32,
    pub lifetime: f32,
    pub hit_interval: f32,
    pub cooldown: f32,
    pub stat_modifiers: Vec<StatModifier>,
    pub spawn: AttackSpawn,
    pub offset: Vec3,
}

pub fn quick_attack() -> AttackDefinition {
    AttackDefinition {
        damage: 10.0,
        range: 30.0,
        lifetime: 2.0,
        hit_interval: 2.1,
        cooldown: 3.0,
        offset: Vec3::ZERO,
        spawn: AttackSpawn::Hitbox {
            color: Color::srgb(1.0, 0.0, 0.0),
            size: Vec2::new(10.0, 10.0),
        },
        stat_modifiers: vec![
            StatModifier {
                stat_type: StatType::Speed,
                flat: 0.0,
                multiplier: 2.0,
                duration: Some(2.0),
                trigger: ModifierTrigger::Cast,
            },
            StatModifier {
                stat_type: StatType::Acceleration,
                flat: 0.0,
                multiplier: 10.0,
                duration: Some(2.0),
                trigger: ModifierTrigger::Cast,
            },
        ],
    }
}
pub fn simple_beam() -> AttackDefinition {
    AttackDefinition {
        damage: 10.0,
        range: 100.0,
        lifetime: 0.1,
        hit_interval: 0.05,
        cooldown: 0.6,
        stat_modifiers: vec![],
        offset: Vec3::ZERO,
        spawn: AttackSpawn::Hitbox {
            color: Color::srgb(1.0, 0.0, 0.0),
            size: Vec2::new(100.0, 10.0),
        },
    }
}
