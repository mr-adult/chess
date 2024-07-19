use super::move_err::MoveErr;

#[derive(Debug)]
pub enum AcnMoveErr {
    /// Signifies an error in parsing the algebraic chess notation string.
    Acn,
    /// Signifies that the ACN string's check status mismatched the board state.
    CheckStateMismatch,
    /// Signifies that multiple legal moves matched the ACN string.
    AmbiguousMove,
    /// Signifies that the move could not be made with the reason.
    Move(MoveErr),
}

impl From<MoveErr> for AcnMoveErr {
    fn from(value: MoveErr) -> Self {
        Self::Move(value)
    }
}
