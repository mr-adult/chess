#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Player {
    Black,
    White,
}

impl Player {
    pub fn as_char(self) -> char {
        match self {
            Player::Black => 'B',
            Player::White => 'W',
        }
    }

    pub fn as_index(&self) -> usize {
        match self {
            Player::White => 0,
            Player::Black => 1,
        }
    }
}

#[macro_export]
macro_rules! white {
    () => {
        0_usize
    };
}

#[macro_export]
macro_rules! black {
    () => {
        1_usize
    };
}
