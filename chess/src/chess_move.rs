use std::str::FromStr;

use chess_common::{Location, PieceKind, Player};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Move {
    pub(crate) from: Location,
    pub(crate) to: Location,
}

impl Move {}

impl FromStr for Move {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}



pub(crate) enum PawnMoveKind {
    Push,
    DoublePush,
    Capture,
    EnPassant,
    Promotion,
    CapturePromotion,
}

pub(crate) enum KingMoveKind {
    Normal,
    Castles,
}

pub(crate) enum MoveKind {
    /// A move where the piece starts and ends at the same
    /// position, resulting in no net change to the board.
    NonMove,
    /// A standard move.
    Normal,
    /// A pawn move. This is separate from normal so that
    /// en passant can be correctly classified.
    Pawn(PawnMoveKind),
    /// A king move. This is separate from normal so that 
    /// castling can be correctly classified.
    King(KingMoveKind),
}

pub(crate) enum IllegalMove {
    NoPieceAtFromLocation,
    PieceOwnedByOtherPlayer,
    IllegalMove,
    Check,
}
