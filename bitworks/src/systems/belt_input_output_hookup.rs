use bevy::prelude::*;

use crate::{CompassDir, MapPos};

pub struct SingleInput {
    pub pos: MapPos,
    pub dir: CompassDir,
}

pub fn input(pos: MapPos, dir: CompassDir) -> SingleInput {
    SingleInput { pos, dir }
}

pub struct SingleOutput {
    pub pos: MapPos,
    pub dir: CompassDir,
    pub entity: Option<Entity>,
}

pub struct MultipleOutputs {
    pub outputs: Vec<SingleOutput>,
}

impl MultipleOutputs {
    fn new(entries: &[(MapPos, CompassDir)]) -> Self {
        Self {
            outputs: entries
                .iter()
                .map(|(pos, dir)| SingleOutput {
                    pos: *pos,
                    dir: *dir,
                    entity: None,
                })
                .collect(),
        }
    }
}

pub fn output<P: Into<MapPos>>(pos: P, dir: CompassDir) -> MultipleOutputs {
    MultipleOutputs {
        outputs: vec![SingleOutput {
            pos: pos.into(),
            dir,
            entity: None,
        }],
    }
}

pub fn outputs(entries: &[(MapPos, CompassDir)]) -> MultipleOutputs {
    MultipleOutputs::new(entries)
}
