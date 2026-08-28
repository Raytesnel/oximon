use crate::combat::attack_definition::{
    AttackDefinition, AttackEffect, DamageEffect, EffectTrigger, KnockbackEffectDef,
    ModifierTarget, ProjectileDef, StatModifierEffect, StatusEffect, TimedEffect,
};
use crate::common::components::{ModifierLifetime, StatModifier, StatType};
use bevy::prelude::*;

#[allow(dead_code)]
#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub enum KnockbackMode {
    _Additive, // adds to velocity (smooth, keeps momentum)
    Override,  // replaces velocity (sharp hits)
    Impulse,   // instant burst (like smash knockback)
}
#[derive(Clone, Debug)]
pub enum KnockbackDirection {
    SourceToTarget,  // classic hit
    _TargetToSource, // recoil pull
    Fixed(Vec3),     // e.g. always upward
}
#[derive(Clone, Debug)]
pub enum HitBehavior {
    Single,        // stop after first hit (Quick Attack)
    MultiHit,      // keep hitting (beam / fire)
    _Limited(u32), // e.g. triple kick (3 hits max)
}

pub fn fireball() -> AttackDefinition {
    let residue = AttackDefinition {
        name: "fire_patch".to_string(),
        cooldown: 0.01,
        collision: false,
        follow_caster: false,
        projectile: None,
        residue: None,
        effects: vec![
            TimedEffect {
                trigger: EffectTrigger::OnHit,
                effect: AttackEffect::Damage(DamageEffect {
                    amount: 10.0,
                    target: ModifierTarget::TargetEntity,
                }),
            },
            TimedEffect {
                trigger: EffectTrigger::OnHit,
                effect: AttackEffect::Knockback(KnockbackEffectDef {
                    force: 10.0,
                    direction: KnockbackDirection::SourceToTarget,
                    _mode: KnockbackMode::Impulse,
                    hitstun: 0.01,
                    target: ModifierTarget::TargetEntity,
                }),
            },
            TimedEffect {
                trigger: EffectTrigger::OnHit,
                effect: AttackEffect::Damage(DamageEffect {
                    amount: 5.0,
                    target: ModifierTarget::SelfEntity,
                }),
            },
            TimedEffect {
                trigger: EffectTrigger::OnHit,
                effect: AttackEffect::Knockback(KnockbackEffectDef {
                    force: 10.0,
                    direction: KnockbackDirection::SourceToTarget,
                    _mode: KnockbackMode::Override,
                    hitstun: 1.0,
                    target: ModifierTarget::SelfEntity,
                }),
            },
        ],
        lifetime: 20.0,
        hit_interval: 0.5,
        hit_behavior: HitBehavior::MultiHit,
        offset: Vec3::ZERO,
        spawn: AttackSpawn::Hitbox {
            color: Color::srgba(1.0, 0.5, 0.0, 0.2),
            size: Vec2::new(60.0, 60.0),
        },
    };

    AttackDefinition {
        name: "fireball".to_string(),
        follow_caster: false,
        collision: false,
        projectile: Some(ProjectileDef { speed: 500.0 }),
        residue: Some(Box::new(residue)),
        effects: vec![
            TimedEffect {
                trigger: EffectTrigger::OnHit,
                effect: AttackEffect::Damage(DamageEffect {
                    amount: 10.0,
                    target: ModifierTarget::TargetEntity,
                }),
            },
            TimedEffect {
                trigger: EffectTrigger::OnHit,
                effect: AttackEffect::Knockback(KnockbackEffectDef {
                    force: 800.0,
                    direction: KnockbackDirection::SourceToTarget,
                    _mode: KnockbackMode::Override,
                    hitstun: 1.0,
                    target: ModifierTarget::TargetEntity,
                }),
            },
        ],

        lifetime: 2.0,
        hit_interval: 0.1,
        cooldown: 0.1,

        hit_behavior: HitBehavior::Single,
        offset: Vec3::ZERO,

        spawn: AttackSpawn::Hitbox {
            color: Color::srgb(1.0, 0.5, 0.5),
            size: Vec2::new(20.0, 20.0),
        },
    }
}

