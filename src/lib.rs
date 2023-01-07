use std::f32::consts::PI;

use axgeom::{vec2, vec2same, Vec2};
use cgmath::Transform;
use gloo::console::log;
use serde::{Deserialize, Serialize};
use shogo::simple2d;
use shogo::{simple2d::DynamicBuffer, utils};
use wasm_bindgen::prelude::*;

use duckduckgeo::grid;

pub mod matrix;
mod scroll;

const COLORS: &[[f32; 4]] = &[
    [1.0, 0.0, 0.0, 0.5],
    [0.0, 1.0, 0.0, 0.5],
    [0.0, 0.0, 1.0, 0.5],
];

///Common data sent from the main thread to the worker.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MEvent {
    CanvasMouseMove { x: f32, y: f32 },
    CanvasMouseDown { x: f32, y: f32 },
    CanvasMouseUp,
    ButtonClick,
    ShutdownClick,
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

    let _: () = response.next().await.unwrap_throw();
    log!("main thread is closing");
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

    //TODO move?
    ctx.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
    ctx.setup_alpha();

    // setup game data
    let mut color_iter = COLORS.iter().cycle().peekable();
    let game_dim = [1000.0f32, 1000.0];
    let viewport = [canvas.width() as f32, canvas.height() as f32];

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
                    -0.1,
                );
            }
        }
        let j = (0..positions.len()).map(|_| [0.0; 2]).collect();
        model::ModelData {
            positions,
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

    let mut scroll_manager = scroll::ScrollController::new([0.0; 2]);

    //let foo=load_glb(BLOCK_GLB);
    let foo = model::load_glb(CAT_GLB);

    let data = {
        let mut data = foo.gen();

        use matrix::*;

        //for person
        let s = matrix::scale(400.0, 400.0, 400.0).generate();

        for p in data.positions.iter_mut() {
            *p = s.transform_point((*p).into()).into();
        }
        data
    };

    let cat = ModelGpu::new(&ctx, &data);

    'outer: loop {
        let mut j = false;
        let res = frame_timer.next().await;
        //log!(format!("number of events:{:?}", res.len()));
        for e in res {
            match e {
                MEvent::CanvasMouseUp => {
                    if scroll_manager.handle_mouse_up() {
                        j = true;
                    }
                }
                MEvent::CanvasMouseMove { x, y } => {
                    scroll_manager.handle_mouse_move([*x, *y]);
                }
                MEvent::CanvasMouseDown { x, y } => {
                    scroll_manager.handle_mouse_down([*x, *y]);
                }
                MEvent::ButtonClick => {
                    let _ = color_iter.next();
                }
                MEvent::ShutdownClick => break 'outer,
            }
        }
        let mouse_world = mouse_to_world(
            scroll_manager.cursor_canvas,
            scroll_manager.camera(),
            viewport,
        );

        //let mm = mouse_ray(scroll_manager.cursor_canvas, scroll_manager.camera(),viewport);

        //log!(format!("{:?}", (scroll_manager.cursor_canvas,scroll_manager.camera(),mouse_world)));

        // if j {
        //      grid_walls.set(grid_viewport.to_grid((mouse_world).into()), true);
        //      update_walls(&grid_viewport, cache, &mut walls, &grid_walls);
        // }
        scroll_manager.step();

        use matrix::*;

        // for a in mouse_to_world_coord(scroll_manager.cursor_canvas,
        //     scroll_manager.camera(),
        //     viewport){
        //     simple2d::shapes(cache).rect(
        //         simple2d::Rect {
        //             x: a[0] - grid_viewport.spacing / 2.0,
        //             y: a[1] - grid_viewport.spacing / 2.0,
        //             w: grid_viewport.spacing,
        //             h: grid_viewport.spacing,
        //         },
        //         a[2],
        //     );
        // }
        simple2d::shapes(cache).rect(
            simple2d::Rect {
                x: mouse_world[0] - grid_viewport.spacing / 2.0,
                y: mouse_world[1] - grid_viewport.spacing / 2.0,
                w: grid_viewport.spacing,
                h: grid_viewport.spacing,
            },
            0.0,
        );

        buffer.update_clear(cache);

        ctx.draw_clear([0.0, 0.0, 0.0, 0.0]);

        let matrix = projection(scroll_manager.camera(), viewport).generate();

        //let k = matrix::z_rotation(counter).chain(matrix).generate();
        let mut v = draw_sys.view(matrix.as_ref());

        // v.draw_triangles(&checker, None,&[0.3, 0.3, 0.3, 0.3]);

        checkers_gpu.draw_pos(&mut v, &buffer);
        //v.draw_triangles(&walls,None, &[1.0, 0.5, 0.5, 1.0]);
        // v.draw_triangles(&buffer,None, color_iter.peek().unwrap_throw());

        {
            buffer.update_no_clear(&checkers.positions);
            checkers_gpu.draw_pos(&mut v, &buffer);
        }

        //let k = matrix::z_rotation(counter).chain(matrix).generate();
        //let mut v = draw_sys.view(&k);
        cat.draw(&mut v);

        ctx.flush();
    }

    w.post_message(());

    log!("worker thread closing");
}

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
                -0.1,
            );
        }
    }
    buffer.update_clear(cache);
}

