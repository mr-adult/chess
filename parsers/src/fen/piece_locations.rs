use std::ops::{Index, IndexMut};

use chess_common::{Location, Piece};

#[derive(Clone, Debug, Default)]
pub struct PieceLocations {
    pieces: [[Option<Piece>; 8]; 8],
}

impl Index<&Location> for PieceLocations {
    type Output = Option<Piece>;

    fn index(&self, index: &Location) -> &Self::Output {
        &self.pieces[index.rank().as_index()][index.file().as_index()]
    }
}

impl IndexMut<&Location> for PieceLocations {
    fn index_mut(&mut self, index: &Location) -> &mut Self::Output {
        &mut self.pieces[index.rank().as_index()][index.file().as_index()]
    }
}
