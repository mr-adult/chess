/** @type {Element} */
var dragged;
/** @typedef { {file: string, rank: string} } Location */
/** @typedef { {from: Location, to: Location} } Move */
/** @type {Move[]} */
var legalMoves = [];

function updateLegalMoves() {
    let game_board = document.querySelector("#game_fen").textContent;
    let url = "/api/v0/legal_moves?board_fen=" + encodeURI(game_board);
    fetch(url, {
        method: "GET"
    })
        .then(function (res) { return res.json() })
        /** @param {Move[]} res */
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

function onDragStart(e) {
    var file = e.target.getAttribute("file");
    var rank = e.target.getAttribute("rank");
    dragged = e.target;
    e.target.classList.add("legal_move_square");
    for (var i = 0; i < legalMoves.length; i++) {
        if (legalMoves[i].from.file === file && legalMoves[i].from.rank == rank) {
            var selector = ".board_square[file=\"" + legalMoves[i].to.file + "\"][rank=\"" + legalMoves[i].to.rank + "\"]";
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
    var to_rank = e.target.getAttribute("rank") || "";
    var to_rank_int = parseInt(to_rank);

    fetch("/api/v0/make_move", {
        method: "POST",
        headers: {
            "Content-type": "application/json; charset=UTF-8"
        },
        body: JSON.stringify({
            move: {
                from: {
                    rank: from_rank_int,
                    file: dragged.getAttribute("file") || "",
                },
                to: {
                    rank: to_rank_int,
                    file: e.target.getAttribute("file") || "",
                },
            },
            board_fen: board_fen,
        }),
    }).then(function (res) { return res.text() })
        .then(function (text) {
            if (text == null || text.length == 0) { return; }
            unbindBoardEventListeners();
            document.getElementById("game_state_wrapper").innerHTML = text

            updateLegalMoves();
            bindBoardEventListeners();
        })
        .catch(function (err) { console.error(err); });

    e.stopPropagation();
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
