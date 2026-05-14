#[allow(unused)]
use crate::common::components::{ComputedStats, Player, Stats};
use crate::movement::MovementPlugin;
#[allow(unused)]
use crate::movement::components::{Dash, Facing, MovementState, Velocity};
#[allow(unused)]
use crate::movement::systems::compute_direction;
use bevy::app::FixedMain;
use bevy::prelude::*;
use bevy::time::Time;
#[allow(dead_code)]

const UP_BUTTON: KeyCode = KeyCode::ArrowUp;
#[allow(dead_code)]
const DOWN_BUTTON: KeyCode = KeyCode::ArrowDown;
#[allow(dead_code)]
const LEFT_BUTTON: KeyCode = KeyCode::ArrowLeft;
#[allow(dead_code)]
const RIGHT_BUTTON: KeyCode = KeyCode::ArrowRight;
#[allow(dead_code)]
const DASH_BUTTON: KeyCode = KeyCode::ShiftLeft;

#[allow(dead_code)]
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

#[test]
fn test_diagonal_normalized() {
    let mut input = ButtonInput::<KeyCode>::default();
    input.press(UP_BUTTON);
    input.press(RIGHT_BUTTON);

    let dir = compute_direction(&input);

    assert!(dir.length() <= 1.0);
}

#[allow(dead_code)]
fn run_movement(keys: &[KeyCode]) -> Vec3 {
    let mut app = test_app();

    // input
    let mut input = ButtonInput::<KeyCode>::default();
    for key in keys {
        input.press(*key);
    }
    app.insert_resource(input);
    tick(&mut app, 0.016);
    // simulate multiple frames
    for _ in 0..10 {
        tick(&mut app, 0.016);
    }

    let world = app.world_mut();
    let mut query = world.query::<&Velocity>();
    let transform = query.single(world);

    transform.unwrap().value
}

#[test]
fn movement_cardinal_directions() {
    let cases = vec![
        (vec![UP_BUTTON], Vec3::Y),
        (vec![DOWN_BUTTON], -Vec3::Y),
        (vec![LEFT_BUTTON], -Vec3::X),
        (vec![RIGHT_BUTTON], Vec3::X),
    ];
    for (keys, expected) in cases {
        let movement = run_movement(&keys);

        assert!(
            movement.dot(expected) > 0.0,
            "Wrong direction for {:?}, got {:?}",
            keys,
            movement
        );
    }
}

#[test]
fn movement_diagonal_directions() {
    let cases = vec![
        (vec![UP_BUTTON, RIGHT_BUTTON], Vec3::new(1.0, 1.0, 0.0)),
        (vec![UP_BUTTON, LEFT_BUTTON], Vec3::new(-1.0, 1.0, 0.0)),
        (vec![DOWN_BUTTON, RIGHT_BUTTON], Vec3::new(1.0, -1.0, 0.0)),
        (vec![DOWN_BUTTON, LEFT_BUTTON], Vec3::new(-1.0, -1.0, 0.0)),
    ];

    for (keys, expected) in cases {
        let movement = run_movement(&keys);

        assert!(
            movement.dot(expected.normalize()) > 0.0,
            "Wrong diagonal for {:?}, got {:?}",
            keys,
            movement
        );
    }
}

#[test]
fn movement_is_normalized() {
    let straight = run_movement(&[RIGHT_BUTTON]).length();
    let diagonal = run_movement(&[RIGHT_BUTTON, UP_BUTTON]).length();
    assert!(
        (straight - diagonal).abs() < 10.0, // 👈 relaxed tolerance due to acceleration
        "Diagonal movement is faster than straight movement"
    );
}

#[allow(dead_code)]
fn run_frames(input_per_frame: Vec<Vec<KeyCode>>) -> Vec<Vec3> {
    let mut app = test_app();

    let mut positions = Vec::new();

    for keys in input_per_frame {
        let mut input = ButtonInput::<KeyCode>::default();
        for key in keys {
            input.press(key);
        }
        app.insert_resource(input);

        tick(&mut app, 0.016);

        let world = app.world_mut();
        let mut query = world.query::<&Transform>();
        let transform = query.single(world);

        positions.push(transform.unwrap().translation);
    }

    positions
}

