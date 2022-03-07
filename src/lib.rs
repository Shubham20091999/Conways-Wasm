extern crate console_error_panic_hook;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

use web_sys::{
    WebGl2RenderingContext, WebGlFramebuffer, WebGlProgram, WebGlShader, WebGlTexture, Window,
};

mod utils;

use WebGl2RenderingContext as GL;
//-------------------------------------------------------------

const PXSIZE: u32 = 4;

const VERT_SHADER: &str = r##"#version 300 es
 
in vec4 position;

void main() {

    gl_Position = position;
}
"##;

const COMPUTE_FRAG_SHADER: &str = r##"#version 300 es

precision lowp float;
precision lowp int;

uniform sampler2D u_texture;
uniform vec2 u_size;

out vec4 outColor;

vec4 getValueAt(int i, int j) {
    return texelFetch(u_texture, ivec2(mod((gl_FragCoord.xy) - vec2(i, j), u_size)), 0);
}

int getAliveOrDeadAt(int i, int j) {
    return int(getValueAt(i, j).r > 0.50);
}

void main() {
    int isAlive = getAliveOrDeadAt(0, 0);
    float isAlive_f = float(isAlive);
    int aliveNeighbourCount = getAliveOrDeadAt(1, 1) + getAliveOrDeadAt(1, -1) + getAliveOrDeadAt(-1, 1) + getAliveOrDeadAt(-1, -1) + getAliveOrDeadAt(0, 1) + getAliveOrDeadAt(1, 0) + getAliveOrDeadAt(0, -1) + getAliveOrDeadAt(-1, 0);

    float willBeAlive = float((aliveNeighbourCount == 3) || (bool(isAlive) && aliveNeighbourCount == 2));

    //For diming effect for newly alive cell and newly dead cells
    // outColor.r = willBeAlive = willBeAlive * (isAlive_f + 0.60) + (isAlive_f - 0.85) * (1.0 - willBeAlive);
    // isAlive_f - (1.0-a) + (1.0-a+b) * willBeAlive;
    // a-> brightness of willBeDead pixel
    // b-> brightness of will Be alive pixel
    outColor.r = isAlive_f - 0.85 + 1.45 * willBeAlive;
}
"##;

const DISPLAY_FRAG_SHADER: &str = r##"#version 300 es

precision highp float;

uniform sampler2D u_texture;
uniform float u_px_size;

out vec4 outColor;

void main() {
    float val = texelFetch(u_texture, ivec2(gl_FragCoord.xy / u_px_size), 0).r;
    outColor = vec4(val,val,0.0,1.0);
    // outColor = vec4(1.0, 0.0, 0.0, 1.0);
}
"##;

//-------------------------------------------------------------
struct Program<T> {
    display: T,
    compute: T,
}

struct Size {
    width: u32,
    height: u32,
}
#[wasm_bindgen]
pub struct GOL {
    gl: GL,
    size: Program<Size>,
    program: Program<WebGlProgram>,
}

impl GOL {
    pub fn new() -> Self {
        let window = utils::get_window();

        let gl = utils::get_canvas("canvas")
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<GL>()
            .unwrap();

        let display_size = Size {
            height: window.inner_height().unwrap().as_f64().unwrap() as u32,
            width: window.inner_width().unwrap().as_f64().unwrap() as u32,
        };

        let compute_size = Size {
            height: display_size.height / PXSIZE,
            width: display_size.width / PXSIZE,
        };

        let vert_shader = utils::compile_shader(&gl, GL::VERTEX_SHADER, VERT_SHADER)
            .ok()
            .unwrap();
        let compute_frag = utils::compile_shader(&gl, GL::FRAGMENT_SHADER, COMPUTE_FRAG_SHADER)
            .ok()
            .unwrap();
        let disp_frag = utils::compile_shader(&gl, GL::FRAGMENT_SHADER, DISPLAY_FRAG_SHADER)
            .ok()
            .unwrap();

        let program = Program {
            display: utils::link_program(&gl, &vert_shader, &disp_frag)
                .ok()
                .unwrap(),
            compute: utils::link_program(&gl, &vert_shader, &compute_frag)
                .ok()
                .unwrap(),
        };

        return Self {
            gl: gl,
            size: Program {
                display: display_size,
                compute: compute_size,
            },
            program: program,
        };
    }
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    // Your code goes here!
    log("Hello world!");

    Ok(())
}