//convert DOM coordinate to canvas relative coordinate
fn convert_coord(canvas: &web_sys::HtmlElement, event: &web_sys::Event) -> [f32; 2] {
    use wasm_bindgen::JsCast;
    shogo::simple2d::convert_coord(canvas, event.dyn_ref().unwrap_throw())
}

fn convert_coord_touch(canvas: &web_sys::HtmlElement, event: &web_sys::Event) -> [f32; 2] {
    event.prevent_default();
    event.stop_propagation();
    use wasm_bindgen::JsCast;
    convert_coord_touch_inner(canvas, event.dyn_ref().unwrap_throw())[0]
}

///
/// Convert a mouse event to a coordinate for simple2d.
///
pub fn convert_coord_touch_inner(
    canvas: &web_sys::HtmlElement,
    e: &web_sys::TouchEvent,
) -> Vec<[f32; 2]> {
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
        //log!(format!("{:?}",(rx,ry)));
        let [x, y] = [
            (x + rx - rect.left()) * scalex,
            (y + ry - rect.top()) * scaley,
        ];

        ans.push([x as f32, y as f32]);
    }
    ans
}

fn mouse_to_world(mouse: [f32; 2], camera: [f32; 2], viewport: [f32; 2]) -> [f32; 2] {
    //generate some mouse points
    use matrix::*;

    let clip_x = mouse[0] / viewport[0] * 2. - 1.;
    let clip_y = mouse[1] / viewport[1] * -2. + 1.;

    let startc = [clip_x, clip_y, -0.9];
    let endc = [clip_x, clip_y, 0.999];

    let matrix = projection(camera, viewport).inverse().generate();

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

//project world to clip space
fn projection(offset: [f32; 2], dim: [f32; 2]) -> impl matrix::MyMatrix + matrix::Inverse {
    use matrix::*;
    //clip space positive z is into the screen. negative z is towards your face.
    let m = matrix::perspective(0.3, dim[0] / dim[1], 1.0, 1610.0);
    let t = translation(offset[0], offset[1], -500.0);
    let r = z_rotation(PI / 4.0);
    let r2 = x_rotation(PI / 4.0);
    scale(1.0, -1.0, 1.0).chain(m).chain(r2).chain(t).chain(r) //.chain(r2).chain(t)//.chain(r)
}

const KEY_GLB: &'static [u8] = include_bytes!("../assets/key.glb");
const PERSON_GLB: &'static [u8] = include_bytes!("../assets/person-v1.glb");
const CAT_GLB: &'static [u8] = include_bytes!("../assets/cat2.glb");
