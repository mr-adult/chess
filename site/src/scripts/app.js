var PieceKind = {
    pawn: "Pawn",
    knight: "Knight",
    bishop: "Bishop",
    rook: "Rook",
    queen: "Queen",
    king: "King",
};

/** @type {Element} */
var dragged;
/** @typedef { {file: string, rank: string} } Location */
/** @typedef { {from: Location, to: Location} } Move */
/** @typedef { {type: "Normal" | "Promotion", move: Move} PossibleMove } */
/** @type {PossibleMove[]} */
var legalMoves = [];

function getBoardFEN() {
    var element = document.querySelector("#game_fen");
    if (!element) { return null; }
    let game_board = element.textContent;
    return game_board;
};

function updateLegalMoves() {
    let game_board = getBoardFEN();
    let url = "/api/v0/legal_moves?board_fen=" + encodeURI(game_board);
    fetch(url, {
        method: "GET"
    })
        .then(function (res) { return res.json() })
        /** @param {PossibleMove[]} res */
        .then(function (res) {
            if (!Array.isArray(res)) {
                console.error("Response was not JSON: " + res);
            } else {
                legalMoves = res;
            }
        })
        .catch(function (err) {
            console.error(err);
        });
};

function replaceBoard(newBoardHtml, shouldUpdateLegalMoves) {
    if (!newBoardHtml || newBoardHtml.length === 0) { return; }
    unbindBoardEventListeners();
    document.getElementById("game_state_wrapper").innerHTML = newBoardHtml; 

    if (shouldUpdateLegalMoves) {
        updateLegalMoves();
    }
    bindBoardEventListeners();
}

function onDragStart(e) {
    var file = e.target.getAttribute("file");
    var rank = e.target.getAttribute("rank");
    dragged = e.target;
    e.target.classList.add("legal_move_square");
    for (var i = 0; i < legalMoves.length; i++) {
        var move = legalMoves[i].move;
        if (move.from.file === file && move.from.rank == rank) {
            var selector = ".board_square[file=\"" + move.to.file + "\"][rank=\"" + move.to.rank + "\"]";
            var element = document.querySelector(selector);
            if (!element) { continue; }
            element
                .classList
                .add("legal_move_square");
        }
    }
};

function onDragOver(e) {
    e.preventDefault();
}

function onDrop(e) {
    document.querySelectorAll(".legal_move_square")
        .forEach(function (element) {
            element.classList.remove("legal_move_square");
        });

    var board_fen = document.querySelector("#game_fen").textContent;

    var from_rank = dragged.getAttribute("rank") || "";
    var from_rank_int = parseInt(from_rank);
    var from_file = dragged.getAttribute("file") || "";
    var to_file = e.target.getAttribute("file") || "";
    var to_rank = e.target.getAttribute("rank") || "";
    var to_rank_int = parseInt(to_rank);

    if (from_rank === "" 
        || to_rank === "" 
        || from_file === "" 
        || to_file === ""
        || (from_rank === to_rank && from_file === to_file)) {
        return;
    }

    var is_legal_move = false;
    var is_promotion = false;
    for (var i = 0; i < legalMoves.length; i++) {
        var move = legalMoves[i].move;
        if (move.from.file === from_file
            && move.from.rank.toString() === from_rank
            && move.to.file === to_file
            && move.to.rank.toString() === to_rank) {
                
            is_legal_move = true;
            is_promotion = legalMoves[i].type === "Promotion"
            break;
        }
    }

    if (!is_legal_move) { return; }

    if (is_promotion) {
        var selected_promotion = "";
        while (!selected_promotion || 
            (selected_promotion !== "q" 
                && selected_promotion !== "r" 
                && selected_promotion !== "b" 
                && selected_promotion !== "k"
                && selected_promotion !== "n")) {

            var dirty = prompt(
                "Please select a promotion piece.\n" +
                "Options are:\n" + 
                "[Q]ueen\n" +
                "[R]ook\n" + 
                "[B]ishop\n" + 
                "K[n]ight");

            if (!dirty) { continue; }
            selected_promotion = dirty.trim()[0].toLocaleLowerCase();
        }
    }

    var starting_fen = document.querySelector("#initial_board_fen").textContent;
    var history_elements = document.querySelectorAll("td[move]");
    var history = [];
    for (var i = 0; i < history_elements.length; i++) {
        var move_number = parseInt(history_elements[i].getAttribute("move"));
        if (isNaN(move_number)) { throw new Error("NaN"); }
        history[move_number - 1] = history_elements[i].textContent
    }
    
    var payload = {
        starting_fen: starting_fen,
        history: history,
        board_fen: board_fen,
        move: {
            type: is_promotion ? "Promotion" : "Normal",
            move: {
                from: {
                    rank: from_rank_int,
                    file: from_file,
                },
                to: {
                    rank: to_rank_int,
                    file: to_file,
                },
            }
        },
    };

    if (is_promotion) {
        var move = payload.move;
        switch(selected_promotion) {
            case "q":
                move.promotion_kind = PieceKind.queen;
                break;

            case "r":
                move.promotion_kind = PieceKind.rook;
                break;

            case "b":
                move.promotion_kind = PieceKind.bishop;
                break;

            case "k":
            case "n":
                move.promotion_kind = PieceKind.knight;
                break;

            default:
                var error = new Error("Invalid promotion selection " + selected_promotion);
                console.error(error);
                throw error;
        }
    }

    fetch("/api/v0/make_move", {
        method: "POST",
        headers: {
            "Content-type": "application/json; charset=UTF-8"
        },
        body: JSON.stringify(payload),
    }).then(function (res) { return res.text() })
        .then(function (text) { replaceBoard(text, true); })
        .catch(function (err) { console.error(err); });

    e.stopPropagation();
}

function onDragEnd() {
    document.querySelectorAll(".legal_move_square")
        .forEach(function (legal_move_square) {
            legal_move_square.classList.remove("legal_move_square");
        });
}

function bindBoardEventListeners() {
    document.querySelectorAll(".piece_drag_target")
        .forEach(function (target) {
            target.addEventListener("dragstart", onDragStart);
            target.addEventListener("dragover", onDragOver);
            target.addEventListener("drop", onDrop);
        });
}

function unbindBoardEventListeners() {
    document.querySelectorAll(".piece_drag_target")
        .forEach(function (target) {
            target.removeEventListener("dragstart", onDragStart);
            target.removeEventListener("dragover", onDragOver);
            target.removeEventListener("drop", onDrop);
        })
}

window.addEventListener("load", function (e) {
    bindBoardEventListeners();
    updateLegalMoves();
});

window.addEventListener("dragend", onDragEnd);
