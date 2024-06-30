use axum::extract::Query;
use axum::response::Html;
use axum::{routing::get, Router};
use chess_core::Board;
use http::StatusCode;
use std::str::FromStr;

use super::game_board::new_game;
use super::render_gameboard;
use crate::common::FenRequest;

pub(crate) fn create_ssr_router() -> Router {
    Router::new()
        .route("/new_game", get(new_game))
        .route("/render_board", get(render_board_handler))
}

pub(crate) async fn render_board_handler(
    board_fen: Query<FenRequest>,
) -> Result<Html<String>, StatusCode> {
    let board = Board::from_str(&board_fen.0.board_fen).map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(render_gameboard(&board))
}
