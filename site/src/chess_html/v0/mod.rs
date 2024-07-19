use axum::extract::Query;
use axum::response::Html;
use axum::{routing::get, Router};
use chess_core::Board;
use http::StatusCode;
use std::str::FromStr;

use super::game_board::render_gameboard_full_page;
use crate::common::{FenRequest, PerftRequest};

pub(crate) fn create_ssr_router() -> Router {
    Router::new()
        .route("/new_game", get(new_game))
        .route("/render_board", get(render_board_handler))
        .route("/perft", get(perft_handler))
}

pub(crate) async fn new_game() -> Html<String> {
    let board = Board::default();
    render_gameboard_full_page(&board)
}

pub(crate) async fn render_board_handler(
    board_fen: Query<FenRequest>,
) -> Result<Html<String>, StatusCode> {
    let board = Board::from_str(&board_fen.0.board_fen).map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(render_gameboard_full_page(&board))
}

async fn perft_handler(req: Query<PerftRequest>) -> Result<Html<String>, StatusCode> {
    let mut board = Board::from_str(&req.0.board_fen).map_err(|_| StatusCode::BAD_REQUEST)?;
    let perft = board.perft(req.0.depth);

    #[cfg(debug_assertions)]
    let moves_raw = perft
        .iter()
        .map(|perft_move| {
            match board.make_move_acn(&perft_move.0.to_string()) {
                Err(err) => println!(
                    "Failed to make move: '{}'. Err: {err:?}",
                    perft_move.0.to_string()
                ),
                Ok(val) => val,
            };
            let move_ = board.undo().unwrap();
            (move_.move_().clone(), perft_move.1)
        })
        .collect::<Vec<_>>();

    let mut html = String::new();
    let mut total = 0;
    html.push_str("<table><tbody>");

    // perft with raw moves (like stockfish)
    #[cfg(debug_assertions)]
    for item in moves_raw {
        let move_string = format!("{:?}", item.0);
        html.push_str("<tr>");

        html.push_str("<td>");
        html.push_str(&move_string);
        html.push_str("</td>");

        html.push_str("<td>");
        html.push_str(&item.1.to_string());
        html.push_str("</td>");

        html.push_str("</tr>");

        total += item.1;
    }

    #[cfg(not(debug_assertions))]
    for item in perft {
        let acn = item.0.to_string();

        html.push_str("<tr>");

        html.push_str("<td>");
        html.push_str(&acn);
        html.push_str("</td>");

        html.push_str("<td>");
        html.push_str(&item.1.to_string());
        html.push_str("</td>");

        html.push_str("</tr>");

        total += item.1;
    }

    html.push_str("<tr>");
    html.push_str("<td>Total</td>");
    html.push_str("<td>");
    html.push_str(&total.to_string());
    html.push_str("</td>");
    html.push_str("</tr>");

    html.push_str("</tbody></table>");

    Ok(Html(html))
}
