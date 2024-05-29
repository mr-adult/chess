use std::str::FromStr;

use axum::{extract::Query, routing::get, Json, Router};
use chess_core::{Board, Move};
use http::StatusCode;
use serde::Deserialize;

pub(crate) fn create_api_router() -> Router {
    Router::new().route("/legal_moves", get(get_legal_moves_handler))
}

async fn get_legal_moves_handler(
    req: Query<LegalMovesRequest>,
) -> Result<Json<Vec<Move>>, StatusCode> {
    println!("{}", req.board_fen);
    let board = Board::from_str(&req.board_fen).map_err(|fen_err| {
        println!("{:?}", fen_err);
        StatusCode::BAD_REQUEST
    })?;

    let mut result = Vec::new();
    for move_ in board.legal_moves() {
        println!("{:?}", move_);
        result.push(move_);
    }

    Ok(Json(result))
}

#[derive(Deserialize)]
struct LegalMovesRequest {
    board_fen: String,
}
