#![macro_use]

// pub fn check<T: Default>(b: bool) -> Option<T> {
//     if b {
//         Some(T::default())
//     } else {
//         None
//     }
// }

// use lazy_static::lazy_static;
// use prometheus::{self, register_int_gauge, IntGauge};

// #[macro_export]
// macro_rules! gauge {
//     ($NAME:ident) => {
//         lazy_static! {
//             static ref $NAME: IntGauge =
//                 register_int_gauge!(stringify!($NAME).to_lowercase(), "no help").unwrap();
//         }
//     };
// }
