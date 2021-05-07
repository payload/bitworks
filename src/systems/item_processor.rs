use bevy::prelude::*;

use crate::components::*;

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
        processor.process(me, acceptor, ejector, wiring);
    }
}
