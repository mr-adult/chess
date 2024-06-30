use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct FenRequest {
    pub(crate) board_fen: String,
}
