use bevy::{
    diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::vec3,
    prelude::*,
};

use bevy_inspector_egui::{InspectableRegistry, WorldInspectorPlugin};
use bevy_rapier2d::prelude::*;

use bitworks::*;

const TILE_SIZE: f32 = 48.0;
const TILE_HALFSIZE: f32 = 24.0;

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
        .add_plugin(DebugPlugin)
        .add_plugin(LyonPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(MapPlugin)
        .add_plugin(AssetsPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(WasdPlayerMovementPlugin)
        .add_plugin(SetupPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(GameStatePlugin)
        .add_system_to_stage(CoreStage::PreUpdate, simple_spawner_system.system())
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

struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(setup.system())
            .add_startup_system(setup_rapier.system())
            .add_startup_system(spawn_player.system());
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

#[derive(Debug, Clone)]
enum Simple {
    /// pos, out direction
    ItemGenerator(MapPos, CompassDir),
    /// pos, in direction, out direction
    Belt(MapPos, CompassDir, CompassDir),
    /// pos, in direction
    NullSink(MapPos, CompassDir),
    // pos of cell 1, output direction, pos of cell 2 is right of output direction
    Merger2x2(MapPos, CompassDir),
}

// fn simple_spawner_system(simples: Query<(Entity, &Simple), Added<Simple>>, mut cmds: Commands) {
fn simple_spawner_system(
    simples: Query<(Entity, &Simple), Added<Simple>>,
    mut cmds: Commands,
    belt_atlas: Res<BeltAtlasHandle>,
) {
    for (entity, simple) in simples.iter() {
        cmds.entity(entity).remove::<Simple>();

        match simple {
            Simple::ItemGenerator(pos, out_dir) => {
                cmds.entity(entity)
                    .insert(Name::new("ItemGenerator"))
                    .insert(*pos)
                    .insert(RandomItemGenerator {
                        cooldown: 0.0,
                        next_time: 1.0,
                        output: None,
                    })
                    .insert(output((0, 0), *out_dir))
                    .insert_bundle(lyon().polygon(6, TILE_HALFSIZE).outlined(
                        Color::TEAL,
                        Color::BLACK,
                        4.0,
                    ));
            }
            Simple::Belt(pos, in_dir, out_dir) => {
                let pos_vec = pos.vec2();
                let in_vec = 0.5 * in_dir.vec2();
                let out_vec = 0.5 * out_dir.vec2();
                let start = TILE_SIZE * vec3(pos_vec.x + in_vec.x, pos_vec.y + in_vec.y, 0.0);
                let end = TILE_SIZE * vec3(pos_vec.x + out_vec.x, pos_vec.y + out_vec.y, 0.0);
                let segment = BeltSegment { start, end };

                cmds.entity(entity)
                    .insert(Name::new("Belt"))
                    .insert(*pos)
                    .insert(Belt {
                        segments: vec![segment],
                        items: vec![],
                        output: None,
                    })
                    .insert(ItemInput::new(2))
                    .insert(input(map_pos(0, 0), *in_dir))
                    .insert(output((0, 0), *out_dir))
                    .insert_bundle(SpriteSheetBundle {
                        sprite: TextureAtlasSprite::new(0),
                        texture_atlas: belt_atlas.clone(),
                        ..Default::default()
                    });
            }
            Simple::NullSink(pos, in_dir) => {
                cmds.entity(entity)
                    .insert(Name::new("NullSink"))
                    .insert(pos.clone())
                    .insert(NullSink::new(&[entity]))
                    .insert(ItemInput::new(2))
                    .insert(input(map_pos(0, 0), *in_dir))
                    .insert_bundle(lyon().circle(TILE_HALFSIZE).outlined(
                        Color::RED,
                        Color::BLACK,
                        4.0,
                    ));
            }
            Simple::Merger2x2(pos1, out_dir) => {
                let pos1 = *pos1;
                let out_dir = *out_dir;
                let in_dir = out_dir.opposite();
                let pos2 = pos1.step(out_dir.right());
                let right = map_pos(0, 0).step(out_dir.right());

                let in1 = cmds
                    .spawn()
                    .insert(pos1)
                    .insert(ItemInput::new(2))
                    .insert(input(map_pos(0, 0), in_dir))
                    .id();
                let in2 = cmds
                    .spawn()
                    .insert(pos2)
                    .insert(ItemInput::new(2))
                    .insert(input(right, in_dir))
                    .id();
                cmds.entity(entity)
                    .insert(Name::new("Merger"))
                    .insert(pos1)
                    .insert(Transform::default())
                    .insert(GlobalTransform::default())
                    .insert(Merger {
                        cooldown: 0.0,
                        next_time: 0.0,
                        items_per_step: 1,
                        input_cursor: 0,
                        output_cursor: 0,
                        inputs: vec![in1, in2],
                        outputs: vec![],
                    })
                    .insert(outputs(&[
                        (map_pos(0, 0), out_dir),
                        (map_pos(0, -1), out_dir),
                    ]))
                    .with_children(|child| {
                        child.spawn().insert_bundle(
                            lyon().rectangle(TILE_SIZE, 2.0 * TILE_SIZE).outlined_pos(
                                Color::DARK_GRAY,
                                Color::BLACK,
                                4.0,
                                vec2(-TILE_HALFSIZE, TILE_HALFSIZE),
                            ),
                        );
                    });
            }
        }
    }
}
