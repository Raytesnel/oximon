use bevy::prelude::*;
use bevy::asset::AssetServer;
use bevy::color::Color;
use crate::overworld::components::*;

pub fn on_sign_interaction(
    trigger: On<InteractionEvent>,
    query: Query<(&InteractionType, &SignText)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let entity = trigger.event().entity;
    let Ok((InteractionType::Sign, sign_text)) = query.get(entity) else {
        return;
    };

    // Spawn popup as a child — inherits the sign's Transform
    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            SignPopup {
                timer: Timer::from_seconds(3.0, TimerMode::Once),
            },
            Text2d::new(sign_text.0.clone()),
            TextFont {
                font: asset_server.load("fonts/your_font.otf"),
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::WHITE),
            // Float 32px above the sign
            Transform::from_xyz(0.0, 32.0, 5.0),
        ));
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

pub fn on_lamp_interaction(
    trigger: On<InteractionEvent>,
    mut query: Query<(&InteractionType, &mut InteractionState)>,
    children_q: Query<&Children>,
    name_q: Query<&Name>,
    mut sprite_q: Query<&mut Sprite>,
    spritesheet: Option<Res<LampSpriteSheet>>,
    mut commands: Commands,
) {
    let Some(spritesheet) = spritesheet else {
        warn!("LampSpriteSheet resource not loaded yet");
        return;
    };

    let entity = trigger.event().entity;
    let Ok((InteractionType::Lamp, mut state)) = query.get_mut(entity) else {
        return;
    };

    // Find the TiledObjectVisual child that holds the sprite
    let Ok(children) = children_q.get(entity) else {
        return;
    };

    let visual_child = children.iter().find(|child| {
        name_q.get(child.clone().clone())
            .map(|n| n.as_str() == "TiledObjectVisual")
            .unwrap_or(false)
    });

    let Some(visual_entity) = visual_child else {
        warn!("no TiledObjectVisual child found");
        return;
    };

    let Ok(mut sprite) = sprite_q.get_mut(visual_entity.clone()) else {
        warn!("TiledObjectVisual has no Sprite");
        return;
    };

    let frames = match *state {
        InteractionState::Off => {
            *state = InteractionState::On;
            info!("turning ON: lamp");
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        }
        _ => {
            *state = InteractionState::Off;
            info!("turning OFF: lamp");
            vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0]
        }
    };
    info!("inserting frames: {:?}", frames);

    sprite.image = spritesheet.image.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: spritesheet.layout.clone(),
        index: frames[0],
    });

    // Put the animation state on the visual child, not the parent
    // so tick_lamp_animation can find the Sprite easily
    commands.entity(visual_entity.clone()).insert(LampAnimationState {
        timer: Timer::from_seconds(0.08, TimerMode::Repeating),
        frames,
        current: 0,
        hold_on_last: true,
    });

    info!("lamp animation started on visual child {:?}", visual_entity);
}

pub fn load_lamp_spritesheet(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(32, 65),
        10, 1,
        None, None,
    ));
    commands.insert_resource(LampSpriteSheet {
        image: asset_server.load("tiles/dungeon/Sprite-lamp.png"),
        layout,
    });
}
pub fn tick_lamp_animation(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut LampAnimationState, &mut Sprite)>,
) {
    for (entity, mut anim, mut sprite) in &mut query {
        anim.timer.tick(time.delta());
        if !anim.timer.just_finished() {
            continue;
        }

        let next = anim.current + 1;

        if next >= anim.frames.len() {
            // Reached the last frame
            if anim.hold_on_last {
                // Remove the animator — sprite stays on last frame
                commands.entity(entity).remove::<LampAnimationState>();
            } else {
                anim.current = 0;  // loop
            }
        } else {
            anim.current = next;
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = anim.frames[next];
            }
        }
    }
}