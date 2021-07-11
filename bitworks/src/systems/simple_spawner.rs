use bevy::{math::vec3, prelude::*};

use crate::*;

pub struct SimpleSpawnerPlugin;

impl Plugin for SimpleSpawnerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_to_stage(CoreStage::PreUpdate, simple_spawner_system.system());
    }
}

#[derive(Debug, Clone)]
pub enum Simple {
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
