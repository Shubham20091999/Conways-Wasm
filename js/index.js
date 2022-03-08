let rust = import("../pkg/index.js");

document.getElementById("main")

rust.then(m => {
    const gol = new m.GOL();

    let drawScene = function(){
        gol.draw();
        requestAnimationFrame(drawScene);
    } 
    // setInterval(m => { gol.draw() }, 100);
    requestAnimationFrame(drawScene);
}).catch(console.error);

