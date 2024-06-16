use chess_common::Location;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Move {
    pub(crate) from: Location,
    pub(crate) to: Location,
}
