use std::str::FromStr;

use axum::{
    extract::Query,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use chess_core::{Board, PossibleMove, SelectedMove};
use http::StatusCode;
use serde::Deserialize;

use crate::{
    chess_html::render_gameboard_with_history,
    common::{FenRequest, PerftRequest},
};

pub(crate) fn create_api_router() -> Router {
    Router::new()
        .route("/legal_moves", get(get_legal_moves_handler))
        .route("/make_move", post(make_move_handler))
        .route("/perft", get(perft_handler))
        .route("/pgn", post(get_pgn_handler))
}

async fn get_legal_moves_handler(
    req: Query<FenRequest>,
) -> Result<Json<Vec<PossibleMove>>, StatusCode> {
    let board = Board::from_str(&req.board_fen).map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(Json(board.legal_moves().collect::<Vec<_>>()))
}

async fn get_pgn_handler(req: Json<MakeMovesRequest>) -> Result<String, StatusCode> {
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

    let pgn = board.to_pgn().to_string();
    return Ok(pgn);
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
    return Ok(render_gameboard_with_history(&board));
}

#[derive(Debug, Deserialize)]
struct MakeMovesRequest {
    board_fen: String,
    #[serde(alias = "move")]
    move_: SelectedMove,

    starting_fen: Option<String>,
    history: Option<Vec<String>>,
}

async fn perft_handler(req: Query<PerftRequest>) -> Result<Json<Vec<(String, usize)>>, StatusCode> {
    let mut board = Board::from_str(&req.board_fen).map_err(|_| StatusCode::BAD_REQUEST)?;
    let perft_result = board.perft(
        req.depth,
        std::thread::available_parallelism()
            .ok()
            .and_then(|non_zero| Some(non_zero.get()))
            .unwrap_or(1),
    );
    let mut total = 0;
    let result = perft_result.into_iter().map(|tuple| {
        total += tuple.1;

        (tuple.0.to_string(), tuple.1)
    });

    let mut result_vec = result.collect::<Vec<(String, usize)>>();
    result_vec.push(("Total".to_string(), total));

    Ok(Json(result_vec))
}
