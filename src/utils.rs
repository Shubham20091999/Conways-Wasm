use wasm_bindgen::JsCast;

use web_sys::{
    Document, HtmlCanvasElement, WebGl2RenderingContext, WebGlFramebuffer, WebGlProgram,
    WebGlShader, WebGlTexture, Window,
};

pub fn get_window() -> Window {
    return web_sys::window().unwrap();
}

pub fn get_document() -> Document {
    return get_window().document().unwrap();
}

pub fn get_canvas(id: &str) -> HtmlCanvasElement {
    let canvas = get_document().get_element_by_id(id).unwrap();
    return canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
}

pub fn get_gl(id: &str) -> WebGl2RenderingContext {
    return get_canvas(id)
        .get_context("webgl2")
        .unwrap()
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()
        .unwrap();
}

pub fn compile_shader(
    gl: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))
        .unwrap();
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    gl: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = gl
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))
        .unwrap();

    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

pub fn create_texture(
    gl: &WebGl2RenderingContext,
    sizew: i32,
    sizeh: i32,
    data: Option<&js_sys::Object>,
) -> Result<WebGlTexture, String> {
    let texture = gl.create_texture();
    if let None = texture {
        return Err("unable to initialize".to_string());
    }
    gl.active_texture(WebGl2RenderingContext::TEXTURE0 + 0);
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, texture.as_ref());

    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
        WebGl2RenderingContext::TEXTURE_2D,
        0,
        WebGl2RenderingContext::R8 as i32,
        sizew,
        sizeh,
        0,
        WebGl2RenderingContext::RED,
        WebGl2RenderingContext::UNSIGNED_BYTE,
        data,
    )
    .map_err(|err| println!("{:?}", err))
    .ok();

    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MIN_FILTER,
        WebGl2RenderingContext::NEAREST as i32,
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MAG_FILTER,
        WebGl2RenderingContext::NEAREST as i32,
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_WRAP_S,
        WebGl2RenderingContext::REPEAT as i32,
    );
    gl.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_WRAP_T,
        WebGl2RenderingContext::REPEAT as i32,
    );

    return Ok(texture.unwrap());
}
