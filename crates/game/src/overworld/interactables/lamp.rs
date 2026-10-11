use crate::overworld::components::*;
use bevy::asset::AssetServer;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

type LampQueryItem = (
    &'static InteractionType,
    &'static mut InteractionState,
    Option<&'static SpriteSheetHandle>,
    Option<&'static SpriteSheetProps>,
);

#[derive(SystemParam)]
pub struct LampInteractionContext<'w, 's> {
    pub lamp_q: Query<'w, 's, LampQueryItem>,
    pub children_q: Query<'w, 's, &'static Children>,
    pub name_q: Query<'w, 's, &'static Name>,
    pub sprite_q: Query<'w, 's, &'static mut Sprite>,
}

pub fn on_lamp_interaction(
    trigger: On<InteractionEvent>,
    mut lamp_ctx: LampInteractionContext,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let entity = trigger.event().entity;
    let Ok((InteractionType::Lamp, mut state, maybe_handle, maybe_props)) =
        lamp_ctx.lamp_q.get_mut(entity)
    else {
        return;
    };
    let handle = if let Some(h) = maybe_handle {
        h.clone()
    } else if let Some(props) = maybe_props {
        let layout = layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(props.width, props.height),
            props.columns,
            props.rows,
            None,
            None,
        ));
        let h = SpriteSheetHandle {
            image: asset_server.load(props.path.clone()),
            layout,
        };
        commands.entity(entity).insert(h.clone());
        h
    } else {
        warn!("lamp has no spritesheet props");
        return;
    };
    // Find the TiledObjectVisual child that holds the sprite
    let Ok(children) = lamp_ctx.children_q.get(entity) else {
        return;
    };

    let visual_child = children.iter().find(|child| {
        lamp_ctx
            .name_q
            .get(*child)
            .map(|n| n.as_str() == "TiledObjectVisual")
            .unwrap_or(false)
    });

    let Some(visual_entity) = visual_child else {
        warn!("no TiledObjectVisual child found");
        return;
    };

    let Ok(mut sprite) = lamp_ctx.sprite_q.get_mut(visual_entity) else {
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

    sprite.image = handle.image.clone();
    sprite.texture_atlas = Some(TextureAtlas {
        layout: handle.layout.clone(),
        index: frames[0],
    });

    // Put the animation state on the visual child, not the parent
    // so tick_lamp_animation can find the Sprite easily
    commands.entity(visual_entity).insert(LampAnimationState {
        timer: Timer::from_seconds(0.08, TimerMode::Repeating),
        frames,
        current: 0,
        hold_on_last: true,
    });

    info!("lamp animation started on visual child {:?}", visual_entity);
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
                anim.current = 0; // loop
            }
        } else {
            anim.current = next;
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = anim.frames[next];
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::overworld::components::LampAnimationState;
    use crate::overworld::interactables::tick_lamp_animation;
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    fn make_app_with_time(step_seconds: f32) -> App {
        let mut app = App::new();
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(step_seconds),
        ));
        app.add_plugins(MinimalPlugins);
        app
    }

    fn spawn_lamp_visual(app: &mut App, frames: Vec<usize>) -> Entity {
        app.world_mut()
            .spawn((
                Sprite::default(),
                LampAnimationState {
                    // Use a 1-second frame interval so our 2-second step fires it
                    timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                    frames,
                    current: 0,
                    hold_on_last: true,
                },
            ))
            .id()
    }

    #[test]
    fn lamp_animation_advances_frame() {
        let mut app = make_app_with_time(1.1); // advances past the 1 s frame timer
        app.add_systems(Update, tick_lamp_animation);

        let entity = spawn_lamp_visual(&mut app, vec![0, 1, 2, 3]);
        for _ in 0..5 {
            app.update();
        }
        let anim = app.world().get::<LampAnimationState>(entity).unwrap();
        assert_eq!(anim.current, 1, "frame should have advanced to index 1");
    }

    #[test]
    fn lamp_animation_hold_on_last_removes_component() {
        let mut app = make_app_with_time(1.1);
        app.add_systems(Update, tick_lamp_animation);

        let entity = spawn_lamp_visual(&mut app, vec![0, 1]);
        for _ in 0..15 {
            app.update();
        }

        assert!(
            app.world().get::<LampAnimationState>(entity).is_none(),
            "LampAnimationState should be removed on last frame with hold_on_last"
        );
    }
}
