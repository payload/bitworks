use bevy::{
    diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};

use bevy_inspector_egui::{InspectableRegistry, WorldInspectorPlugin};
use bevy_rapier2d::prelude::*;

use bitworks::*;

fn main() {
    belts_example_app().run();
}

pub fn belts_example_app() -> AppBuilder {
    let config = Config::from_ron("config.ron").expect("config.ron");

    let mut app = App::build();

    if config.log_diagnostics {
        app.add_plugin(LogDiagnosticsPlugin::default());
    }

    app.add_state(AppState::GamePaused)
        .add_plugins(DefaultPlugins)
        .add_plugin(DiagnosticsPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(DebugLinesPlugin)
        .add_plugin(BeltDebugPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(MapPlugin)
        .add_plugin(AssetsPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(WasdPlayerMovementPlugin)
        .add_plugin(SetupPlugin)
        .add_plugin(SimpleSpawnerPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(GameStatePlugin)
        .add_plugin(BeltInputOutputHookupPlugin)
        .add_plugin(BeltPlugin)
        .add_plugin(BeltGraphicsPlugin);

    let mut registry = app
        .world_mut()
        .get_resource_or_insert_with(InspectableRegistry::default);
    registry.register::<RandomItemGenerator>();
    registry.register::<MapPos>();

    app
}

struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup.system())
            .add_startup_system(setup_rapier.system())
            .add_startup_system(spawn_player.system());
    }
}

fn setup(mut cmds: Commands) {
    let cmds = &mut cmds;

    spawn_3d_orbit_camera(cmds);

    use CompassDir::*;
    for simple in vec![
        Simple::ItemGenerator(map_pos(-3, 2), E),
        Simple::Belt(map_pos(-2, 2), W, E),
        Simple::Belt(map_pos(-1, 2), W, E),
        //
        Simple::ItemGenerator(map_pos(-3, 0), E),
        Simple::Belt(map_pos(-2, 0), W, N),
        Simple::Belt(map_pos(-2, 1), S, E),
        Simple::Belt(map_pos(-1, 1), W, E),
        //
        Simple::Merger2x2(map_pos(0, 2), E),
        //
        Simple::Belt(map_pos(1, 2), W, E),
        Simple::Belt(map_pos(1, 1), W, E),
        Simple::NullSink(map_pos(2, 2), W),
    ] {
        cmds.spawn_bundle((simple,));
    }
}

fn setup_rapier(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.gravity = Default::default();
    rapier_config.scale = TILE_SIZE;
}

fn spawn_player(
    mut cmds: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    rapier_config: Res<RapierConfiguration>,
) {
    let sprite_size = vec2(0.75 * TILE_SIZE, 0.75 * TILE_SIZE);

    cmds.spawn_bundle(SpriteBundle {
        material: materials.add(ColorMaterial {
            color: Color::rgb(0.4, 0.4, 0.9),
            texture: None,
        }),
        sprite: Sprite::new(sprite_size),
        transform: Transform::from_xyz(0.0, 0.0, 10.0),
        ..Default::default()
    })
    .wasd_player_movement_insert_default_rb_collider(sprite_size, &rapier_config)
    .insert(WasdPlayerMovment {
        velocity: 6.0 * sprite_size.x,
    });
}
