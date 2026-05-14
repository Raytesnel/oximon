mod combat;
mod common;
pub mod movement;
mod overworld;

use crate::combat::CombatPlugin;
use crate::common::CommonPlugin;
use crate::movement::MovementPlugin;
use crate::overworld::OverworldPlugin;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_tiled::prelude::*;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    Overworld,
    Combat,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "oximon".into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .init_state::<GameState>()
        .add_plugins(TiledPlugin::default())
        .add_plugins(TiledPhysicsPlugin::<TiledPhysicsAvianBackend>::default())
        .add_plugins(PhysicsPlugins::default().with_length_unit(32.0))
        .add_plugins(CommonPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(CombatPlugin)
        .add_plugins(OverworldPlugin)
        .add_systems(Startup, setup_camera)
        .run();
}

#[derive(Component)]
struct MainCamera;

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera, Msaa::Off, Transform::default()));
}
