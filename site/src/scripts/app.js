var dragged;
/** @typedef { {file: string, rank: string} } Location */
/** @typedef { {from: Location, to: Location} } Move */
/** @type {Move[]} */
var legalMoves = [];

window.addEventListener("load", function (e) {
    document.querySelectorAll(".piece_drag_target")
        .forEach(function (target) {
            target.addEventListener("dragstart", function (e) {
                var file = e.target.getAttribute("file");
                var rank = e.target.getAttribute("rank");
                dragged = e.target;
                e.target.classList.add("legal_move_square");
                for (var i = 0; i < legalMoves.length; i++) {
                    if (legalMoves[i].from.file === file && legalMoves[i].from.rank == rank) {
                        var selector = ".board_square[file=\"" + legalMoves[i].to.file + "\"][rank=\"" + legalMoves[i].to.rank + "\"]";
                        console.log(selector);
                        var element = document.querySelector(selector);
                        console.log(element);
                        if (!element) { continue; }
                        element
                            .classList
                            .add("legal_move_square");
                    }
                }
            });

            target.addEventListener("dragover", function (e) {
                e.preventDefault();
                console.log("dragover");
                console.log(e.target);
            });

            target.addEventListener("drop", function (e) {
                document.querySelectorAll(".legal_move_square")
                    .forEach(function (element) {
                        element.classList.remove("legal_move_square");
                    });

                e.stopPropagation();
            })
        });

    let game_board = document.querySelector("#game_fen").innerHTML;
    let url = "/api/legal_moves?board_fen=" + encodeURI(game_board);
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
});
