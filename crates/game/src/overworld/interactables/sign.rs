use crate::overworld::components::*;
use crate::overworld::dialogs::{CurrentDialogue, DialogueState};
use bevy::prelude::*;

pub fn on_sign_interaction(
    trigger: On<InteractionEvent>,
    query: Query<(&InteractionType, &SignText)>,
    mut current_dialogue: ResMut<CurrentDialogue>,
) {
    let entity = trigger.event().entity;
    let Ok((InteractionType::Sign, sign_text)) = query.get(entity) else {
        return;
    };

    let lines: Vec<String> = sign_text.0.split('\n').map(|s| s.to_string()).collect();

    current_dialogue.state = Some(DialogueState {
        lines,
        current_line: 0,
        speaker_name: "Sign".to_string(),
    });
}

pub fn tick_sign_popups(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut SignPopup)>,
) {
    for (entity, mut popup) in &mut query {
        popup.timer.tick(time.delta());
        if popup.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    use crate::overworld::components::SignPopup;
    use crate::overworld::interactables::tick_sign_popups;

    fn make_app_with_time(step_seconds: f32) -> App {
        let mut app = App::new();
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(step_seconds),
        ));
        app.add_plugins(MinimalPlugins);
        app
    }

    #[test]
    fn sign_popup_despawns_after_timer() {
        // Each update() advances time by 2 seconds
        let mut app = make_app_with_time(2.0);
        app.add_systems(Update, tick_sign_popups);

        let popup = app
            .world_mut()
            .spawn(SignPopup {
                timer: Timer::from_seconds(3.0, TimerMode::Once),
            })
            .id();
        app.update();
        app.update();
        assert!(app.world().get_entity(popup).is_ok(), "popup alive at 2 s");
        for _ in 0..15 {
            app.update();
        }
        assert!(
            app.world().get_entity(popup).is_err(),
            "popup must be despawned after timer expires"
        );
    }

    #[test]
    fn sign_popup_survives_before_timer() {
        let mut app = make_app_with_time(1.0);
        app.add_systems(Update, tick_sign_popups);

        let popup = app
            .world_mut()
            .spawn(SignPopup {
                timer: Timer::from_seconds(3.0, TimerMode::Once),
            })
            .id();

        app.update(); // +1 s
        app.update(); // +2 s
        assert!(app.world().get_entity(popup).is_ok());
    }
}
