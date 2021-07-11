use bevy::math::{vec2, Vec2};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum CompassDir {
    N,
    E,
    S,
    W,
}

impl CompassDir {
    pub fn opposite(&self) -> Self {
        use CompassDir::*;
        match self {
            N => S,
            E => W,
            S => N,
            W => E,
        }
    }

    pub fn right(&self) -> Self {
        use CompassDir::*;
        match self {
            N => E,
            E => S,
            S => W,
            W => N,
        }
    }

    pub fn left(&self) -> Self {
        use CompassDir::*;
        match self {
            N => W,
            E => N,
            S => E,
            W => S,
        }
    }

    pub fn vec2(&self) -> Vec2 {
        use CompassDir::*;
        match self {
            N => vec2(0.0, 1.0),
            E => vec2(1.0, 0.0),
            S => vec2(0.0, -1.0),
            W => vec2(-1.0, 0.0),
        }
    }
}
