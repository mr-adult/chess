use std::{
    array::{from_fn, IntoIter},
    fmt::Debug,
    ops::{BitAnd, BitOr, BitXor},
};

use chess_common::{File, Rank};

use crate::arr_deque::ArrDeque;

#[derive(Clone, Default)]
pub(crate) struct BitBoard(pub(crate) u64);

impl BitBoard {
    pub(crate) fn left(&self) -> Self {
        Self(self.0.wrapping_shr(1) & !File::h_bit_filter())
    }

    pub(crate) fn right(&self) -> Self {
        Self(self.0.wrapping_shl(1) & !File::a_bit_filter())
    }

    pub(crate) fn up(&self) -> Self {
        Self(self.0.wrapping_shl(8))
    }

    pub(crate) fn down(&self) -> Self {
        Self(self.0.wrapping_shr(8))
    }

    pub(crate) fn up_left(&self) -> Self {
        Self(self.0.wrapping_shl(7) & !File::h_bit_filter())
    }

    pub(crate) fn up_right(&self) -> Self {
        Self(self.0.wrapping_shl(9) & !File::a_bit_filter())
    }

    pub(crate) fn down_left(&self) -> Self {
        Self(self.0.wrapping_shr(7) & !File::h_bit_filter())
    }

    pub(crate) fn down_right(&self) -> Self {
        Self(self.0.wrapping_shr(9) & !File::a_bit_filter())
    }

    pub(crate) fn diagonal_moves(&self) -> DiagonalMovesIterator {
        DiagonalMovesIterator::new(self.clone())
    }

    pub(crate) fn straight_moves(&self) -> StraightMovesIterator {
        StraightMovesIterator::new(self.clone())
    }

    pub(crate) fn knight_moves(&self) -> KnightMovesIterator {
        KnightMovesIterator::new(self.clone())
    }

    pub(crate) fn intersects_with(&self, other: &BitBoard) -> bool {
        self.intersects_with_u64(other.0)
    }

    pub(crate) fn intersects_with_u64(&self, other: u64) -> bool {
        (self.0 & other) != 0
    }
}

impl BitAnd for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitOr for BitBoard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitXor for BitBoard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result: [String; 8] = from_fn(|_| String::with_capacity(0));
        for rank in Rank::all_ranks_ascending().rev() {
            match rank {
                Rank::One => {
                    let bits = (self.0 & Rank::one_bit_filter()) as u8;
                    result[rank.as_index()] = format!("{:08b}", bits);
                }
                Rank::Two => {
                    let bits = (self.0 & Rank::two_bit_filter()) >> 8 as u8;
                    result[rank.as_index()] = format!("{:08b}", bits);
                }
                Rank::Three => {
                    let bits = (self.0 & Rank::three_bit_filter()) >> 16 as u8;
                    result[rank.as_index()] = format!("{:08b}", bits);
                }
                Rank::Four => {
                    let bits = (self.0 & Rank::four_bit_filter()) >> 24 as u8;
                    result[rank.as_index()] = format!("{:08b}", bits);
                }
                Rank::Five => {
                    let bits = (self.0 & Rank::five_bit_filter()) >> 32 as u8;
                    result[rank.as_index()] = format!("{:08b}", bits);
                }
                Rank::Six => {
                    let bits = (self.0 & Rank::six_bit_filter()) >> 40 as u8;
                    result[rank.as_index()] = format!("{:08b}", bits);
                }
                Rank::Seven => {
                    let bits = (self.0 & Rank::seven_bit_filter()) >> 48 as u8;
                    result[rank.as_index()] = format!("{:08b}", bits);
                }
                Rank::Eight => {
                    let bits = (self.0 & Rank::eight_bit_filter()) >> 56 as u8;
                    result[rank.as_index()] = format!("{:08b}", bits);
                }
            }
        }

        let mut result_string = '\n'.to_string();
        result_string.push_str(&result.join("\n"));
        result_string.push('\n');
        write!(f, "{}", result_string)
    }
}

#[derive(Debug, Clone, Copy)]
enum KnightDirection {
    UpLeftx2,
    Upx2Left,
    UpRightx2,
    Upx2Right,
    DownLeftx2,
    Downx2Left,
    DownRightx2,
    Downx2Right,
}

