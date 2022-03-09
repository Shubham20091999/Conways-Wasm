let rust = import("../pkg/index.js");

// Get A WebGL context
var canvas = document.querySelector("#main");
var gl = canvas.getContext("webgl2");
if (!gl) {
    alert("WebGL2 not supported!!!");
    throw new Error("WebGL2 not Supported");
}

//milli seconds per frame
const MSpF = 50;
//Pixel Size
const PxSize = 4;

canvas.width = Math.floor(window.innerWidth / PxSize) * PxSize;
canvas.height = Math.floor(window.innerHeight / PxSize) * PxSize;

rust.then(m => {
    const gol = new m.GOL(gl, PxSize);

    let last_frame = Number.NEGATIVE_INFINITY;
    function drawScene(time) {
        requestAnimationFrame(arguments.callee);
        if ((time - last_frame) > MSpF) {
            gol.draw();
            last_frame = time;
        }
    }
    requestAnimationFrame(drawScene);
}).catch(console.error);

