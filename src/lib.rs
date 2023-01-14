use axgeom::{vec2, vec2same};
use cgmath::{SquareMatrix, Transform};
use gloo::console::log;
use model::matrix;
use serde::{Deserialize, Serialize};
use shogo::simple2d::{self, TextureBuffer};
use shogo::{simple2d::DynamicBuffer, utils};
use std::f32::consts::PI;
use wasm_bindgen::prelude::*;

use duckduckgeo::grid;

mod scroll;

const COLORS: &[[f32; 4]] = &[
    [1.0, 0.0, 0.0, 0.5],
    [0.0, 1.0, 0.0, 0.5],
    [0.0, 0.0, 1.0, 0.5],
];

///Common data sent from the main thread to the worker.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MEvent {
    CanvasMouseMove {
        x: f32,
        y: f32,
    },
    CanvasMouseDown {
        x: f32,
        y: f32,
    },
    CanvasMouseUp,
    ButtonClick,
    ShutdownClick,
    Resize {
        canvasx: u32,
        canvasy: u32,
        x: f32,
        y: f32,
    },
}

#[wasm_bindgen]
pub async fn main_entry() {
    console_error_panic_hook::set_once();

    use futures::StreamExt;

    log!("demo start");

    let (canvas, button, shutdown_button) = (
        utils::get_by_id_canvas("mycanvas"),
        utils::get_by_id_elem("mybutton"),
        utils::get_by_id_elem("shutdownbutton"),
    );

    canvas.set_width(gloo::utils::body().client_width() as u32);
    canvas.set_height(gloo::utils::body().client_height() as u32);

    let offscreen = canvas.transfer_control_to_offscreen().unwrap_throw();

    let (mut worker, mut response) =
        shogo::EngineMain::new("./gridlock_worker.js", offscreen).await;

    let _handler = worker.register_event(&canvas, "mousemove", |e| {
        let [x, y] = convert_coord(e.elem, e.event);
        MEvent::CanvasMouseMove { x, y }
    });

    let _handler = worker.register_event(&canvas, "mousedown", |e| {
        let [x, y] = convert_coord(e.elem, e.event);
        MEvent::CanvasMouseDown { x, y }
    });

    let _handler = worker.register_event(&canvas, "mouseup", |_| MEvent::CanvasMouseUp);

    let _handler = worker.register_event(&canvas, "touchstart", |e| {
        let [x, y] = convert_coord_touch(e.elem, e.event);
        MEvent::CanvasMouseDown { x, y }
    });

    let _handler = worker.register_event(&canvas, "touchend", |_| MEvent::CanvasMouseUp);

    let _handler = worker.register_event(&canvas, "touchmove", |e| {
        let [x, y] = convert_coord_touch(e.elem, e.event);
        MEvent::CanvasMouseMove { x, y }
    });

    let _handler = worker.register_event(&button, "click", |_| MEvent::ButtonClick);

    let _handler = worker.register_event(&shutdown_button, "click", |_| MEvent::ShutdownClick);

    let w = gloo::utils::window();

    let _handler = worker.register_event(&w, "resize", |_| resize());

    worker.post_message(resize());

    let _: () = response.next().await.unwrap_throw();
    log!("main thread is closing");
}
fn resize() -> MEvent {
    let canvas = utils::get_by_id_canvas("mycanvas");
    //canvas.set_width(gloo::utils::body().client_width() as u32);
    //canvas.set_height(gloo::utils::body().client_height() as u32);

    // let width = gloo::utils::document().body().unwrap_throw().client_width();
    // let height = gloo::utils::document()
    //     .body()
    //     .unwrap_throw()
    //     .client_height();

    let width = canvas.client_width();
    let height = canvas.client_height();

    let realpixels = gloo::utils::window().device_pixel_ratio();
    log!(format!("pixel ratio:{:?}", realpixels));
    // .body.clientWidth;
    // var height=document.body.clientHeight;

    // var realToCSSPixels = window.devicePixelRatio;
    // var gl_width  = Math.floor(width  * realToCSSPixels);
    // var gl_height = Math.floor(height * realToCSSPixels);

    let gl_width = (width as f64 * realpixels).floor();
    let gl_height = (height as f64 * realpixels).floor();

    MEvent::Resize {
        canvasx: gloo::utils::body().client_width() as u32,
        canvasy: gloo::utils::body().client_height() as u32,
        x: gl_width as f32,
        y: gl_height as f32,
    }
}

