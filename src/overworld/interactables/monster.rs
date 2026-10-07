use crate::common::components::{BattleState, CombatSpawnContext};
use crate::overworld::components::*;
use avian2d::prelude::*;
use bevy::prelude::*;

type PlayerQueryItem = (&'static Facing, &'static Transform);
type ObstacleQueryItem = &'static Transform;
type ObstacleFilter = (With<RigidBody>, With<PushableBlock>);

pub fn on_monster_interaction(
    trigger: On<InteractionEvent>,
    monster_q: Query<&InteractionType>,
    player_q: Query<&Transform, With<OverworldPlayer>>,
    mut next_battle_state: ResMut<NextState<BattleState>>,
    mut commands: Commands,
) {
    let entity = trigger.event().entity;
    let Ok(InteractionType::Monster) = monster_q.get(entity) else {
        return;
    };

    if let Ok(player_tf) = player_q.single() {
        commands.insert_resource(CombatSpawnContext {
            player_world_pos: player_tf.translation,
        });
    }

    next_battle_state.set(BattleState::Entering);
}
