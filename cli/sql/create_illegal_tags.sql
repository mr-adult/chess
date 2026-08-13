CREATE TABLE IF NOT EXISTS illegal_game_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tag_name TEXT NOT NULL,
    tag_value TEXT NOT NULL,
    game_id INTEGER NOT NULL,
    FOREIGN KEY(game_id) REFERENCES illegal_games(id)
);
