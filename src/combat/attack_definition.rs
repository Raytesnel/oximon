use crate::combat::attacks::{AttackSpawn, HitBehavior, KnockbackDirection, KnockbackMode};
use crate::common::components::StatModifier;
use bevy::math::Vec3;
use bevy::prelude::Component;

#[derive(Clone, Copy, Debug)]
pub enum ModifierTarget {
    SelfEntity,   // attacker
    TargetEntity, // enemy
}

#[derive(Clone, Debug)]
pub struct DamageEffect {
    pub amount: f32,
    pub target: ModifierTarget,
}

#[derive(Clone, Debug)]
pub struct KnockbackEffectDef {
    pub force: f32,
    pub direction: KnockbackDirection,
    pub _mode: KnockbackMode,
    pub hitstun: f32,
    pub target: ModifierTarget,
}

#[derive(Clone, Debug)]
pub struct StatModifierEffect {
    pub modifier: StatModifier,
    pub target: ModifierTarget,
}
#[derive(Clone, Debug)]
pub enum EffectTrigger {
    OnCast,
    OnHit,
}
#[derive(Clone, Debug)]
pub struct TimedEffect {
    pub trigger: EffectTrigger,
    pub effect: AttackEffect,
}

#[derive(Clone, Debug)]
pub enum StatusEffect {
    Poison {
        dps: f32,
        duration: f32,
        tick_rate: f32,
    },
}

#[derive(Clone, Debug)]
pub enum AttackEffect {
    Damage(DamageEffect),
    Knockback(KnockbackEffectDef),
    StatModifier(StatModifierEffect),
    ApplyStatus(StatusEffect),
    // later:
    // Status(StatusEffectDef),
    // Protect(ProtectEffect),
}

#[derive(Component, Clone, Debug)]
pub struct AttackDefinition {
    pub name: String,
    pub effects: Vec<TimedEffect>,

    pub lifetime: f32,
    pub hit_interval: f32,
    pub cooldown: f32,

    pub spawn: AttackSpawn,
    pub offset: Vec3,
    pub hit_behavior: HitBehavior,
}
