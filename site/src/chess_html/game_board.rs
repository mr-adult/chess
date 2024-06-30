use axum::response::Html;
use chess_common::{File, Location, Rank};
use chess_core::Board;
use html_to_string_macro::html;

use super::svg::create_option_piece_svg;

pub(crate) async fn new_game() -> Html<String> {
    let raw_board = Board::default();
    Html(html! {
        <div id="game_state_wrapper">
            {render_gameboard(&raw_board).0}
        </div>
    })
}

pub(crate) fn render_gameboard(board: &Board) -> Html<String> {
    let ergo_board = board.into_ergo_board();

    let board_html = Rank::all_ranks_ascending().rev().map(|rank| {
        let row = File::all_files_ascending()
            .map(|file| {
                let piece = ergo_board[Location::new(file, rank)];
                let is_light_square = (rank.as_int() + file.as_int()) % 2 != 0;
                html!{
                    <span class={
                        if is_light_square {
                            "board_square light"
                        } else {
                            "board_square dark"
                        }
                    } rank={rank.as_char()} file={file.as_char()}>
                        <span class="column_centering piece_drag_target" rank={rank.as_char()} file={file.as_char()} draggable=true>
                            {create_option_piece_svg(piece).0}
                        </span>
                    </span>
                }
            })
            .collect::<String>();

        html!{
            <div class="rank" rank={rank.as_char()}>
                {row}
            </div>
        }
    }).collect::<String>();

    Html(html! {
        <div id="game_board">
            {board_html}
        </div>
        <div style="text-align: center;">
            <h5 style="margin-bottom: 5px;">"Current Position:"</h5>
            <textarea readonly disabled id="game_fen" style="text-align: center; overflow: hidden; width: 400px; resize: none;">
                {board.to_string()}
            </textarea>
        </div>
    })
}
