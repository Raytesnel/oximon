use super::components::*;
use bevy::prelude::*;

const UP_BUTTON: KeyCode = KeyCode::ArrowUp;
const DOWN_BUTTON: KeyCode = KeyCode::ArrowDown;
const LEFT_BUTTON: KeyCode = KeyCode::ArrowLeft;
const RIGHT_BUTTON: KeyCode = KeyCode::ArrowRight;
const MAX_SPEED: f32 = 200.0;
const ACCELERATION: f32 = 1000.0;
const WALK_STOP_TIME: f32 = 0.05;
const FRICTION: f32 = MAX_SPEED / WALK_STOP_TIME;

const DASH_ACCELERATION: f32 = 2000.0;
const DASH_MAX_SPEED: f32 = 1000.0;
const DASH_BUTTON: KeyCode = KeyCode::ShiftLeft;
const DASH_FRICTION: f32 = 500.0;
const DASH_STOP_TIME: f32 = 0.2;
const POST_DASH_FRICTION: f32 = DASH_MAX_SPEED / DASH_STOP_TIME;
const DASH_TIME: f32 = 0.3;

pub fn apply_acceleration(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, Option<&Dash>), With<Player>>,
) {
    let direction = compute_direction(&keyboard);

    for (mut velocity, dash) in &mut query {
        if let Some(dash) = dash {
            velocity.0 += dash.direction * DASH_ACCELERATION * time.delta_secs();
            if velocity.0.length() > DASH_MAX_SPEED {
                velocity.0 = velocity.0.normalize() * DASH_MAX_SPEED;
            }
            continue;
        } else {
            velocity.0 += direction * ACCELERATION * time.delta_secs();
            if velocity.0.length() > MAX_SPEED {
                velocity.0 = velocity.0.normalize() * MAX_SPEED;
            }
        }
    }
}
pub fn apply_friction(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, Option<&Dash>), With<Player>>,
) {
    for (mut velocity, dash) in &mut query {
        let speed = velocity.0.length();
        let friction = if dash.is_some() {
            DASH_FRICTION
        } else if velocity.0.length() > MAX_SPEED {
            POST_DASH_FRICTION
        } else {
            FRICTION
        };
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
    _time: Res<Time>,
    query: Query<(Entity, Option<&Dash>), With<Player>>,
) {
    if !keyboard.just_pressed(DASH_BUTTON) {
        return;
    }

    for (entity, dash) in &query {
        // prevent re-dashing while already dashing
        if dash.is_some() {
            continue;
        }

        let direction = compute_direction(&keyboard);

        // don't dash if no direction
        if direction == Vec3::ZERO {
            continue;
        }

        commands.entity(entity).insert(Dash {
            direction,
            timer: Timer::from_seconds(DASH_TIME, TimerMode::Once),
        });
    }
}

pub fn update_dash(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Dash)>) {
    for (entity, mut dash) in &mut query {
        dash.timer.tick(time.delta());
        if dash.timer.is_finished() {
            commands.entity(entity).remove::<Dash>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::Time;

    fn add_systems() -> App {
        let mut app = App::new();

        app.add_systems(
            Update,
            (
                apply_acceleration,
                apply_friction.after(apply_acceleration),
                apply_velocity.after(apply_friction),
            ),
        );

        app.world_mut()
            .spawn((Player, Transform::default(), Velocity::default()));
        app
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
        let mut app = add_systems();

        // input
        let mut input = ButtonInput::<KeyCode>::default();
        for key in keys {
            input.press(*key);
        }
        app.insert_resource(input);

        // simulate multiple frames
        for _ in 0..10 {
            let mut time = Time::<()>::default();
            time.advance_by(std::time::Duration::from_secs_f32(0.016));
            app.insert_resource(time);

            app.update();
        }

        let world = app.world_mut();
        let mut query = world.query::<&Transform>();
        let transform = query.single(world);

        transform.unwrap().translation
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
            (straight - diagonal).abs() < 1.0, // 👈 relaxed tolerance due to acceleration
            "Diagonal movement is faster than straight movement"
        );
    }

    fn run_frames(input_per_frame: Vec<Vec<KeyCode>>) -> Vec<Vec3> {
        let mut app = add_systems();

        let mut positions = Vec::new();

        for keys in input_per_frame {
            let mut input = ButtonInput::<KeyCode>::default();
            for key in keys {
                input.press(key);
            }
            app.insert_resource(input);

            let mut time = Time::<()>::default();
            time.advance_by(std::time::Duration::from_secs_f32(0.016));
            app.insert_resource(time);

            app.update();

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
            vec![RIGHT_BUTTON]; 30 // hold key for many frames
        ]);

        let mut deltas = vec![];
        for i in 1..frames.len() {
            deltas.push(frames[i].x - frames[i - 1].x);
        }

        let last = deltas[deltas.len() - 1];
        let prev = deltas[deltas.len() - 2];
        assert!(
            (last - prev).abs() < 0.1,
            "Speed did not stabilize near max speed"
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
    fn add_dash_systems() -> App {
        let mut app = add_systems();

        app.add_systems(Update, (handle_dash_input, update_dash));

        app
    }
    #[test]
    fn dash_is_triggered() {
        let mut app = add_dash_systems();

        // input: press right + dash
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(RIGHT_BUTTON);
        input.press(DASH_BUTTON);
        app.insert_resource(input);

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs_f32(0.016));
        app.insert_resource(time);

        app.update();

        let world = app.world_mut();
        let mut q = world.query::<&Dash>();

        let dash_count = q.iter(world).count();

        assert!(dash_count == 1, "Dash was not created");
    }
    #[test]
    fn dash_has_correct_direction() {
        let mut app = add_dash_systems();

        let mut input = ButtonInput::<KeyCode>::default();
        input.press(RIGHT_BUTTON);
        input.press(DASH_BUTTON);
        app.insert_resource(input);

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs_f32(0.016));
        app.insert_resource(time);

        app.update();

        let world = app.world_mut();
        let mut q = world.query::<&Dash>();

        let dash = q.single(world).unwrap();

        assert_eq!(dash.direction.x > 0.0, true);
        assert_eq!(dash.direction.y == 0.0, true);
    }

    #[test]
    fn dash_increases_movement_speed() {
        let normal = run_movement(&[RIGHT_BUTTON]);

        let mut app = add_dash_systems();

        let mut input = ButtonInput::<KeyCode>::default();
        input.press(RIGHT_BUTTON);
        input.press(DASH_BUTTON);
        app.insert_resource(input);

        for _ in 0..3 {
            let mut time = Time::<()>::default();
            time.advance_by(std::time::Duration::from_secs_f32(0.016));
            app.insert_resource(time);
            app.update();
        }

        let world = app.world_mut();
        let mut q = world.query::<&Transform>();
        let dash_pos = q.single(world).unwrap().translation;

        assert!(
            dash_pos.x > normal.x,
            "Dash did not increase movement speed"
        );
    }

    #[test]
    fn dash_respects_max_speed() {
        let mut app = add_dash_systems();

        let mut input = ButtonInput::<KeyCode>::default();
        input.press(RIGHT_BUTTON);
        input.press(DASH_BUTTON);
        app.insert_resource(input);

        for _ in 0..20 {
            let mut time = Time::<()>::default();
            time.advance_by(std::time::Duration::from_secs_f32(DASH_TIME / 20f32));
            app.insert_resource(time);
            app.update();
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
        let mut app = add_dash_systems();

        // trigger dash
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(RIGHT_BUTTON);
        input.press(DASH_BUTTON);
        app.insert_resource(input);

        // first frame: start dash
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs_f32(0.016));
        app.insert_resource(time);
        app.update();

        // release input
        let input = ButtonInput::<KeyCode>::default();
        app.insert_resource(input);
        app.update();

        let total_time = DASH_TIME + DASH_STOP_TIME + WALK_STOP_TIME;
        let steps = (total_time / 0.016) as usize;

        for _ in 0..steps {
            let mut time = Time::<()>::default();
            time.advance_by(std::time::Duration::from_secs_f32(0.016));
            app.insert_resource(time);
            app.update();
        }

        let world = app.world_mut();
        let mut q = world.query::<&Velocity>();
        let velocity = q.single(world).unwrap();
        assert!(
            velocity.0.length() < 1.0,
            "Dash did not stop within expected time"
        );
    }
}
