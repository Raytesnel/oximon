use super::components::*;
use crate::combat::components::Hitstun;
use crate::common::components::{ComputedStats, Player, Stats};
use bevy::ecs::query::QueryData;
use bevy::prelude::*;

const UP_BUTTON: KeyCode = KeyCode::ArrowUp;
const DOWN_BUTTON: KeyCode = KeyCode::ArrowDown;
const LEFT_BUTTON: KeyCode = KeyCode::ArrowLeft;
const RIGHT_BUTTON: KeyCode = KeyCode::ArrowRight;
const DASH_BUTTON: KeyCode = KeyCode::ShiftLeft;

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
        (With<Movable>, Without<Hitstun>),
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
    mut query: Query<(&mut Velocity, &MovementState, &ComputedStats), (With<Movable>)>,
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
    mut query: Query<(&mut Transform, &Velocity), With<Movable>>,
) {
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.value * time.delta_secs();
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
    query: Query<(Entity, &MovementState, &ComputedStats), (With<Movable>, Without<Hitstun>)>,
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
        (With<Movable>, Without<Hitstun>),
    >,
) {
    for (mut velocity, mut movement_state, dash, recover, move_intent) in &mut query {
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

    if keyboard.pressed(UP_BUTTON) {
        direction.y += 1.0;
    }
    if keyboard.pressed(DOWN_BUTTON) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(LEFT_BUTTON) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(RIGHT_BUTTON) {
        direction.x += 1.0;
    }

    for mut intent in &mut query {
        intent.direction = direction.normalize_or_zero();
    }
}
