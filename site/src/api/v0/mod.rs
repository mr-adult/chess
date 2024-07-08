use std::str::FromStr;

use axum::{
    extract::Query,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use chess_common::Player;
use chess_core::{Board, PossibleMove, SelectedMove};
use http::StatusCode;
use serde::Deserialize;

use crate::{chess_html::render_gameboard, common::FenRequest};

pub(crate) fn create_api_router() -> Router {
    Router::new()
        .route("/legal_moves", get(get_legal_moves_handler))
        .route("/make_move", post(make_move_handler))
}

async fn get_legal_moves_handler(
    req: Query<FenRequest>,
) -> Result<Json<Vec<PossibleMove>>, StatusCode> {
    let board = Board::from_str(&req.board_fen).map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(Json(board.legal_moves().collect::<Vec<_>>()))
}

async fn make_move_handler(req: Json<MakeMovesRequest>) -> Result<Html<String>, StatusCode> {
    let mut board = Board::from_str(&req.board_fen).map_err(|_| StatusCode::BAD_REQUEST)?;

    if let Some(starting_fen) = &req.starting_fen {
        let mut starting_board =
            Board::from_str(&starting_fen).map_err(|_| StatusCode::BAD_REQUEST)?;

        if let Some(history) = req.history.as_ref() {
            for historical_move in history {
                starting_board
                    .make_move_acn(&historical_move)
                    .map_err(|_| StatusCode::BAD_REQUEST)?;
            }
        }

        if starting_board.to_fen_string() != req.board_fen {
            return Err(StatusCode::BAD_REQUEST);
        }

        board = starting_board;
    }

    board
        .make_move(req.move_.clone())
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    return Ok(render_gameboard(&board));
}

#[derive(Debug, Deserialize)]
struct MakeMovesRequest {
    board_fen: String,
    #[serde(alias = "move")]
    move_: SelectedMove,

    starting_fen: Option<String>,
    history: Option<Vec<String>>,
}
