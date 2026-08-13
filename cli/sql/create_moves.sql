CREATE TABLE IF NOT EXISTS moves (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    move_number INTEGER NOT NULL,
    from_rank INTEGER check(from_rank BETWEEN 1 AND 8),
    from_file TEXT check(from_file IN ('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h')),
    to_rank INTEGER check(to_rank BETWEEN 1 AND 8),
    to_file TEXT check(to_file IN ('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h')),
    player TEXT check(player = 'white' OR player = 'black'),
    is_castle_kingside INTEGER check(is_castle_kingside BETWEEN 0 AND 1),
    is_castle_queenside INTEGER check(is_castle_queenside BETWEEN 0 AND 1),
    is_check INTEGER check(is_check BETWEEN 0 AND 1),
    is_checkmate INTEGER check(is_checkmate BETWEEN 0 AND 1),
    piece INT,
    fen_after TEXT NOT NULL,
    acn TEXT NOT NULL,
    game_id INTEGER NOT NULL,
    FOREIGN KEY(game_id) REFERENCES games(id),
    FOREIGN KEY(piece) REFERENCES pieces(id)
);
