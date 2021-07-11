use bevy::prelude::*;

use crate::{Belt, CompassDir, MultipleOutputs, SingleInput};

pub struct BeltSpriteAnimationPlugin;

impl Plugin for BeltSpriteAnimationPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(belt_sprite_animation_system.system());
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
