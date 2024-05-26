/*
CREATE TABLE IF NOT EXISTS users (
    id uuid PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    user_name text NOT NULL,
    email text NOT NULL,

    -- Auth
    -- If the password_hash is ever null, it means
    -- the user is in the middle of the sign up workflow
    -- (or never completed it)
    password_hash VARCHAR(256),
    password_salt uuid NOT NULL DEFAULT gen_random_uuid(),
    token_salt uuid NOT NULL DEFAULT gen_random_uuid(),
);

CREATE TABLE IF NOT EXISTS games (
    id uuid PRIMARY KEY NOT NULL DEFAULT gen_random_uuid(),
    white_player text NOT NULL,
    black_player text NOT NULL,
);

CREATE TABLE IF NOT EXISTS moves (
    game_id uuid NOT NULL,
    move_number smallint NOT NULL CHECK (move_number > 0),
    from_rank char NOT NULL,
    from_file char NOT NULL,
    to_rank char NOT NULL,
    to_file char NOT NULL,

    PRIMARY KEY (game_id, move_number),

    CONSTRAINT game_id_fk 
        FOREIGN KEY(game_id)
        REFERENCES games(game_id)
        ON UPDATE CASCADE,
        ON DELETE CASCADE,

    CONSTRAINT no_non_moves
        CHECK (
            from_file <> to_file OR from_rank <> to_rank
        )
);
*/