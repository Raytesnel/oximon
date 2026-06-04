use crate::MainCamera;
use crate::movement::input::*;
use crate::overworld::components::{Facing, OverworldEntity, OverworldPlayer, YSort};
use avian2d::prelude::*;
use bevy::prelude::*;

pub fn spawn_player_overworld(commands: &mut Commands) {
    commands.spawn((
        OverworldPlayer,
        OverworldEntity,
        Sprite {
            color: Color::srgb(1.0, 0., 1.0),
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },
        YSort,
        Facing::Down,
        Transform::from_xyz(0., 0., 10.),
        RigidBody::Dynamic,
        Collider::rectangle(18.0, 18.0),
        LockedAxes::ROTATION_LOCKED,
        LinearDamping(8.0),
        Friction::new(0.0),
        Restitution::new(0.0),
        GravityScale(0.0),
        CollisionEventsEnabled,
    ));
}

pub fn y_sort(mut query: Query<&mut Transform, With<YSort>>) {
    for mut transform in &mut query {
        transform.translation.z = -transform.translation.y / 1000.0;
    }
}

pub fn camera_follow(
    mut cam_q: Query<&mut Transform, With<MainCamera>>,
    player_q: Query<&Transform, (With<OverworldPlayer>, Without<MainCamera>)>,
) {
    let player_transform = player_q.single().expect("Expected exactly one player");
    let mut cam_transform = cam_q.single_mut().expect("Expected exactly one camera");

    let deadzone = Vec2::new(200.0, 120.0);

    let delta = player_transform.translation - cam_transform.translation;
    let lerp_factor = 0.005;

    if delta.x.abs() > deadzone.x {
        cam_transform.translation.x = cam_transform
            .translation
            .x
            .lerp(player_transform.translation.x, lerp_factor);
    }

    if delta.y.abs() > deadzone.y {
        cam_transform.translation.y = cam_transform
            .translation
            .y
            .lerp(player_transform.translation.y, lerp_factor);
    }
}

pub fn update_facing(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Facing, With<OverworldPlayer>>,
) {
    for mut facing in &mut query {
        if keyboard.pressed(MOVE_UP_BUTTON) {
            *facing = Facing::Up;
        } else if keyboard.pressed(MOVE_DOWN_BUTTON) {
            *facing = Facing::Down;
        } else if keyboard.pressed(MOVE_LEFT_BUTTON) {
            *facing = Facing::Left;
        } else if keyboard.pressed(MOVE_RIGHT_BUTTON) {
            *facing = Facing::Right;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::movement::input::{MOVE_LEFT_BUTTON, MOVE_UP_BUTTON};
    use crate::overworld::components::Facing;
    use crate::overworld::components::YSort;
    use crate::overworld::player_movement::update_facing;
    use crate::overworld::player_movement::y_sort;
    use bevy::prelude::*;
    // ── helpers ──────────────────────────────────────────────────────────────

    fn make_app() -> App {
        let mut app = App::new();
        // MinimalPlugins gives us time + scheduling without a window.
        app.add_plugins(MinimalPlugins);
        app
    }

    #[test]
    fn ysort_higher_y_gets_lower_z() {
        let mut app = make_app();
        app.add_systems(Update, y_sort);

        let high = app
            .world_mut()
            .spawn((YSort, Transform::from_xyz(0.0, 200.0, 0.0)))
            .id();

        let low = app
            .world_mut()
            .spawn((YSort, Transform::from_xyz(0.0, 50.0, 0.0)))
            .id();

        app.update();

        let high_z = app.world().get::<Transform>(high).unwrap().translation.z;
        let low_z = app.world().get::<Transform>(low).unwrap().translation.z;

        assert!(
            high_z < low_z,
            "entity at y=200 should have z={high_z} < z={low_z} (entity at y=50)"
        );
    }

    #[test]
    fn ysort_z_formula_correct() {
        let mut app = make_app();
        app.add_systems(Update, y_sort);

        let entity = app
            .world_mut()
            .spawn((YSort, Transform::from_xyz(0.0, 300.0, 0.0)))
            .id();

        app.update();

        let z = app.world().get::<Transform>(entity).unwrap().translation.z;

        assert!(
            (z - (-300.0_f32 / 1000.0)).abs() < f32::EPSILON,
            "expected z = -0.3, got {z}"
        );
    }

    #[test]
    fn ysort_does_not_change_xy() {
        let mut app = make_app();
        app.add_systems(Update, y_sort);

        let entity = app
            .world_mut()
            .spawn((YSort, Transform::from_xyz(42.0, 99.0, 0.0)))
            .id();

        app.update();

        let tf = app.world().get::<Transform>(entity).unwrap().translation;
        assert_eq!(tf.x, 42.0);
        assert_eq!(tf.y, 99.0);
    }

    fn app_with_facing() -> (App, Entity) {
        let mut app = make_app();
        app.add_plugins(bevy::input::InputPlugin); // gives us ButtonInput<KeyCode>
        app.add_systems(Update, update_facing);

        let player = app
            .world_mut()
            .spawn((crate::overworld::components::OverworldPlayer, Facing::Down))
            .id();

        (app, player)
    }

    #[test]
    fn facing_updates_to_up_on_up_key() {
        let (mut app, player) = app_with_facing();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(MOVE_UP_BUTTON);

        app.update();

        assert_eq!(*app.world().get::<Facing>(player).unwrap(), Facing::Up);
    }

    #[test]
    fn facing_updates_to_left_on_left_key() {
        let (mut app, player) = app_with_facing();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(MOVE_LEFT_BUTTON);

        app.update();

        assert_eq!(*app.world().get::<Facing>(player).unwrap(), Facing::Left);
    }

    #[test]
    fn facing_unchanged_when_no_key_pressed() {
        let (mut app, player) = app_with_facing();
        // default is Facing::Down, no keys pressed
        app.update();

        assert_eq!(*app.world().get::<Facing>(player).unwrap(), Facing::Down);
    }
}
