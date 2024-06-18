use std::array::IntoIter;

use crate::bitboard::BitBoard;

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
            ]
            .into_iter(),
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
