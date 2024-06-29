use crate::bitboard::BitBoard;
use arr_deque::ArrDeque;

#[derive(Debug, Clone, Copy)]
pub(crate) enum DiagonalDirection {
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

impl DiagonalDirection {
    pub(crate) fn all() -> [DiagonalDirection; 4] {
        [Self::UpLeft, Self::UpRight, Self::DownLeft, Self::DownRight]
    }
}

#[derive(Debug)]
pub(crate) struct BishopMovesIterator {
    original: BitBoard,
    previous: BitBoard,
    directions_to_check: ArrDeque<DiagonalDirection, 4>,
}

impl BishopMovesIterator {
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

    pub(crate) fn with_directions<T: IntoIterator<Item = DiagonalDirection>>(
        directions: T,
        board: BitBoard,
    ) -> Self {
        let mut directions_to_check = ArrDeque::new();
        for direction in directions {
            assert!(directions_to_check.push_back(direction).is_ok());
        }

        Self {
            original: board.clone(),
            previous: board,
            directions_to_check,
        }
    }

    pub(crate) fn next_direction(&mut self) -> bool {
        self.directions_to_check.pop_front();
        if self.previous.0 != self.original.0 {
            self.previous = self.original.clone();
        }
        return self.directions_to_check.len() > 0;
    }
}

impl Iterator for BishopMovesIterator {
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
                self.next_direction();
            } else {
                return Some(self.previous.clone());
            }
        }

        return None;
    }
}
