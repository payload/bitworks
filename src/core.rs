use crate::components::Signal;

#[allow(dead_code)]
pub enum Piece {
    TR,
    BR,
    BL,
    TL,
}

pub type ItemPiece = (Item, Piece);

pub enum ItemFilter {
    Color,
    Shape,
    All,
}

pub type Pos = (usize, usize);

#[allow(dead_code)]
pub enum Dir {
    W,
    E,
    N,
    S,
}

impl Dir {
    fn pos(&self, base: &Pos) -> Pos {
        let (x, y) = base;
        match *self {
            Dir::W => (x - 1, y + 0),
            Dir::E => (x + 1, y + 0),
            Dir::N => (x + 0, y - 1),
            Dir::S => (x + 0, y + 1),
        }
    }

    fn invert(&self) -> Self {
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

#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq)]
pub enum Item {
    Red,
    Green,
    Blue,
}

impl Item {
    pub fn paint(self, other: Item) -> Item {
        Item::Blue
    }

    pub fn signal(&self) -> Signal {
        Signal::None
    }

    pub fn pieces(self) -> std::array::IntoIter<ItemPiece, 1> {
        std::array::IntoIter::new([(self, Piece::TR)])
    }
}
