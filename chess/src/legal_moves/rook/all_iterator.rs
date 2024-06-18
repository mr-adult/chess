use crate::{arr_deque::ArrDeque, bitboard::BitBoard};

#[derive(Debug, Clone, Copy)]
enum StraightDirection {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Debug)]
pub(crate) struct RookMovesIterator {
    original: BitBoard,
    previous: BitBoard,
    directions_to_check: ArrDeque<StraightDirection, 4>,
}

impl RookMovesIterator {
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

    pub(crate) fn next_direction(&mut self) -> bool {
        self.directions_to_check.pop_front();
        self.previous = self.original.clone();
        return self.directions_to_check.len() > 0;
    }
}

impl Iterator for RookMovesIterator {
    type Item = BitBoard;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(direction) = self.directions_to_check.peek_front() {
            match *direction {
                StraightDirection::Up => self.previous = self.previous.up(),
                StraightDirection::Right => self.previous = self.previous.right(),
                StraightDirection::Down => self.previous = self.previous.down(),
                StraightDirection::Left => self.previous = self.previous.left(),
            }

            if self.previous.0 == 0 {
                self.next_direction();
            } else {
                return Some(self.previous.clone());
            }
        }
        return None;
    }
}