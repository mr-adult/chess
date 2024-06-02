#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Player {
    Black,
    White,
}

impl Player {
    pub const fn as_char(self) -> char {
        match self {
            Player::Black => 'b',
            Player::White => 'w',
        }
    }

    pub const fn as_index(&self) -> usize {
        match self {
            Player::White => 0,
            Player::Black => 1,
        }
    }

    pub const fn other_player(&self) -> Player {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::Black,
        }
    }

    pub const fn other_player_usize(player: usize) -> usize {
        debug_assert!(player < 2);
        player + 1 % 2
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
