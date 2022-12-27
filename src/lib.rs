use axgeom::vec2;
use gloo::console::log;
use serde::{Deserialize, Serialize};
use shogo::utils;
use wasm_bindgen::prelude::*;



use duckduckgeo::grid;





const COLORS: &[[f32; 4]] = &[
    [1.0, 0.0, 0.0, 0.5],
    [0.0, 1.0, 0.0, 0.5],
    [0.0, 0.0, 1.0, 0.5],
];

///Common data sent from the main thread to the worker.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MEvent {
    CanvasMouseMove { x: f32, y: f32 },
    CanvasMouseDown,
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

    let (mut worker, mut response) = shogo::EngineMain::new("./gridlock_worker.js", offscreen).await;

    let _handler = worker.register_event(&canvas, "mousemove", |e| {
        let [x, y] = convert_coord(e.elem, e.event);
        MEvent::CanvasMouseMove { x, y }
    });

    let _handler = worker.register_event(&canvas, "mousedown", |e| {
        MEvent::CanvasMouseDown
    });

    let _handler = worker.register_event(&canvas, "mouseup", |e| {
        MEvent::CanvasMouseUp
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
    let mut mouse_pos = [0.0f32; 2];
    let mut color_iter = COLORS.iter().cycle().peekable();
    let radius = 4.0;
    let game_dim = [canvas.width() as f32, canvas.height() as f32];


    let grid_viewport=duckduckgeo::grid::GridViewPort{origin:vec2(0.0,0.0),spacing:game_dim[0]/(64 as f32)};

    let mut grid_walls=grid::Grid2D::new(axgeom::Vec2{x:64,y:64});


    //let mut scrolling=false;
    //let mut mouse_is_down=false;
    'outer: loop {
        for e in frame_timer.next().await {
            match e {
                MEvent::CanvasMouseUp=>{
                
                    grid_walls.set(grid_viewport.to_grid(mouse_pos.into()),true);
                    let mut s=simple2d::shapes(cache);
                    for (p,val) in grid_walls.iter(){
                        if val{
                            let top_left=grid_viewport.to_world_topleft(p);
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
                MEvent::CanvasMouseMove { x, y } => {
                    mouse_pos = [*x, *y];
                    // if mouse_is_down{
                    //     scrolling=true;
                    // }
                },
                MEvent::CanvasMouseDown=>{

                }
                MEvent::ButtonClick => {
                    let _ = color_iter.next();
                }
                MEvent::ShutdownClick => break 'outer,
            }
        }

        simple2d::shapes(cache)
            .line(radius, mouse_pos, [0.0, 0.0])
            .line(radius, mouse_pos, game_dim)
            .line(radius, mouse_pos, [0.0, game_dim[1]])
            .line(radius, mouse_pos, [game_dim[0], 0.0]);

        buffer.update_clear(cache);

        ctx.draw_clear([0.13, 0.13, 0.13, 1.0]);

        let mut v = draw_sys.view(game_dim, [0.0, 0.0]);
        v.draw_triangles(&walls, &[1.0, 1.0, 1.0, 0.2]);
        v.draw_triangles(&buffer, color_iter.peek().unwrap_throw());

        ctx.flush();
    }

    w.post_message(());

    log!("worker thread closing");
}


fn convert_canvas_to_grid(){

}
fn convert_grid_canvas(){

}



//convert DOM coordinate to canvas relative coordinate
fn convert_coord(canvas: &web_sys::HtmlElement, event: &web_sys::Event) -> [f32; 2] {
    use wasm_bindgen::JsCast;
    shogo::simple2d::convert_coord(canvas, event.dyn_ref().unwrap_throw())
}
