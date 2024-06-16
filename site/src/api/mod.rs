use std::str::FromStr;

use axum::{
    extract::Query,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use chess_core::{Board, Move};
use http::StatusCode;
use serde::Deserialize;

use crate::chess_html::render_gameboard;

pub(crate) fn create_api_router() -> Router {
    Router::new()
        .route("/legal_moves", get(get_legal_moves_handler))
        .route("/make_move", post(make_move_handler))
}

async fn get_legal_moves_handler(
    req: Query<LegalMovesRequest>,
) -> Result<Json<Vec<Move>>, StatusCode> {
    println!("{}", req.board_fen);
    let board = Board::from_str(&req.board_fen).map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut result = Vec::new();
    for move_ in board.legal_moves() {
        println!("{:?}", move_);
        result.push(move_);
    }
    println!("done");

    Ok(Json(result))
}

#[derive(Deserialize)]
struct LegalMovesRequest {
    board_fen: String,
}

async fn make_move_handler(req: Json<MakeMovesRequest>) -> Result<Html<String>, StatusCode> {
    println!("Attempting to make move {req:?}");
    let mut board = Board::from_str(&req.board_fen).map_err(|_| StatusCode::BAD_REQUEST)?;
    board
        .make_move(req.move_.clone())
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    return Ok(render_gameboard(&board));
}

#[derive(Debug, Deserialize)]
struct MakeMovesRequest {
    board_fen: String,
    #[serde(alias = "move")]
    move_: Move,
}
