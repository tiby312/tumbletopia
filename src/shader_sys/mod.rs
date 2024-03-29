//!
//! A simple webgl 2d drawing system that draws shapes using many small circles or squares.
//!
//! The data sent to the gpu is minimized by only sending the positions of the vertex.
//! The color can also be changed for all vertices in a buffer.
//!
use gloo::console::log;
pub mod shader;
mod util;

use web_sys::WebGlBuffer;
use web_sys::WebGlShader;
use web_sys::WebGlUniformLocation;
use web_sys::{WebGl2RenderingContext, WebGlProgram};

use WebGl2RenderingContext as GL;

use shader::*;

pub type Vertex = [f32; 3];

pub struct TextureBuffer {
    pub(crate) texture: web_sys::WebGlTexture,
    pub(crate) ctx: WebGl2RenderingContext,
    width: i32,
    height: i32,
}
impl Drop for TextureBuffer {
    fn drop(&mut self) {
        self.ctx.delete_texture(Some(&self.texture));
    }
}
impl TextureBuffer {
    pub fn texture(&self) -> &web_sys::WebGlTexture {
        &self.texture
    }
    pub fn width(&self) -> i32 {
        self.width
    }
    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn bind(&self, ctx: &WebGl2RenderingContext) {
        ctx.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
    }

    pub fn new(ctx: &WebGl2RenderingContext) -> TextureBuffer {
        let texture = ctx.create_texture().unwrap_throw();
        ctx.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

        let width = 1;
        let height = 1;
        // Fill the texture with a 1x1 blue pixel.
        ctx.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGBA as i32,
            width,  //width
            height, //height
            0,      //border
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            Some(&[0, 0, 255, 255]),
        )
        .unwrap_throw();

        Self {
            ctx: ctx.clone(),
            texture,
            width,
            height,
        }
    }
    pub fn update(&mut self, width: usize, height: usize, image: &[u8]) {
        //log!(format!("image bytes:{:?}",image.len()));
        self.ctx
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
        // self.ctx.compressed_tex_image_2d_with_u8_array(
        //     WebGl2RenderingContext::TEXTURE_2D,
        //     0,
        //     WebGl2RenderingContext::RGBA,
        //     width as i32,
        //     height as i32,
        //     0,
        //     image
        // );

        //log!("attemptying to make image!!!");
        // let arr=js_sys::Uint8ClampedArray::new_with_length(image.len() as u32);
        // arr.copy_from(image);

        //TODO leverage javascript to load png instead to avoid image dependancy??

        //let image = image::load_from_memory_with_format(&image, image::ImageFormat::Png).unwrap();
        //let rgba_image = image.to_rgba8();

        //https://stackoverflow.com/questions/70309403/updating-html-canvas-imagedata-using-rust-webassembly
        let clamped_buf: Clamped<&[u8]> = Clamped(image);
        let image = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
            clamped_buf,
            width as u32,
            height as u32,
        )
        .map_err(|e| log!(e))
        .unwrap_throw();
        self.width = width as i32;
        self.height = height as i32;
        self.ctx
            .tex_image_2d_with_u32_and_u32_and_image_data(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::RGBA as i32,
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                &image,
            )
            .unwrap_throw();

        self.ctx.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        self.ctx.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::LINEAR as i32,
        );
        self.ctx.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_S,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );
        self.ctx.tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_WRAP_T,
            WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
        );

        // self.ctx.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
        //     WebGl2RenderingContext::TEXTURE_2D,
        //     0,
        //     WebGl2RenderingContext::RGBA as i32,
        //     width as i32, //width
        //     height as i32, //height
        //     0, //border
        //     WebGl2RenderingContext::RGBA,
        //     WebGl2RenderingContext::UNSIGNED_BYTE,
        //     Some(image)).unwrap_throw();
    }
}

use wasm_bindgen::{prelude::*, Clamped};

pub fn draw_clear(ctx: &WebGl2RenderingContext, color: [f32; 4]) {
    let [a, b, c, d] = color;
    ctx.clear_color(a, b, c, d);
    ctx.clear(
        web_sys::WebGl2RenderingContext::COLOR_BUFFER_BIT
            | web_sys::WebGl2RenderingContext::DEPTH_BUFFER_BIT,
    );
}

///
/// A simple shader program that allows the user to draw simple primitives.
///
pub struct ShaderSystem {
    pub program: GlProgram,
    pub ctx: WebGl2RenderingContext,
}

impl Drop for ShaderSystem {
    fn drop(&mut self) {
        self.ctx.delete_program(Some(&self.program.program));
    }
}

impl ShaderSystem {
    pub fn new(ctx: &WebGl2RenderingContext) -> Result<ShaderSystem, String> {
        //https://webglfundamentals.org/webgl/lessons/webgl-text-texture.html

        ctx.pixel_storei(WebGl2RenderingContext::UNPACK_PREMULTIPLY_ALPHA_WEBGL, 1);
        ctx.enable(WebGl2RenderingContext::BLEND);

        ctx.blend_func(
            WebGl2RenderingContext::ONE,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );

        ctx.enable(WebGl2RenderingContext::DEPTH_TEST);
        ctx.enable(WebGl2RenderingContext::CULL_FACE);

        let program = GlProgram::new(ctx)?;

        Ok(ShaderSystem {
            program,
            ctx: ctx.clone(),
        })
    }

    pub fn draw(
        &mut self,
        res: &VaoData,
        texture: &TextureBuffer,
        mmatrix: &[[f32; 16]],
        grayscale: bool,
        text: bool,
        lighting: bool,
    ) {
        self.program.draw(shader::Argss {
            texture,
            mmatrix,
            res,
            point_size: 1.0,
            grayscale,
            text,
            lighting,
        })
    }
}

//TODO why is this here?
///
/// Convert a mouse event to a coordinate for simple2d.
///
pub fn convert_coord(canvas: &web_sys::HtmlElement, e: &web_sys::MouseEvent) -> [f32; 2] {
    let rect = canvas.get_bounding_client_rect();

    let canvas_width: f64 = canvas
        .get_attribute("width")
        .unwrap_throw()
        .parse()
        .unwrap_throw();
    let canvas_height: f64 = canvas
        .get_attribute("height")
        .unwrap_throw()
        .parse()
        .unwrap_throw();

    let scalex = canvas_width / rect.width();
    let scaley = canvas_height / rect.height();

    let [x, y] = [e.client_x() as f64, e.client_y() as f64];

    let [x, y] = [(x - rect.left()) * scalex, (y - rect.top()) * scaley];
    [x as f32, y as f32]
}
