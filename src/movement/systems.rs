use super::components::*;
use crate::common::components::Player;
use bevy::ecs::query::QueryData;
use bevy::prelude::*;

const UP_BUTTON: KeyCode = KeyCode::ArrowUp;
const DOWN_BUTTON: KeyCode = KeyCode::ArrowDown;
const LEFT_BUTTON: KeyCode = KeyCode::ArrowLeft;
const RIGHT_BUTTON: KeyCode = KeyCode::ArrowRight;
const MAX_SPEED: f32 = 250.0;
const SPEED_UP_TIME: f32 = 0.2;
const ACCELERATION: f32 = MAX_SPEED / SPEED_UP_TIME;
const WALK_STOP_TIME: f32 = 0.4;
const FRICTION: f32 = MAX_SPEED / WALK_STOP_TIME;

const DASH_MAX_SPEED: f32 = 600.0;
const DASH_BUTTON: KeyCode = KeyCode::ShiftLeft;
const DASH_FRICTION: f32 = 500.0;
const DASH_STOP_TIME: f32 = 0.1;
const POST_DASH_FRICTION: f32 = DASH_MAX_SPEED / DASH_STOP_TIME;
const DASH_TIME: f32 = 0.3;

#[derive(QueryData)]
#[query_data(mutable)]
pub struct MovementData {
    pub velocity: &'static Velocity,
    pub state: &'static mut MovementState,
    pub dash: Option<&'static Dash>,
    pub recover: Option<&'static Recover>,
}

pub fn apply_acceleration(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &MovementState, Option<&Dash>), With<Player>>,
) {
    let input_dir = compute_direction(&keyboard);
    for (mut velocity, state, dash) in &mut query {
        match state {
            MovementState::Dashing => {
                let dash = dash.expect("Dashing state must have Dash component");

                velocity.0 = dash.direction * DASH_MAX_SPEED;

                if velocity.0.length() > DASH_MAX_SPEED {
                    velocity.0 = velocity.0.normalize() * DASH_MAX_SPEED;
                }
            }

            MovementState::Recovering => {}

            MovementState::Moving | MovementState::Idle => {
                velocity.0 += input_dir * ACCELERATION * time.delta_secs();
                if velocity.0.length() > MAX_SPEED {
                    velocity.0 = velocity.0.normalize() * MAX_SPEED;
                }
            }
        }
    }
}
pub fn apply_friction(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &MovementState), With<Player>>,
) {
    for (mut velocity, state) in &mut query {
        let speed = velocity.0.length();

        let friction = match state {
            MovementState::Dashing => DASH_FRICTION,
            MovementState::Recovering => POST_DASH_FRICTION,
            MovementState::Moving | MovementState::Idle => FRICTION,
        };
        debug!(
            "velocity before: {:?}, with friction: {:?}",
            velocity, friction
        );
        if speed > 0.0 {
            let drop = friction * time.delta_secs();
            let new_speed = (speed - drop).max(0.0);

            velocity.0 = velocity.0.normalize_or_zero() * new_speed;
        }
    }
}
pub fn apply_velocity(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity), With<Player>>,
) {
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.0 * time.delta_secs();
    }
}

pub fn compute_direction(input: &ButtonInput<KeyCode>) -> Vec3 {
    let mut direction = Vec3::ZERO;

    if input.pressed(UP_BUTTON) {
        direction.y += 1.0;
    }
    if input.pressed(DOWN_BUTTON) {
        direction.y -= 1.0;
    }
    if input.pressed(LEFT_BUTTON) {
        direction.x -= 1.0;
    }
    if input.pressed(RIGHT_BUTTON) {
        direction.x += 1.0;
    }

    direction.normalize_or_zero()
}

pub fn handle_dash_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &MovementState), With<Player>>,
) {
    if !keyboard.pressed(DASH_BUTTON) {
        return;
    }

    for (entity, state) in &query {
        if *state == MovementState::Dashing {
            info!("Ignore dash input, already dashing.");
            continue;
        }

        let direction = compute_direction(&keyboard);
        if direction == Vec3::ZERO {
            info!("Ignore dash input, no direction input.");
            continue;
        }

        commands.entity(entity).insert(Dash {
            direction,
            timer: Timer::from_seconds(DASH_TIME, TimerMode::Once),
        });
    }
}

