use arr_deque::ArrDeque;
use chess_common::PieceKind;

use crate::{legal_moves::LegalMovesIterator, PossibleMove, SelectedMove};

pub struct PossibleMovesIterator<'board> {
    legal_moves: LegalMovesIterator<'board>,
    lookahead: ArrDeque<SelectedMove, 3>,
}

impl<'board> PossibleMovesIterator<'board> {
    pub(crate) fn new(legal_moves: LegalMovesIterator<'board>) -> Self {
        Self {
            legal_moves: legal_moves,
            lookahead: ArrDeque::new(),
        }
    }
}

impl<'board> Iterator for PossibleMovesIterator<'board> {
    type Item = SelectedMove;

    fn next(&mut self) -> Option<Self::Item> {
        let front = self.lookahead.pop_front();
        if front.is_some() {
            return front;
        }

        if let Some(move_) = self.legal_moves.next() {
            match move_ {
                PossibleMove::Normal { move_ } => return Some(SelectedMove::Normal { move_ }),
                PossibleMove::Promotion { move_ } => {
                    for i in 0..3 {
                        assert!(self
                            .lookahead
                            .push_back(SelectedMove::Promotion {
                                move_: move_.clone(),
                                promotion_kind: match i {
                                    0 => PieceKind::Knight,
                                    1 => PieceKind::Bishop,
                                    2 => PieceKind::Rook,
                                    _ => unreachable!(),
                                }
                            })
                            .is_ok());
                    }

                    return Some(SelectedMove::Promotion {
                        move_,
                        promotion_kind: PieceKind::Queen,
                    });
                }
            }
        }

        return None;
    }
}
