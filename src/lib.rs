extern crate console_error_panic_hook;

use wasm_bindgen::prelude::*;

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

use web_sys::{WebGl2RenderingContext, WebGlFramebuffer, WebGlProgram, WebGlTexture};
mod utils;

use WebGl2RenderingContext as GL;
//-------------------------------------------------------------

const PXSIZE: i32 = 4;

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
	outColor = vec4(val,val,val,1.0);
	// outColor = vec4(1.0, 0.0, 0.0, 1.0);
}
"##;

//-------------------------------------------------------------
struct Program<T> {
	display: T,
	compute: T,
}

struct Size {
	width: i32,
	height: i32,
}
#[wasm_bindgen]
pub struct GOL {
	gl: GL,
	size: Program<Size>,
	program: Program<WebGlProgram>,
	texture: Program<WebGlTexture>,
	framebuffer: WebGlFramebuffer,
}

#[wasm_bindgen]
impl GOL {
	#[wasm_bindgen(constructor)]
	pub fn new() -> Self {
		let window = utils::get_window();
		let canvas = utils::get_canvas("main");
		let gl = utils::get_gl("main");
		let display_size = Size {
			height: window.inner_height().unwrap().as_f64().unwrap() as i32,
			width: window.inner_width().unwrap().as_f64().unwrap() as i32,
		};

		canvas.set_height(display_size.height as u32);
		canvas.set_width(display_size.width as u32);

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

		gl.use_program(Some(&program.compute));

		//Vertex Initialization
		{
			let vertices: [f32; 12] = [
				-1.0, -1.0, 1.0, -1.0, -1.0, 1.0, //Triangle 1
				1.0, -1.0, 1.0, 1.0, -1.0, 1.0, //Triangle 2
			];

			let position_attribute_location = gl.get_attrib_location(&program.compute, "position");
			let buffer = gl.create_buffer().ok_or("Failed to create buffer").unwrap();
			gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));

			// Note that `Float32Array::view` is somewhat dangerous (hence the
			// `unsafe`!). This is creating a raw view into our module's
			// `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
			// (aka do a memory allocation in Rust) it'll cause the buffer to change,
			// causing the `Float32Array` to be invalid.
			//
			// As a result, after `Float32Array::view` we have to be very careful not to
			// do any memory allocations before it's dropped.
			unsafe {
				let positions_array_buf_view = js_sys::Float32Array::view(&vertices);

				gl.buffer_data_with_array_buffer_view(
					GL::ARRAY_BUFFER,
					&positions_array_buf_view,
					GL::STATIC_DRAW,
				);
			}

			let vao = gl
				.create_vertex_array()
				.ok_or("Could not create vertex array object")
				.unwrap();
			gl.bind_vertex_array(Some(&vao));

			gl.vertex_attrib_pointer_with_i32(0, 2, GL::FLOAT, false, 0, 0);
			gl.enable_vertex_attrib_array(position_attribute_location as u32);

			gl.bind_vertex_array(Some(&vao));
		}

		let texture_location_compute = gl.get_uniform_location(&program.compute, "u_texture");
		let texture_location_display = gl.get_uniform_location(&program.display, "u_texture");

		gl.pixel_storei(GL::UNPACK_ALIGNMENT, 1);

		let compute_texture = utils::create_texture(
			&gl,
			compute_size.width as i32,
			compute_size.height as i32,
			None,
		)
		.ok()
		.unwrap();

		let values: Vec<u8> = utils::gen_random_byte(compute_size.width * compute_size.height);

		let display_texture = utils::create_texture(
			&gl,
			compute_size.width as i32,
			compute_size.height as i32,
			Some(&js_sys::Uint8Array::from(&values[..])),
		)
		.ok()
		.unwrap();

		gl.bind_texture(GL::TEXTURE_2D, Some(&display_texture));

		let pxsize_location_display = gl.get_uniform_location(&program.display, "u_px_size");
		let size_location = gl.get_uniform_location(&program.compute, "u_size");

		gl.use_program(Some(&program.compute));
		gl.uniform1i(texture_location_compute.as_ref(), 0);
		gl.uniform2f(
			size_location.as_ref(),
			compute_size.width as f32,
			compute_size.height as f32,
		);

		gl.use_program(Some(&program.display));
		gl.uniform1i(texture_location_display.as_ref(), 0);
		gl.uniform1f(pxsize_location_display.as_ref(), PXSIZE as f32);

		let framebuffer = gl.create_framebuffer().unwrap();

		return Self {
			gl: gl,
			size: Program {
				display: display_size,
				compute: compute_size,
			},
			program: program,
			texture: Program {
				compute: compute_texture,
				display: display_texture,
			},
			framebuffer: framebuffer,
		};
	}

	#[wasm_bindgen]
	pub fn draw(&mut self) {
		let gl = &self.gl;

		self.gl.use_program(Some(&self.program.display));
		self.gl
			.viewport(0, 0, self.size.display.width, self.size.display.height);

		gl.clear_color(0.0, 0.0, 0.0, 1.0);
		gl.clear(GL::COLOR_BUFFER_BIT);
		gl.draw_arrays(GL::TRIANGLES, 0, 6);

		gl.use_program(Some(&self.program.compute));
		gl.viewport(0, 0, self.size.compute.width, self.size.compute.height);

		gl.bind_framebuffer(GL::FRAMEBUFFER, Some(&self.framebuffer));

		gl.framebuffer_texture_2d(
			GL::FRAMEBUFFER,
			GL::COLOR_ATTACHMENT0,
			GL::TEXTURE_2D,
			Some(&self.texture.compute),
			0,
		);

		gl.clear_color(0.0, 0.0, 0.0, 1.0);
		gl.clear(GL::COLOR_BUFFER_BIT);

		gl.draw_arrays(GL::TRIANGLES, 0, 6);

		gl.bind_framebuffer(GL::FRAMEBUFFER, None);

		gl.bind_texture(GL::TEXTURE_2D, Some(&self.texture.compute));
		std::mem::swap(&mut self.texture.compute, &mut self.texture.display);
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
