use bevy::prelude::*;

use crate::components::*;
use crate::core::*;

#[macro_use]
use crate::tools::*;

pub fn paintcutter_building() -> (String, ItemProcessor, ItemAcceptor, ItemEjector, Wiring, Paintcutter) {
    use Dir::*;
    let mut processor = ItemProcessor::default();
    processor.label(ProcessorLabel::Paintcutter);
    let mut acceptor = ItemAcceptor::default();
    acceptor.slot(1).pos(0, 0).dir(W).filter(ItemFilter::Color);
    acceptor.slot(2).pos(1, 0).dir(W).filter(ItemFilter::Shape);
    let mut ejector = ItemEjector::default();
    ejector.slot(1).pos(0, 1).dir(E);
    ejector.slot(1).pos(1, 1).dir(E);
    ejector.slot(1).pos(2, 1).dir(E);
    ejector.slot(1).pos(3, 1).dir(E);
    let mut wiring = Wiring::default();
    wiring.pin(1).input().pos(0, 0).dir(W);
    wiring.pin(2).output().pos(1, 0).dir(W);

    ("cutter4".to_string(), processor, acceptor, ejector, wiring, Paintcutter { charges: vec!() })
}

pub fn paintcutter_accept_charge(processor: &mut ItemProcessor, acceptor: &ItemAcceptor, ejector: &ItemEjector, wiring: &Wiring) -> bool {
    processor.is_free() &&
    acceptor.slot(1).is_color() &&
    acceptor.slot(2).is_shape() &&
    wiring.value_pin(1).is_truthy()
}

pub fn paintcutter_process(
    processor: &mut ItemProcessor,
    acceptor: &mut ItemAcceptor,
    wiring: &mut Wiring,
) -> Option<Charge> {
    let color = acceptor.take_slot(1)?;
    let shape = acceptor.take_slot(2)?;
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

struct Paintcutter {
    charges: Vec<Charge>,
}

struct Charge {
    items: Vec<(usize, Item)>,
}

pub fn paintcutter_eject(paintcutter: Paintcutter, ejector: &mut ItemEjector) {
    if let Some(charge) = paintcutter.charges.get_mut(0) {
        for (slot, item) in charge
            .items
            .drain_filter(|(slot, item)| ejector.slot(slot).is_free())
        {
            ejector.eject(slot, item);
        }

        if charge.items.is_empty() {
            paintcutter.charges.remove(0);
        }
    }
}

#[test]
fn example() {
    let mut bundle = paintcutter_building();

    if bundle.5.charges.len() < 2 {

        let charge = paintcutter_process(&mut bundle.1, &mut bundle.2, &mut bundle.4);
        
        if let Some(charge) = charge {
            bundle.5.charges.push(charge);
        }

    }
}