SELECT m.move_number, m.acn FROM illegal_games g
INNER JOIN illegal_game_moves m
    ON g.id = m.game_id
WHERE m.move_number <= g.illegal_move_number
ORDER BY g.id, m.move_number;
