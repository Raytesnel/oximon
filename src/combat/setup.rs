use crate::combat::ai::{AI, AIConfig, AIIntent, AIState, Target};
use crate::combat::components::{
    AttackId, AttackStats, CombatEntity, CombatSceneEntity, CombatState, Cooldowns, Health, Hurtbox,
};
use crate::common::components::{
    CombatSpawnContext, ComputedStats, Enemy, GameLayer, ModifierLifetime, Player, RuntimeModifier,
    StatType, Stats,
};
use crate::movement::components::{Facing, Movable, MoveIntent, MovementState};
use avian2d::prelude::*;
use bevy::color::Color;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;
use std::collections::HashMap;

pub fn setup_combat_players(mut commands: Commands, spawn_ctx: Res<CombatSpawnContext>) {
    let origin = spawn_ctx.player_world_pos;
    let player_entity = spawn_combat_player(&mut commands, origin);
    spawn_enemy(
        &mut commands,
        player_entity,
        origin + Vec3::new(150.0, 0.0, 0.0),
    );
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

    pub movement_state: MovementState,
    pub facing: Facing,

    pub health: Health,
    pub attack: AttackStats,
    pub combat: CombatState,
    pub move_intent: MoveIntent,
    pub hurtbox: Hurtbox,
    pub rigidbody: RigidBody,
    pub collider: Collider,
    pub loackaxes: LockedAxes,
    pub gravityscale: GravityScale,
    pub colission_layer: CollisionLayers,
}

fn spawn_combat_player(commands: &mut Commands, pos: Vec3) -> Entity {
    commands
        .spawn((
            Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..default()
            },
            CombatEntity,
            Transform::from_translation(pos),
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
                    source: AttackId(0),
                    flat: 250.0,
                    multiplier: 1.0,
                    timer: None,
                    stat_type: StatType::Speed,
                    lifetime: ModifierLifetime::Permanent,
                }],
                acceleration: vec![RuntimeModifier {
                    source: AttackId(0),
                    flat: 1250.0,
                    multiplier: 1.0,
                    timer: None,
                    lifetime: ModifierLifetime::Permanent,
                    stat_type: StatType::Acceleration,
                }],
                friction: vec![RuntimeModifier {
                    source: AttackId(0),
                    flat: 625.0,
                    multiplier: 1.0,
                    timer: None,
                    lifetime: ModifierLifetime::Permanent,
                    stat_type: StatType::Friction,
                }],
                dash_speed: vec![RuntimeModifier {
                    source: AttackId(0),
                    flat: 600.0,
                    multiplier: 1.0,
                    timer: None,
                    lifetime: ModifierLifetime::Permanent,
                    stat_type: StatType::DashSpeed,
                }],
                dash_time: vec![RuntimeModifier {
                    source: AttackId(0),
                    flat: 0.01,
                    multiplier: 1.0,
                    timer: None,
                    lifetime: ModifierLifetime::Permanent,
                    stat_type: StatType::DashTime,
                }],
                dash_friction: vec![RuntimeModifier {
                    source: AttackId(0),
                    flat: 500.0,
                    multiplier: 1.0,
                    timer: None,
                    lifetime: ModifierLifetime::Permanent,
                    stat_type: StatType::DashFriction,
                }],
                dash_stop_time: vec![RuntimeModifier {
                    source: AttackId(0),
                    flat: 0.01,
                    multiplier: 1.0,
                    timer: None,
                    lifetime: ModifierLifetime::Permanent,
                    stat_type: StatType::DashStopTime,
                }],
            },
            Facing(Vec2::X),
            MovementState::Idle,
            Health {
                current: 100.0,
                _max: 100.0,
            },
        ))
        .insert((
            Facing(Vec2::X),
            CollisionLayers::new(GameLayer::Combat, [GameLayer::Combat]),
            RigidBody::Dynamic,
            Collider::rectangle(18.0, 18.0),
            LockedAxes::ROTATION_LOCKED,
            LinearDamping(0.0),
            GravityScale(0.0),
            LinearVelocity::default(),
            AttackStats { _attack: 25.0 },
            CombatState::Idle,
            MoveIntent {
                direction: Vec3::ZERO,
            },
        ))
        .id()
}

