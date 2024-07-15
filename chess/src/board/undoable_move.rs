use chess_common::{Location, Piece, PieceKind};

use crate::{Move, SelectedMove};

#[derive(Clone, Debug)]
pub enum UndoableMove {
    Promotion {
        move_: Move,
        // algebraic_notation: String,
        promoted_to: PieceKind,
    },
    EnPassant {
        move_: Move,
        // algebraic_notation: String,
        captured_pawn_location: Location,
    },
    Normal {
        move_: Move,
        // algebraic_notation: String,
    },
    Capture {
        move_: Move,
        // algebraic_notation: String,
        captured_piece: Piece,
    },
    CapturePromotion {
        move_: Move,
        captured_piece: Piece,
        promoted_to: PieceKind,
    },
    Castles {
        move_: Move,
        rook_move: Move,
    },
}

impl UndoableMove {
    pub fn move_(&self) -> &Move {
        match self {
            Self::Promotion { move_, .. } => move_,
            Self::EnPassant { move_, .. } => move_,
            Self::Normal { move_, .. } => move_,
            Self::Capture { move_, .. } => move_,
            Self::CapturePromotion { move_, .. } => move_,
            Self::Castles { move_, .. } => move_,
        }
    }
}

impl Into<SelectedMove> for &UndoableMove {
    fn into(self) -> SelectedMove {
        match self {
            UndoableMove::Promotion { move_, promoted_to }
            | UndoableMove::CapturePromotion {
                move_, promoted_to, ..
            } => SelectedMove::Promotion {
                move_: move_.clone(),
                promotion_kind: *promoted_to,
            },
            UndoableMove::EnPassant { move_, .. }
            | UndoableMove::Normal { move_ }
            | UndoableMove::Capture { move_, .. }
            | UndoableMove::Castles { move_, .. } => SelectedMove::Normal {
                move_: move_.clone(),
            },
        }
    }
}
