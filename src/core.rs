#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct Pos(pub usize, pub usize);

impl Pos {
    pub fn _add(&self, other: &Self) -> Self {
        Self(self.0 + other.0, self.1 + other.1)
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    W,
    E,
    N,
    S,
}

impl Dir {
    pub fn pos(&self, base: &Pos) -> Pos {
        let Pos(x, y) = base;
        match *self {
            Dir::W => Pos(x - 1, y + 0),
            Dir::E => Pos(x + 1, y + 0),
            Dir::N => Pos(x + 0, y - 1),
            Dir::S => Pos(x + 0, y + 1),
        }
    }

    pub fn _invert(&self) -> Self {
        match *self {
            Dir::W => Dir::E,
            Dir::E => Dir::W,
            Dir::N => Dir::S,
            Dir::S => Dir::N,
        }
    }
}

impl Default for Dir {
    fn default() -> Self {
        Self::E
    }
}
