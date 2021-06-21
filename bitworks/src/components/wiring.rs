use bevy::utils::HashMap;

use crate::core::*;

#[derive(Default)]
pub struct Wiring {
    pins: HashMap<usize, Pin>,
}

#[derive(Default)]
pub struct Pin {
    pin_dir: PinDir,
    pos: Pos,
    dir: Dir,
    signal: Signal,
}

pub enum PinDir {
    Input,
    Output,
}

impl Default for PinDir {
    fn default() -> Self {
        PinDir::Input
    }
}

impl Wiring {
    pub fn pin(&mut self, pin: usize) -> &mut Pin {
        self.pins.insert(pin, Pin::default());
        self.pins.get_mut(&pin).unwrap()
    }

    pub fn value_pin(&self, pin: usize) -> Signal {
        Signal::None
    }

    pub fn set_pin(&mut self, pin: usize, signal: Signal) {}
}

impl Pin {
    pub fn input(&mut self) -> &mut Self {
        self.pin_dir = PinDir::Input;
        self
    }

    pub fn output(&mut self) -> &mut Self {
        self.pin_dir = PinDir::Output;
        self
    }

    pub fn pos(&mut self, x: usize, y: usize) -> &mut Self {
        self.pos = Pos(x, y);
        self
    }

    pub fn dir(&mut self, dir: Dir) -> &mut Self {
        self.dir = dir;
        self
    }
}

pub enum Signal {
    None,
}

impl Default for Signal {
    fn default() -> Self {
        Signal::None
    }
}

impl Signal {
    pub fn is_truthy(&self) -> bool {
        false
    }
}