#[wasm_bindgen]
pub async fn worker_entry() {
    let (mut w, ss) = shogo::EngineWorker::new().await;
    let mut frame_timer = shogo::FrameTimer::new(30, ss);

    let canvas = w.canvas();
    let ctx = simple2d::ctx_wrap(&utils::get_context_webgl2_offscreen(&canvas));

    let mut draw_sys = ctx.shader_system();
    let mut buffer = ctx.buffer_dynamic();
    let cache = &mut vec![];
    let mut walls = ctx.buffer_dynamic();

    //TODO get rid of this somehow.
    //these values are incorrect.
    //they are set correctly after resize is called on startup.
    let gl_width = canvas.width(); // as f32*1.6;
    let gl_height = canvas.height(); // as f32*1.6;
    ctx.viewport(0, 0, gl_width as i32, gl_height as i32);
    let mut viewport = [canvas.width() as f32, canvas.height() as f32];

    ctx.setup_alpha();

    // setup game data
    let mut color_iter = COLORS.iter().cycle().peekable();
    let game_dim = [1000.0f32, 1000.0];

    let grid_width = 32;

    let grid_viewport = duckduckgeo::grid::GridViewPort {
        origin: vec2(0.0, 0.0),
        spacing: game_dim[0] / (grid_width as f32),
    };

    let checkers = {
        let mut positions = Vec::new();
        let mut k = simple2d::shapes(&mut positions);
        for x in 0..grid_width {
            let offset = if x % 2 == 0 {
                0..grid_width
            } else {
                1..grid_width - 1
            };
            for y in offset.step_by(2) {
                k.rect(
                    simple2d::Rect {
                        x: x as f32 * grid_viewport.spacing,
                        y: y as f32 * grid_viewport.spacing,
                        w: grid_viewport.spacing,
                        h: grid_viewport.spacing,
                    },
                    -1.0,
                );
            }
        }
        let j = (0..positions.len()).map(|_| [0.0; 2]).collect();
        let normals = (0..positions.len()).map(|_| [0.0,0.0,1.0]).collect();
        
        model::ModelData {
            matrix: cgmath::Matrix4::identity(),
            positions,
            normals,
            indices: None,
            texture: model::single_tex(),
            tex_coords: j,
        }
    };

    let checkers_gpu = ModelGpu::new(&ctx, &checkers);

    //let checker = ctx.buffer_static_clear(cache);

    let mut grid_walls = grid::Grid2D::new(vec2same(grid_width));

    grid_walls.set(vec2(0, 0), true);
    grid_walls.set(vec2(2, 1), true);

    update_walls(&grid_viewport, cache, &mut walls, &grid_walls);

    let mut scroll_manager = scroll::ScrollController::new([0., 0.].into());

    let cat = {
        let data = model::load_glb(CAT_GLB).gen_ext(grid_viewport.spacing);
        ModelGpu::new(&ctx, &data)
    };

    let grass = {
        let data = model::load_glb(GRASS_GLB).gen_ext(grid_viewport.spacing);
        ModelGpu::new(&ctx, &data)
    };

    'outer: loop {
        let mut j = false;
        let res = frame_timer.next().await;
        for e in res {
            match e {
                MEvent::Resize {
                    canvasx,
                    canvasy,
                    x,
                    y,
                } => {
                    let xx = *x as u32;
                    let yy = *y as u32;
                    canvas.set_width(xx);
                    canvas.set_height(yy);
                    ctx.viewport(0, 0, xx as i32, yy as i32);

                    viewport = [xx as f32, yy as f32];
                    log!(format!("updating viewport to be:{:?}", viewport));
                }
                MEvent::CanvasMouseUp => {
                    if scroll_manager.handle_mouse_up() {
                        j = true;
                    }
                }
                MEvent::CanvasMouseMove { x, y } => {
                    //log!(format!("{:?}",(x,y)));

                    scroll_manager.handle_mouse_move([*x, *y], viewport);
                }
                MEvent::CanvasMouseDown { x, y } => {
                    //log!(format!("{:?}",(x,y)));

                    scroll_manager.handle_mouse_down([*x, *y]);
                }
                MEvent::ButtonClick => {
                    let _ = color_iter.next();
                }
                MEvent::ShutdownClick => break 'outer,
            }
        }

        //log!(format!("{:?}",scroll_manager.camera()));
        let mouse_world = mouse_to_world(
            scroll_manager.cursor_canvas.into(),
            scroll_manager.camera(),
            viewport,
        );
        //log!(format!("mouse:{:?}",mouse_world));

        //let mm = mouse_ray(scroll_manager.cursor_canvas, scroll_manager.camera(),viewport);

        //log!(format!("{:?}", (scroll_manager.cursor_canvas,scroll_manager.camera(),mouse_world)));

        // if j {
        //      grid_walls.set(grid_viewport.to_grid((mouse_world).into()), true);
        //      update_walls(&grid_viewport, cache, &mut walls, &grid_walls);
        // }
        scroll_manager.step();

        use matrix::*;

        // simple2d::shapes(cache).rect(
        //     simple2d::Rect {
        //         x: mouse_world[0] - grid_viewport.spacing / 2.0,
        //         y: mouse_world[1] - grid_viewport.spacing / 2.0,
        //         w: grid_viewport.spacing,
        //         h: grid_viewport.spacing,
        //     },
        //     mouse_world[2] - 10.0,
        // );

        buffer.update_clear(cache);

        ctx.draw_clear([0.0, 0.0, 0.0, 0.0]);

        let matrix = view_projection(scroll_manager.camera(), viewport).generate();

        let mut v = draw_sys.view(matrix.as_ref());

        // v.draw_triangles(&checker, None,&[0.3, 0.3, 0.3, 0.3]);

        checkers_gpu.draw_pos(&mut v, &buffer);
        //v.draw_triangles(&walls,None, &[1.0, 0.5, 0.5, 1.0]);
        // v.draw_triangles(&buffer,None, color_iter.peek().unwrap_throw());

        {
            buffer.update_no_clear(&checkers.positions);
            checkers_gpu.draw_pos(&mut v, &buffer);
        }

        {
            let j = grid_viewport.spacing / 2.0;
            let t = matrix::translation(mouse_world[0] - j, mouse_world[1] - j, 50.0);
            let s = matrix::scale(1.0, 1.0, 1.0);
            let m = matrix.chain(t).chain(s).generate();
            let mut v = draw_sys.view(m.as_ref());
            cat.draw(&mut v);
        }
        //let k = matrix::z_rotation(counter).chain(matrix).generate();
        //let mut v = draw_sys.view(&k);
        //cat.draw(&mut v);

        for a in 0..6 {
            for b in 0..5 {
                use matrix::*;
                let x1 = grid_viewport.spacing * a as f32;
                let y1 = grid_viewport.spacing * b as f32;
                let mm = view_projection(scroll_manager.camera(), viewport)
                    .chain(translation(x1, y1, 1.0))
                    .generate();

                let mut v = draw_sys.view(mm.as_ref());
                grass.draw(&mut v);
            }
        }

        ctx.flush();
    }

    w.post_message(());

    log!("worker thread closing");
}

