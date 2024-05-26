use std::fmt::Debug;

use chess_common::{PieceKind, Player};

#[derive(Clone, Copy)]
pub struct Piece {
    kind: PieceKind,
    player: Player,
}

impl Piece {
    pub fn new(kind: PieceKind, player: Player) -> Self {
        Self {
            kind,
            player,
        }
    }

    pub fn kind(&self) -> PieceKind {
         self.kind
    }

    pub fn player(&self) -> Player {
        self.player
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
