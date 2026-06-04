use crate::movement::input::{MOVE_LEFT_BUTTON, MOVE_RIGHT_BUTTON, MOVE_UP_BUTTON};
use crate::overworld::{
    components::*,
    input_systems::{interaction_input_system, overworld_movement},
    interactables::*,
    player_movement::{update_facing, y_sort},
};
use avian2d::prelude::{CollidingEntities, LinearVelocity};
use bevy::input::InputPlugin;
use bevy::prelude::*;

fn make_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin));
    app
}

#[test]
fn walking_behind_object_gets_lower_z_value() {
    let mut app = make_app();
    app.add_systems(Update, y_sort);

    // Player is high on screen (y=200) — "behind" the tree below them.
    let player = app
        .world_mut()
        .spawn((OverworldPlayer, YSort, Transform::from_xyz(0.0, 200.0, 0.0)))
        .id();

    // Tree is lower on screen (y=50) — should render in front.
    let tree = app
        .world_mut()
        .spawn((YSort, Transform::from_xyz(0.0, 50.0, 0.0)))
        .id();

    app.update();

    let player_z = app.world().get::<Transform>(player).unwrap().translation.z;
    let tree_z = app.world().get::<Transform>(tree).unwrap().translation.z;

    assert!(
        player_z < tree_z,
        "player at y=200 (behind tree) must have z={player_z} < tree z={tree_z}"
    );
}

#[test]
fn moving_player_down_increases_z_relative_to_static_object() {
    // Simulates the player walking "in front of" an object.
    let mut app = make_app();
    app.add_systems(Update, y_sort);

    let player = app
        .world_mut()
        .spawn((YSort, Transform::from_xyz(0.0, 100.0, 0.0)))
        .id();

    let npc = app
        .world_mut()
        .spawn((YSort, Transform::from_xyz(0.0, 120.0, 0.0)))
        .id();

    app.update();

    let player_z_far = app.world().get::<Transform>(player).unwrap().translation.z;
    let npc_z = app.world().get::<Transform>(npc).unwrap().translation.z;
    assert!(
        player_z_far > npc_z,
        "player below npc should have higher z (in front)"
    );

    // Now move player above the npc.
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation
        .y = 140.0;

    app.update();

    let player_z_behind = app.world().get::<Transform>(player).unwrap().translation.z;
    assert!(
        player_z_behind < npc_z,
        "player above npc should have lower z (behind)"
    );
}

