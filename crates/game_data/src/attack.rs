use bevy_color::Color;
use bevy_math::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

mod color_rgba {
    use bevy_color::Color;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(c: &Color, s: S) -> Result<S::Ok, S::Error> {
        let c = c.to_srgba();
        [c.red, c.green, c.blue, c.alpha].serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Color, D::Error> {
        let [r, g, b, a] = <[f32; 4]>::deserialize(d)?;
        Ok(Color::srgba(r, g, b, a))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MonsterId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModifierTarget {
    SelfEntity,   // attacker
    TargetEntity, // enemy
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageEffect {
    pub amount: f32,
    pub target: ModifierTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnockbackMode {
    Additive, // adds to velocity (smooth, keeps momentum)
    Override, // replaces velocity (sharp hits)
    Impulse,  // instant burst (like smash knockback)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnockbackDirection {
    SourceToTarget, // classic hit
    TargetToSource, // recoil pull
    Fixed(Vec3),    // e.g. always upward
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnockbackEffectDef {
    pub force: f32,
    pub direction: KnockbackDirection,
    pub mode: KnockbackMode,
    pub hitstun: f32,
    pub target: ModifierTarget,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatType {
    Speed,
    Acceleration,
    Friction,
    DashSpeed,
    DashTime,
    DashFriction,
    DashStopTime,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModifierLifetime {
    Duration, // ignores all, always waiting till defined lifetime.
    WhileAttacking,
    OnAttackEnd,
    Permanent,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatModifier {
    pub stat_type: StatType,
    pub flat: f32,
    pub multiplier: f32,
    pub duration: Option<f32>,
    pub lifetime: ModifierLifetime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatModifierEffect {
    pub modifier: StatModifier,
    pub target: ModifierTarget,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatusEffect {
    Poison {
        dps: f32,
        duration: f32,
        tick_rate: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackEffect {
    Damage(DamageEffect),
    Knockback(KnockbackEffectDef),
    StatModifier(StatModifierEffect),
    ApplyStatus(StatusEffect),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectTrigger {
    OnCast,
    OnHit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimedEffect {
    pub trigger: EffectTrigger,
    pub effect: AttackEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackSpawn {
    Hitbox {
        size: Vec2,
        #[serde(with = "color_rgba")]
        color: Color,
    },
    // later:
    // Sprite { texture: Handle<Image> },
    // Animation { atlas: Handle<TextureAtlas>, ... }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HitBehavior {
    Single,       // stop after first hit (Quick Attack)
    MultiHit,     // keep hitting (beam / fire)
    Limited(u32), // e.g. triple kick (3 hits max)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectileDef {
    pub speed: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)] // serializes as a plain string: "fireball"
pub struct AttackId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackDefinition {
    pub name: String,
    pub effects: Vec<TimedEffect>,
    pub collision: bool,
    pub lifetime: f32,
    pub hit_interval: f32,
    pub cooldown: f32,
    pub residue_tile_key: Option<String>,
    pub spawn: AttackSpawn,
    pub residue: Option<AttackId>,
    pub offset: Vec3,
    pub hit_behavior: HitBehavior,
    pub follow_caster: bool,
    pub projectile: Option<ProjectileDef>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_attack_files() {
        for json in [
            include_str!("/home/raymond/Projects/oximon/assets/attacks/fireball.json"),
            include_str!("/home/raymond/Projects/oximon/assets/attacks/fire_patch.json"),
        ] {
            serde_json::from_str::<AttackDefinition>(json).unwrap();
        }
    }

    #[test]
    fn load_attacks() {}
}
