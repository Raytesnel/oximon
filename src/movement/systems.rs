use super::components::*;
use crate::combat::components::Hitstun;
use crate::common::components::{ComputedStats, Player};
use crate::movement::input::{
    MOVE_DOWN_BUTTON, MOVE_LEFT_BUTTON, MOVE_RIGHT_BUTTON, MOVE_UP_BUTTON,
};
use crate::movement::types::*;
use bevy::prelude::*;

pub fn apply_friction(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &MovementState, &ComputedStats), With<Movable>>,
) {
    for (mut velocity, state, stats) in &mut query {
        let speed = velocity.value.length();

        let friction = match state {
            MovementState::Recovering => stats.dash_speed / stats.dash_stop_time,
            MovementState::Moving | MovementState::Idle => stats.friction,
        };

        if speed > 0.001 {
            let drop = friction * time.delta_secs();
            let new_speed = (speed - drop).max(0.0);

            velocity.value = velocity.value.normalize_or_zero() * new_speed;
        }
    }
}

pub fn apply_velocity(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity), NoneOverWorldMovable>,
) {
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.value * time.delta_secs();
    }
}

fn compute_direction(input: &ButtonInput<KeyCode>) -> Vec3 {
    let mut direction = Vec3::ZERO;

    if input.pressed(MOVE_UP_BUTTON) {
        direction.y += 1.0;
    }
    if input.pressed(MOVE_DOWN_BUTTON) {
        direction.y -= 1.0;
    }
    if input.pressed(MOVE_LEFT_BUTTON) {
        direction.x -= 1.0;
    }
    if input.pressed(MOVE_RIGHT_BUTTON) {
        direction.x += 1.0;
    }

    direction.normalize_or_zero()
}

pub fn apply_acceleration(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &MovementState, &ComputedStats, &MoveIntent), AllowedMovable>,
) {
    for (mut velocity, state, stats, move_intent) in &mut query {
        match state {
            MovementState::Recovering => {
                info!("we are still need to remove recovering");
            }

            MovementState::Moving | MovementState::Idle => {
                let acceleration = stats.acceleration;
                let speed = stats.speed;
                let input_dir = move_intent.direction;
                velocity.value += input_dir * acceleration * time.delta_secs();

                if velocity.value.length() > speed {
                    velocity.value = velocity.value.normalize() * speed;
                }
            }
        }
    }
}

pub fn update_movement_state(
    mut query: Query<(&Velocity, &mut MovementState, &MoveIntent), AllowedMovable>,
) {
    for (velocity, mut movement_state, move_intent) in &mut query {
        let input_dir = move_intent.direction;
        let speed = velocity.value.length();
        let new_state = if input_dir != Vec3::ZERO {
            MovementState::Moving
        } else if speed > 1.0 {
            // still sliding
            MovementState::Moving
        } else {
            MovementState::Idle
        };

        if *movement_state != new_state {
            debug!("State change: {:?} -> {:?}", *movement_state, new_state);
            *movement_state = new_state;
        }
    }
}

pub fn update_facing(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Facing, With<Player>>,
) {
    let input_dir = compute_direction(&keyboard).truncate();

    if input_dir.length() > 0.1 {
        let dir = input_dir.normalize();

        for mut facing in &mut query {
            facing.0 = dir;
        }
    }
}

