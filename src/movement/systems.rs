use super::components::*;
use crate::combat::components::Hitstun;
use crate::common::components::{ComputedStats, Player};
use crate::movement::input::{
    DASH_BUTTON, MOVE_DOWN_BUTTON, MOVE_LEFT_BUTTON, MOVE_RIGHT_BUTTON, MOVE_UP_BUTTON,
};
use crate::movement::types::*;
use bevy::ecs::query::QueryData;
use bevy::prelude::*;

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
    mut query: Query<
        (
            &mut Velocity,
            &MovementState,
            Option<&Dash>,
            &ComputedStats,
            &MoveIntent,
        ),
        AllowedMovable,
    >,
) {
    for (mut velocity, state, dash, stats, move_intent) in &mut query {
        match state {
            MovementState::Dashing => {
                let dash = dash.expect("Dashing state must have Dash component");
                velocity.value = dash.direction * stats.dash_speed;

                if velocity.value.length() > stats.dash_speed {
                    velocity.value = velocity.value.normalize() * stats.dash_speed;
                }
            }

            MovementState::Recovering => {}

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

pub fn apply_friction(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &MovementState, &ComputedStats), With<Movable>>,
) {
    for (mut velocity, state, stats) in &mut query {
        let speed = velocity.value.length();

        let friction = match state {
            MovementState::Dashing => stats.dash_friction,
            MovementState::Recovering => stats.dash_speed / stats.dash_stop_time,
            MovementState::Moving | MovementState::Idle => stats.friction,
        };

        if speed > 0.0 {
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

pub fn compute_direction(input: &ButtonInput<KeyCode>) -> Vec3 {
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

pub fn handle_dash_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &MovementState, &ComputedStats), AllowedMovable>,
) {
    if !keyboard.pressed(DASH_BUTTON) {
        return;
    }
    for (entity, state, stats) in &query {
        if *state == MovementState::Dashing {
            continue;
        }

        let direction = compute_direction(&keyboard);
        if direction == Vec3::ZERO {
            continue;
        }

        commands.entity(entity).insert(Dash {
            direction,
            timer: Timer::from_seconds(stats.dash_time, TimerMode::Once),
        });
    }
}

pub fn update_dash_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Dash, &ComputedStats)>,
) {
    for (entity, mut dash, stats) in &mut query {
        dash.timer.tick(time.delta());
        if dash.timer.is_finished() {
            commands.entity(entity).remove::<Dash>().insert(Recover {
                timer: Timer::from_seconds(stats.dash_stop_time, TimerMode::Once),
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
        if recover.timer.is_finished() {
            commands.entity(entity).remove::<Recover>();
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_movement_state(
    mut query: Query<
        (
            &Velocity,
            &mut MovementState,
            Option<&Dash>,
            Option<&Recover>,
            &MoveIntent,
        ),
        AllowedMovable,
    >,
) {
    for (velocity, mut movement_state, dash, recover, move_intent) in &mut query {
        let input_dir = move_intent.direction;
        let speed = velocity.value.length();
        let new_state = if dash.is_some() {
            MovementState::Dashing
        } else if recover.is_some() {
            MovementState::Recovering
        } else if input_dir != Vec3::ZERO {
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

pub fn debug_movement_state_changes(
    mut query: Query<(Entity, &MovementState), Changed<MovementState>>,
) {
    for (entity, state) in &mut query {
        debug!("Entity {:?} changed state to {:?}", entity, state);
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
    macro_rules! test_facing {
        ($($name:ident: $keys:expr => $expected:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    run_facing_test($keys, $expected);
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
                #[test]
                fn $name() {
                    run_input_system_test($keys, $expected);
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
    fn run_input_system_test(keys: &[KeyCode], expected: Vec3) {
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
}
