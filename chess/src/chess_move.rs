use std::fmt::Debug;

use chess_common::{Location, PieceKind};
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Move {
    pub(crate) from: Location,
    pub(crate) to: Location,
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
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum SelectedMove {
    Promotion {
        #[serde(rename = "move")]
        move_: Move,
        promotion_kind: PieceKind,
    },
    Normal {
        #[serde(rename = "move")]
        move_: Move,
    },
}

impl SelectedMove {
    pub fn move_(&self) -> &Move {
        match self {
            Self::Promotion { move_, .. } | Self::Normal { move_ } => move_,
        }
    }

    pub(super) fn take_move(self) -> Move {
        match self {
            Self::Promotion { move_, .. } | Self::Normal { move_ } => move_,
        }
    }

    pub fn promotion_kind(&self) -> Option<PieceKind> {
        match self {
            Self::Promotion { promotion_kind, .. } => Some(*promotion_kind),
            Self::Normal { .. } => None,
        }
    }
}
