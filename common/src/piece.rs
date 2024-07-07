use std::fmt::Debug;

use crate::{PieceKind, Player};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    kind: PieceKind,
    player: Player,
}

impl Piece {
    pub const WHITE_PAWN: Piece = Piece::new(Player::White, PieceKind::Pawn);
    pub const BLACK_PAWN: Piece = Piece::new(Player::Black, PieceKind::Pawn);

    pub const fn new(player: Player, kind: PieceKind) -> Self {
        Self { kind, player }
    }

    pub const fn kind(&self) -> PieceKind {
        self.kind
    }

    pub const fn kind_ref(&self) -> &PieceKind {
        &self.kind
    }

    pub const fn player(&self) -> Player {
        self.player
    }

    pub const fn to_fen(&self) -> char {
        match self.player() {
            Player::White => self.kind().as_char().to_ascii_uppercase(),
            Player::Black => self.kind().as_char().to_ascii_lowercase(),
        }
    }
}

impl Debug for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut repr_string = String::with_capacity(3);
        repr_string.push(self.player.as_char());
        repr_string.push(self.kind.as_char());
        f.write_str(&repr_string)
    }
}
