use std::fmt::Debug;

use chess_common::Location;
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
