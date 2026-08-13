use std::{fs::OpenOptions, io::Read, process::ExitCode};

use chess_common::{File, PieceKind, Player, Rank};
use chess_core::{AcnMoveErr, Board};
use chess_parsers::{Check, ParsedGame, PgnErr, PieceMoveKind};
use clap::{command, Arg, Command};
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use log::error;
use rusqlite::{Connection, Error};

fn main() -> ExitCode {
    env_logger::init();

    let matches = create_command().get_matches();

    match matches.subcommand() {
        Some(("load", args)) => {
            let sqlite_db = args
                .get_one::<String>("destination sqlite file")
                .expect("'destination sqlite file' is required");

            let files = args
                .get_many::<String>("pgn files")
                .expect("'pgn files' is required")
                .collect::<Vec<_>>();

            if handle_load_subcommand(sqlite_db, files).is_ok() {
                return ExitCode::SUCCESS;
            } else {
                return ExitCode::FAILURE;
            }
        }
        Some((_, _)) => {
            unreachable!("clap should prevent coming to this branch");
        }
        None => {
            create_command().print_long_help().ok();
            return ExitCode::FAILURE;
        }
    }
}

fn create_command() -> Command {
    command!().subcommand(
        Command::new("load")
            .arg(
                Arg::new("destination sqlite file")
                    .required(true)
                    .help("the file where the sqlite database should be created"),
            )
            .arg(
                Arg::last(Arg::new("pgn files"), true)
                .num_args(1..)
                    .required(true)
                    .help("the pgn files to be loaded into the sqlite"),
            ),
    )
}

fn handle_load_subcommand(sqlite_db: &str, files: Vec<&String>) -> Result<(), ()> {
    let sqlite_conn = match Connection::open(sqlite_db) {
        Err(err) => {
            error!("Failed to open SQLite database. Inner error: {err}");
            return Err(());
        }
        Ok(conn) => conn,
    };

    if let Err(err) = initialize_sqlite_db(&sqlite_conn) {
        error!("Failed to initialize SQLite database. Inner error: {err}");
        return Err(());
    }

    for file_name in files {
        match OpenOptions::new().read(true).write(false).open(file_name) {
            Err(err) => {
                error!("Failed to open {file_name}. Inner error: {err}");
                return Err(());
            }
            Ok(mut file) => {
                let mut pgn = String::new();
                if let Err(err) = file.read_to_string(&mut pgn) {
                    error!("Failed to read {file_name}. Inner error: {err}");
                    return Err(());
                }

                let pgn_string = pgn.to_string();
                let parsed_pgn = chess_parsers::parse_pgn(&pgn);

                match parsed_pgn {
                    Err(err) => {
                        let mut files = SimpleFiles::new();
                        let file_id = files.add(file_name, &pgn_string);

                        let writer = StandardStream::stderr(ColorChoice::Always);
                        let config = codespan_reporting::term::Config::default();

                        let diagnostic = match err {
                            PgnErr::UnexpectedCharacter(char_err) => {
                                let mut start = char_err.location().byte_index();
                                let mut end = start;
                                let ch_len = &pgn_string[start..]
                                    .chars()
                                    .next()
                                    .map(char::len_utf8)
                                    .unwrap_or(0);
                                if end < (pgn_string.len() - 1) {
                                    end += 1;
                                } else if start > 0 {
                                    start -= 1;
                                }

                                let message = "Unexpected character.".to_string();

                                Diagnostic::error()
                                    .with_message(&message)
                                    .with_label(Label::primary(file_id, start..end))
                            }
                            PgnErr::Token(token_err) => Diagnostic::error()
                                .with_message("unexpected token")
                                .with_label(Label::primary(
                                    file_id,
                                    match token_err.found_span() {
                                        None => {
                                            let end = pgn_string.len();
                                            let mut start = end;
                                            if start > 0 {
                                                start -= 1;
                                            }

                                            start..end
                                        }
                                        Some(span) => span.into(),
                                    },
                                )),
                            PgnErr::InvalidAlgebraicChessNotation { span, value } => {
                                Diagnostic::error()
                                    .with_message(&format!(
                                        "Invalid algebraic chess notation '{value}'"
                                    ))
                                    .with_label(Label::primary(file_id, &span))
                            }
                            PgnErr::InvalidTagName { span, tag } => Diagnostic::error()
                                .with_message(&format!("invalid tag name '{tag}'"))
                                .with_label(Label::primary(file_id, span)),
                        };

                        term::emit(&mut writer.lock(), &config, &files, &diagnostic).ok();
                        return Err(());
                    }
                    Ok(pgn) => {
                        let ProcessedTables {
                            legal_games,
                            illegal_games,
                        } = process_tables(pgn);

                        if let Err(()) = insert_legal_games(&sqlite_conn, legal_games) {
                            return Err(());
                        }

                        if let Err(()) = insert_illegal_games(&sqlite_conn, illegal_games) {
                            return Err(());
                        }

                        if let Err(_) = sqlite_conn.pragma_update(None, "journal_mode", "DELETE") {
                            error!("Failed to reset SQLite to standard journal_mode");
                        }

                        return Ok(());
                    }
                }
            }
        };
    }

    Ok(())
}