pub fn player_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut MoveIntent, (With<Player>, Without<Hitstun>)>,
) {
    let mut direction = Vec3::ZERO;

    if keyboard.pressed(MOVE_UP_BUTTON) {
        direction.y += 1.0;
    }
    if keyboard.pressed(MOVE_DOWN_BUTTON) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(MOVE_LEFT_BUTTON) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(MOVE_RIGHT_BUTTON) {
        direction.x += 1.0;
    }

    for mut intent in &mut query {
        intent.direction = direction.normalize_or_zero();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::FixedMain;
    use paste::paste;

    fn tick(app: &mut App, dt: f32) {
        let delta = std::time::Duration::from_secs_f32(dt);

        // FIXED TIME (for FixedUpdate systems)
        {
            let mut time = app.world_mut().resource_mut::<Time<Fixed>>();
            time.advance_by(delta);
        }

        // REAL TIME (for normal systems)
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(delta);
        }
        app.world_mut().run_schedule(FixedMain);
        app.update();
    }

    macro_rules! test_facing {
        ($($name:ident: $keys:expr => $expected:expr,)*) => {
            $(
            paste! {
                #[test]
                fn [<test_input_$name _results_new_facing_direction>]() {
                    run_facing_test($keys, $expected);
                }
        }
            )*
        }
    }
    test_facing! {
        facing_up:         &[MOVE_UP_BUTTON]                           => Vec2::Y,
        facing_down:       &[MOVE_DOWN_BUTTON]                         => Vec2::NEG_Y,
        facing_left:       &[MOVE_LEFT_BUTTON]                         => Vec2::NEG_X,
        facing_right:      &[MOVE_RIGHT_BUTTON]                        => Vec2::X,
        facing_up_right:   &[MOVE_UP_BUTTON, MOVE_RIGHT_BUTTON]        => Vec2::new( 1.0,  1.0).normalize(),
        facing_up_left:    &[MOVE_UP_BUTTON, MOVE_LEFT_BUTTON]         => Vec2::new(-1.0,  1.0).normalize(),
        facing_down_right: &[MOVE_DOWN_BUTTON, MOVE_RIGHT_BUTTON]      => Vec2::new( 1.0, -1.0).normalize(),
        facing_down_left:  &[MOVE_DOWN_BUTTON, MOVE_LEFT_BUTTON]       => Vec2::new(-1.0, -1.0).normalize(),
    }
    fn run_facing_test(keys: &[KeyCode], expected: Vec2) {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, super::update_facing);
        app.world_mut().spawn((Player, Facing(Vec2::X)));

        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        for &key in keys {
            input.press(key);
        }

        app.update();

        let world = app.world_mut();
        let mut q = world.query::<&Facing>();
        let facing = q.single(world).unwrap();

        assert!(
            facing.0.distance(expected) < 0.01,
            "Facing was {:?}, expected up-left",
            facing.0
        );
    }

    macro_rules! test_input_system {
        ($($name:ident: $keys:expr => $expected:expr,)*) => {
            $(
            paste!{
                #[test]
                fn [<test_$name _will_result_in_movement_intent>]() {
                    run_test_user_input_give_move_intent($keys, $expected);
                }
        }
            )*
        }
    }
    test_input_system! {
        input_up:         &[MOVE_UP_BUTTON]                      => Vec3::Y,
        input_down:       &[MOVE_DOWN_BUTTON]                    => Vec3::NEG_Y,
        input_left:       &[MOVE_LEFT_BUTTON]                    => Vec3::NEG_X,
        input_right:      &[MOVE_RIGHT_BUTTON]                   => Vec3::X,
        input_up_right:   &[MOVE_UP_BUTTON, MOVE_RIGHT_BUTTON]   => Vec3::new( 1.0,  1.0, 0.0).normalize(),
        input_cancelled:  &[MOVE_LEFT_BUTTON, MOVE_RIGHT_BUTTON] => Vec3::ZERO,
    }
    fn run_test_user_input_give_move_intent(keys: &[KeyCode], expected: Vec3) {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, super::player_input_system);
        app.world_mut().spawn((
            Player,
            MoveIntent {
                direction: Vec3::ZERO,
            },
        ));

        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        for &key in keys {
            input.press(key);
        }

        app.update();

        let world = app.world_mut();
        let mut q = world.query::<&MoveIntent>();
        let intent = q.single(world).unwrap();

        assert!(
            intent.direction.distance(expected) < 0.01,
            "Keys {:?} → direction {:?}, expected {:?}",
            keys,
            intent.direction,
            expected
        );
    }

    #[test]
    fn test_player_with_hit_stun_stops_player_intent() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, super::player_input_system);
        app.world_mut().spawn((
            Player,
            Hitstun { remaining: 1.0 },
            MoveIntent {
                direction: Vec3::ZERO,
            },
        ));

        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(MOVE_RIGHT_BUTTON);

        app.update();

        let world = app.world_mut();
        let mut q = world.query::<&MoveIntent>();
        let intent = q.single(world).unwrap();
        let expected = Vec3::new(0.0, 0.0, 0.0);
        assert!(
            intent.direction.distance(expected) < 0.01,
            "Keys {:?} → direction {:?}, expected {:?}",
            MOVE_RIGHT_BUTTON,
            intent.direction,
            expected
        );
    }

    fn setup_acceleration_app() -> App {
        let mut app = App::new();
        app.init_resource::<Time>();
        app.insert_resource(Time::<Fixed>::from_seconds(0.016));
        app.init_resource::<ButtonInput<KeyCode>>();
        app
    }
    fn base_stats() -> ComputedStats {
        ComputedStats {
            speed: 10.0,
            acceleration: 10.0,
            friction: 10.0,
            dash_speed: 10.0,
            dash_time: 10.0,
            dash_friction: 10.0,
            dash_stop_time: 10.0,
        }
    }
    fn assert_acceleration(app: &mut App, expected_speed: f32) {
        let world = app.world_mut();
        let mut q = world.query::<&Velocity>();
        let velocity = q.single(world).unwrap();
        let expected_speed = Vec3::new(expected_speed, 0.0, 0.0);
        assert!(
            velocity.value.distance(expected_speed) < 0.01,
            "Velocity {:?}, expected {:?}",
            velocity.value,
            expected_speed
        );
    }
    #[test]
    fn test_movement_intent_with_high_acceleration_results_in_max_speed() {
        // setup
        let mut app = setup_acceleration_app();
        app.add_systems(Update, super::apply_acceleration);

        let max_speed = 2.0;
        app.world_mut().spawn((
            Player,
            MovementState::Idle,
            ComputedStats {
                speed: max_speed,
                acceleration: 200.0,
                friction: 10.0,
                dash_speed: 10.0,
                dash_time: 10.0,
                dash_friction: 10.0,
                dash_stop_time: 10.0,
            },
            Velocity::default(),
            Movable,
            MoveIntent {
                direction: Vec3::new(1.0, 0.0, 0.0),
            },
        ));
        app.update();
        // act
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(MOVE_LEFT_BUTTON);
        tick(&mut app, 0.016);
        // assert
        assert_acceleration(&mut app, max_speed);
    }
    #[test]
    fn test_movement_intent_with_low_acceleration_results_in_acceleration_increase() {
        // setup
        let mut app = setup_acceleration_app();
        app.add_systems(Update, super::apply_acceleration);

        let fixed_time = 0.016;
        let acceleration = 2.0;
        app.world_mut().spawn((
            Player,
            MovementState::Idle,
            ComputedStats {
                speed: 10.0,
                acceleration: acceleration,
                friction: 10.0,
                dash_speed: 10.0,
                dash_time: 10.0,
                dash_friction: 10.0,
                dash_stop_time: 10.0,
            },
            Velocity::default(),
            Movable,
            MoveIntent {
                direction: Vec3::new(1.0, 0.0, 0.0),
            },
        ));
        app.update();
        // act
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(MOVE_LEFT_BUTTON);
        tick(&mut app, fixed_time);
        // assert
        let expected = acceleration * fixed_time;
        assert_acceleration(&mut app, expected);
    }
    #[test]
    fn test_stunned_moveable_with_movement_intent_results_in_no_acceleration() {
        // setup
        let mut app = setup_acceleration_app();
        app.add_systems(Update, super::apply_acceleration);

        let fixed_time = 0.016;
        app.world_mut().spawn((
            Player,
            MovementState::Idle,
            base_stats(),
            Hitstun { remaining: 10.0 },
            Movable,
            Velocity::default(),
            MoveIntent {
                direction: Vec3::new(1.0, 0.0, 0.0),
            },
        ));
        app.update();
        // act
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(MOVE_LEFT_BUTTON);
        tick(&mut app, fixed_time);
        // assert
        assert_acceleration(&mut app, 0.0);
    }
    #[test]
    fn test_non_moveable_with_movement_intent_results_in_no_acceleration() {
        // setup
        let mut app = setup_acceleration_app();
        app.add_systems(Update, super::apply_acceleration);

        let fixed_time = 0.016;
        app.world_mut().spawn((
            Player,
            MovementState::Idle,
            base_stats(),
            Velocity::default(),
            MoveIntent {
                direction: Vec3::new(1.0, 0.0, 0.0),
            },
        ));
        app.update();
        // act
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(MOVE_LEFT_BUTTON);
        tick(&mut app, fixed_time);
        // assert
        assert_acceleration(&mut app, 0.0);
    }
    #[test]
    fn test_compute_direction_diagonal_normalized() {
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(MOVE_UP_BUTTON);
        input.press(MOVE_RIGHT_BUTTON);

        let dir = compute_direction(&input);

        assert!(dir.length() <= 1.0);
    }

    #[test]
    fn test_velocity_translates_to_translation() {
        let mut app = setup_acceleration_app();
        app.add_systems(Update, super::apply_velocity);
        let fixed_time = 0.016;
        let velocity = 10.0;
        app.world_mut().spawn((
            Player,
            Movable,
            Transform::default(),
            Velocity {
                value: Vec3::new(velocity, 0.0, 0.0),
            },
        ));
        tick(&mut app, fixed_time);

        let world = app.world_mut();
        let mut q = world.query::<&mut Transform>();
        let transform = q.single(world).unwrap();

        assert_eq!(transform.translation.x, (fixed_time * velocity));
    }

    #[test]
    fn test_velocity_keeps_growing_with_updates() {
        let mut app = setup_acceleration_app();
        app.add_systems(Update, super::apply_velocity);
        let fixed_time = 0.16;
        let velocity = 10.0;
        let total_update = 20;
        app.world_mut().spawn((
            Player,
            Movable,
            Transform::default(),
            Velocity {
                value: Vec3::new(velocity, 0.0, 0.0),
            },
        ));
        for _ in 0..total_update {
            tick(&mut app, fixed_time);
        }

        let world = app.world_mut();
        let mut q = world.query::<&mut Transform>();
        let transform = q.single(world).unwrap();

        assert!((transform.translation.x - (fixed_time * velocity) * total_update as f32) < 0.2);
    }
    #[test]
    fn test_apply_friction_reduces_velocity() {
        let mut app = setup_acceleration_app();
        app.add_systems(Update, super::apply_friction);

        let initial_speed = 10.0;
        let friction = 2.0;
        let dt = 0.016;

        app.world_mut().spawn((
            Player,
            Movable,
            MovementState::Idle,
            ComputedStats {
                speed: 10.0,
                acceleration: 10.0,
                friction,
                dash_speed: 10.0,
                dash_time: 10.0,
                dash_friction: 10.0,
                dash_stop_time: 10.0,
            },
            Velocity {
                value: Vec3::new(initial_speed, 0.0, 0.0),
            },
        ));

        tick(&mut app, dt);

        let world = app.world_mut();
        let mut q = world.query::<&Velocity>();
        let velocity = q.single(world).unwrap();

        let expected = initial_speed - friction * dt;

        assert!(
            (velocity.value.x - expected).abs() < 0.01,
            "velocity {:?}, expected {}",
            velocity.value,
            expected
        );
    }
    #[test]
    fn test_apply_friction_clamps_to_zero() {
        let mut app = setup_acceleration_app();
        app.add_systems(Update, super::apply_friction);

        let dt = 0.016;

        app.world_mut().spawn((
            Player,
            Movable,
            MovementState::Idle,
            ComputedStats {
                speed: 10.0,
                acceleration: 10.0,
                friction: 1000.0, // extreme friction
                dash_speed: 10.0,
                dash_time: 10.0,
                dash_friction: 10.0,
                dash_stop_time: 10.0,
            },
            Velocity {
                value: Vec3::new(1.0, 0.0, 0.0),
            },
        ));

        tick(&mut app, dt);

        let world = app.world_mut();
        let mut q = world.query::<&Velocity>();
        let velocity = q.single(world).unwrap();

        assert!(
            velocity.value.length() == 0.0,
            "velocity should be zero but was {:?}",
            velocity.value
        );
    }
    #[test]
    fn test_apply_friction_preserves_direction() {
        let mut app = setup_acceleration_app();
        app.add_systems(Update, super::apply_friction);

        let dt = 0.016;

        let initial = Vec3::new(3.0, 4.0, 0.0); // direction ≠ axis-aligned

        app.world_mut().spawn((
            Player,
            Movable,
            MovementState::Idle,
            ComputedStats {
                speed: 10.0,
                acceleration: 10.0,
                friction: 1.0,
                dash_speed: 10.0,
                dash_time: 10.0,
                dash_friction: 10.0,
                dash_stop_time: 10.0,
            },
            Velocity { value: initial },
        ));

        tick(&mut app, dt);

        let world = app.world_mut();
        let mut q = world.query::<&Velocity>();
        let velocity = q.single(world).unwrap();

        let original_dir = initial.normalize();
        let new_dir = velocity.value.normalize_or_zero();

        assert!(
            original_dir.distance(new_dir) < 0.01,
            "direction changed: {:?} -> {:?}",
            original_dir,
            new_dir
        );
    }
}
