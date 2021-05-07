use bevy::prelude::*;

use crate::components::*;
use crate::core::*;
use crate::tools::*;

pub fn cutter4_building(cmds: &mut Commands) {
    use Dir::*;
    let mut processor = ItemProcessor::default();
    processor.label(ProcessorLabel::Cutter4);
    let mut acceptor = ItemAcceptor::default();
    acceptor.slot(1).pos(0, 0).dir(S);
    acceptor.slot(2).pos(1, 0).dir(S);
    let mut ejector = ItemEjector::default();
    ejector.slot(1).pos(0, 1).dir(N);
    ejector.slot(1).pos(1, 1).dir(N);
    ejector.slot(1).pos(2, 1).dir(N);
    ejector.slot(1).pos(3, 1).dir(N);
    let mut wiring = Wiring::default();
    wiring.pin(1).input().pos(0, 0).dir(S);
    wiring.pin(2).output().pos(1, 0).dir(S);

    cmds.spawn_bundle(("cutter4".to_string(), processor, acceptor, ejector, wiring));
}

pub fn cutter4_process(
    _me: Entity,
    _processor: &mut ItemProcessor,
    mut acceptor: Mut<ItemAcceptor>,
    mut ejector: Mut<ItemEjector>,
    mut wiring: Mut<Wiring>,
) -> Option<()> {
    check(ejector.slots().all(ItemEjectorSlot::is_free))?;
    check(wiring.value_pin(1).is_truthy())?;
    check(acceptor.slot(1).has_color())?;
    check(acceptor.slot(2).has_shape())?;

    let color = acceptor.take_slot(1).unwrap();
    let shape = acceptor.take_slot(2).unwrap();
    let shape = color.paint(shape);
    wiring.set_pin(2, shape.signal());

    for (shape, piece) in shape.pieces() {
        match piece {
            Piece::TR => ejector.eject(1, shape),
            Piece::BR => ejector.eject(2, shape),
            Piece::BL => ejector.eject(3, shape),
            Piece::TL => ejector.eject(4, shape),
        }
    }

    Some(())
}
