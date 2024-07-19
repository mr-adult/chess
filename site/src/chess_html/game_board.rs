use axum::response::Html;
use chess_common::{File, Location, Player, Rank};
use chess_core::Board;
use chess_parsers::PieceLocations;
use html_to_string_macro::html;

use super::svg::create_option_piece_svg;

pub(crate) fn render_gameboard_full_page(board: &Board) -> Html<String> {
    Html(html! {
        <!DOCTYPE html>
        <head>
            <title>"Chess"</title>
            <link rel="icon" type="image/x-icon" href="/favicon.ico" />
            <meta charset="UTF-8" />
            <meta name="description" content="A chess website" />
            <link rel="stylesheet" href="/styles/app.css" />
            <script src="/scripts/app.js"></script>
        </head>
        <body>
            <div id="game_state_wrapper">
                {render_gameboard(&board).0}
            </div>
        </body>
    })
}

pub(crate) fn render_gameboard(board: &Board) -> Html<String> {
    let ergo_board: PieceLocations = board.into();

    let board_html = Rank::all_ranks_ascending().rev().map(|rank| {
        let row = File::all_files_ascending()
            .map(|file| {
                let piece = ergo_board[&Location::new(file, rank)];
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

    let move_history_table = board.get_move_history_acn().into_iter().enumerate().fold(
        Vec::new(),
        |mut aggregate, (i, move_)| {
            let expect_message = "To always have a pair in the pairs list";
            match board.starting_position().player_to_move() {
                Player::White => {
                    if i % 2 == 0 {
                        aggregate.push([Some(move_), None]);
                    } else {
                        let pair_to_add_to = aggregate.last_mut().expect(expect_message);
                        pair_to_add_to[1] = Some(move_);
                    }
                }
                Player::Black => {
                    if i == 0 {
                        aggregate.push([None, Some(move_)]);
                    } else if i % 2 == 1 {
                        aggregate.push([Some(move_), None]);
                    } else {
                        aggregate.last_mut().expect(&expect_message)[1] = Some(move_);
                    }
                }
            }

            aggregate
        },
    );

    let td_style = "padding: 10px; border: 1px solid black; border-collapse: collapse;";
    let mut final_html = vec![html! {
        <tr>
            <td style={td_style}></td>
            <td style={td_style}>"White"</td>
            <td style={td_style}>"Black"</td>
        </tr>
    }];

    let mut move_num_counter = 1;
    let mut num_rows = 0;
    move_history_table
        .into_iter()
        .enumerate()
        .map(|(row_num, row)| {
            num_rows += 1;
            html! {
                <tr style="padding: 10px; border: 1px solid black; border-collapse: collapse;">
                    {row.into_iter()
                        .enumerate()
                        .map(|(i, move_)| {
                            html!{
                                {if i == 0 {
                                    html!{<td style="text-align: right; padding: 10px;">{row_num + 1}"."</td>}
                                } else { "".to_string() }}
                                <td {if move_.is_some() {
                                    let mut move_attr = "move=\"".to_string();
                                    move_attr.push_str(&move_num_counter.to_string());
                                    move_attr.push('"');
                                    move_attr
                                } else { "".to_string() }} style={td_style}>
                                    {match move_ {
                                        None => "".to_string(),
                                        Some(move_) => {
                                            move_num_counter += 1;
                                            move_.to_string()
                                        }
                                    }}
                                </td>
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("")}
                </tr>
            }
        })
        .for_each(|row_html| final_html.push(row_html));

    let move_history_table_html = final_html.join("");

    Html(html! {
        <div style="display: flex; flex-direction: row; justify-content: center; column-gap: 20px">
            <div id="game_board">
                {board_html}
            </div>
            <div id="initial_board_fen" style="display: none;">{board.starting_position().to_string()}</div>
            <div id="game_history">
                <table border="1" style="padding: 10px; border: 1px solid black; border-collapse: collapse;">
                    <tbody>
                        {move_history_table_html}
                    </tbody>
                </table>
            </div>
        </div>
        <div style="text-align: center;">
            <h5 style="margin-bottom: 5px;">"Current Position:"</h5>
            <textarea readonly disabled id="game_fen" style="text-align: center; overflow: hidden; width: 400px; resize: none;">
                {board.to_fen_string()}
            </textarea>
        </div>
    })
}
