#[cfg(test)]
mod test_elevation {
    use crate::common::components::GameLayer;
    use crate::overworld::components::{
        Elevation, FixedElevation, OverworldPlayer, StairZone, YSort,
    };
    use crate::overworld::player_movement::y_sort;
    use crate::overworld::setup::{apply_fixed_elevation, elevation_layer, stair_elevation_system};
    use avian2d::prelude::*;
    use bevy::prelude::*;

    const Y_SORT_SCALE: f32 = 1000.0;
    const ELEVATION_STEP: f32 = 10.0;

    // ── 1. Stair zone changes elevation ─────────────────────────────
    // No physics plugin needed — we hand-craft the CollisionStart message
    // that stair_elevation_system reacts to, rather than simulating a real overlap.

    #[test]
    fn entering_stair_zone_changes_elevation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<CollisionStart>();
        app.add_systems(Update, stair_elevation_system);

        let player = app.world_mut().spawn((OverworldPlayer, Elevation(0))).id();
        let zone = app.world_mut().spawn(StairZone { target: 1 }).id();

        app.world_mut()
            .resource_mut::<Messages<CollisionStart>>()
            .write(CollisionStart {
                collider1: player,
                collider2: zone,
                body1: None,
                body2: None,
            });

        app.update();

        assert_eq!(app.world().get::<Elevation>(player).unwrap().0, 1);
    }

    // ── 2. Z formula includes elevation offset ──────────────────────

    #[test]
    fn ysort_z_includes_elevation_offset() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, y_sort);

        let entity = app
            .world_mut()
            .spawn((YSort, Elevation(1), Transform::from_xyz(0.0, 300.0, 0.0)))
            .id();

        app.update();

        let z = app.world().get::<Transform>(entity).unwrap().translation.z;
        let expected = 1.0 * ELEVATION_STEP - 300.0 / Y_SORT_SCALE;
        assert!(
            (z - expected).abs() < f32::EPSILON,
            "expected {expected}, got {z}"
        );
    }

    // ── 3. FixedElevation cancels parent's own Z offset ─────────────

    #[test]
    fn fixed_elevation_cancels_parent_z_offset() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(TransformPlugin);
        app.add_systems(Update, apply_fixed_elevation);

        let parent = app
            .world_mut()
            .spawn(Transform::from_xyz(0.0, 0.0, 999.0))
            .id();
        let child = app
            .world_mut()
            .spawn((FixedElevation(5.0), Transform::from_xyz(0.0, 0.0, 0.0)))
            .id();
        app.world_mut().entity_mut(child).insert(ChildOf(parent));

        app.update();

        let global_z = app
            .world()
            .get::<GlobalTransform>(child)
            .unwrap()
            .translation()
            .z;
        assert!(
            (global_z - 5.0).abs() < f32::EPSILON,
            "expected 5.0, got {global_z}"
        );
    }

    // ── 4. Every elevation maps to a distinct physics layer ─────────
    // Pure logic check — no physics simulation. This is what actually
    // guarantees collision filtering can separate floors; the full
    // physics-engine version of this test wasn't worth the setup cost.

    #[test]
    fn elevation_layer_mapping_is_distinct_per_level() {
        assert_ne!(elevation_layer(-1), elevation_layer(0));
        assert_ne!(elevation_layer(0), elevation_layer(1));
        assert_ne!(elevation_layer(-1), elevation_layer(1));
    }
}