#[test]
fn movement_accelerates_over_time() {
    let frames = run_frames(vec![
        vec![RIGHT_BUTTON],
        vec![RIGHT_BUTTON],
        vec![RIGHT_BUTTON],
        vec![RIGHT_BUTTON],
    ]);

    let d1 = frames[1].x - frames[0].x;
    let d2 = frames[2].x - frames[1].x;
    let d3 = frames[3].x - frames[2].x;

    assert!(d2 >= d1, "Velocity did not increase between frame 1 and 2");
    assert!(d3 >= d2, "Velocity did not increase between frame 2 and 3");
}
#[test]
fn movement_has_momentum_after_input_release() {
    let frames = run_frames(vec![
        vec![RIGHT_BUTTON], // accelerate
        vec![RIGHT_BUTTON],
        vec![RIGHT_BUTTON],
        vec![RIGHT_BUTTON],
        vec![RIGHT_BUTTON],
        vec![RIGHT_BUTTON],
        vec![RIGHT_BUTTON],
        vec![RIGHT_BUTTON],
        vec![RIGHT_BUTTON],
        vec![RIGHT_BUTTON],
        vec![RIGHT_BUTTON],
        vec![RIGHT_BUTTON],
        vec![], // release
        vec![],
    ]);

    let d_release_1 = frames[frames.len() - 2].x - frames[frames.len() - 3].x;
    let d_release_2 = frames[frames.len() - 1].x - frames[frames.len() - 2].x;

    assert!(
        d_release_1 > 0.0,
        "Player stopped immediately after releasing input"
    );

    assert!(
        d_release_2 <= d_release_1,
        "Friction not slowing player down"
    );
}
#[test]
fn movement_has_max_speed_cap() {
    let frames = run_frames(vec![
            vec![RIGHT_BUTTON]; 300 // hold key for many frames
        ]);

    let mut deltas = vec![];
    for i in 1..frames.len() {
        deltas.push(frames[i].x - frames[i - 1].x);
    }

    let last = deltas[deltas.len() - 1];
    let prev = deltas[deltas.len() - 2];
    assert!(
        (last - prev).abs() < 0.2,
        "Speed did not stabilize near max speed, but got div {:?} of last:{:?}, previous:{:?}",
        (last - prev).abs(),
        last,
        prev
    );

    assert!(last < 10.0, "Speed is unreasonably high");
}
#[test]
fn movement_eventually_stops_due_to_friction() {
    let frames = run_frames(vec![
        vec![RIGHT_BUTTON],
        vec![RIGHT_BUTTON],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
    ]);

    let last_delta = frames.last().unwrap().x - frames[frames.len() - 2].x;

    assert!(last_delta.abs() < 0.01, "Player did not come to a stop");
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_plugins(MovementPlugin);

    app.insert_resource(Time::<Fixed>::from_seconds(0.016));

    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.world_mut().spawn((
        Player,
        Transform::default(),
        Velocity::default(),
        Facing(Vec2::X),
        MovementState::Idle,
        ComputedStats {
            speed: 250.0,
            acceleration: 1250.0,
            friction: 625.0,
            dash_speed: 600.0,
            dash_time: 0.2,
            dash_friction: 200.0,
            dash_stop_time: 0.3,
        },
    ));
    // tick(&mut app, 0.016);
    app.update();
    app
}
#[test]
fn check_face_position_is_same_as_last_input_vector() {
    let mut app = test_app();
    let mut input = ButtonInput::<KeyCode>::default();
    input.press(RIGHT_BUTTON);
    app.insert_resource(input);
    tick(&mut app, 0.016);

    let world = app.world_mut();
    let mut q = world.query::<&Facing>();
    let facing = q.single(world).unwrap();

    assert!(
        facing.0.distance(Vec2::X) < 0.01,
        "Facing was {:?}, expected RIGHT",
        facing.0
    );
}
#[test]
fn check_face_position_updates_when_changing_orientation() {
    let mut app = test_app();
    tick(&mut app, 0.016);
    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(LEFT_BUTTON);
        input.press(UP_BUTTON);
    }
    tick(&mut app, 0.016);
    let world = app.world_mut();
    let mut q = world.query::<&Facing>();
    let facing = q.single(world).unwrap();

    let expected = Vec2::new(-1.0, 1.0).normalize();
    assert!(
        facing.0.distance(expected) < 0.01,
        "Facing was {:?}, expected up-left",
        facing.0
    );
}
#[test]
fn dash_is_triggered() {
    let mut app = test_app();

    // input: press right + dash
    let mut input = ButtonInput::<KeyCode>::default();
    input.press(RIGHT_BUTTON);
    input.press(DASH_BUTTON);
    app.insert_resource(input);

    tick(&mut app, 0.016);

    let world = app.world_mut();
    let mut q = world.query::<&Dash>();

    let dash_count = q.iter(world).count();

    assert_eq!(dash_count, 1, "Dash was not created");
}
#[test]
fn dash_has_correct_direction() {
    let mut app = test_app();

    let mut input = ButtonInput::<KeyCode>::default();
    input.press(RIGHT_BUTTON);
    input.press(DASH_BUTTON);
    app.insert_resource(input);

    tick(&mut app, 0.016);

    let world = app.world_mut();
    let mut q = world.query::<&Dash>();

    let dash = q.single(world).unwrap();

    assert_eq!(dash.direction.x > 0.0, true);
    assert_eq!(dash.direction.y == 0.0, true);
}

