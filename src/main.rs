mod combat;
mod common;
pub mod movement;
mod overworld;

use crate::combat::CombatPlugin;
use crate::combat::ai::{AI, AIConfig, AIIntent, AIState, Target};
use crate::combat::components::{AttackId, AttackIdCounter, AttackStats, CombatEntity, CombatState, Cooldowns, Health, Hitstop, Hurtbox};
use crate::common::CommonPlugin;
use crate::common::components::{
    ComputedStats, ModifierLifetime, Player, RuntimeModifier, StatType, Stats,
};
use crate::movement::MovementPlugin;
use crate::movement::components::{Facing, Movable, MoveIntent};
use bevy::prelude::*;
use common::components::Enemy;
use movement::components::{MovementState, Velocity};
use std::collections::HashMap;
use bevy_ecs_tiled::prelude::*;
use crate::overworld::OverworldPlugin;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    Overworld,
    Combat,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes_override: Some(true),
            ..default()
        }))
        .init_state::<GameState>()
        .add_plugins(TiledPlugin::default())
        .add_plugins(CommonPlugin)        .add_plugins(MovementPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(OverworldPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

#[derive(Component)]
struct MainCamera;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        MainCamera,
        Transform::default(),
    ));
}