use web_sys::{WebGl2RenderingContext, WebGlTexture};

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

pub struct ModelGpu {
    index: Option<simple2d::IndexBuffer>,
    tex_coord: simple2d::TextureCoordBuffer,
    texture: simple2d::TextureBuffer,
    position: simple2d::DynamicBuffer,
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
        ModelGpu {
            index,
            tex_coord,
            texture,
            position,
        }
    }
    pub fn draw_pos(&self, view: &mut simple2d::View, pos: &simple2d::Buffer) {
        view.draw_triangles(&self.texture, &self.tex_coord, pos, self.index.as_ref());
    }
    pub fn draw(&self, view: &mut simple2d::View) {
        view.draw_triangles(
            &self.texture,
            &self.tex_coord,
            &self.position,
            self.index.as_ref(),
        );
    }
}

use shogo::simple2d::Vertex;

fn update_walls(
    grid_viewport: &duckduckgeo::grid::GridViewPort,
    cache: &mut Vec<Vertex>,
    buffer: &mut DynamicBuffer,
    grid_walls: &grid::Grid2D,
) {
    let mut s = simple2d::shapes(cache);
    for (p, val) in grid_walls.iter() {
        if val {
            let top_left = grid_viewport.to_world_topleft(p);
            s.rect(
                simple2d::Rect {
                    x: top_left.x,
                    y: top_left.y,
                    w: grid_viewport.spacing,
                    h: grid_viewport.spacing,
                },
                0.1,
            );
        }
    }
    buffer.update_clear(cache);
}

//convert DOM coordinate to canvas relative coordinate
fn convert_coord(canvas: &web_sys::EventTarget, event: &web_sys::Event) -> [f32; 2] {
    use wasm_bindgen::JsCast;
    shogo::simple2d::convert_coord(
        canvas.dyn_ref().unwrap_throw(),
        event.dyn_ref().unwrap_throw(),
    )
}

