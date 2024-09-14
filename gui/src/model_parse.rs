use super::*;

pub struct TextureGpu {
    pub texture: shader_sys::TextureBuffer,
}
impl TextureGpu {
    pub fn new(ctx: &web_sys::WebGl2RenderingContext, tt: &model::Img) -> Self {
        let mut texture = shader_sys::TextureBuffer::new(ctx);
        texture.update(tt.width as usize, tt.height as usize, &tt.data);

        TextureGpu { texture }
    }
}

pub struct Foo<A, B> {
    pub texture: A,
    pub model: B,
    pub height: f32,
    pub width: f32,
}

pub struct ModelGpu {
    pub res: shader_sys::shader::VaoData,
}

impl ModelGpu {
    pub fn new(shader: &ShaderSystem, data: &model::ModelData) -> Self {
        let program = &shader.program;
        let mat = &shader.program.matrix_buffer;
        let res: shader_sys::shader::VaoData = shader_sys::shader::create_vao(
            &shader.ctx,
            program,
            &data.tex_coords,
            &data.positions,
            &data.normals,
            data.indices.as_ref().unwrap(),
            mat,
        );
        ModelGpu { res }
    }
}