fn initialize_sqlite_db(conn: &Connection) -> Result<(), Error> {
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "wal")?;

    for create in [
        include_str!("../sql/create_pieces.sql"),
        include_str!("../sql/create_games.sql"),
        include_str!("../sql/create_tags.sql"),
        include_str!("../sql/create_moves.sql"),
        include_str!("../sql/create_illegal_games.sql"),
        include_str!("../sql/create_illegal_tags.sql"),
        include_str!("../sql/create_illegal_moves.sql"),
    ] {
        conn.execute(create, ())?;
    }

    let inserts = [
        PieceKind::Pawn,
        PieceKind::Knight,
        PieceKind::Bishop,
        PieceKind::Rook,
        PieceKind::Queen,
        PieceKind::King,
    ]
    .into_iter()
    .map(|piece_kind| {
        let piece_kind_id: u8 = piece_kind as u8;
        format!("({},'{}')", piece_kind_id, piece_kind.to_string())
    })
    .collect::<Vec<_>>()
    .join(",");

    let mut insert_stmt = "INSERT INTO pieces (id, value) VALUES ".to_string();
    insert_stmt.push_str(&inserts);
    conn.execute(&insert_stmt, [])?;

    Ok(())
}

fn process_tables(parsed_games: Vec<ParsedGame>) -> ProcessedTables {
    let mut legal_games = Vec::with_capacity(parsed_games.len());
    let mut illegal_games = Vec::new();

    'game_loop: for game in parsed_games {
        let mut board = Board::default();

        let mut moves = Vec::new();
        for (i, move_) in game.moves.iter().enumerate() {
            let player = board.player_to_move();
            let selected_move = match board.make_move_acn(&move_.to_string()) {
                Ok(selected_move) => selected_move,
                Err(AcnMoveErr::CheckStateMismatch(selected_move)) => {
                    if i == game.moves.len() - 1 {
                        selected_move
                    } else {
                        error!("Illegal move: check state mismatch");
                        illegal_games.push(IllegalMoveRowModel {
                            parsed_game: game,
                            illegal_move_number: i,
                            fen_at_illegal_move: board.to_fen_string(),
                        });
                        continue 'game_loop;
                    }
                }
                Err(err) => {
                    error!("Illegal move: {err:?}");
                    illegal_games.push(IllegalMoveRowModel {
                        parsed_game: game,
                        illegal_move_number: i,
                        fen_at_illegal_move: board.to_fen_string(),
                    });
                    continue 'game_loop;
                }
            };

            let fen_after = board.to_fen_string();
            moves.push(FullyPopulatedMoveRowModel {
                from_file: selected_move.move_().from().file(),
                from_rank: selected_move.move_().from().rank(),
                to_file: selected_move.move_().to().file(),
                to_rank: selected_move.move_().to().rank(),
                acn: move_.to_string(),
                player,
                fen_after,
                piece: match &move_.move_kind {
                    PieceMoveKind::CastleKingside | PieceMoveKind::CastleQueenside => None,
                    PieceMoveKind::Normal(normal_move) => Some(normal_move.piece_kind),
                },
                is_castle_kingside: matches!(move_.move_kind, PieceMoveKind::CastleKingside),
                is_castle_queenside: matches!(move_.move_kind, PieceMoveKind::CastleQueenside),
            })
        }

        let mut uncategorized_tag_pairs = Vec::new();
        let mut event = None;
        let mut site = None;
        let mut date = None;
        let mut round = None;
        let mut white = None;
        let mut black = None;
        for tag in game.tag_pairs {
            let tag_name_raw = tag.0.to_string();
            let tag_name = tag.0.to_string().to_lowercase();
            let tag_value = tag.1.to_string();
            match tag_name.as_str() {
                "event" => event = Some(tag_value),
                "site" => site = Some(tag_value),
                "date" => date = Some(tag_value),
                "round" => round = Some(tag_value),
                "white" => white = Some(tag_value),
                "black" => black = Some(tag_value),
                _ => {
                    uncategorized_tag_pairs.push((tag_name_raw, tag_value));
                }
            }
        }

        legal_games.push(FullyPopulatedBoardRowModel {
            event,
            site,
            date,
            round,
            white,
            black,
            other_tags: uncategorized_tag_pairs,
            moves,
        });
    }

    ProcessedTables {
        legal_games,
        illegal_games,
    }
}