#[derive(Debug)]
pub(crate) struct KnightMovesIterator {
    original: BitBoard,
    directions_to_check: IntoIter<KnightDirection, 8>,
}

impl KnightMovesIterator {
    pub(crate) fn new(board: BitBoard) -> Self {
        Self {
            original: board,
            directions_to_check: [
                KnightDirection::UpLeftx2,
                KnightDirection::Upx2Left,
                KnightDirection::UpRightx2,
                KnightDirection::Upx2Right,
                KnightDirection::DownLeftx2,
                KnightDirection::Downx2Left,
                KnightDirection::DownRightx2,
                KnightDirection::Downx2Right,
            ].into_iter(),
        }
    }
}

impl Iterator for KnightMovesIterator {
    type Item = BitBoard;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(direction) = self.directions_to_check.next() {
            let new_square = match direction {
                KnightDirection::UpLeftx2 => self.original.up_left().left(),
                KnightDirection::Upx2Left => self.original.up_left().up(),
                KnightDirection::UpRightx2 => self.original.up_right().right(),
                KnightDirection::Upx2Right => self.original.up_right().up(),
                KnightDirection::DownLeftx2 => self.original.down_left().left(),
                KnightDirection::Downx2Left => self.original.down_left().down(),
                KnightDirection::DownRightx2 => self.original.down_right().right(),
                KnightDirection::Downx2Right => self.original.down_right().down(),
            };

            if new_square.0 != 0 {
                return Some(new_square);
            }
        }
        return None;
    }
}

#[derive(Debug, Clone, Copy)]
enum DiagonalDirection {
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

#[derive(Debug)]
pub(crate) struct DiagonalMovesIterator {
    original: BitBoard,
    previous: BitBoard,
    directions_to_check: ArrDeque<DiagonalDirection, 4>,
}

impl DiagonalMovesIterator {
    pub(crate) fn new(board: BitBoard) -> Self {
        Self {
            original: board.clone(),
            previous: board,
            directions_to_check: ArrDeque::from_fn(|i| match i {
                0 => DiagonalDirection::DownLeft,
                1 => DiagonalDirection::DownRight,
                2 => DiagonalDirection::UpLeft,
                3 => DiagonalDirection::UpRight,
                _ => unreachable!(),
            }),
        }
    }
}

impl Iterator for DiagonalMovesIterator {
    type Item = BitBoard;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(direction) = self.directions_to_check.peek_front() {
            match *direction {
                DiagonalDirection::UpLeft => self.previous = self.previous.up_left(),
                DiagonalDirection::UpRight => self.previous = self.previous.up_right(),
                DiagonalDirection::DownLeft => self.previous = self.previous.down_left(),
                DiagonalDirection::DownRight => self.previous = self.previous.down_right(),
            }

            if self.previous.0 == 0 {
                self.directions_to_check.pop_front();
                self.previous = self.original.clone();
            } else {
                return Some(self.previous.clone());
            }
        }
        return None;
    }
}

#[derive(Debug, Clone, Copy)]
enum StraightDirection {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug)]
pub(crate) struct StraightMovesIterator {
    original: BitBoard,
    previous: BitBoard,
    directions_to_check: ArrDeque<StraightDirection, 4>,
}

impl StraightMovesIterator {
    pub(crate) fn new(board: BitBoard) -> Self {
        Self {
            original: board.clone(),
            previous: board,
            directions_to_check: ArrDeque::from_fn(|i| match i {
                0 => StraightDirection::Up,
                1 => StraightDirection::Right,
                2 => StraightDirection::Down,
                3 => StraightDirection::Left,
                _ => unreachable!(),
            }),
        }
    }
}

impl Iterator for StraightMovesIterator {
    type Item = BitBoard;

    fn next(&mut self) -> Option<Self::Item> {
        return None;
        while let Some(direction) = self.directions_to_check.peek_front() {
            match *direction {
                StraightDirection::Up => self.previous = self.previous.up(),
                StraightDirection::Right => self.previous = self.previous.right(),
                StraightDirection::Down => self.previous = self.previous.down(),
                StraightDirection::Left => self.previous = self.previous.left(),
            }

            if self.previous.0 == 0 {
                self.directions_to_check.pop_front();
                self.previous = self.original.clone();
            } else {
                return Some(self.previous.clone());
            }
        }
        return None;
    }
}
