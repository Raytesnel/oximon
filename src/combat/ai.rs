use crate::combat::components::Attack;
use crate::movement::components::{MoveIntent, Velocity};
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
    pub move_dir: Vec3,
    pub wants_attack: bool,
}
#[derive(Component)]
pub struct Target {
    pub entity: Entity,
}

pub fn ai_decision_system(
    time: Res<Time<Fixed>>,
    mut query: Query<(
        &mut AI,
        &AIConfig, // TODO: replace this with computed state.
        &Transform,
        &Target,
        &mut MoveIntent,
        &mut AIIntent,
    )>,
    transforms: Query<&Transform>,
) {
    for (mut ai, config, transform, target, mut move_intent, mut ai_intent) in &mut query {
        let target_pos = transforms.get(target.entity).unwrap().translation;
        let dist = transform.translation.distance(target_pos);
        ai_intent.wants_attack = false;

        match ai.state {
            AIState::Wander => {
                if dist < config.vision_range {
                    ai.state = AIState::Chase;
                }

                ai.timer -= time.delta().as_secs_f32();

                if ai.timer <= 0.0 {
                    move_intent.direction = Vec3::new(
                        rand::random::<f32>() * 2.0 - 1.0,
                        rand::random::<f32>() * 2.0 - 1.0,
                        0.0,
                    )
                    .normalize_or_zero();
                    info!("new route chosen");
                    info!("AI intent dir: {:?}", move_intent.direction);
                    ai.timer = 1.5;
                }
            }

            AIState::Chase => {
                if dist < config.attack_range {
                    ai.state = AIState::Attack;
                } else if dist > config.vision_range {
                    ai.state = AIState::Wander;
                }

                move_intent.direction = (target_pos - transform.translation).normalize_or_zero();
            }

            AIState::Attack => {
                if dist > config.attack_range {
                    ai.state = AIState::Chase;
                }
                move_intent.direction = Vec3::ZERO;
                ai_intent.wants_attack = true;
            }
        }
    }
}

pub fn ai_attack_system(mut query: Query<(&AI, &mut AIIntent, &mut Attack)>) {
    for (ai, intent, mut attack) in &mut query {
        if intent.wants_attack {
            if ai.state == AIState::Attack && attack.hit_timer.is_finished() {
                info!("attacking!");
            }
        }
    }
}