fn insert_legal_games(
    connection: &Connection,
    legal_games: Vec<FullyPopulatedBoardRowModel>,
) -> Result<(), ()> {
    let mut global_move_number = 0;
    let mut insert_game = connection.prepare("INSERT INTO games (id, event, site, date, round, white, black) VALUES (?, ?, ?, ?, ?, ?, ?);").unwrap();
    let mut insert_moves_stmt = "INSERT INTO moves (move_number, from_rank, from_file, to_rank, to_file, player, is_castle_kingside, is_castle_queenside, piece, fen_after, acn, game_id) VALUES ".to_string();

    for (game_id, game) in legal_games.into_iter().enumerate() {
        let params = (
            game_id as i64, game.event, game.site, game.date, game.round, game.white, game.black,
        );
        if let Err(err) = insert_game.execute(params) {
            error!("Failed to insert game into the database. Inner error: {err}");
            return Err(());
        }

        for tag in game.other_tags {
            if let Err(err) = connection.execute(
                "INSERT INTO tags (tag_name, tag_value, game_id) VALUES (?, ?, ?);",
                (tag.0, tag.1, game_id as i64),
            ) {
                error!("Failed to insert game tags into the database. Inner error: {err}");
                return Err(());
            }
        }

        for (move_number, move_) in game.moves.into_iter().enumerate() {
            if global_move_number != 0 {
                insert_moves_stmt.push(',');
            }

            let insert_moves_line = &format!(
                "({move_number},{},'{}',{},'{}','{}',{},{},{},'{}','{}',{game_id})",
                move_.from_rank.as_int(),
                move_.from_file.as_char(),
                move_.to_rank.as_int(),
                move_.to_file.as_char(),
                match move_.player {
                    Player::Black => "black",
                    Player::White => "white",
                },
                if move_.is_castle_kingside { 1 } else { 0 },
                if move_.is_castle_queenside { 1 } else { 0 },
                match move_.piece.map(|piece| {
                    let piece_bit: u8 = piece as u8;
                    piece_bit
                }) {
                    None => "NULL".to_string(),
                    Some(bit) => bit.to_string(),
                },
                move_.fen_after,
                move_.acn,
            );

            insert_moves_stmt.push_str(&insert_moves_line);
            global_move_number += 1;
        }
    }

    if global_move_number != 0 {
        if let Err(err) = connection.execute(&insert_moves_stmt, []) {
            error!("Failed to insert moves into the database. Inner error: {err}");
            return Err(());
        }
    }

    Ok(())
}