#[test]
fn interacting_with_sign_spawns_popup_child() {
    let mut app = make_app();
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::text::Font>();
    app.add_observer(on_sign_interaction);
    app.add_systems(Update, tick_sign_popups);

    let sign = app
        .world_mut()
        .spawn((
            InteractionType::Sign,
            SignText("Hello".to_string()),
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    // Fire the interaction event.
    app.world_mut()
        .commands()
        .trigger(InteractionEvent { entity: sign });
    app.update();

    // The sign should now have a child with SignPopup.
    let children = app
        .world()
        .get::<Children>(sign)
        .expect("sign must have children after interaction");

    let has_popup = children
        .iter()
        .any(|c| app.world().get::<SignPopup>(c).is_some());

    assert!(
        has_popup,
        "sign should have a SignPopup child after interaction"
    );
}

#[test]
fn sign_popup_despawns_after_three_seconds() {
    let mut app = make_app();
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<bevy::text::Font>();
    app.add_observer(on_sign_interaction);
    app.add_systems(Update, tick_sign_popups);

    let sign = app
        .world_mut()
        .spawn((
            InteractionType::Sign,
            SignText("Hello".to_string()),
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();

    // immediate trigger
    app.world_mut().trigger(InteractionEvent { entity: sign });
    app.update(); // observer runs, children spawned
    app.update(); // children flushed into world

    // verify popup exists before testing despawn
    let popup = app
        .world()
        .get::<Children>(sign)
        .expect("sign must have children")
        .iter()
        .find(|c| app.world().get::<SignPopup>(*c).is_some())
        .expect("must have a SignPopup child");

    // use a short timer instead of fighting the time issue
    // OR just do enough ticks — at ~0.25s each, 25 ticks = ~6.25s > 3s
    // but first check the popup actually has time ticking:
    let remaining = app
        .world()
        .get::<SignPopup>(popup)
        .unwrap()
        .timer
        .remaining_secs();
    println!("remaining before loop: {remaining}");

    for _ in 0..15 {
        if app.world().get_entity(popup).is_err() {
            break; // already despawned, no need to continue
        }
        app.world_mut()
            .get_mut::<SignPopup>(popup)
            .unwrap()
            .timer
            .tick(std::time::Duration::from_secs_f32(0.5));
        app.update();
    }

    let remaining_after = app
        .world()
        .get::<SignPopup>(popup)
        .map(|p| p.timer.remaining_secs());
    println!("remaining after loop: {remaining_after:?}");

    assert!(
        app.world().get_entity(popup).is_err(),
        "SignPopup must despawn"
    );
}

#[test]
fn lamp_toggles_off_to_on() {
    let mut app = make_app();
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.add_observer(on_lamp_interaction);
    app.init_resource::<Assets<TextureAtlasLayout>>();
    app.update();

    let lamp = app
        .world_mut()
        .spawn((
            InteractionType::Lamp,
            InteractionState::On,
            SpriteSheetHandle {
                image: Handle::default(),
                layout: Handle::default(),
            },
            // Also need a TiledObjectVisual child with a Sprite
        ))
        .id();

    app.world_mut()
        .spawn((Name::new("TiledObjectVisual"), Sprite::default()))
        .set_parent_in_place(lamp);

    app.update();

    let state = app.world().get::<InteractionState>(lamp).unwrap();
    assert_eq!(*state, InteractionState::On);
}

#[test]
fn lamp_toggles_on_to_off() {
    let mut app = make_app();
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.add_observer(on_lamp_interaction);
    app.init_resource::<Assets<TextureAtlasLayout>>();

    let lamp = app
        .world_mut()
        .spawn((
            InteractionType::Lamp,
            InteractionState::Off,
            SpriteSheetHandle {
                image: Handle::default(),
                layout: Handle::default(),
            },
        ))
        .id();

    app.world_mut()
        .spawn((Name::new("TiledObjectVisual"), Sprite::default()))
        .set_parent_in_place(lamp);
    app.update();

    let state = app.world().get::<InteractionState>(lamp).unwrap();
    assert_eq!(*state, InteractionState::Off);
}

#[test]
fn pushing_block_right_sets_sliding_target_to_the_right() {
    let mut app = make_app();
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.add_observer(on_block_interaction);
    app.init_resource::<Assets<TextureAtlasLayout>>();

    let player = app
        .world_mut()
        .spawn((
            OverworldPlayer,
            Facing::Right,
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    let block_pos = Vec2::new(32.0, 0.0);
    let block = app
        .world_mut()
        .spawn((
            InteractionType::Block,
            PushableBlock { grid_size: 32.0 },
            Transform::from_xyz(block_pos.x, block_pos.y, 0.0),
            GlobalTransform::default(),
        ))
        .id();

    let _ = player;

    app.world_mut()
        .commands()
        .trigger(InteractionEvent { entity: block });
    app.update();

    let sliding = app
        .world()
        .get::<BlockSliding>(block)
        .expect("block should be sliding after interaction");

    assert!(
        sliding.to.x > block_pos.x,
        "block pushed right must have to.x > {}, got {}",
        block_pos.x,
        sliding.to.x
    );
    assert_eq!(
        sliding.to.x,
        block_pos.x + 32.0,
        "block should slide exactly one grid cell (32px)"
    );
}

#[test]
fn pushing_block_that_is_already_sliding_does_nothing() {
    let mut app = make_app();
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.add_observer(on_block_interaction);
    app.init_resource::<Assets<TextureAtlasLayout>>();

    app.world_mut().spawn((
        OverworldPlayer,
        Facing::Right,
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    let block = app
        .world_mut()
        .spawn((
            InteractionType::Block,
            PushableBlock { grid_size: 32.0 },
            Transform::from_xyz(32.0, 0.0, 0.0),
            GlobalTransform::default(),
            BlockSliding {
                from: Vec2::new(32.0, 0.0),
                to: Vec2::new(64.0, 0.0),
                timer: Timer::from_seconds(1.0, TimerMode::Once),
            },
        ))
        .id();

    // Try pushing again — should be ignored.
    app.world_mut()
        .commands()
        .trigger(InteractionEvent { entity: block });
    app.update();

    // The timer should not have been reset (from unchanged at 1.0 s).
    let sliding = app.world().get::<BlockSliding>(block).unwrap();
    assert_eq!(
        sliding.from,
        Vec2::new(32.0, 0.0),
        "existing slide must not be replaced"
    );
}

#[derive(Resource, Default)]
struct EventLog(Vec<Entity>);

#[test]
fn e_press_outside_field_fires_no_event() {
    let mut app = make_app();
    app.add_systems(Update, interaction_input_system);
    app.init_resource::<EventLog>();
    app.add_observer(|trigger: On<InteractionEvent>, mut log: ResMut<EventLog>| {
        log.0.push(trigger.event().entity);
    });

    let player = app
        .world_mut()
        .spawn((OverworldPlayer, Transform::default()))
        .id();

    // Field that does NOT contain the player.
    let owner = app.world_mut().spawn_empty().id();
    app.world_mut().spawn((
        InteractionFieldMarker,
        InteractionField { owner },
        CollidingEntities::default(),
    ));

    let _ = player;

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyE);

    app.update();

    assert!(
        app.world().resource::<EventLog>().0.is_empty(),
        "no interaction event should fire when player is outside all fields"
    );
}