pub fn update_dash_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Dash)>,
) {
    for (entity, mut dash) in &mut query {
        dash.timer.tick(time.delta());
        debug!("dash time left: {:?}", dash.timer.remaining());
        if dash.timer.is_finished() {
            commands
                .entity(entity)
                .remove::<Dash>()
                .insert(MovementState::Recovering)
                .insert(Recover {
                    timer: Timer::from_seconds(DASH_STOP_TIME, TimerMode::Once),
                });
        }
    }
}

pub fn update_recover(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Recover)>,
) {
    for (entity, mut recover) in &mut query {
        recover.timer.tick(time.delta());
        debug!("current time left: {:?}", recover.timer.remaining());
        if recover.timer.is_finished() {
            commands.entity(entity).remove::<Recover>();
        }
    }
}

pub fn update_movement_state(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<MovementData, With<Player>>,
) {
    for mut data in &mut query {
        let input_dir = compute_direction(&keyboard);
        let speed = data.velocity.0.length();

        let new_state = if data.dash.is_some() {
            MovementState::Dashing
        } else if data.recover.is_some() {
            MovementState::Recovering
        } else if input_dir != Vec3::ZERO {
            MovementState::Moving
        } else if speed > 1.0 {
            // still sliding
            MovementState::Moving
        } else {
            MovementState::Idle
        };

        if *data.state != new_state {
            info!("State change: {:?} -> {:?}", *data.state, new_state);
            debug!("velocity: {:?}", data.velocity);
            *data.state = new_state;
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
            info!("Facing direction: {:?}", dir);
            facing.0 = dir;
        }
    }
}

pub fn debug_movement_state_changes(
    mut query: Query<(Entity, &MovementState), Changed<MovementState>>,
) {
    for (entity, state) in &mut query {
        debug!("Entity {:?} changed state to {:?}", entity, state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::components::Player;
    use crate::movement::MovementPlugin;
    use bevy::app::FixedMain;
    use bevy::time::Time;

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

        transform.unwrap().0
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
            (last - prev).abs() < 0.1,
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
        ));
        // tick(&mut app, 0.016);
        app.update();
        app
    }
    #[test]
    fn check_face_position_is_same_as_last_input_vector(){
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
    fn check_face_position_updates_when_changing_orientation(){
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

        assert!(dash_count == 1, "Dash was not created");
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
            velocity.0.x > normal.x,
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
        let mut q = world.query::<&Velocity>();
        let velocity = q.single(world).unwrap();

        assert!(
            velocity.0.length() <= DASH_MAX_SPEED + 1.0, // tolerance
            "Dash exceeded max speed"
        );
    }
    #[test]
    fn dash_stops_within_expected_time() {
        let mut app = test_app();

        // trigger dash
        let mut input = ButtonInput::<KeyCode>::default();

        input.press(RIGHT_BUTTON);
        input.press(DASH_BUTTON);
        app.insert_resource(input);
        // first frame: start dash
        tick(&mut app, 0.016);
        // release input
        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.release(RIGHT_BUTTON);
            input.release(DASH_BUTTON);
        }
        tick(&mut app, 0.016);
        let world = app.world_mut();
        let mut q = world.query::<&MovementState>();
        let state = q.single(world).unwrap();
        assert_eq!(*state, MovementState::Dashing);
        let total_time = DASH_TIME + DASH_STOP_TIME + WALK_STOP_TIME;
        let steps = (total_time / 0.016) as usize + 200;

        for _ in 0..steps {
            tick(&mut app, 0.016);
            let world = app.world_mut();
            let mut q = world.query::<(&Velocity, &MovementState)>();
            let (velocity, state) = q.single(world).unwrap();
        }

        let world = app.world_mut();
        let mut q = world.query::<&Velocity>();
        let velocity = q.single(world).unwrap();
        assert!(
            velocity.0.length() < 1.0,
            "Dash did not stop within expected time"
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
        let total_time = DASH_TIME;
        let steps = (total_time / 0.016) as usize;
        for _ in 0..steps {
            tick(&mut app, 0.016);
        }

        let world = app.world_mut();

        let mut dash_q = world.query::<&Dash>();
        let dash_count = dash_q.iter(world).count();
        let mut velocity_q = world.query::<&Velocity>();
        let mut state_q = world.query::<&MovementState>();
        let state = state_q.single(world).unwrap();

        assert_eq!(dash_count, 0, "Dash component should be removed");
        assert_eq!(*state, MovementState::Recovering);
    }
}
