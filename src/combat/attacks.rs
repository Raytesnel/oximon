use crate::common::components::{ModifierLifetime, ModifierTrigger, StatModifier, StatType};
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

#[derive(Clone)]
pub enum KnockbackMode {
    Additive, // adds to velocity (smooth, keeps momentum)
    Override, // replaces velocity (sharp hits)
    Impulse,  // instant burst (like smash knockback)
}
#[derive(Clone)]
pub enum KnockbackDirection {
    SourceToTarget, // classic hit
    TargetToSource, // recoil pull
    Fixed(Vec3),    // e.g. always upward
}
#[derive(Clone)]
pub struct KnockbackDefinition {
    pub force: f32,
    pub direction: KnockbackDirection,
    pub mode: KnockbackMode,
    pub hitstun: f32,
}
#[derive(Clone)]
pub enum HitBehavior {
    Debuff,       // Attack only changes stats of enemy
    Buff,         // no Attack only stat modifiers
    Single,       // stop after first hit (Quick Attack)
    MultiHit,     // keep hitting (beam / fire)
    Limited(u32), // e.g. triple kick (3 hits max)
}

#[derive(Component, Clone)]
pub struct AttackDefinition {
    pub name: String,
    pub damage: f32,
    pub range: f32,
    pub lifetime: f32,
    pub hit_interval: f32,
    pub cooldown: f32,
    pub stat_modifiers: Vec<StatModifier>,
    pub spawn: AttackSpawn,
    pub offset: Vec3,
    pub hit_behavior: HitBehavior,

    pub knockback_target: Option<KnockbackDefinition>,
    pub knockback_self: Option<KnockbackDefinition>,
}

pub fn quick_attack() -> AttackDefinition {
    AttackDefinition {
        name: "quick_attack".to_string(),
        damage: 10.0,
        range: 30.0,
        lifetime: 2.0,
        hit_interval: 0.1,
        cooldown: 3.0,
        hit_behavior: HitBehavior::Single,
        offset: Vec3::ZERO,
        spawn: AttackSpawn::Hitbox {
            color: Color::srgb(1.0, 0.0, 0.0),
            size: Vec2::new(10.0, 10.0),
        },
        knockback_target: Some(KnockbackDefinition {
            force: 3000.0,
            direction: KnockbackDirection::SourceToTarget,
            mode: KnockbackMode::Override,
            hitstun: 2.0,
        }),
        knockback_self: Some(KnockbackDefinition {
            force: 1.0,
            direction: KnockbackDirection::Fixed(Vec3::ZERO),
            mode: KnockbackMode::Override, // stops player
            hitstun: 2.0,
        }),
        stat_modifiers: vec![
            StatModifier {
                stat_type: StatType::Speed,
                flat: 0.0,
                multiplier: 3.0,
                duration: Some(2.0),
                trigger: ModifierTrigger::Cast,
                lifetime: ModifierLifetime::OnAttackEnd,
            },
            StatModifier {
                stat_type: StatType::Acceleration,
                flat: 0.0,
                multiplier: 15.0,
                duration: Some(2.0),
                trigger: ModifierTrigger::Cast,
                lifetime: ModifierLifetime::OnAttackEnd,
            },
        ],
    }
}
pub fn simple_beam() -> AttackDefinition {
    AttackDefinition {
        name: "simple_beam".to_string(),
        damage: 1.0,
        range: 100.0,
        lifetime: 2.0,
        hit_interval: 0.05,
        cooldown: 0.6,
        hit_behavior: HitBehavior::MultiHit,
        stat_modifiers: vec![StatModifier {
            stat_type: StatType::Speed,
            flat: 0.0,
            multiplier: 0.1,
            duration: Some(2.0),
            trigger: ModifierTrigger::Cast,
            lifetime: ModifierLifetime::OnAttackEnd,
        }],
        offset: Vec3::ZERO,
        spawn: AttackSpawn::Hitbox {
            color: Color::srgb(1.0, 0.0, 0.0),
            size: Vec2::new(100.0, 10.0),
        },
        knockback_target: Some(KnockbackDefinition {
            force: 300.0,
            direction: KnockbackDirection::SourceToTarget,
            mode: KnockbackMode::Impulse,
            hitstun: 0.05,
        }),
        knockback_self: None,
    }
}
pub fn speedo() -> AttackDefinition {
    AttackDefinition {
        name: "speedo".to_string(),
        damage: 0.0,
        range: 0.0,
        lifetime: 3.0,
        hit_interval: 0.00,
        cooldown: 3.0,
        hit_behavior: HitBehavior::Buff,
        stat_modifiers: vec![
            StatModifier {
                stat_type: StatType::Speed,
                flat: 0.0,
                multiplier: 2.0,
                duration: Some(20.0),
                trigger: ModifierTrigger::Cast,
                lifetime: ModifierLifetime::Duration,
            },
            StatModifier {
                stat_type: StatType::Acceleration,
                flat: 0.0,
                multiplier: 2.0,
                duration: Some(20.0),
                trigger: ModifierTrigger::Cast,
                lifetime: ModifierLifetime::Duration,
            },
        ],
        offset: Vec3::ZERO,
        spawn: AttackSpawn::Hitbox {
            color: Color::srgb(0.0, 1.0, 0.0),
            size: Vec2::new(10.0, 10.0),
        },
        knockback_target: None,
        knockback_self: None,
    }
}

pub fn slow_attack() -> AttackDefinition {
    AttackDefinition {
        name: "simple_beam".to_string(),
        damage: 0.0,
        range: 100.0,
        lifetime: 2.0,
        hit_interval: 0.05,
        cooldown: 0.6,
        hit_behavior: HitBehavior::MultiHit,
        stat_modifiers: vec![StatModifier {
            stat_type: StatType::Speed,
            flat: 0.0,
            multiplier: 0.1,
            duration: Some(2.0),
            trigger: ModifierTrigger::Cast,
            lifetime: ModifierLifetime::OnAttackEnd,
        }],
        offset: Vec3::ZERO,
        spawn: AttackSpawn::Hitbox {
            color: Color::srgb(1.0, 0.0, 0.0),
            size: Vec2::new(100.0, 10.0),
        },
        knockback_target: Some(KnockbackDefinition {
            force: 300.0,
            direction: KnockbackDirection::SourceToTarget,
            mode: KnockbackMode::Impulse,
            hitstun: 0.05,
        }),
        knockback_self: None,
    }
}