pub fn spawn_enemy(commands: &mut Commands, target: Entity, pos: Vec3) {
    commands.spawn((
        CombatEntity,
        EnemyBundle {
            colission_layer: CollisionLayers::new(GameLayer::Combat, [GameLayer::Combat]),
            transform: Transform::from_translation(pos),
            enemy: Enemy,
            movable: Movable,

            stats: Stats {
                speed: vec![RuntimeModifier {
                    source: AttackId(0),
                    flat: 250.0,
                    multiplier: 1.0,
                    timer: None,
                    lifetime: ModifierLifetime::Permanent,
                    stat_type: StatType::Speed,
                }],
                acceleration: vec![RuntimeModifier {
                    source: AttackId(0),
                    flat: 1250.0,
                    multiplier: 1.0,
                    timer: None,
                    lifetime: ModifierLifetime::Permanent,
                    stat_type: StatType::Acceleration,
                }],
                friction: vec![RuntimeModifier {
                    source: AttackId(0),
                    flat: 625.0,
                    multiplier: 1.0,
                    timer: None,
                    lifetime: ModifierLifetime::Permanent,
                    stat_type: StatType::Friction,
                }],
                dash_speed: vec![RuntimeModifier {
                    source: AttackId(0),
                    flat: 600.0,
                    multiplier: 1.0,
                    timer: None,
                    lifetime: ModifierLifetime::Permanent,
                    stat_type: StatType::DashSpeed,
                }],
                dash_time: vec![RuntimeModifier {
                    source: AttackId(0),
                    flat: 0.01,
                    multiplier: 1.0,
                    timer: None,
                    lifetime: ModifierLifetime::Permanent,
                    stat_type: StatType::DashTime,
                }],
                dash_friction: vec![RuntimeModifier {
                    source: AttackId(0),
                    flat: 500.0,
                    multiplier: 1.0,
                    timer: None,
                    lifetime: ModifierLifetime::Permanent,
                    stat_type: StatType::DashFriction,
                }],
                dash_stop_time: vec![RuntimeModifier {
                    source: AttackId(0),
                    flat: 0.01,
                    multiplier: 1.0,
                    timer: None,
                    lifetime: ModifierLifetime::Permanent,
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
            hurtbox: Hurtbox {
                size: Vec2::new(20.0, 20.0),
            },
            rigidbody: RigidBody::Dynamic,
            collider: Collider::rectangle(18.0, 18.0),
            loackaxes: LockedAxes::ROTATION_LOCKED,
            gravityscale: GravityScale(0.0),
            ai: AI {
                state: AIState::Wander,
                timer: 0.0,
            },
            ai_config: AIConfig {
                vision_range: 250.0,
                attack_range: 40.0,
            },
            ai_intent: AIIntent {
                wants_attack: false,
            },
            target: Target { entity: target },

            movement_state: MovementState::Idle,
            facing: Facing(Vec2::X),

            health: Health {
                current: 100.0,
                _max: 100.0,
            },
            attack: AttackStats { _attack: 25.0 },
            combat: CombatState::Idle,
            move_intent: MoveIntent {
                direction: Vec3::ZERO,
            },
        },
    ));
}
pub fn hide_combat(mut commands: Commands, query: Query<Entity, With<CombatSceneEntity>>) {
    for e in &query {
        commands.entity(e).insert(Visibility::Hidden);
    }
}

pub fn show_combat(mut commands: Commands, query: Query<Entity, With<CombatSceneEntity>>) {
    for e in &query {
        commands.entity(e).insert(Visibility::Visible);
    }
}
pub fn setup_combat_world(mut commands: Commands, asset_server: Option<Res<AssetServer>>) {
    let Some(asset_server) = asset_server else {
        return; // skip in tests
    };
    let map_handle: Handle<TiledMapAsset> = asset_server.load("map/2d_main.tmx");
    commands
        .spawn((
            TiledMap(map_handle),
            CombatSceneEntity,
            TiledPhysicsSettings::<TiledPhysicsAvianBackend>::default(),
        ))
        .observe(
            |collider_created: On<TiledEvent<ColliderCreated>>, mut commands: Commands| {
                commands.entity(collider_created.event().origin).insert((
                    RigidBody::Static,
                    CollisionLayers::new(GameLayer::Combat, [GameLayer::Combat]),
                ));
            },
        );
}
