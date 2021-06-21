use bevy::{prelude::*, utils::HashMap};

use crate::components::*;
use crate::core::*;

/**
    Moves items from ejector of one building to acceptor of another.
    Acceptor filter is used to decide.
*/
pub fn item_ejector_system(
    mut ejectors: Query<(Entity, &mut ItemEjector, &Pos)>,
    mut acceptors: Query<(Entity, &mut ItemAcceptor, &Pos)>,
    mut processor: Query<&mut ItemProcessor>,
) {
    let mut e_map: HashMap<Pos, (Entity, usize)> = HashMap::default();
    for (e, ejector, pos) in ejectors.iter_mut() {
        for (slot_index, slot) in ejector.slots() {
            let slot_pos = slot.get_pos().add(pos);
            e_map.insert(slot_pos, (e, *slot_index));
        }
    }

    for (_, mut acceptor, a_pos) in acceptors.iter_mut() {
        for (_a_index, mut a_slot) in acceptor.slots_mut() {
            if a_slot.is_free() {
                let e_pos = a_slot.dir.pos(&a_slot.pos.add(a_pos));

                if let Some((e, e_index)) = e_map.get(&e_pos) {
                    if let Ok((_, mut ejector, _)) = ejectors.get_mut(*e) {
                        let e_slot = ejector.slot(*e_index);

                        if let Some(e_item) = e_slot.item() {
                            if a_slot.filter.matches(e_item) {
                                let item = e_slot.items.remove(0); // e_slot is borrows immutable and here it is mutated?!
                                a_slot.items.push(item);
                            }
                        }
                    }
                }
            }
        }
    }
}