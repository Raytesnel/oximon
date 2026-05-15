use crate::movement::input::{
    MOVE_DOWN_BUTTON, MOVE_LEFT_BUTTON, MOVE_RIGHT_BUTTON, MOVE_UP_BUTTON,
};
use crate::overworld::components::{
    InteractionEvent, InteractionField, InteractionFieldMarker, OverworldPlayer,
};
use avian2d::prelude::{CollidingEntities, LinearVelocity};
use bevy::input::ButtonInput;
use bevy::log::info;
use bevy::math::Vec2;
use bevy::prelude::{Commands, Entity, KeyCode, Query, Res, With};

pub fn interaction_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_q: Query<Entity, With<OverworldPlayer>>,
    fields: Query<(&InteractionField, &CollidingEntities), With<InteractionFieldMarker>>,
    mut commands: Commands,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok(player_entity) = player_q.single() else {
        return;
    };

    // Find any interaction field the player is currently inside
    for (field, colliding) in &fields {
        if colliding.contains(&player_entity) {
            info!("triggering interaction for owner {:?}", field.owner);
            commands.trigger(InteractionEvent {
                entity: field.owner,
            });
            break;
        }
    }
}

pub fn overworld_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut LinearVelocity, With<OverworldPlayer>>,
) {
    for mut lin_vel in &mut query {
        let mut dir = Vec2::ZERO;
        if keyboard.pressed(MOVE_UP_BUTTON) {
            dir.y += 1.0;
        }
        if keyboard.pressed(MOVE_DOWN_BUTTON) {
            dir.y -= 1.0;
        }
        if keyboard.pressed(MOVE_LEFT_BUTTON) {
            dir.x -= 1.0;
        }
        if keyboard.pressed(MOVE_RIGHT_BUTTON) {
            dir.x += 1.0;
        }

        lin_vel.0 = dir.normalize_or_zero() * 150.0;
    }
}
