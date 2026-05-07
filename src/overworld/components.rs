use bevy::prelude::*;
#[derive(Resource)]
pub struct LampSpriteSheet {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(Component)]
pub struct LampAnimationState {
    pub timer: Timer,
    pub frames: Vec<usize>,      // frame indices to play
    pub current: usize,          // index into frames vec
    pub hold_on_last: bool,      // true = stop at last frame, false = loop
}

#[derive(Component)]
pub struct OverworldEntity;

#[derive(Component)]
pub struct Interactable;

#[derive(Component)]
pub enum InteractionType {
    Chest,
    Lamp,
    Sign,
}

#[derive(Component, PartialEq)]
pub enum InteractionState {
    Closed,
    Open,
    Off,
    On,
}

#[derive(Component)]
pub struct InteractionField {
    pub owner: Entity,
}

#[derive(Component)]
pub struct InteractionFieldMarker;

#[derive(Component)]
pub struct SignText(pub String);

#[derive(Event,Clone)]
pub struct InteractionEvent {
    pub entity: Entity,
}

#[derive(Component, Clone, Copy)]
pub enum Facing {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component)]
pub struct OverworldPlayer;

#[derive(Component)]
pub struct YSort;

#[derive(Component)]
pub struct SignPopup {
    pub timer: Timer,
}