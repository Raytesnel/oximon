use bevy::prelude::*;

use crate::overworld::components::DialogueTitle;

#[derive(Component)]
pub struct DialogueBox;

#[derive(Component)]
pub struct DialogueText;

#[derive(Component)]
pub struct DialogueState {
    pub lines: Vec<String>,
    pub current_line: usize,
    pub speaker_name: String,
}

#[derive(Component)]
pub struct DialogueActive;

#[derive(Resource, Default)]
pub struct CurrentDialogue {
    pub state: Option<DialogueState>,
}

pub fn spawn_dialogue_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                display: Display::None, // Hidden by default
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::FlexEnd,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            DialogueBox,
            DialogueActive,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Percent(80.0),
                        height: Val::Px(120.0),
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(16.0)),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceBetween,
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.8, 0.8, 0.8)),
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                ))
                .with_children(|parent| {
                    // Speaker name
                    parent.spawn((
                        Text::new("NPC"), // Default value
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.8, 0.0)),
                        DialogueTitle, // Add marker component
                    ));

                    // Dialogue text
                    parent.spawn((
                        Text::new("..."),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        DialogueText,
                    ));

                    // Continue prompt
                    parent.spawn((
                        Text::new("Press E to continue"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));
                });
        });
}

pub fn show_dialogue(
    mut dialogue_box: Query<&mut Node, With<DialogueBox>>,
    mut param_set: ParamSet<(
        Query<&mut Text, With<DialogueText>>,
        Query<&mut Text, With<DialogueTitle>>,
    )>,
    current_dialogue: Res<CurrentDialogue>,
) {
    if let Some(dialogue_state) = &current_dialogue.state {
        // Show box
        for mut node in dialogue_box.iter_mut() {
            node.display = Display::Flex;
        }

        // Update speaker name
        if let Ok(mut title) = param_set.p1().single_mut() {
            title.0 = dialogue_state.speaker_name.clone();
        }

        // Update text
        if dialogue_state.current_line < dialogue_state.lines.len()
            && let Ok(mut text) = param_set.p0().single_mut() {
                text.0 = dialogue_state.lines[dialogue_state.current_line].clone();
            }
    } else {
        // Hide box
        for mut node in dialogue_box.iter_mut() {
            node.display = Display::None;
        }
    }
}

pub fn handle_dialogue_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_dialogue: Option<ResMut<CurrentDialogue>>,
) {
    let Some(mut current_dialogue) = current_dialogue else {
        return;
    };

    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    if let Some(dialogue_state) = &mut current_dialogue.state {
        // Move to next line
        dialogue_state.current_line += 1;

        // Close dialogue if we've reached the end
        if dialogue_state.current_line >= dialogue_state.lines.len() {
            current_dialogue.state = None;
        }
    }
}
