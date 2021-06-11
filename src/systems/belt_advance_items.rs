use crate::game_types::*;
use bevy::prelude::*;

pub fn belt_advance_items_system(
    mut belts: Query<&mut Belt>,
    mut item_inputs: Query<&mut ItemInput>,
    time: Res<Time>,
) {
    let time = time.delta_seconds();

    for mut belt in belts.iter_mut() {
        let mut output = belt
            .output()
            .and_then(|output| item_inputs.get_mut(output).ok());

        let speed = 10.0;
        let advance = speed * time;

        let total_length = belt.total_length();
        let mut next_stop = if let Some(ref mut output) = output {
            NextStop::Output(output)
        } else {
            NextStop::End(total_length)
        };

        let mut pass_on = 0usize;

        for item in belt.items_mut().rev() {
            let padding = item.padding();
            match next_stop {
                NextStop::End(stop) => {
                    item.pos = stop.min(item.pos + advance);
                    next_stop = NextStop::Item(item.pos - padding);
                }
                NextStop::Item(stop) => {
                    item.pos = (stop - padding).min(item.pos + advance);
                    next_stop = NextStop::Item(item.pos - padding);
                }
                NextStop::Output(ref output) => {
                    if item.pos + advance > total_length {
                        // when item is passed on, item.pos is set to the overflow after total length
                        let space = output.space();
                        if space > 0 {
                            pass_on += 1;
                            item.pos = item.pos + advance - total_length;

                            if space == 1 {
                                next_stop = NextStop::End(total_length);
                            }
                        } else {
                            item.pos = total_length;
                            next_stop = NextStop::Item(item.pos - padding);
                        }
                    } else {
                        item.pos += advance;
                        next_stop = NextStop::Item(item.pos - padding);
                    }
                }
            };
        }

        if pass_on > 0 {
            let mut output = output.expect("only pass on if output exists");
            output.add_items(belt.pass_on(pass_on));
        }
    }
}

enum NextStop<'a> {
    End(f32),
    Item(f32),
    Output(&'a mut ItemInput),
}
