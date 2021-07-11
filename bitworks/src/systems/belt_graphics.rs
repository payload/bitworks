use bevy::{math::vec3, prelude::*};

use bevy_prototype_lyon::prelude::Geometry;
use lyon_path::{builder::BorderRadii, traits::PathBuilder};

use crate::{
    lyon_geom, Belt, BeltItem, CompassDir, GetColor, ItemAtlasHandle, LyonBuilder, MultipleOutputs,
    SingleInput,
};

pub struct BeltGraphicsPlugin;

impl Plugin for BeltGraphicsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(belt_sprite_animation_system.system())
            .add_system_to_stage(CoreStage::PreUpdate, draw_belt_system.system());
    }
}

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

#[derive(Clone, Default)]
struct BeltSpriteAnimation {
    row: u32,
    col: u32,
    flip_x: bool,
    flip_y: bool,
    backwards: bool,
}

fn belt_sprite_animation_system(
    mut cmds: Commands,
    mut belts: Query<(
        Entity,
        Option<&BeltSpriteAnimation>,
        &Belt,
        &SingleInput,
        &MultipleOutputs,
        &mut TextureAtlasSprite,
    )>,
    time: Res<Time>,
) {
    use CompassDir::*;
    let time = time.seconds_since_startup();
    let anim_col = (time.fract() * 8.0) as u32;

    for (entity, anim, _belt, input, output, mut sprite) in belts.iter_mut() {
        let mut new_anim: BeltSpriteAnimation = anim.cloned().unwrap_or_else(|| {
            let (row, flip_x, flip_y, backwards) =
                match (input.dir, output.outputs.first().unwrap().dir) {
                    (W, E) => (0, false, false, false),
                    (N, S) => (1, false, true, false),
                    (W, N) => (2, false, false, false),
                    (S, E) => (3, false, false, false),
                    _ => (0, false, false, true),
                };
            BeltSpriteAnimation {
                col: anim_col,
                row,
                flip_x,
                flip_y,
                backwards,
            }
        });

        new_anim.col = anim_col;

        if sprite.flip_x != new_anim.flip_x {
            sprite.flip_x = new_anim.flip_x;
        }
        if sprite.flip_y != new_anim.flip_y {
            sprite.flip_y = new_anim.flip_y;
        }
        let new_index = if new_anim.backwards {
            7 - new_anim.col + new_anim.row * 8
        } else {
            new_anim.col + new_anim.row * 8
        };
        if sprite.index != new_index {
            sprite.index = new_index;
        }

        cmds.entity(entity).insert(new_anim);
    }
}
