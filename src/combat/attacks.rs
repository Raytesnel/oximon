use crate::combat::attack_definition::{
    AttackDefinition, AttackEffect, DamageEffect, EffectTrigger, KnockbackEffectDef,
    ModifierTarget, StatModifierEffect, TimedEffect,
};
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
    Single,       // stop after first hit (Quick Attack)
    MultiHit,     // keep hitting (beam / fire)
    Limited(u32), // e.g. triple kick (3 hits max)
}

pub fn quick_attack() -> AttackDefinition {
    AttackDefinition {
        name: "quick_attack".to_string(),

        effects: vec![
            // DAMAGE
            TimedEffect {
                trigger: EffectTrigger::OnHit,
                effect: AttackEffect::Damage(DamageEffect {
                    amount: 10.0,
                    target: ModifierTarget::TargetEntity,
                }),
            },
            // KNOCKBACK TARGET
            TimedEffect {
                trigger: EffectTrigger::OnHit,
                effect: AttackEffect::Knockback(KnockbackEffectDef {
                    force: 3000.0,
                    direction: KnockbackDirection::SourceToTarget,
                    mode: KnockbackMode::Override,
                    hitstun: 1.0,
                    target: ModifierTarget::TargetEntity,
                }),
            },
            // KNOCKBACK SELF
            TimedEffect {
                trigger: EffectTrigger::OnHit,
                effect: AttackEffect::Knockback(KnockbackEffectDef {
                    force: 1.0,
                    direction: KnockbackDirection::Fixed(Vec3::ZERO),
                    mode: KnockbackMode::Override,
                    hitstun: 0.5,
                    target: ModifierTarget::SelfEntity,
                }),
            },
            // BUFFS (ON CAST!)
            TimedEffect {
                trigger: EffectTrigger::OnCast,
                effect: AttackEffect::StatModifier(StatModifierEffect {
                    target: ModifierTarget::SelfEntity,
                    modifier: StatModifier {
                        stat_type: StatType::Speed,
                        flat: 0.0,
                        multiplier: 3.0,
                        duration: Some(2.0),
                        lifetime: ModifierLifetime::OnAttackEnd,
                        trigger: ModifierTrigger::Cast, // mag later weg
                    },
                }),
            },
            TimedEffect {
                trigger: EffectTrigger::OnCast,
                effect: AttackEffect::StatModifier(StatModifierEffect {
                    target: ModifierTarget::SelfEntity,
                    modifier: StatModifier {
                        stat_type: StatType::Acceleration,
                        flat: 0.0,
                        multiplier: 15.0,
                        duration: Some(2.0),
                        lifetime: ModifierLifetime::OnAttackEnd,
                        trigger: ModifierTrigger::Cast,
                    },
                }),
            },
        ],

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
    }
}
pub fn simple_beam() -> AttackDefinition {
    AttackDefinition {
        name: "simple_beam".to_string(),

        effects: vec![
            TimedEffect {
                trigger: EffectTrigger::OnHit,
                effect: AttackEffect::Damage(DamageEffect {
                    amount: 1.0,
                    target: ModifierTarget::TargetEntity,
                }),
            },
            TimedEffect {
                trigger: EffectTrigger::OnHit,
                effect: AttackEffect::Knockback(KnockbackEffectDef {
                    force: 300.0,
                    direction: KnockbackDirection::SourceToTarget,
                    mode: KnockbackMode::Impulse,
                    hitstun: 0.05,
                    target: ModifierTarget::TargetEntity,
                }),
            },
        ],

        range: 100.0,
        lifetime: 2.0,
        hit_interval: 0.05,
        cooldown: 0.6,

        hit_behavior: HitBehavior::MultiHit,

        offset: Vec3::ZERO,

        spawn: AttackSpawn::Hitbox {
            color: Color::srgb(1.0, 0.0, 0.0),
            size: Vec2::new(100.0, 10.0),
        },
    }
}
pub fn speedo() -> AttackDefinition {
    AttackDefinition {
        name: "speedo".to_string(),

        effects: vec![
            TimedEffect {
                trigger: EffectTrigger::OnCast,
                effect: AttackEffect::StatModifier(StatModifierEffect {
                    target: ModifierTarget::SelfEntity,
                    modifier: StatModifier {
                        stat_type: StatType::Speed,
                        flat: 0.0,
                        multiplier: 2.0,
                        duration: Some(20.0),
                        lifetime: ModifierLifetime::Duration,
                        trigger: ModifierTrigger::Cast,
                    },
                }),
            },
            TimedEffect {
                trigger: EffectTrigger::OnCast,
                effect: AttackEffect::StatModifier(StatModifierEffect {
                    target: ModifierTarget::SelfEntity,
                    modifier: StatModifier {
                        stat_type: StatType::Acceleration,
                        flat: 0.0,
                        multiplier: 2.0,
                        duration: Some(20.0),
                        lifetime: ModifierLifetime::Duration,
                        trigger: ModifierTrigger::Cast,
                    },
                }),
            },
        ],

        // 👇 deze worden bijna irrelevant
        range: 0.0,
        lifetime: 0.1, // kan zelfs laag
        hit_interval: 0.0,
        cooldown: 3.0,

        hit_behavior: HitBehavior::Single, // maakt niet meer uit

        offset: Vec3::ZERO,

        spawn: AttackSpawn::Hitbox {
            color: Color::srgb(0.0, 1.0, 0.0),
            size: Vec2::new(10.0, 10.0),
        },
    }
}
