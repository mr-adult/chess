use std::array::IntoIter;

use chess_common::{File, Location, Player, Rank};

use crate::{bitboard::BitBoard, Board, Move};

use super::captures_at_location::LegalCapturesAtLocationIterator;

pub(crate) struct LegalKingMovesIterator<'board> {
    board: &'board Board,
    player: Player,
    king_bitboard: BitBoard,
    moves: IntoIter<BitBoard, 8>,
    checked_castle_queenside: bool,
    checked_castle_kingside: bool,
    friendly_pieces: BitBoard,
}

impl<'board> LegalKingMovesIterator<'board> {
    pub(crate) fn new(board: &'board Board, player: Player) -> Self {
        let player_index = player.as_index();
        let king_bitboard = board.kings[player_index].clone();
        Self {
            board,
            player,
            king_bitboard: king_bitboard.clone(),
            moves: [
                king_bitboard.up(),
                king_bitboard.up_right(),
                king_bitboard.right(),
                king_bitboard.down_right(),
                king_bitboard.down(),
                king_bitboard.down_left(),
                king_bitboard.left(),
                king_bitboard.up_left(),
            ]
            .into_iter(),
            friendly_pieces: board.create_mailbox_for_player(player),
            checked_castle_kingside: false,
            checked_castle_queenside: false,
        }
    }

    pub(crate) fn is_check(&self, player: Player, king_position: u64) -> bool {
        let mut iterator =
            LegalCapturesAtLocationIterator::new(&self.board, player.other_player(), king_position);
        iterator.next().is_some()
    }
}

impl<'board> Iterator for LegalKingMovesIterator<'board> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        let king_bitboard = self.king_bitboard.clone();

        while let Some(king_move) = self.moves.next() {
            if king_move.0 == 0 {
                continue;
            }

            if king_move.intersects_with(&self.friendly_pieces) {
                continue;
            }

            if self.is_check(self.player, self.king_bitboard.0) {
                continue;
            }

            return Some(Move {
                from: Location::try_from(king_bitboard.0)
                    .expect(Location::failed_from_usize_message()),
                to: Location::try_from(king_move.0).expect(Location::failed_from_usize_message()),
            });
        }

        if self.checked_castle_queenside && self.checked_castle_kingside {
            return None;
        }

        let castle_rank = match self.player {
            Player::White => Rank::One,
            Player::Black => Rank::Eight,
        };

        if !self.checked_castle_queenside {
            self.checked_castle_queenside = true;

            if self.board.player_can_castle_queenside(&self.player) {
                let any_pieces_in_way = [File::b, File::c, File::d]
                    .into_iter()
                    .map(|file| Location::new(file, castle_rank))
                    .map(|loc| loc.as_u64())
                    .any(|bitboard| self.friendly_pieces.intersects_with_u64(bitboard));

                if !any_pieces_in_way {
                    return Some(Move {
                        from: Location::try_from(self.king_bitboard.0)
                            .expect(Location::failed_from_usize_message()),
                        to: Location::new(File::c, castle_rank),
                    });
                }
            }
        }

        if !self.checked_castle_kingside {
            self.checked_castle_kingside = true;

            if self.board.player_can_castle_kingside(&self.player) {
                let any_pieces_in_way = [File::f, File::g]
                    .into_iter()
                    .map(|file| Location::new(file, castle_rank))
                    .map(|loc| loc.as_u64())
                    .any(|bitboard| self.friendly_pieces.intersects_with_u64(bitboard));

                if !any_pieces_in_way {
                    return Some(Move {
                        from: Location::try_from(self.king_bitboard.0)
                            .expect(Location::failed_from_usize_message()),
                        to: Location::new(File::g, castle_rank),
                    });
                }
            }
        }

        return None;
    }
}

// Tests should live in the legal_moves module
