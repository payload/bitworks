use bevy::{
    diagnostic::{DiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::vec3,
    prelude::*,
    utils::HashSet,
};

use bevy_inspector_egui::{InspectableRegistry, WorldInspectorPlugin};
use bevy_prototype_lyon::prelude::Geometry;
use bevy_rapier2d::prelude::*;

use lyon_path::{builder::BorderRadii, traits::PathBuilder};

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
        .add_system_to_stage(CoreStage::PreUpdate, draw_belt_system.system())
        .add_system_to_stage(CoreStage::PreUpdate, simple_spawner_system.system())
        .add_plugin(BeltInputOutputHookupPlugin)
        .add_plugin(BeltPlugin)
        .add_plugin(BeltSpriteAnimationPlugin);

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

struct BeltInputOutputHookupPlugin;

impl Plugin for BeltInputOutputHookupPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(
            CoreStage::PreUpdate,
            input_output_hookup_system.system().label("io_hookup"),
        )
        .add_system_to_stage(
            CoreStage::PreUpdate,
            output_item_stuff_hookup_system.system().after("io_hookup"),
        );
    }
}

#[derive(Default)]
struct DebugIOHookupSystem {
    hookup: Vec<(Entity, Entity)>,
    wrong_dir: Vec<(Entity, CompassDir, CompassDir)>,
    no_input: Vec<(Entity, MapPos)>,
    none_at: Vec<(Entity, MapPos)>,
    filter_set: HashSet<Entity>,
}

impl DebugIOHookupSystem {
    fn print(&mut self) {
        let set = &mut self.filter_set;
        for it in self.hookup.iter().filter(|e| set.insert(e.0)) {
            debug!("check {:?} hookup {:?}", it.0, it.1);
        }
        for it in self.wrong_dir.iter().filter(|e| set.insert(e.0)) {
            debug!("check {:?} wrong dir {:?} to {:?}", it.0, it.1, it.2);
        }
        for it in self.no_input.iter().filter(|e| set.insert(e.0)) {
            debug!("check {:?} no input {:?}", it.0, it.1);
        }
        for it in self.none_at.iter().filter(|e| set.insert(e.0)) {
            debug!("check {:?} none at {:?}", it.0, it.1);
        }
    }
}

fn input_output_hookup_system(
    inputs: Query<(&MapPos, &SingleInput)>,
    mut outputs: Query<(Entity, &MapPos, &mut MultipleOutputs)>,
    map: Res<MapCache>,
    mut debug: Local<DebugIOHookupSystem>,
) {
    for (o_entity, pos, mut outputs) in outputs.iter_mut() {
        // NOTE to make sure not to trigger unnecessary change detection
        //      this loop uses an index range to not trigger a mut deref of Mut<> prematurely
        for i in 0..outputs.outputs.len() {
            let output = &outputs.outputs[i];
            let other_pos = (*pos + output.pos).step(output.dir);

            if output.entity.is_some() {
                continue;
            } else if let Some(input_entity) = map.at(&other_pos) {
                if let Some(input) = inputs.get_component::<SingleInput>(input_entity).ok() {
                    if input.dir == output.dir.opposite() {
                        outputs.outputs[i].entity = Some(input_entity);

                        debug.hookup.push((o_entity, input_entity));
                    } else {
                        debug.wrong_dir.push((o_entity, output.dir, input.dir));
                    }
                } else {
                    debug.no_input.push((o_entity, other_pos));
                }
            } else {
                debug.none_at.push((o_entity, other_pos));
            }
        }
    }

    debug.print();
}

fn output_item_stuff_hookup_system(
    mut entities: Query<
        (
            (Entity, &MultipleOutputs),
            (
                Option<&mut Merger>,
                Option<&mut RandomItemGenerator>,
                Option<&mut Belt>,
            ),
        ),
        Changed<MultipleOutputs>,
    >,
) {
    for ((_entity, outputs), it) in entities.iter_mut() {
        if let Some(mut merger) = it.0 {
            let len = outputs.outputs.len();
            merger.outputs.resize(len, Entity::new(0));
            for i in 0..len {
                if let Some(e) = outputs.outputs[i].entity {
                    merger.outputs[i] = e;
                }
            }
            // debug!("output  {:?} set to {:?}", entity, merger.outputs);
        } else if let Some(mut item_gen) = it.1 {
            item_gen.output = outputs.outputs[0].entity;
            // debug!("output  {:?} set to {:?}", entity, item_gen.output);
        } else if let Some(mut belt) = it.2 {
            belt.output = outputs.outputs[0].entity;
            // debug!("output  {:?} set to {:?}", entity, belt.output);
        }
    }
}

//////

#[derive(Default)]
struct DrawItems {
    entities: Vec<Entity>,
}

fn draw_belt_system(
    belts: Query<&Belt>,
    mut sprite: Query<(&mut Transform, &mut TextureAtlasSprite)>,
    mut draw_items: Local<DrawItems>,
    mut cmds: Commands,
    item_atlas: Res<ItemAtlasHandle>,
) {
    let mut index = 0;

    for belt in belts.iter() {
        for item in belt.items() {
            let item: &BeltItem = item;
            let (pos, _dir) = belt.location_on_path(item.pos) as (Vec3, Vec3);
            let pos = vec3(pos.x, pos.y, 0.1);

            if index >= draw_items.entities.len() {
                let entity = cmds
                    .spawn_bundle(SpriteSheetBundle {
                        texture_atlas: item_atlas.clone(),
                        transform: Transform::from_translation(pos),
                        visible: Visible {
                            is_visible: true,
                            is_transparent: true,
                        },
                        sprite: TextureAtlasSprite {
                            index: 0,
                            color: item.color(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .id();
                draw_items.entities.push(entity);
            } else if let Ok((mut trans, mut sprite)) = sprite.get_mut(draw_items.entities[index]) {
                if sprite.color != item.color() {
                    sprite.color = item.color();
                }
                if trans.translation != pos {
                    trans.translation = pos;
                }
            }

            index += 1;
        }
    }
}

struct ItemBubble;
impl Geometry for ItemBubble {
    fn add_geometry(&self, b: &mut LyonBuilder) {
        b.add_rounded_rectangle(
            &lyon_geom::rect(-2.0, -2.0, 4.0, 4.0),
            &BorderRadii::new(1.0),
            lyon_path::Winding::Positive,
        )
    }
}