fn insert_illegal_games(
    connection: &Connection,
    illegal_games: Vec<IllegalMoveRowModel>,
) -> Result<(), ()> {
    let mut global_move_number = 0;
    let mut insert_game = connection.prepare("INSERT INTO illegal_games (id, event, site, date, round, white, black, illegal_move_number, fen_at_illegal_move) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?);").unwrap();
    let mut insert_moves_stmt = "INSERT INTO illegal_game_moves (move_number, from_rank, from_file, to_rank, to_file, is_castle_kingside, is_castle_queenside, is_check, is_checkmate, piece, acn, game_id) VALUES ".to_string();

    for (game_id, game) in illegal_games.into_iter().enumerate() {
        let mut uncategorized_tag_pairs = Vec::new();
        let mut event = None;
        let mut site = None;
        let mut date = None;
        let mut round = None;
        let mut white = None;
        let mut black = None;
        for tag in game.parsed_game.tag_pairs {
            let tag_name_raw = tag.0.to_string();
            let tag_name = tag.0.to_string().to_lowercase();
            let tag_value = tag.1.to_string();
            match tag_name.as_str() {
                "event" => event = Some(tag_value),
                "site" => site = Some(tag_value),
                "date" => date = Some(tag_value),
                "round" => round = Some(tag_value),
                "white" => white = Some(tag_value),
                "black" => black = Some(tag_value),
                _ => {
                    uncategorized_tag_pairs.push((tag_name_raw, tag_value));
                }
            }
        }

        let params = (
            game_id as i64,
            event,
            site,
            date,
            round,
            white,
            black,
            game.illegal_move_number as i64,
            game.fen_at_illegal_move,
        );
        if let Err(err) = insert_game.execute(params) {
            error!("Failed to insert illegal_games into the database. Inner error: {err}");
            return Err(());
        }

        for tag in uncategorized_tag_pairs {
            if let Err(err) = connection.execute(
                "INSERT INTO illegal_game_tags (tag_name, tag_value, game_id) VALUES (?, ?, ?);",
                (tag.0, tag.1, game_id as i64),
            ) {
                error!("Failed to insert game tags into the database. Inner error: {err}");
                return Err(());
            }
        }

        for (move_number, move_) in game.parsed_game.moves.into_iter().enumerate() {
            if global_move_number != 0 {
                insert_moves_stmt.push(',');
            }

            let is_check;
            let is_checkmate;
            match move_.check_kind {
                Check::None => {
                    is_check = 0;
                    is_checkmate = 0;
                }
                Check::Check => {
                    is_check = 1;
                    is_checkmate = 0;
                }
                Check::Mate => {
                    is_check = 1;
                    is_checkmate = 1;
                }
            }

            let is_castle_queenside;
            let is_castle_kingside;
            let from_rank;
            let from_file;
            let to_rank;
            let to_file;
            let piece;
            match &move_.move_kind {
                PieceMoveKind::CastleKingside => {
                    is_castle_kingside = 1;
                    is_castle_queenside = 0;
                    from_rank = None;
                    from_file = None;
                    to_rank = None;
                    to_file = None;
                    piece = None;
                }
                PieceMoveKind::CastleQueenside => {
                    is_castle_kingside = 0;
                    is_castle_queenside = 1;
                    from_rank = None;
                    from_file = None;
                    to_rank = None;
                    to_file = None;
                    piece = None;
                }
                PieceMoveKind::Normal(normal_move) => {
                    is_castle_kingside = 0;
                    is_castle_queenside = 0;
                    from_rank = normal_move.disambiguation_rank;
                    from_file = normal_move.disambiguation_file;
                    to_rank = Some(normal_move.destination.rank());
                    to_file = Some(normal_move.destination.file());
                    piece = Some(normal_move.piece_kind);
                }
            }

            let insert_moves_line = &format!(
                "({move_number},{},{},{},{},{},{},{},{},{},'{}',{game_id})",
                from_rank
                    .map(|rank| rank.as_int().to_string())
                    .unwrap_or("NULL".to_string()),
                from_file
                    .map(|file| format!("'{}'", file.as_char()))
                    .unwrap_or("NULL".to_string()),
                to_rank
                    .map(|rank| rank.as_int().to_string())
                    .unwrap_or("NULL".to_string()),
                to_file
                    .map(|file| format!("'{}'", file.as_char().to_string()))
                    .unwrap_or("NULL".to_string()),
                is_castle_kingside,
                is_castle_queenside,
                is_check,
                is_checkmate,
                piece
                    .map(|piece| format!("'{}'", piece as u8))
                    .unwrap_or("NULL".to_string()),
                move_.to_string(),
            );

            insert_moves_stmt.push_str(&insert_moves_line);
            global_move_number += 1;
        }
    }

    if global_move_number != 0 {
        if let Err(err) = connection.execute(&insert_moves_stmt, []) {
            error!("Failed to insert illegal_game_moves into the database. Inner error: {err}");
            return Err(());
        }
    }

    Ok(())
}

struct ProcessedTables<'pgn> {
    legal_games: Vec<FullyPopulatedBoardRowModel>,
    illegal_games: Vec<IllegalMoveRowModel<'pgn>>,
}

struct IllegalMoveRowModel<'pgn> {
    illegal_move_number: usize,
    parsed_game: ParsedGame<'pgn>,
    fen_at_illegal_move: String,
}

struct FullyPopulatedBoardRowModel {
    event: Option<String>,
    site: Option<String>,
    date: Option<String>,
    round: Option<String>,
    white: Option<String>,
    black: Option<String>,
    other_tags: Vec<(String, String)>,
    moves: Vec<FullyPopulatedMoveRowModel>,
}

struct FullyPopulatedMoveRowModel {
    from_rank: Rank,
    from_file: File,
    to_rank: Rank,
    to_file: File,
    player: Player,
    is_castle_kingside: bool,
    is_castle_queenside: bool,
    piece: Option<PieceKind>,
    fen_after: String,
    acn: String,
}
