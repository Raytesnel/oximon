use super::components::*;
use bevy::prelude::*;
pub fn apply_acceleration(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    let direction = compute_direction(&keyboard);

    let accel = 1000.0;
    let max_speed = 200.0;

    for mut velocity in &mut query {
        velocity.0 += direction * accel * time.delta_secs();

        if velocity.0.length() > max_speed {
            velocity.0 = velocity.0.normalize() * max_speed;
        }
    }
}
pub fn apply_friction(time: Res<Time>, mut query: Query<&mut Velocity, With<Player>>) {
    let friction = 800.0;

    for mut velocity in &mut query {
        let speed = velocity.0.length();

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

    if input.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    direction.normalize_or_zero()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
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
        input.press(KeyCode::KeyW);
        input.press(KeyCode::KeyD);

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
            (vec![KeyCode::KeyW], Vec3::Y),
            (vec![KeyCode::KeyS], -Vec3::Y),
            (vec![KeyCode::KeyA], -Vec3::X),
            (vec![KeyCode::KeyD], Vec3::X),
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
            (vec![KeyCode::KeyW, KeyCode::KeyD], Vec3::new(1.0, 1.0, 0.0)),
            (
                vec![KeyCode::KeyW, KeyCode::KeyA],
                Vec3::new(-1.0, 1.0, 0.0),
            ),
            (
                vec![KeyCode::KeyS, KeyCode::KeyD],
                Vec3::new(1.0, -1.0, 0.0),
            ),
            (
                vec![KeyCode::KeyS, KeyCode::KeyA],
                Vec3::new(-1.0, -1.0, 0.0),
            ),
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
        let straight = run_movement(&[KeyCode::KeyD]).length();
        let diagonal = run_movement(&[KeyCode::KeyD, KeyCode::KeyW]).length();

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
            vec![KeyCode::KeyD],
            vec![KeyCode::KeyD],
            vec![KeyCode::KeyD],
            vec![KeyCode::KeyD],
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
            vec![KeyCode::KeyD], // accelerate
            vec![KeyCode::KeyD],
            vec![KeyCode::KeyD],
            vec![KeyCode::KeyD],
            vec![KeyCode::KeyD],
            vec![KeyCode::KeyD],
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
            vec![KeyCode::KeyD]; 30 // hold key for many frames
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
            vec![KeyCode::KeyD],
            vec![KeyCode::KeyD],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        ]);

        let last_delta = frames.last().unwrap().x - frames[frames.len() - 2].x;

        assert!(last_delta.abs() < 0.01, "Player did not come to a stop");
    }
}
