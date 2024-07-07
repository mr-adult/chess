use serde_derive::Serialize;

use super::move_::Move;

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum PossibleMove {
    Promotion {
        #[serde(rename = "move")]
        move_: Move,
    },
    Normal {
        #[serde(rename = "move")]
        move_: Move,
    },
}

impl PossibleMove {
    pub fn move_(&self) -> &Move {
        match self {
            Self::Promotion { move_ } | Self::Normal { move_ } => move_,
        }
    }

    pub fn take_move(self) -> Move {
        match self {
            Self::Promotion { move_ } | Self::Normal { move_ } => move_,
        }
    }
}
