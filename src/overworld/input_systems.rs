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

pub const INTERACTION_KEY: KeyCode = KeyCode::KeyE;
pub fn interaction_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_q: Query<Entity, With<OverworldPlayer>>,
    fields: Query<(&InteractionField, &CollidingEntities), With<InteractionFieldMarker>>,
    mut commands: Commands,
) {
    if !keyboard.just_pressed(INTERACTION_KEY) {
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

#[cfg(test)]
mod tests {
    use crate::movement::input::{
        MOVE_DOWN_BUTTON, MOVE_LEFT_BUTTON, MOVE_RIGHT_BUTTON, MOVE_UP_BUTTON,
    };
    use crate::overworld::components::{
        InteractionEvent, InteractionField, InteractionFieldMarker, OverworldPlayer,
    };
    use crate::overworld::input_systems::{
        INTERACTION_KEY, interaction_input_system, overworld_movement,
    };
    use avian2d::prelude::{CollidingEntities, LinearVelocity};
    use bevy::app::FixedMain;
    use bevy::input::InputPlugin;
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    fn tick(app: &mut App, dt: f32) {
        let delta = std::time::Duration::from_secs_f32(dt);
        // TimeUpdateStrategy makes app.update() advance the real Time by exactly
        // dt instead of using wall-clock time (which is ~0 in tests).
        app.insert_resource(TimeUpdateStrategy::ManualDuration(delta));
        {
            let mut time = app.world_mut().resource_mut::<Time<Fixed>>();
            time.advance_by(delta);
        }
        app.world_mut().run_schedule(FixedMain);
        app.update();
    }

    fn make_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, InputPlugin));
        app.update();
        app
    }

    fn app_with_movement() -> (App, Entity) {
        let mut app = make_app();
        app.add_systems(FixedUpdate, overworld_movement);

        let player = app
            .world_mut()
            .spawn((OverworldPlayer, LinearVelocity(Vec2::ZERO)))
            .id();
        app.update();
        (app, player)
    }

    #[test]
    fn movement_no_input_gives_zero_velocity() {
        let (mut app, player) = app_with_movement();
        tick(&mut app, 0.016);

        let vel = app.world().get::<LinearVelocity>(player).unwrap().0;
        assert_eq!(vel, Vec2::ZERO);
    }

    #[test]
    fn movement_up_key_gives_positive_y() {
        let (mut app, player) = app_with_movement();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(MOVE_UP_BUTTON);

        tick(&mut app, 0.016);

        let vel = app.world().get::<LinearVelocity>(player).unwrap().0;
        println!("velocity: {:?}", vel);
        assert!(vel.y > 0.0, "up key should produce positive y velocity");
        assert_eq!(vel.x, 0.0);
    }

    #[test]
    fn movement_down_key_gives_negative_y() {
        let (mut app, player) = app_with_movement();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(MOVE_DOWN_BUTTON);

        tick(&mut app, 0.016);

        let vel = app.world().get::<LinearVelocity>(player).unwrap().0;
        assert!(vel.y < 0.0);
    }

    #[test]
    fn movement_left_key_gives_negative_x() {
        let (mut app, player) = app_with_movement();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(MOVE_LEFT_BUTTON);

        tick(&mut app, 0.016);

        let vel = app.world().get::<LinearVelocity>(player).unwrap().0;
        assert!(vel.x < 0.0);
    }

    #[test]
    fn movement_right_key_gives_positive_x() {
        let (mut app, player) = app_with_movement();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(MOVE_RIGHT_BUTTON);

        tick(&mut app, 0.016);

        let vel = app.world().get::<LinearVelocity>(player).unwrap().0;
        assert!(vel.x > 0.0);
    }

    #[test]
    fn movement_diagonal_is_normalised() {
        let (mut app, player) = app_with_movement();

        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.press(MOVE_UP_BUTTON);
            keys.press(MOVE_RIGHT_BUTTON);
        }

        tick(&mut app, 0.016);

        let vel = app.world().get::<LinearVelocity>(player).unwrap().0;
        // Speed must equal the configured constant (150), not 150*√2.
        let speed = vel.length();
        assert!(
            (speed - 150.0).abs() < 1.0,
            "diagonal speed should be ~150, got {speed}"
        );
    }

    #[test]
    fn movement_speed_is_150() {
        let (mut app, player) = app_with_movement();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(MOVE_UP_BUTTON);

        tick(&mut app, 0.016);

        let vel = app.world().get::<LinearVelocity>(player).unwrap().0;
        assert!(
            (vel.length() - 150.0).abs() < f32::EPSILON,
            "speed should be 150.0"
        );
    }

    #[derive(Resource, Default)]
    struct InteractionLog(Vec<Entity>);

    fn app_with_interaction() -> App {
        let mut app = make_app();
        app.add_systems(Update, interaction_input_system);
        app.init_resource::<InteractionLog>();
        // Observer that writes to the log.
        app.add_observer(
            |trigger: On<InteractionEvent>, mut log: ResMut<InteractionLog>| {
                log.0.push(trigger.event().entity);
            },
        );
        app
    }

    fn spawn_interaction_setup(app: &mut App) -> (Entity, Entity) {
        let player = app
            .world_mut()
            .spawn((OverworldPlayer, Transform::default()))
            .id();

        let owner = app.world_mut().spawn_empty().id();

        // Bypass CollidingEntities entirely — write a test-only version of the
        // system that accepts a fake "inside field" condition, OR just test the
        // observer separately from the input system.
        //
        // For now: insert CollidingEntities and manually add the player to it.
        let mut colliding = CollidingEntities::default();
        // Check if this method exists on your avian2d version:
        colliding.0.insert(player); // direct field access if it's a newtype

        app.world_mut().spawn((
            InteractionFieldMarker,
            InteractionField { owner },
            colliding,
        ));

        (player, owner)
    }

    #[test]
    fn interaction_does_not_trigger_without_e_press() {
        let mut app = app_with_interaction();
        spawn_interaction_setup(&mut app);
        // No key press
        tick(&mut app, 0.016);

        let log = app.world().resource::<InteractionLog>();
        assert!(log.0.is_empty(), "no event expected without key press");
    }

    #[test]
    fn interaction_does_not_trigger_when_not_in_field() {
        let mut app = app_with_interaction();

        let player = app
            .world_mut()
            .spawn((OverworldPlayer, Transform::default()))
            .id();

        let owner = app.world_mut().spawn_empty().id();
        app.world_mut().spawn((
            InteractionFieldMarker,
            InteractionField { owner },
            CollidingEntities::default(), // empty — player not inside
        ));

        let _ = player;

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(INTERACTION_KEY);

        app.update();

        let log = app.world().resource::<InteractionLog>();
        assert!(
            log.0.is_empty(),
            "no event when player is not inside the interaction field"
        );
    }
}
