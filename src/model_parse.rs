use super::*;

pub struct TextureGpu {
    pub texture: simple2d::TextureBuffer,
}
impl TextureGpu {
    pub fn new(ctx: &web_sys::WebGl2RenderingContext, tt: &model::Img) -> Self {
        let mut texture = simple2d::TextureBuffer::new(ctx);

        texture.update(tt.width as usize, tt.height as usize, &tt.data);
        TextureGpu { texture }
    }
}

pub struct Foo<A, B> {
    pub texture: A,
    pub model: B,
}

pub struct ModelGpu {
    pub res: simple2d::shader::VaoResult,
}

impl ModelGpu {
    pub fn new(shader: &ShaderSystem, data: &model::ModelData) -> Self {
        let program = &shader.program;
        let mat = &shader.program.matrix_buffer;
        let res = simple2d::shader::create_vao2(
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
