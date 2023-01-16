use super::*;

pub struct ModelGpu {
    index: Option<simple2d::IndexBuffer>,
    tex_coord: simple2d::TextureCoordBuffer,
    texture: simple2d::TextureBuffer,
    position: simple2d::DynamicBuffer,
    normals: simple2d::DynamicBuffer,
}
impl ModelGpu {
    pub fn new(ctx: &web_sys::WebGl2RenderingContext, data: &model::ModelData) -> Self {
        let index = if let Some(indices) = &data.indices {
            let mut index = simple2d::IndexBuffer::new(&ctx).unwrap_throw();
            index.update(&indices);
            Some(index)
        } else {
            None
        };

        let mut tex_coord = simple2d::TextureCoordBuffer::new(&ctx).unwrap_throw();
        tex_coord.update(&data.tex_coords);

        let mut texture = simple2d::TextureBuffer::new(&ctx);

        texture.update(
            data.texture.width as usize,
            data.texture.height as usize,
            &data.texture.data,
        );

        let mut position = simple2d::DynamicBuffer::new(&ctx).unwrap_throw();
        position.update_no_clear(&data.positions);

        let mut normals = simple2d::DynamicBuffer::new(&ctx).unwrap_throw();
        normals.update_no_clear(&data.normals);

        ModelGpu {
            index,
            tex_coord,
            texture,
            position,
            normals,
        }
    }
    // pub fn draw_pos(&self, view: &mut simple2d::View, pos: &simple2d::Buffer) {
    //     view.draw_triangles(&self.texture, &self.tex_coord, pos, self.index.as_ref());
    // }
    pub fn draw(&self, view: &mut simple2d::View) {
        view.draw_triangles(
            &self.texture,
            &self.tex_coord,
            &self.position,
            self.index.as_ref(),
            &self.normals,
        );
    }
}