pub fn stone_block() -> AttackDefinition {
    let residue = AttackDefinition {
        name: "stone_residue".to_string(),
        follow_caster: false,
        projectile: None,
        collision: true,
        residue: None,
        effects: vec![],
        lifetime: 999.0,
        hit_interval: 0.1,
        cooldown: 1.0,
        hit_behavior: HitBehavior::MultiHit,
        offset: Vec3::ZERO,
        spawn: AttackSpawn::Hitbox {
            color: Color::srgb(0.5, 0.5, 0.5),
            size: Vec2::new(20.0, 20.0),
        },
    };

    AttackDefinition {
        name: "stone_block".to_string(),
        follow_caster: false,
        collision: false,
        projectile: Some(ProjectileDef { speed: 500.0 }),
        residue: Some(Box::new(residue)),
        effects: vec![/* knockback only */],
        lifetime: 10.0,
        hit_interval: 0.1,
        cooldown: 1.0,
        hit_behavior: HitBehavior::Single,
        offset: Vec3::ZERO,
        spawn: AttackSpawn::Hitbox {
            color: Color::srgb(0.6, 0.6, 0.6),
            size: Vec2::new(20.0, 20.0),
        },
    }
}

pub fn quick_attack() -> AttackDefinition {
    AttackDefinition {
        name: "quick_attack".to_string(),
        projectile: None,
        follow_caster: true,
        residue: None,
        collision: false,
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
                    _mode: KnockbackMode::Override,
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
                    _mode: KnockbackMode::Override,
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
                    },
                }),
            },
        ],

        lifetime: 2.0,
        hit_interval: 0.1,
        cooldown: 3.0,

        hit_behavior: HitBehavior::Single,
        offset: Vec3::ZERO,

        spawn: AttackSpawn::Hitbox {
            color: Color::srgb(1.0, 0.0, 0.0),
            size: Vec2::new(20.0, 20.0),
        },
    }
}
#[allow(dead_code)]
pub fn simple_beam() -> AttackDefinition {
    AttackDefinition {
        name: "simple_beam".to_string(),
        follow_caster: true,
        projectile: None,
        residue: None,
        collision: false,
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
                    _mode: KnockbackMode::Impulse,
                    hitstun: 0.05,
                    target: ModifierTarget::TargetEntity,
                }),
            },
        ],

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
        follow_caster: true,
        residue: None,
        projectile: None,
        collision: false,
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
                    },
                }),
            },
        ],

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
#[allow(dead_code)]
pub fn slow_down() -> AttackDefinition {
    AttackDefinition {
        name: "speedo".to_string(),
        follow_caster: true,
        projectile: None,
        residue: None,
        lifetime: 2.0,
        hit_interval: 2.0,
        cooldown: 3.0,
        collision: false,
        hit_behavior: HitBehavior::Single,
        offset: Vec3::ZERO,
        spawn: AttackSpawn::Hitbox {
            color: Color::srgb(1.0, 0.0, 0.0),
            size: Vec2::new(100.0, 10.0),
        },
        effects: vec![
            TimedEffect {
                trigger: EffectTrigger::OnHit,
                effect: AttackEffect::StatModifier(StatModifierEffect {
                    target: ModifierTarget::TargetEntity,
                    modifier: StatModifier {
                        stat_type: StatType::Speed,
                        flat: 0.0,
                        multiplier: 0.9,
                        duration: Some(2.0),
                        lifetime: ModifierLifetime::Duration,
                    },
                }),
            },
            TimedEffect {
                trigger: EffectTrigger::OnHit,
                effect: AttackEffect::ApplyStatus(StatusEffect::Poison {
                    dps: 5.0,
                    tick_rate: 2.0,
                    duration: 20.0,
                }),
            },
        ],
    }
}
pub fn shoot_square() -> AttackDefinition {
    AttackDefinition {
        name: "shoot_square".to_string(),
        projectile: Some(ProjectileDef { speed: 500.0 }),
        follow_caster: false,
        residue: None,
        collision: false,
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
                    _mode: KnockbackMode::Override,
                    hitstun: 1.0,
                    target: ModifierTarget::TargetEntity,
                }),
            },
        ],

        lifetime: 2.0,
        hit_interval: 0.1,
        cooldown: 0.1,

        hit_behavior: HitBehavior::Single,
        offset: Vec3::ZERO,

        spawn: AttackSpawn::Hitbox {
            color: Color::srgb(1.0, 0.0, 0.0),
            size: Vec2::new(20.0, 20.0),
        },
    }
}
