#![feature(portable_simd)]

use std::{
    borrow::Cow,
    fs::OpenOptions,
    io::Read,
    process::ExitCode,
    simd::{cmp::SimdPartialEq, u8x32},
};

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
use log::{error, info};
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
                                    end += ch_len;
                                } else if start > 0 {
                                    let ch_len = &pgn_string[..end]
                                        .chars()
                                        .rev()
                                        .next()
                                        .map(char::len_utf8)
                                        .unwrap_or(0);
                                    start -= ch_len;
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

                        term::emit_to_write_style(&mut writer.lock(), &config, &files, &diagnostic).ok();
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
    info!("{}", insert_stmt);
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
                check_kind: move_.check_kind.clone(),
            })
        }

        let mut uncategorized_tag_pairs = Vec::new();
        let mut event = None;
        let mut site = None;
        let mut date = None;
        let mut round = None;
        let mut white = None;
        let mut black = None;
        let mut result = None;
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
                "result" => result = Some(tag_value),
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
            result,
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
    let mut buffers = [(0, String::new()), (0, String::new()), (0, String::new())];

    let mut iterator = legal_games.into_iter();

    let mut done = false;
    while !done {
        let mut chunk = Vec::with_capacity(1000);

        for _ in 0..1000 {
            if let Some(next) = iterator.next() {
                chunk.push(next);
            } else {
                done = true;
                break;
            }
        }

        insert_legal_game_batch(connection, chunk, &mut buffers)?;
    }

    Ok(())
}

fn insert_legal_game_batch(
    connection: &Connection,
    batch: impl IntoIterator<Item = FullyPopulatedBoardRowModel>,
    buffers: &mut [(usize, String); 3],
) -> Result<(), ()> {
    for buffer in buffers.iter_mut() {
        buffer.1.clear();
    }

    let [(global_game_index, game_insert), (_, tag_insert), (_, move_insert)] = buffers;

    game_insert.push_str("INSERT INTO games (id, event, site, date, round, white, black, result) VALUES ");
    tag_insert.push_str("INSERT INTO tags (tag_name, tag_value, game_id) VALUES ");
    move_insert.push_str("INSERT INTO moves (move_number, from_rank, from_file, to_rank, to_file, player, is_castle_kingside, is_castle_queenside, is_check, is_checkmate, piece, fen_after, acn, game_id) VALUES ");

    let mut game_index = 0;
    let mut tag_index = 0;
    let mut move_index = 0;

    for game in batch {
        if game_index != 0 {
            game_insert.push_str(", ");
        }

        game_insert.push('(');
        {
            game_insert.push_str(&global_game_index.to_string());

            for string_param in [
                game.event, game.site, game.date, game.round, game.white, game.black, game.result,
            ]
            .iter()
            {
                game_insert.push(',');

                game_insert.push('\'');
                game_insert.push_str(
                    &string_param
                        .as_ref()
                        .map(|param| escape_sqlite_string_literal(&param))
                        .unwrap_or(Cow::Borrowed("NULL")),
                );
                game_insert.push('\'');
            }
        }
        game_insert.push(')');
        game_index += 1;

        for tag in game.other_tags {
            if tag_index != 0 {
                tag_insert.push_str(", ");
            }

            tag_insert.push('(');

            for tag_pair_piece in [tag.0, tag.1] {
                tag_insert.push('\'');
                tag_insert.push_str(&escape_sqlite_string_literal(&tag_pair_piece));
                tag_insert.push('\'');
                tag_insert.push(',');
            }

            tag_insert.push_str(&global_game_index.to_string());
            tag_insert.push(')');
            tag_index += 1;
        }

        for (move_number, move_) in game.moves.into_iter().enumerate() {
            if move_index != 0 {
                move_insert.push(',');
            }

            move_insert.push('(');
            {
                move_insert.push_str(&move_number.to_string());
                move_insert.push(',');

                move_insert.push_str(&move_.from_rank.as_int().to_string());
                move_insert.push(',');

                move_insert.push('\'');
                move_insert.push(move_.from_file.as_char());
                move_insert.push('\'');
                move_insert.push(',');

                move_insert.push_str(&move_.to_rank.as_int().to_string());
                move_insert.push(',');

                move_insert.push('\'');
                move_insert.push(move_.to_file.as_char());
                move_insert.push('\'');
                move_insert.push(',');

                move_insert.push_str(match move_.player {
                    Player::Black => "'black'",
                    Player::White => "'white'",
                });
                move_insert.push(',');

                move_insert.push_str(&if move_.is_castle_kingside { 1 } else { 0 }.to_string());
                move_insert.push(',');

                move_insert.push_str(&if move_.is_castle_queenside { 1 } else { 0 }.to_string());
                move_insert.push(',');

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

                move_insert.push_str(&is_check.to_string());
                move_insert.push(',');

                move_insert.push_str(&is_checkmate.to_string());
                move_insert.push(',');

                move_insert.push_str(&match move_.piece.map(|piece| {
                    let piece_bit: u8 = piece as u8;
                    piece_bit
                }) {
                    None => "NULL".to_string(),
                    Some(bit) => bit.to_string(),
                });
                move_insert.push(',');

                move_insert.push('\'');
                move_insert.push_str(&escape_sqlite_string_literal(&move_.fen_after));
                move_insert.push('\'');
                move_insert.push(',');

                move_insert.push('\'');
                move_insert.push_str(&escape_sqlite_string_literal(&move_.acn));
                move_insert.push('\'');
                move_insert.push(',');

                move_insert.push_str(&global_game_index.to_string());
            }
            move_insert.push(')');

            move_index += 1;
        }

        *global_game_index += 1;
    }

    info!("{}", game_index);
    if let Err(err) = connection.execute(&game_insert, ()) {
        error!("Failed to insert game into the database. Inner error: {err}");
        return Err(());
    }

    info!("{}", tag_insert);
    if let Err(err) = connection.execute(&tag_insert, ()) {
        error!("Failed to insert game tags into the database. Inner error: {err}");
        return Err(());
    }

    info!("{}", move_insert);
    if let Err(err) = connection.execute(&move_insert, ()) {
        error!("Failed to insert moves into the database. Inner error: {err}");
        return Err(());
    }

    Ok(())
}

