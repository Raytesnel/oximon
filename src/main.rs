use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, sprite_movement)
        .run();
}

#[derive(Component)]
struct Player;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(20.0, 20.0)), // your "block"
            ..default()
        },
        Transform::from_xyz(0., 0., 0.),
        Player,
    ));
}

fn compute_direction(input: &ButtonInput<KeyCode>) -> Vec3 {
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

fn sprite_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    if let Ok(mut transform) = query.single_mut() {
        let direction = compute_direction(&keyboard);

        let speed = 200.0;

        transform.translation += direction.normalize_or_zero() * speed * time.delta_secs();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::Time;

    #[test]
    fn test_direction_up() {
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(KeyCode::KeyW);

        let dir = compute_direction(&input);

        assert_eq!(dir, Vec3::new(0.0, 1.0, 0.0));
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
        let mut app = App::new();
        app.add_systems(Update, sprite_movement);

        app.world_mut().spawn((Player, Transform::default()));

        let mut input = ButtonInput::<KeyCode>::default();
        for key in keys {
            input.press(*key);
        }
        app.insert_resource(input);

        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs_f32(0.016));
        app.insert_resource(time);

        app.update();

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
            (straight - diagonal).abs() < 0.001,
            "Diagonal movement is faster than straight movement"
        );
    }
}
