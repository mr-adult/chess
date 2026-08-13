CREATE TABLE IF NOT EXISTS illegal_game_moves (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    move_number INTEGER NOT NULL,
    from_rank INTEGER CHECK(from_rank BETWEEN 1 AND 8 OR from_rank IS NULL),
    from_file TEXT CHECK(from_file IN ('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h') OR from_file IS NULL),
    to_rank INTEGER CHECK(to_rank BETWEEN 1 AND 8 OR to_rank IS NULL),
    to_file TEXT CHECK(to_file IN ('a', 'b', 'c', 'd', 'e', 'f', 'g', 'h') OR to_file IS NULL),
    is_castle_kingside INTEGER CHECK(is_castle_kingside BETWEEN 0 AND 1),
    is_castle_queenside INTEGER CHECK(is_castle_queenside BETWEEN 0 AND 1),
    is_check INTEGER CHECK(is_check BETWEEN 0 AND 1),
    is_checkmate INTEGER CHECK(is_check BETWEEN 0 AND 1),
    piece INT,
    acn TEXT NOT NULL,
    game_id INTEGER NOT NULL,
    FOREIGN KEY(game_id) REFERENCES games(id),
    FOREIGN KEY(piece) REFERENCES pieces(id)
);
