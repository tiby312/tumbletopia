// pub fn remove_common<T: Ord>(a: &mut Vec<T>, b: &mut Vec<T>) {
//     use std::collections::BTreeSet;

//     let mut k = BTreeSet::from_iter(b.drain(..));

//     a.retain(|j| !k.remove(j));

//     b.extend(k.into_iter());
// }
// use std::collections::BTreeSet;

// pub fn remove_common_set<T: Ord>(a: &mut BTreeSet<T>, k: &mut BTreeSet<T>) {
//     a.retain(|j| !k.remove(j));
// }
// pub struct AATexture<'a> {
//     ctx: WebGl2RenderingContext,
//     color_rend_buffer: web_sys::WebGlRenderbuffer,
//     rend_buffer: web_sys::WebGlFramebuffer,
//     color_buffer: web_sys::WebGlFramebuffer,
//     texture:&'a TextureBuffer
// }
// impl<'a> AATexture<'a> {
//     pub fn phase1(&mut self){
//         self.ctx.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&self.rend_buffer));
//     }
//     pub fn finish_phase1(&mut self){

//         self.ctx.bind_framebuffer(WebGl2RenderingContext::READ_FRAMEBUFFER, Some(&self.rend_buffer));
//         self.ctx.bind_framebuffer(WebGl2RenderingContext::DRAW_FRAMEBUFFER, Some(&self.color_buffer));
//         self.ctx.clear_bufferfv_with_f32_array(WebGl2RenderingContext::COLOR, 0, &[1.0, 1.0, 1.0, 1.0]);

//         let w=self.texture.width();
//         let h=self.texture.height();
//         self.ctx.blit_framebuffer(0, 0, w, h,
//             0, 0, w, h,
//             WebGl2RenderingContext::COLOR_BUFFER_BIT, WebGl2RenderingContext::LINEAR); //TODO nearest?

//         self.ctx.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&self.rend_buffer));

//         self.ctx.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);

//         //gl.bindFramebuffer(gl.FRAMEBUFFER, null);
//     }

//     pub fn phase2(&mut self){

//     }
//     pub fn new(
//         ctx: &WebGl2RenderingContext,
//         width: usize,
//         height: usize,
//         texture: &'a TextureBuffer,
//     ) -> Self {

//         let rend_buffer = {
//             let rend_buffer=ctx.create_renderbuffer().unwrap_throw();
//             ctx.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(&rend_buffer));
//             use wasm_bindgen::convert::IntoWasmAbi;
//             // let max_sample: i32 = ctx
//             //     .get_parameter(WebGl2RenderingContext::MAX_SAMPLES)
//             //     .unwrap_throw()
//             //     .into_abi() as i32;
//             ctx.renderbuffer_storage_multisample(
//                 WebGl2RenderingContext::RENDERBUFFER,
//                 4,
//                 WebGl2RenderingContext::RGBA8,
//                 width as i32,
//                 height as i32,
//             );
//             rend_buffer
//         };

//         let frame1={
//             let frame1 = ctx.create_framebuffer().unwrap_throw();
//             ctx.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&frame1));
//             ctx.framebuffer_renderbuffer(
//                 WebGl2RenderingContext::FRAMEBUFFER,
//                 WebGl2RenderingContext::COLOR_ATTACHMENT0,
//                 WebGl2RenderingContext::RENDERBUFFER,
//                 Some(&rend_buffer),
//             );
//             ctx.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);

//             frame1
//         };

//         let frame2={
//             let frame2 = ctx.create_framebuffer().unwrap_throw();
//             ctx.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&frame2));
//             ctx.framebuffer_texture_2d(
//                 WebGl2RenderingContext::FRAMEBUFFER,
//                 WebGl2RenderingContext::COLOR_ATTACHMENT0,
//                 WebGl2RenderingContext::TEXTURE_2D,
//                 Some(texture.texture()),
//                 0,
//             );
//             ctx.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);

//             frame2
//         };

//         AATexture {
//             ctx:ctx.clone(),
//             color_rend_buffer: rend_buffer,
//             rend_buffer: frame1,
//             color_buffer: frame2,
//             texture
//         }
//     }
// }