fn convert_coord_touch(canvas: &web_sys::EventTarget, event: &web_sys::Event) -> [f32; 2] {
    event.prevent_default();
    event.stop_propagation();
    use wasm_bindgen::JsCast;
    convert_coord_touch_inner(canvas, event.dyn_ref().unwrap_throw())[0]
}

///
/// Convert a mouse event to a coordinate for simple2d.
///
pub fn convert_coord_touch_inner(
    canvas: &web_sys::EventTarget,
    e: &web_sys::TouchEvent,
) -> Vec<[f32; 2]> {
    use wasm_bindgen::JsCast;

    let canvas: &web_sys::HtmlElement = canvas.dyn_ref().unwrap_throw();
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

    let touches = e.touches();

    let mut ans = vec![];
    for a in 0..touches.length() {
        let touch = touches.get(a).unwrap();
        let x = touch.client_x() as f64;
        let y = touch.client_y() as f64;
        let rx = touch.radius_x() as f64;
        let ry = touch.radius_y() as f64;
        let [x, y] = [
            (x * scalex - rect.left() * scalex),
            (y * scaley - rect.top() * scaley),
        ];

        ans.push([x as f32, y as f32]);
    }
    ans
}

// //TODO don't compute matrix each time!!!!
// fn viewport_to_word(camera: [f32; 2], viewport: [f32; 2]) -> [[f32; 2]; 4] {
//     [
//         mouse_to_world([0.0, 0.0], camera, viewport),
//         mouse_to_world([viewport[0], 0.0], camera, viewport),
//         mouse_to_world([0.0, viewport[1]], camera, viewport),
//         mouse_to_world(viewport, camera, viewport),
//     ]
// }

fn mouse_to_world(mouse: [f32; 2], camera: [f32; 2], viewport: [f32; 2]) -> [f32; 2] {
    //TODO something wrong here. not factoring in zoom!
    //generate some mouse points
    use matrix::*;

    let clip_x = mouse[0] / viewport[0] * 2. - 1.;
    let clip_y = mouse[1] / viewport[1] * -2. + 1.;

    let startc = [clip_x, clip_y, -0.9];
    let endc = [clip_x, clip_y, 0.999];

    let matrix = view_projection(camera, viewport).inverse().generate();

    let a = matrix.transform_point(startc.into());
    let b = matrix.transform_point(endc.into());

    let v = b - a;
    let ray = collision::Ray::new(a, v);

    let p = cgmath::Point3::new(0.0, 0.0, 0.0);
    let up = cgmath::Vector3::new(0.0, 0.0, -1.0);

    let plane = collision::Plane::from_point_normal(p, up);
    use collision::Continuous;

    if let Some(point) = plane.intersection(&ray) {
        [point.x, point.y]
    } else {
        [300.0, -80.0]
    }
}

fn camera(camera: [f32; 2], zoom: f32) -> impl matrix::MyMatrix + matrix::Inverse {
    //world coordinates when viewed with this camera is:
    //x leftdown
    //y right down
    //z+ into the sky (-z into the worlds ground)

    use matrix::*;

    use cgmath::*;

    //position camera in the sky pointing down
    
    let cam = Point3::new(camera[0] + 300.0, camera[1] + 300.0, 500.0);
    let dir = Point3::new(camera[0], camera[1], 0.0);
    let up = Vector3::new(0.0, 0.0, 1.0);
    let g = cgmath::Matrix4::look_at(cam, dir, up).inverse();

    let zoom = translation(0.0, 0.0, zoom);
    g.chain(zoom)
}

fn projection(dim: [f32; 2]) -> impl matrix::MyMatrix + matrix::Inverse {

    use matrix::*;
    matrix::perspective(0.4, dim[0] / dim[1], 1.0, 1000.0)
}

//project world to clip space
fn view_projection(offset: [f32; 2], dim: [f32; 2]) -> impl matrix::MyMatrix + matrix::Inverse {
    use matrix::*;

    projection(dim).chain(camera(offset, -600.0 + dim[1] * 0.5).inverse())
}

const KEY_GLB: &'static [u8] = include_bytes!("../assets/key.glb");
const PERSON_GLB: &'static [u8] = include_bytes!("../assets/person-v1.glb");
const CAT_GLB: &'static [u8] = include_bytes!("../assets/tiger2.glb");
const GRASS_GLB: &'static [u8] = include_bytes!("../assets/grass.glb");
