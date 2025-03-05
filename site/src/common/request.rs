use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct FenRequest {
    // Keep in sync with PerftRequest
    pub(crate) board_fen: String,
}

#[derive(Deserialize)]
pub(crate) struct PerftRequest {
    pub(crate) board_fen: String,
    pub(crate) depth: usize,
}
