use bevy::{
    input::{keyboard::KeyboardInput, ElementState},
    prelude::*,
};

pub fn show_metrics_system(mut keyboard_input_events: EventReader<KeyboardInput>) {
    for event in keyboard_input_events.iter() {
        if let Some(key_code) = event.key_code {
            if event.state == ElementState::Pressed && key_code == KeyCode::M {
                let mfamilies = prometheus::default_registry().gather();
                println!();
                for mfamily in mfamilies {
                    print!("{} ", mfamily.get_name());
                    let mtype = mfamily.get_field_type();
                    for m in mfamily.get_metric() {
                        match mtype {
                            prometheus::proto::MetricType::GAUGE => {
                                print!("{} ", m.get_gauge().get_value())
                            }
                            prometheus::proto::MetricType::COUNTER => {
                                print!("{} ", m.get_counter().get_value())
                            }
                            prometheus::proto::MetricType::SUMMARY => todo!(),
                            prometheus::proto::MetricType::UNTYPED => todo!(),
                            prometheus::proto::MetricType::HISTOGRAM => todo!(),
                        }
                    }
                    println!();
                }
                println!();
            }
        }
    }
}
