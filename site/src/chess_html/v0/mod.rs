use axum::extract::{Path, Query};
use axum::response::Html;
use axum::{routing::get, Router};
use chess_core::Board;
use chess_parsers::GameResult;
use html_to_string_macro::html;
use http::StatusCode;
use rusqlite::Row;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::str::FromStr;

use super::game_board::render_gameboard_full_page;
use crate::chess_html::render_gameboard_without_history;
use crate::common::{FenRequest, PerftRequest};
use crate::open_db;

pub(crate) fn create_ssr_router() -> Router {
    Router::new()
        .route("/new_game", get(new_game))
        .route("/render_board", get(render_board_handler))
        .route("/render_position", get(render_position_handler))
        .route("/perft", get(perft_handler))
        .route("/games", get(get_all_games_handler))
        .route("/games/", get(get_all_games_handler))
        .route("/games/{game_id}", get(get_game_handler))
}

pub(crate) async fn new_game() -> Html<String> {
    let board = Board::default();
    render_gameboard_full_page(&board)
}

pub(crate) async fn analysis_handler() -> Html<String> {
    Html(html!{
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            <title>"Chess Database"</title>
            <script src="/scripts/htmx.min.js"></script>
        </head>
        <body style="margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); min-height: 100vh;">
            <div style="max-width: 1200px; margin: 0 auto; padding: 20px;">
                <header style="text-align: center; margin-bottom: 40px;">
                    <h1 style="color: white; font-size: 2.5em; margin: 0; text-shadow: 0 2px 4px rgba(0,0,0,0.3);">"Chess Database"</h1>
                    <p style="color: rgba(255,255,255,0.9); font-size: 1.2em; margin: 10px 0 0 0;">"Browse and analyze chess games"</p>
                </header>

                <div style="background: white; border-radius: 12px; box-shadow: 0 10px 30px rgba(0,0,0,0.2); overflow: hidden;">
                    <div style="background: linear-gradient(135deg, #4a90e2 0%, #357abd 100%); color: white; padding: 20px; border-bottom: 1px solid #eee;">
                        <h2 style="margin: 0; font-size: 1.5em;">"Games Library"</h2>
                        <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 15px;">
                            <div style="display: flex; gap: 20px;">
                                <span id="totalGames" style="background: rgba(255,255,255,0.2); padding: 8px 16px; border-radius: 20px; font-size: 0.9em;">"Total: 0 games"</span>
                                <span id="validGames" style="background: rgba(76,175,80,0.3); padding: 8px 16px; border-radius: 20px; font-size: 0.9em;">"Valid: 0"</span>
                                <span id="invalidGames" style="background: rgba(244,67,54,0.3); padding: 8px 16px; border-radius: 20px; font-size: 0.9em;">"Invalid: 0"</span>
                            </div>
                            <button 
                                hx-get="/html/v0/games"
                                hx-trigger="click"
                                hx-target="#gamesTable"
                                hx-swap="outerHTML"
                                style="background: rgba(255,255,255,0.2); border: 1px solid rgba(255,255,255,0.3); color: white; padding: 8px 16px; border-radius: 6px; cursor: pointer; font-size: 0.9em;">
                                "🔄 Refresh"
                            </button>
                        </div>
                    </div>

                    <div style="padding: 0;">
                        <div id="gamesTable" 
                            hx-trigger="load"
                            hx-get="/html/v0/games"
                            hx-swap="outerHTML"
                            style="text-align: center; padding: 60px; color: #666; display: block;">
                            <div style="display: inline-block; width: 40px; height: 40px; border: 3px solid #f3f3f3; border-top: 3px solid #4a90e2; border-radius: 50%; animation: spin 1s linear infinite;"></div>
                            <p style="margin-top: 20px; font-size: 1.1em;">"Loading games..."</p>
                        </div>

                        <div id="error" style="text-align: center; padding: 60px; color: #dc3545; display: none;">
                            <div style="font-size: 3em; margin-bottom: 20px;">"⚠️"</div>
                            <h3 style="margin: 0 0 10px 0;">"Error Loading Games"</h3>
                            <p style="margin: 0; color: #6c757d;">"Please make sure the database API is running"</p>
                        </div>
                    </div>
                </div>
            </div>

            <style>r#"
                @keyframes spin {
                    0% { transform: rotate(0deg); }
                    100% { transform: rotate(360deg); }
                }

                tbody tr:hover {
                    background-color: #f8f9fa;
                }

                .status-valid {
                    background: #d4edda;
                    color: #155724;
                    padding: 4px 12px;
                    border-radius: 12px;
                    font-size: 0.8em;
                    font-weight: 600;
                    white-space: nowrap;
                }

                .status-invalid {
                    background: #f8d7da;
                    color: #721c24;
                    padding: 4px 12px;
                    border-radius: 12px;
                    font-size: 0.8em;
                    font-weight: 600;
                    white-space: nowrap;
                }

                .btn-primary {
                    background: linear-gradient(135deg, #4a90e2 0%, #357abd 100%);
                    border: none;
                    color: white;
                    padding: 8px 16px;
                    border-radius: 6px;
                    text-decoration: none;
                    font-size: 0.85em;
                    font-weight: 500;
                    cursor: pointer;
                    transition: all 0.2s;
                    white-space: nowrap;
                }

                .btn-primary:hover {
                    transform: translateY(-1px);
                    box-shadow: 0 4px 12px rgba(74, 144, 226, 0.3);
                }
            "#</style>
            
            <script>r#"
                function updateStats() {
                    let trs = document.querySelectorAll("tr");
                    const total = Math.max(trs.length - 1, 0);
                    const valid = Math.max(trs.length - 1, 0);
                    const invalid = 0;

                    document.getElementById('totalGames').textContent = `Total: ${total} games`;
                    document.getElementById('validGames').textContent = `Valid: ${valid}`;
                    document.getElementById('invalidGames').textContent = `Invalid: ${invalid}`;
                }
            
                document.body.addEventListener("htmx:afterRequest", updateStats)
            "#</script>
        </body>
        </html>
    })
}

pub(crate) async fn render_board_handler(
    board_fen: Query<FenRequest>,
) -> Result<Html<String>, StatusCode> {
    let board = Board::from_str(&board_fen.0.board_fen).map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(render_gameboard_full_page(&board))
}

pub(crate) async fn render_position_handler(
    board_fen: Query<FenRequest>,
) -> Result<Html<String>, StatusCode> {
    let board = Board::from_str(&board_fen.0.board_fen).map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok(render_gameboard_without_history(&board))
}

async fn perft_handler(req: Query<PerftRequest>) -> Result<Html<String>, StatusCode> {
    let mut board = Board::from_str(&req.0.board_fen).map_err(|_| StatusCode::BAD_REQUEST)?;
    let perft = board.perft(
        req.0.depth,
        std::thread::available_parallelism()
            .ok()
            .and_then(|non_zero| Some(non_zero.get()))
            .unwrap_or(1),
    );

    #[cfg(debug_assertions)]
    let moves_raw = perft
        .iter()
        .map(|perft_move| {
            match board.make_move_acn(&perft_move.0.to_string()) {
                Err(err) => println!(
                    "Failed to make move: '{}'. Err: {err:?}",
                    perft_move.0.to_string()
                ),
                Ok(_) => (),
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

async fn get_all_games_handler() -> Html<String> {
    let conn = open_db().unwrap();
    let table_rows = conn
        .prepare("SELECT * FROM games;")
        .unwrap()
        .query_map([], |row| Ok(Game::from_row(row)))
        .unwrap()
        .enumerate()
        .map(|(i, game)| {
            let game = game.unwrap();
            let tr_color = if i % 2 == 0 {
                "#ffffff"
            } else {
                "#f8f9fa"
            };

            let tr_style = format!("border-bottom: 1px solid #dee2e6; color: {}", tr_color);

            let mut player_info = String::new();
            if !game.white.is_empty() {
                player_info.push_str(&html_escape(&game.white));
            } else {
                player_info.push_str("Unknown");
            }
            player_info.push_str(" vs. ");
            if !game.black.is_empty() {
                player_info.push_str(&html_escape(&game.black));
            } else {
                player_info.push_str("Unknown");
            }

            let row_index = i + 1;

            html!{
                <tr style={tr_style}>
                    <td aria-colindex="1"
                        aria-rowindex={row_index}
                        style="padding: 15px 20px; border-right: 1px solid #dee2e6; color: #6c757d">
                        <div style="font-weight: 500;">{
                            if game.event.is_empty() {
                                Cow::Borrowed("Unknown")
                            } else {
                                html_escape(&game.event)
                            }
                        }</div>
                        {
                            if game.round.is_empty() {
                                "".to_string()
                            } else {
                                format!(r#"<div style="font-size: 0.8em; color: #6c757d; margin-top: 2px;">Round {}"</div>"#, html_escape(&game.round))
                            }
                        }
                    </td>
                    <td aria-colindex="2"
                        aria-rowindex={row_index}
                        style="padding: 15px 20px; border-right: 1px solid #dee2e6; color: #6c757d;">
                        <div style="font-weight: 500;">{player_info}</div>
                    </td>
                    <td aria-colindex="3"
                        aria-rowindex={row_index}
                        style="padding: 15px 20px; border-right: 1px solid #dee2e6; color: #6c757d;">
                        {
                            if game.date.is_empty() {
                                Cow::Borrowed("Unknown")
                            } else {
                                html_escape(&game.date)
                            }
                        }
                    </td>
                    <td aria-colindex="4"
                        aria-rowindex={row_index}
                        style="padding: 15px 20px; border-right: 1px solid #dee2e6; color: #6c757d;">
                        {
                            if game.site.is_empty() {
                                Cow::Borrowed("Unknown")
                            } else {
                                html_escape(&game.site)
                            }
                        }
                    </td>
                    <td aria-colindex="4"
                        aria-rowindex={row_index}
                        style="padding: 15px 20px; border-right: 1px solid #dee2e6; color: #6c757d;">
                        {GameResult::try_from(game.result).unwrap_or(GameResult::Inconclusive).as_ref()}
                    </td>
                    <td aria-colindex="5"
                        aria-rowindex={row_index}
                        style="padding: 15px 20px; border-right: 1px solid #dee2e6; color: #6c757d;">
                        {
                            if true /* game.isValid */ {
                                format!(r#"<a href="/html/v0/games/{}?move_number=0" class="btn-primary">View Moves</a>"#, game.id)
                            } else {
                                r#"<span style="font-size: 0.8em;">No moves available</span>"#.to_string()
                            }
                        }
                    </td>
                </tr>
            }
        })
        .collect::<Vec<_>>();

    Html(html! {
        <table id="gamesTable"
            aria-colcount="5"
            aria-rowcount={table_rows.len().to_string()}
            style="width: 100%; border-collapse: collapse; font-size: 0.9em;">
            <thead>
                <tr style="background: #f8f9fa; border-bottom: 2px solid #dee2e6;">
                    <th style="padding: 15px 20px; text-align: left; font-weight: 600; color: #495057;">"Event"</th>
                    <th style="padding: 15px 20px; text-align: left; font-weight: 600; color: #495057;">"Players"</th>
                    <th style="padding: 15px 20px; text-align: left; font-weight: 600; color: #495057;">"Date"</th>
                    <th style="padding: 15px 20px; text-align: left; font-weight: 600; color: #495057;">"Site"</th>
                    <th style="padding: 15px 20px; text-align: left; font-weight: 600; color: #495057;">"Result"</th>
                    <th style="padding: 15px 20px; text-align: center; font-weight: 600; color: #495057;">"Actions"</th>
                </tr>
            </thead>
            <tbody id="gamesTableBody">
                {table_rows.into_iter().collect::<String>()}
            </tbody>
        </table>
    })
}

async fn get_game_handler(
    Path(game_id): Path<i32>,
    Query(move_number): Query<MoveNumber>,
) -> Result<Html<String>, StatusCode> {
    let connection = open_db().unwrap();

    let game = connection
        .query_one("SELECT * FROM games WHERE id = ?", [game_id], |row| {
            Ok(Game::from_row(row))
        })
        .unwrap();

    let mut stmt = connection
        .prepare(
            r#"
SELECT m.move_number,
    m.fen_after,
    m.acn
FROM games g
LEFT OUTER JOIN moves m
ON g.id = m.game_id
WHERE g.id = ?
ORDER BY m.move_number;"#,
        )
        .unwrap();

    let moves = stmt
        .query_map([game_id], |row| {
            Ok(SingleMove {
                move_number: row.get("move_number").unwrap(),
                acn: row.get("acn").unwrap(),
                fen_after: row.get("fen_after").unwrap(),
            })
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let target_move = moves.iter().nth(move_number.move_number);

    let board = if let Some(move_) = target_move {
        match Board::from_str(&move_.fen_after) {
            Ok(board) => board,
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        Board::default()
    };

    Ok(Html(html! {
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8" />
            <meta name="viewport" content="width=device-width, initial-scale=1.0" />
            <title>"Chess Game Details"</title>
            <link rel="stylesheet" href="/styles/app.css" />
            <script src="/scripts/htmx.min.js"></script>
        </head>
        <body style="margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); min-height: 100vh;">
            <div style="max-width: 1200px; margin: 0 auto; padding: 20px;">
                <header style="text-align: center; margin-bottom: 40px;">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                        <a href="/analysis" style="background: rgba(255,255,255,0.2); border: 1px solid rgba(255,255,255,0.3); color: white; padding: 10px 20px; border-radius: 6px; text-decoration: none; font-weight: 500; transition: all 0.2s;">
                            "← Back to Games"
                        </a>
                        <h1 id="gameTitle" style="color: white; font-size: 2.2em; margin: 0; text-shadow: 0 2px 4px rgba(0,0,0,0.3);">"Game Details"</h1>
                        <div style="width: 120px;"></div>
                    </div>
                </header>

                <div style="display: grid; gap: 20px; align-items: start; grid-template-columns: repeat(auto-fill, calc(20% - 5px) calc(60% - 10px) calc(20% - 5px))">
                    <div style="background: white; border-radius: 12px; box-shadow: 0 10px 30px rgba(0,0,0,0.2); overflow: hidden; top: 20px; position: sticky;">
                        <div style="background: linear-gradient(135deg, #4a90e2 0%, #357abd 100%); color: white; padding: 20px;">
                            <h2 style="margin: 0; font-size: 1.5em;">"Game Information"</h2>
                        </div>

                        <div id="gameInfo" style="padding: 10px;">
                            <div style="display: grid; grid-template-columns: 1fr; gap: 10px;">
                                <div>
                                    <h3 style="margin: 0 0 10px 0; color: #495057; font-size: 1.1em;">"Players"</h3>
                                    <div style="font-weight: 600; font-size: 1.1em; margin-bottom: 5px;">
                                        <span style="color: #212529;">"⚪ " {
                                            if game.white.is_empty() {
                                                Cow::Borrowed("Unknown")
                                            } else {
                                                html_escape(&game.white)
                                            }
                                        }</span>
                                    </div>
                                    <div style="font-weight: 600; font-size: 1.1em;">
                                        <span style="color: #212529;">"⚫ " {
                                            if game.black.is_empty() {
                                                Cow::Borrowed("Unknown")
                                            } else {
                                                html_escape(&game.black)
                                            }
                                        }</span>
                                    </div>
                                </div>
                                <div>
                                    <h3 style="margin: 0 0 10px 0; color: #495057; font-size: 1.1em;">"Event"</h3>
                                    <div id="event" style="color: #6c757d;">{html_escape(&game.event)}</div>
                                </div>
                                <div>
                                    <h3 style="margin: 0 0 10px 0; color: #495057; font-size: 1.1em;">"Date"</h3>
                                    <div id="date" style="color: #6c757d;">{html_escape(&game.date)}</div>
                                </div>
                                <div>
                                    <h3 style="margin: 0 0 10px 0; color: #495057; font-size: 1.1em;">"Site"</h3>
                                    <div id="site" style="color: #6c757d;">{html_escape(&game.site)}</div>
                                </div>
                                <div>
                                    <h3 style="margin: 0 0 10px 0; color: #495057; font-size: 1.1em;">"Result"</h3>
                                    <div id="site" style="color: #6c757d;">{html_escape(GameResult::try_from(game.result).unwrap_or_default().as_ref())}</div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div style="background: white; border-radius: 12px; box-shadow: 0 10px 30px rgba(0,0,0,0.2); overflow: hidden; top: 20px; position: sticky;">
                        <div style="background: linear-gradient(135deg, #28a745 0%, #20c997 100%); color: white; padding: 15px; text-align: center;">
                            <h3 style="margin: 0; font-size: 1.2em;">"Current Position"</h3>
                            <div id="currentMoveInfo" style="font-size: 0.9em; margin-top: 5px; opacity: 0.9;">"Click a move to view position"</div>
                        </div>
                        <div id="boardContainer" style="padding: 20px; text-align: center;">
                            <div style="color: #6c757d; padding: 40px;">
                                {render_gameboard_without_history(&board).0}
                            </div>
                        </div>
                    </div>
                    
                    <div style="background: white; border-radius: 12px; box-shadow: 0 10px 30px rgba(0,0,0,0.2); overflow: hidden;">
                        <div style="background: linear-gradient(135deg, #6f42c1 0%, #e83e8c 100%); color: white; padding: 20px;">
                            <h2 style="margin: 0; font-size: 1.5em;">"Game Moves"</h2>
                            <div style="margin-top: 10px; font-size: 0.9em; opacity: 0.9;">"Click any move to view the board position at that moment"</div>
                        </div>
                    
                        <div style="padding: 0;">
                            <div id="movesContainer" style="padding: 10px;">
                                <div id="movesList" style="display: grid; grid-template-columns: repeat(3, minmax(10px, 1fr)); gap: 8px;">
                                    {
                                        moves.into_iter().map(|move_| {
                                            let button = html!{
                                                <div>
                                                    <label for={format!("move{}", move_.move_number)}>{html_escape(&move_.acn)}</label>
                                                    <input id={format!("move{}", move_.move_number)}
                                                        {
                                                            if move_.move_number == move_number.move_number {
                                                                "checked=\"true\""
                                                            } else {
                                                                ""
                                                            }
                                                        }
                                                        type="radio"
                                                        name="move"
                                                        hx-get={format!("/html/v0/render_position?board_fen={}", move_.fen_after.replace(' ', "%20"))}
                                                        hx-target="#chessBoard"
                                                        hx-trigger="click"
                                                        hx-swap="outerHTML"
                                                        hx-replace-url={format!("/html/v0/games/{}?move_number={}", game_id, move_.move_number)}>
                                                    </input>
                                                </div>
                                            };
                                            if move_.move_number % 2 == 0 {
                                                html!{
                                                    <span class="move-number">{move_.move_number / 2 + 1}</span>
                                                    {button}
                                                }
                                            } else {
                                                button
                                            }
                                        }).collect::<String>()
                                    }
                                </div>
                            </div>
                    
                            <div id="error" style="text-align: center; padding: 60px; color: #dc3545; display: none;">
                                <div style="font-size: 3em; margin-bottom: 20px;">"⚠️"</div>
                                <h3 style="margin: 0 0 10px 0;">"Error Loading Game"</h3>
                                <p style="margin: 0; color: #6c757d;">"Game not found or API unavailable"</p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <style>r#"
                @keyframes spin {
                    0% { transform: rotate(0deg); }
                    100% { transform: rotate(360deg); }
                }

                .move-button {
                    background: #f8f9fa;
                    border: 2px solid #dee2e6;
                    padding: 12px 8px;
                    border-radius: 8px;
                    cursor: pointer;
                    font-weight: 600;
                    text-align: center;
                    transition: all 0.2s;
                    font-family: 'Courier New', monospace;
                    font-size: 0.9em;
                    color: #495057;
                    text-decoration: none;
                    display: block;
                }
                
                .move-button:hover {
                    background: #e9ecef;
                    border-color: #6f42c1;
                    transform: translateY(-2px);
                    box-shadow: 0 4px 12px rgba(111, 66, 193, 0.2);
                }
                
                .move-button.active {
                    background: linear-gradient(135deg, #6f42c1 0%, #e83e8c 100%);
                    color: white;
                    border-color: #6f42c1;
                    box-shadow: 0 4px 12px rgba(111, 66, 193, 0.3);
                }
                
                .move-number {
                    font-size: 0.8em;
                    color: #6c757d;
                    display: block;
                    margin-bottom: 4px;
                }
                
                .move-button.active .move-number {
                    color: rgba(255,255,255,0.8);
                }
                
                a[href*="index.html"]:hover {
                    background: rgba(255,255,255,0.3);
                    transform: translateY(-1px);
                }
            "#</style>
        </body>
        </html>
    }))
}

#[derive(Serialize)]
struct Game {
    id: i32,
    event: String,
    site: String,
    date: String,
    round: String,
    white: String,
    black: String,
    result: u8, /* GameResult */
}

impl Game {
    fn from_row(row: &Row) -> Self {
        let id: i32 = row.get("id").unwrap();
        let event: Option<String> = row.get("event").unwrap();
        let site: Option<String> = row.get("site").unwrap();
        let date: Option<String> = row.get("date").unwrap();
        let round: Option<String> = row.get("round").unwrap();
        let white: Option<String> = row.get("white").unwrap();
        let black: Option<String> = row.get("black").unwrap();
        let result: Option<u8> = row.get("result").unwrap();

        Game {
            id,
            event: event.unwrap_or(String::with_capacity(0)),
            site: site.unwrap_or(String::with_capacity(0)),
            date: date.unwrap_or(String::with_capacity(0)),
            round: round.unwrap_or(String::with_capacity(0)),
            white: white.unwrap_or(String::with_capacity(0)),
            black: black.unwrap_or(String::with_capacity(0)),
            result: result.unwrap_or(GameResult::Inconclusive as u8)
        }
    }
}

#[derive(Deserialize)]
struct MoveNumber {
    move_number: usize,
}

#[derive(Serialize)]
struct SingleMove {
    move_number: usize,
    acn: String,
    fen_after: String,
}

fn html_escape(html: &str) -> Cow<str> {
    let mut result = Cow::Borrowed(html);

    for (i, ch) in html.char_indices() {
        match ch {
            '&' => {
                const AMP: &'static str = "&amp;";
                match &mut result {
                    Cow::Borrowed(_) => {
                        let mut string = html[0..i].to_string();
                        string.push_str(AMP);
                        result = Cow::Owned(string);
                    }
                    Cow::Owned(string) => {
                        string.push_str(AMP);
                    }
                }
            }
            '<' => {
                const LT: &'static str = "&lt;";
                match &mut result {
                    Cow::Borrowed(_) => {
                        let mut string = html[0..i].to_string();
                        string.push_str(LT);
                        result = Cow::Owned(string);
                    }
                    Cow::Owned(string) => {
                        string.push_str(LT);
                    }
                }
            }
            '>' => {
                const GT: &'static str = "&gt;";
                match &mut result {
                    Cow::Borrowed(_) => {
                        let mut string = html[0..i].to_string();
                        string.push_str(GT);
                        result = Cow::Owned(string);
                    }
                    Cow::Owned(string) => {
                        string.push_str(GT);
                    }
                }
            }
            '"' => {
                const QUOT: &'static str = "&quot;";
                match &mut result {
                    Cow::Borrowed(_) => {
                        let mut string = html[0..i].to_string();
                        string.push_str(QUOT);
                        result = Cow::Owned(string);
                    }
                    Cow::Owned(string) => {
                        string.push_str(QUOT);
                    }
                }
            }
            '\'' => {
                const APOS: &'static str = "&#x27;";
                match &mut result {
                    Cow::Borrowed(_) => {
                        let mut string = html[0..i].to_string();
                        string.push_str(APOS);
                        result = Cow::Owned(string);
                    }
                    Cow::Owned(string) => {
                        string.push_str(APOS);
                    }
                }
            }
            _ => {
                if let Cow::Owned(string) = &mut result {
                    string.push(ch);
                }
            }
        }
    }

    result
}
