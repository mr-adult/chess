use std::fmt::Debug;

use chess_common::{PieceKind, Player};

pub(crate) struct Piece {
    pub(crate) kind: PieceKind,
    pub(crate) player: Player,
}

impl Piece {
    pub(crate) fn new(kind: PieceKind, player: Player) -> Self {
        Self {
            kind,
            player,
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
