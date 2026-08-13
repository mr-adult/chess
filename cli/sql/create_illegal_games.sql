CREATE TABLE IF NOT EXISTS illegal_games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event TEXT,
    site TEXT,
    date TEXT,
    round TEXT,
    white TEXT,
    black TEXT,
    illegal_move_number INTEGER NOT NULL,
    fen_at_illegal_move TEXT NOT NULL
);
