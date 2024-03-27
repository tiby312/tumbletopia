use std::borrow::Borrow;

use super::*;

pub struct TextureGpu {
    texture: simple2d::TextureBuffer,
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
impl<A: Borrow<TextureGpu>, B: Borrow<ModelGpu>> simple2d::Drawable for Foo<A, B> {
    fn draw(&self, view: &mut simple2d::View) {
        let model = self.model.borrow();
        let tex = self.texture.borrow();
        view.draw(
            WebGl2RenderingContext::TRIANGLES,
            &tex.texture,
            &model.tex_coord,
            &model.position.0,
            model.index.as_ref(),
            &model.normals.0,
            false,
            false,
            false,
            true,
        );
    }
    fn draw_ext(
        &self,
        view: &mut simple2d::View,
        grayscale: bool,
        text: bool,
        _linear: bool,
        lighting: bool,
    ) {
        let model = self.model.borrow();
        let tex = self.texture.borrow();
        view.draw(
            WebGl2RenderingContext::TRIANGLES,
            &tex.texture,
            &model.tex_coord,
            &model.position.0,
            model.index.as_ref(),
            &model.normals.0,
            grayscale,
            text,
            false,
            lighting,
        );
    }
}

pub struct ModelGpu {
    index: Option<simple2d::IndexBuffer>,
    tex_coord: simple2d::TextureCoordBuffer,
    position: simple2d::Vert3Buffer,
    normals: simple2d::Vert3Buffer,
}
impl ModelGpu {
    pub fn new(ctx: &web_sys::WebGl2RenderingContext, data: &model::ModelData) -> Self {
        let index = if let Some(indices) = &data.indices {
            let mut index = simple2d::IndexBuffer::new(ctx).unwrap_throw();
            index.update(indices);
            Some(index)
        } else {
            None
        };

        let mut tex_coord = simple2d::TextureCoordBuffer::new(ctx).unwrap_throw();
        tex_coord.update(&data.tex_coords);

        let mut position = simple2d::Vert3Buffer::new(ctx).unwrap_throw();
        position.update(&data.positions);

        let mut normals = simple2d::Vert3Buffer::new(ctx).unwrap_throw();
        normals.update(&data.normals);

        ModelGpu {
            index,
            tex_coord,
            position,
            normals,
        }
    }
}
