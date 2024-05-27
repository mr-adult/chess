var dragged;

window.addEventListener("load", function(e) {
    document.querySelectorAll(".piece_drag_target")
        .forEach(function (target) {
            target.addEventListener("dragstart", function (e) {
                console.log("dragstart");
                console.log(e.target);
                dragged = e.target;
            });

            target.addEventListener("dragover", function(e) {
                e.preventDefault();
                console.log("dragover");
                console.log(e.target);
            });

            target.addEventListener("drop", function(e) {
                e.stopPropagation();
                console.log("drop");
                console.log(e.target);
                dragged = undefined;
            })
        });

    let game_board = document.querySelector("#game_fen").innerHTML
    let url = "/api/legal_moves?board_fen=" + encodeURI(game_board);
    fetch(url, {
        method: "GET"
    })
        .then((res) => res.json())
        .then((res) => console.log(res));
});
