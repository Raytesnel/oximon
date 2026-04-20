mod combat;
mod common;
pub mod movement;

use crate::combat::CombatPlugin;
use crate::combat::ai::{AI, AIConfig, AIIntent, AIState, Target};
use crate::combat::components::{AttackStats, CombatState, Cooldowns, Health, Hitstop};
use crate::common::CommonPlugin;
use crate::common::components::{
    ComputedStats, ModifierTrigger, Player, RuntimeModifier, StatModifier, StatType, Stats,
};
use crate::movement::MovementPlugin;
use crate::movement::components::{Facing, Movable, MoveIntent};
use bevy::prelude::*;
use common::components::Enemy;
use movement::components::{MovementState, Velocity};
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MovementPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(CommonPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.insert_resource(Hitstop { remaining: 0.0 });
    // Player
    let player_entity = commands
        .spawn((
            Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..default()
            },
            Transform::from_xyz(0., 0., 0.),
            Player,
            Movable,
            Cooldowns {
                timers: HashMap::new(),
            },
            ComputedStats {
                speed: 250.0,
                acceleration: 1250.0,
                friction: 625.0,
                dash_speed: 600.0,
                dash_time: 0.01,
                dash_friction: 50.0,
                dash_stop_time: 0.2,
            },
            Stats {
                speed: vec![RuntimeModifier {
                    flat: 250.0,
                    multiplier: 1.0,
                    timer: None,
                    stat_type: StatType::Speed,
                }],
                acceleration: vec![RuntimeModifier {
                    flat: 1250.0,
                    multiplier: 1.0,
                    timer: None,
                    stat_type: StatType::Acceleration,
                }],
                friction: vec![RuntimeModifier {
                    flat: 625.0,
                    multiplier: 1.0,
                    timer: None,
                    stat_type: StatType::Friction,
                }],
                dash_speed: vec![RuntimeModifier {
                    flat: 600.0,
                    multiplier: 1.0,
                    timer: None,
                    stat_type: StatType::DashSpeed,
                }],
                dash_time: vec![RuntimeModifier {
                    flat: 0.01,
                    multiplier: 1.0,
                    timer: None,
                    stat_type: StatType::DashTime,
                }],
                dash_friction: vec![RuntimeModifier {
                    flat: 500.0,
                    multiplier: 1.0,
                    timer: None,
                    stat_type: StatType::DashFriction,
                }],
                dash_stop_time: vec![RuntimeModifier {
                    flat: 0.01,
                    multiplier: 1.0,
                    timer: None,
                    stat_type: StatType::DashStopTime,
                }],
            },
            Facing(Vec2::X),
            Velocity::default(),
            MovementState::Idle,
            Health {
                current: 100.0,
                max: 100.0,
            },
            AttackStats { attack: 25.0 },
            CombatState::Idle,
            MoveIntent {
                direction: Vec3::ZERO,
            },
        ))
        .id();

    spawn_enemy(&mut commands, player_entity, Vec3::new(100.0, 0.0, 0.0));
}

#[derive(Bundle)]
pub struct EnemyBundle {
    pub transform: Transform,
    pub enemy: Enemy,
    pub movable: Movable,

    pub stats: Stats,
    pub computed: ComputedStats,

    pub sprite: Sprite,

    pub ai: AI,
    pub ai_config: AIConfig,
    pub ai_intent: AIIntent,
    pub target: Target,

    pub velocity: Velocity,
    pub movement_state: MovementState,
    pub facing: Facing,

    pub health: Health,
    pub attack: AttackStats,
    pub combat: CombatState,
    pub move_intent: MoveIntent,
}

pub fn spawn_enemy(commands: &mut Commands, target: Entity, pos: Vec3) {
    commands.spawn(EnemyBundle {
        transform: Transform::from_translation(pos),
        enemy: Enemy,
        movable: Movable,

        stats: Stats {
            speed: vec![RuntimeModifier {
                flat: 250.0,
                multiplier: 1.0,
                timer: None,
                stat_type: StatType::Speed,
            }],
            acceleration: vec![RuntimeModifier {
                flat: 1250.0,
                multiplier: 1.0,
                timer: None,
                stat_type: StatType::Acceleration,
            }],
            friction: vec![RuntimeModifier {
                flat: 625.0,
                multiplier: 1.0,
                timer: None,
                stat_type: StatType::Friction,
            }],
            dash_speed: vec![RuntimeModifier {
                flat: 600.0,
                multiplier: 1.0,
                timer: None,
                stat_type: StatType::DashSpeed,
            }],
            dash_time: vec![RuntimeModifier {
                flat: 0.01,
                multiplier: 1.0,
                timer: None,
                stat_type: StatType::DashTime,
            }],
            dash_friction: vec![RuntimeModifier {
                flat: 500.0,
                multiplier: 1.0,
                timer: None,
                stat_type: StatType::DashFriction,
            }],
            dash_stop_time: vec![RuntimeModifier {
                flat: 0.01,
                multiplier: 1.0,
                timer: None,
                stat_type: StatType::DashStopTime,
            }],
        },
        computed: ComputedStats {
            speed: 100.0,
            acceleration: 1250.0,
            friction: 625.0,
            dash_speed: 600.0,
            dash_time: 0.01,
            dash_friction: 50.0,
            dash_stop_time: 0.2,
        },

        sprite: Sprite {
            color: Color::srgb(0., 0., 1.0),
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },

        ai: AI {
            state: AIState::Wander,
            timer: 0.0,
        },
        ai_config: AIConfig {
            vision_range: 250.0,
            attack_range: 40.0,
            wander_speed: 50.0,
            chase_speed: 120.0,
        },
        ai_intent: AIIntent {
            move_dir: Vec3::ZERO,
            wants_attack: false,
        },
        target: Target { entity: target },

        velocity: Velocity::default(),
        movement_state: MovementState::Idle,
        facing: Facing(Vec2::X),

        health: Health {
            current: 100.0,
            max: 100.0,
        },
        attack: AttackStats { attack: 25.0 },
        combat: CombatState::Idle,
        move_intent: MoveIntent {
            direction: Vec3::ZERO,
        },
    });
}
