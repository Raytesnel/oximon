use crate::combat::components::Attack;
use crate::movement::components::Velocity;
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Eq)]
pub enum AIState {
    Wander,
    Chase,
    Attack,
}
#[derive(Component, Debug)]
pub struct AI {
    pub state: AIState,
    pub timer: f32,
}
#[derive(Component, Debug)]
pub struct AIConfig {
    pub vision_range: f32,
    pub attack_range: f32,
    pub wander_speed: f32,
    pub chase_speed: f32,
}
#[derive(Component, Debug)]
pub struct AIIntent {
    pub move_dir: Vec2,
    pub wants_attack: bool,
}
#[derive(Component)]
pub struct Target {
    pub entity: Entity,
}

pub fn ai_decision_system(
    time: Res<Time>,
    mut query: Query<(&mut AI, &AIConfig, &Transform, &Target, &mut AIIntent)>,
    transforms: Query<&Transform>,
) {
    for (mut ai, config, transform, target, mut intent) in &mut query {
        let target_pos = transforms.get(target.entity).unwrap().translation;
        let dist = transform.translation.distance(target_pos);

        match ai.state {
            AIState::Wander => {
                if dist < config.vision_range {
                    ai.state = AIState::Chase;
                }

                ai.timer -= time.delta_secs();

                if ai.timer <= 0.0 {
                    intent.move_dir = Vec2::new(
                        rand::random::<f32>() * 2.0 - 1.0,
                        rand::random::<f32>() * 2.0 - 1.0,
                    )
                        .normalize_or_zero();

                    ai.timer = 1.5;
                }
            }

            AIState::Chase => {
                if dist < config.attack_range {
                    ai.state = AIState::Attack;
                } else if dist > config.vision_range {
                    ai.state = AIState::Wander;
                }

                intent.move_dir =
                    (target_pos.truncate() - transform.translation.truncate()).normalize_or_zero();
            }

            AIState::Attack => {
                if dist > config.attack_range {
                    ai.state = AIState::Chase;
                }

                intent.move_dir = Vec2::ZERO;
                intent.wants_attack = true;
            }
        }
    }
}

pub fn ai_movement_system(mut query: Query<(&AI, &AIConfig, &mut AIIntent, &mut Velocity)>) {
    for (ai, config, intent, mut vel) in &mut query {
        match ai.state {
            AIState::Wander => {
                vel.value.x = intent.move_dir.x * config.wander_speed;
                vel.value.y = intent.move_dir.y * config.wander_speed;
            }

            AIState::Chase => {
                vel.value.x = intent.move_dir.x * config.chase_speed;
                vel.value.y = intent.move_dir.y * config.chase_speed;
            }

            AIState::Attack => {
                vel.value.x = 0.0;
                vel.value.y = 0.0;
            }
        }
    }
}

pub fn ai_attack_system(mut query: Query<(&AI, &mut AIIntent, &mut Attack)>) {
    for (ai, intent, mut attack) in &mut query {
        if intent.wants_attack {
            if ai.state == AIState::Attack && attack.hit_timer.is_finished() {
                attack.active = true;
            }
        }
    }
}
