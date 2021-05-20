use bevy::prelude::*;

use crate::{buildings::*, components::*};

/**
    Moves items from acceptor through processor to ejector.
*/
pub fn item_processor_system(
    mut query: Query<(
        Entity,
        &mut ItemProcessor,
        &mut ItemAcceptor,
        &mut ItemEjector,
        &mut Wiring,
    )>,
) {
    for (me, mut processor, acceptor, ejector, wiring) in query.iter_mut() {
        let charge = match processor.label {
            ProcessorLabel::Paintcutter => paintcutter_process(me, processor, acceptor, wiring),
            ProcessorLabel::Mixer => None,
            ProcessorLabel::None => None,
        };
        
    }
}

fn foo_system(
    query: Query<Entity, With<(ItemAcceptor, ItemEjector)>>,
    mut query: Query<&mut ItemAcceptor>,
    mut query: Query<&mut ItemEjector>,
) {}