use chess_common::PieceKind;
use serde_derive::Deserialize;

use super::move_::Move;

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

    pub(crate) fn take_move(self) -> Move {
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