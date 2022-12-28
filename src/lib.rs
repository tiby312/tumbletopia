use axgeom::{vec2, vec2same, Vec2};
use gloo::console::log;
use serde::{Deserialize, Serialize};
use shogo::utils;
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
    CanvasMouseMove { x: f32, y: f32 },
    CanvasMouseDown { x: f32, y: f32 },
    CanvasMouseUp,
    ButtonClick,
    ShutdownClick,
}

#[wasm_bindgen]
pub async fn main_entry() {
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
    use shogo::simple2d;

    let (mut w, ss) = shogo::EngineWorker::new().await;
    let mut frame_timer = shogo::FrameTimer::new(30, ss);

    let canvas = w.canvas();
    let ctx = simple2d::ctx_wrap(&utils::get_context_webgl2_offscreen(&canvas));
    let mut draw_sys = ctx.shader_system();
    let mut buffer = ctx.buffer_dynamic();
    let cache = &mut vec![];
    let mut walls = ctx.buffer_dynamic();

    ctx.setup_alpha();

    // setup game data
    let mut color_iter = COLORS.iter().cycle().peekable();
    let radius = 4.0;
    let game_dim = [canvas.width() as f32, canvas.height() as f32];

    let grid_viewport = duckduckgeo::grid::GridViewPort {
        origin: vec2(0.0, 0.0),
        spacing: game_dim[0] / (64 as f32),
    };

    let mut grid_walls = grid::Grid2D::new(axgeom::Vec2 { x: 64, y: 64 });

    let mut scroll_manager = scroll::ScrollController::new(vec2same(0.0));

    'outer: loop {
        for e in frame_timer.next().await {
            log!(format!("{:?}", e));
            match e {
                MEvent::CanvasMouseUp => {
                    if scroll_manager.handle_mouse_up() {
                        grid_walls.set(
                            grid_viewport.to_grid((*scroll_manager.world_cursor()).into()),
                            true,
                        );
                        let mut s = simple2d::shapes(cache);
                        for (p, val) in grid_walls.iter() {
                            if val {
                                let top_left = grid_viewport.to_world_topleft(p);
                                s.rect(simple2d::Rect {
                                    x: top_left.x,
                                    y: top_left.y,
                                    w: grid_viewport.spacing,
                                    h: grid_viewport.spacing,
                                });
                            }
                        }
                        walls.update_clear(cache);
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

        scroll_manager.step();

        let world_cursor = *scroll_manager.world_cursor();
        simple2d::shapes(cache)
            .line(radius, world_cursor, [0.0, 0.0])
            .line(radius, world_cursor, game_dim)
            .line(radius, world_cursor, [0.0, game_dim[1]])
            .line(radius, world_cursor, [game_dim[0], 0.0]);

        buffer.update_clear(cache);

        ctx.draw_clear([0.13, 0.13, 0.13, 1.0]);

        let mut v = draw_sys.view(game_dim, *scroll_manager.camera_pos());
        v.draw_triangles(&walls, &[1.0, 1.0, 1.0, 0.2]);
        v.draw_triangles(&buffer, color_iter.peek().unwrap_throw());

        ctx.flush();
    }

    w.post_message(());

    log!("worker thread closing");
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
        let x = touch.screen_x() as f64;
        let y = touch.screen_y() as f64;

        let [x, y] = [(x - rect.left()) * scalex, (y - rect.top()) * scaley];

        ans.push([x as f32, y as f32]);
    }
    ans
}