#[test]
fn dash_increases_movement_speed() {
    let normal = run_movement(&[RIGHT_BUTTON]);

    let mut app = test_app();

    let mut input = ButtonInput::<KeyCode>::default();
    input.press(RIGHT_BUTTON);
    input.press(DASH_BUTTON);
    app.insert_resource(input);

    for _ in 0..5 {
        tick(&mut app, 0.016);
    }

    let world = app.world_mut();
    let mut q = world.query::<&Velocity>();
    let velocity = q.single(world).unwrap();
    assert!(
        velocity.value.x > normal.x,
        "Dash did not increase movement speed"
    );
}

#[test]
fn dash_respects_max_speed() {
    let mut app = test_app();

    let mut input = ButtonInput::<KeyCode>::default();
    input.press(RIGHT_BUTTON);
    input.press(DASH_BUTTON);
    app.insert_resource(input);

    for _ in 0..20 {
        tick(&mut app, 0.016);
    }

    let world = app.world_mut();
    let mut q = world.query::<(&Velocity, &Stats)>();
    let (velocity, stats) = q.single(world).unwrap();
    assert!(
        velocity.value.length() <= stats.dash_speed() + 1.0, // tolerance
        "Dash exceeded max speed"
    );
}
#[test]
fn player_becomes_idle_when_stopped() {
    let mut app = test_app();

    // no input, no velocity
    app.insert_resource(ButtonInput::<KeyCode>::default());

    for _ in 0..5 {
        tick(&mut app, 0.016);
    }

    let world = app.world_mut();
    let mut q = world.query::<&MovementState>();

    let state = q.single(world).unwrap();
    assert_eq!(*state, MovementState::Idle);
}
#[test]
fn player_enters_moving_state_when_input_pressed() {
    let mut app = test_app();

    let mut input = ButtonInput::<KeyCode>::default();
    input.press(RIGHT_BUTTON);
    app.insert_resource(input);

    tick(&mut app, 0.016);
    let world = app.world_mut();
    let mut q = world.query::<&MovementState>();

    let state = q.single(world).unwrap();
    assert_eq!(*state, MovementState::Moving);
}
#[test]
fn dash_forces_dashing_state() {
    let mut app = test_app();

    let mut input = ButtonInput::<KeyCode>::default();
    input.press(RIGHT_BUTTON);
    input.press(DASH_BUTTON);
    app.insert_resource(input);
    tick(&mut app, 0.016);
    tick(&mut app, 0.016);

    let world = app.world_mut();
    let mut q = world.query::<&MovementState>();
    let state = q.single(world).unwrap();
    assert_eq!(*state, MovementState::Dashing);
}
#[test]
fn dash_transitions_to_recovering() {
    let mut app = test_app();

    let mut input = ButtonInput::<KeyCode>::default();
    input.press(RIGHT_BUTTON);
    input.press(DASH_BUTTON);
    app.insert_resource(input);
    tick(&mut app, 0.016);
    // release input
    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.release(RIGHT_BUTTON);
        input.release(DASH_BUTTON);
    }
    // tick(&mut app, 0.016);
    // run until dash should finish
    let total_time = {
        let world = app.world_mut();
        let mut q = world.query::<&Stats>();
        let stats = q.single(world).unwrap();
        stats.dash_time() + 0.3
    };
    let steps = (total_time / 0.016) as usize;
    for _ in 0..steps {
        tick(&mut app, 0.016);
    }

    let world = app.world_mut();

    let mut dash_q = world.query::<&Dash>();
    let dash_count = dash_q.iter(world).count();
    let mut state_q = world.query::<&MovementState>();
    let state = state_q.single(world).unwrap();

    assert_eq!(dash_count, 0, "Dash component should be removed");
    assert_eq!(*state, MovementState::Recovering);
}
