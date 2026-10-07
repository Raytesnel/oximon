use crate::overworld::components::*;
use crate::overworld::dialogs::{CurrentDialogue, DialogueState};
use bevy::prelude::*;

#[allow(dead_code)]
pub fn on_npc_interaction(
    trigger: On<InteractionEvent>,
    query: Query<(&InteractionType, &NpcDialogue)>,
    mut current_dialogue: ResMut<CurrentDialogue>,
) {
    let entity = trigger.event().entity;
    let Ok((InteractionType::NPC, npc_dialogue)) = query.get(entity) else {
        return;
    };

    current_dialogue.state = Some(DialogueState {
        lines: npc_dialogue.lines.clone(), // Vec<String>
        current_line: 0,
        speaker_name: npc_dialogue.name.clone(),
    });
}