fn insert_illegal_games(
    connection: &Connection,
    illegal_games: Vec<IllegalMoveRowModel>,
) -> Result<(), ()> {
    let mut global_move_number = 0;
    let mut insert_game = connection.prepare("INSERT INTO illegal_games (id, event, site, date, round, white, black, result, illegal_move_number, fen_at_illegal_move) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);").unwrap();
    let mut insert_moves_stmt = "INSERT INTO illegal_game_moves (move_number, from_rank, from_file, to_rank, to_file, is_castle_kingside, is_castle_queenside, is_check, is_checkmate, piece, acn, game_id) VALUES ".to_string();

    for (game_id, game) in illegal_games.into_iter().enumerate() {
        let mut uncategorized_tag_pairs = Vec::new();
        let mut event = None;
        let mut site = None;
        let mut date = None;
        let mut round = None;
        let mut white = None;
        let mut black = None;
        let mut result = None;
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
                "result" => result = Some(tag_value),
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
            result,
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

#[derive(Debug)]
struct FullyPopulatedBoardRowModel {
    event: Option<String>,
    site: Option<String>,
    date: Option<String>,
    round: Option<String>,
    white: Option<String>,
    black: Option<String>,
    result: Option<String>,
    other_tags: Vec<(String, String)>,
    moves: Vec<FullyPopulatedMoveRowModel>,
}

#[derive(Debug)]
struct FullyPopulatedMoveRowModel {
    from_rank: Rank,
    from_file: File,
    to_rank: Rank,
    to_file: File,
    player: Player,
    is_castle_kingside: bool,
    is_castle_queenside: bool,
    check_kind: Check,
    piece: Option<PieceKind>,
    fen_after: String,
    acn: String,
}

fn escape_sqlite_string_literal(contents: &str) -> Cow<'_, str> {
    let bytes = contents.as_bytes();
    let mut i = 0;
    const SIMD_SIZE: usize = 32;
    let needle = u8x32::splat(b'\'');
    while i + SIMD_SIZE < bytes.len() {
        let simd = u8x32::from_slice(&bytes[i..(i + SIMD_SIZE)]);
        if needle.simd_eq(simd).any() {
            return Cow::Owned(contents.replace('\'', "''"));
        }

        i += SIMD_SIZE;
    }

    for j in i..bytes.len() {
        if bytes[j] == b'\'' {
            return Cow::Owned(contents.replace('\'', "''"));
        }
    }

    return Cow::Borrowed(contents);
}

#[test]
fn test_escape() {
    let test_cases: [(&str, Cow<'static, str>); 20] = [
        // Inputs without single quotes use Cow::Borrowed.
        ("", Cow::Borrowed("")),
        ("plain text", Cow::Borrowed("plain text")),
        // Inputs containing single quotes use Cow::Owned.
        ("'", Cow::Owned("''".to_owned())),
        ("a'b", Cow::Owned("a''b".to_owned())),
        ("don't", Cow::Owned("don''t".to_owned())),
        ("''", Cow::Owned("''''".to_owned())),
        (
            "O'Reilly's book",
            Cow::Owned("O''Reilly''s book".to_owned()),
        ),
        (
            "It''s already quoted",
            Cow::Owned("It''''s already quoted".to_owned()),
        ),
        ("'leading quote", Cow::Owned("''leading quote".to_owned())),
        ("trailing quote'", Cow::Owned("trailing quote''".to_owned())),
        ("''both ends''", Cow::Owned("''''both ends''''".to_owned())),
        (
            "This string contains 'one' quoted phrase.",
            Cow::Owned("This string contains ''one'' quoted phrase.".to_owned()),
        ),
        (
            "She said, 'It's a test.'",
            Cow::Owned("She said, ''It''s a test.''".to_owned()),
        ),
        ("''''", Cow::Owned("''''''''".to_owned())),
        ("A'B'C'D'E", Cow::Owned("A''B''C''D''E".to_owned())),
        (
            "The user entered: 'DROP TABLE users;'",
            Cow::Owned("The user entered: ''DROP TABLE users;''".to_owned()),
        ),
        (
            "Line one'\nLine two'\nLine three'",
            Cow::Owned("Line one''\nLine two''\nLine three''".to_owned()),
        ),
        (
            "Apostrophes: ''' ''' '''",
            Cow::Owned("Apostrophes: '''''' '''''' ''''''".to_owned()),
        ),
        (
            "This is a long test string containing several apostrophes:\n\
         Don't stop testing. John's value isn't equal to Mary's value,\n\
         and the input includes 'quoted text' plus a final quote'.",
            Cow::Owned(
                "This is a long test string containing several apostrophes:\n\
             Don''t stop testing. John''s value isn''t equal to Mary''s value,\n\
             and the input includes ''quoted text'' plus a final quote''."
                    .to_owned(),
            ),
        ),
        (
            "A string with no apostrophes at all",
            Cow::Borrowed("A string with no apostrophes at all"),
        ),
    ];

    for (input, expected_output) in test_cases {
        assert_eq!(expected_output, escape_sqlite_string_literal(input));
    }
}
