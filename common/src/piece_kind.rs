use serde_derive::Deserialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceKind {
    pub const fn as_char(self) -> char {
        match self {
            Self::Pawn => 'P',
            Self::Knight => 'N',
            Self::Bishop => 'B',
            Self::Rook => 'R',
            Self::Queen => 'Q',
            Self::King => 'K',
        }
    }
}

impl TryFrom<char> for PieceKind {
    type Error = ();
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'P' | 'p' => Ok(PieceKind::Pawn),
            'N' | 'n' => Ok(PieceKind::Knight),
            'B' | 'b' => Ok(PieceKind::Bishop),
            'R' | 'r' => Ok(PieceKind::Rook),
            'Q' | 'q' => Ok(PieceKind::Queen),
            'K' | 'k' => Ok(PieceKind::King),
            _ => Err(()),
        }
    }
}
