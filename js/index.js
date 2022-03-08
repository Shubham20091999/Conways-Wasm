let rust = import("../pkg/index.js");

document.getElementById("main")

//milli seconds per frame
const MSpF = 100;

rust.then(m => {
    const gol = new m.GOL();

    let last_frame = Number.NEGATIVE_INFINITY;
    let drawScene = function (time) {
        requestAnimationFrame(drawScene);
        if ((time - last_frame) > MSpF) {
            gol.draw();
            last_frame = time;
        }
    }
    drawScene();
}).catch(console.error);

