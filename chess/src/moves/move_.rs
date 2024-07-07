use std::fmt::Debug;

use chess_common::{File, Location, Player, Rank};
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Move {
    pub(crate) from: Location,
    pub(crate) to: Location,
}

impl Move {
    pub const WHITE_CASTLE_KINGSIDE: Move = Move {
        from: Location::new(File::king_starting(), Rank::castle(&Player::White)),
        to: Location::new(
            File::castle_kingside_destination(),
            Rank::castle(&Player::White),
        ),
    };

    pub const BLACK_CASTLE_KINGSIDE: Move = Move {
        from: Location::new(File::king_starting(), Rank::castle(&Player::Black)),
        to: Location::new(
            File::castle_kingside_destination(),
            Rank::castle(&Player::Black),
        ),
    };

    pub const WHITE_CASTLE_QUEENSIDE: Move = Move {
        from: Location::new(File::king_starting(), Rank::castle(&Player::White)),
        to: Location::new(
            File::castle_queenside_destination(),
            Rank::castle(&Player::White),
        ),
    };

    pub const BLACK_CASTLE_QUEENSIDE: Move = Move {
        from: Location::new(File::king_starting(), Rank::castle(&Player::Black)),
        to: Location::new(
            File::castle_queenside_destination(),
            Rank::castle(&Player::Black),
        ),
    };
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::with_capacity(8);
        result.push_str(&self.from.to_string());
        result.push_str(" -> ");
        result.push_str(&self.to.to_string());

        write!(f, "{}", result)
    }
}
