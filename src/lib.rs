use axgeom::{vec2, Vec2, vec2same};
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


    #[derive(PartialEq,Debug)]
    pub enum Scrollin{
        MouseDown{
            anchor:[f32;2]
        },
        Scrolling{
            anchor:[f32;2]
        },
        NotScrolling
    }

    let mut camera_velocity=vec2same(0.0);
    let mut camera=vec2same(0.0);


    let mut scrolling=Scrollin::NotScrolling;
    'outer: loop {
        for e in frame_timer.next().await {
            match e {
                MEvent::CanvasMouseUp=>{
                    match scrolling{
                        Scrollin::MouseDown{..}=>{
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
                        Scrollin::Scrolling{anchor}=>{
                            let curr:Vec2<_>=mouse_pos.into();
                            let anchor:Vec2<_>=anchor.into();
                            camera_velocity=(curr-anchor)*0.02;
                            scrolling=Scrollin::NotScrolling;
                        }
                        Scrollin::NotScrolling=>{
                            panic!("not possible?")
                        }
                    }
                }
                MEvent::CanvasMouseMove { x, y } => {
                    mouse_pos = [*x, *y];
                    match scrolling{
                        Scrollin::MouseDown{anchor}=>{
                            scrolling=Scrollin::Scrolling{anchor}
                        },
                        _=>{}
                    }
                },
                MEvent::CanvasMouseDown=>{
                    scrolling=Scrollin::MouseDown{
                        anchor:mouse_pos
                    };
                }
                MEvent::ButtonClick => {
                    let _ = color_iter.next();
                }
                MEvent::ShutdownClick => break 'outer,
            }
        }

        log!(format!("{:?}",&scrolling));



        {
            camera+=camera_velocity;
            camera_velocity*=0.99;
        }


        simple2d::shapes(cache)
            .line(radius, mouse_pos, [0.0, 0.0])
            .line(radius, mouse_pos, game_dim)
            .line(radius, mouse_pos, [0.0, game_dim[1]])
            .line(radius, mouse_pos, [game_dim[0], 0.0]);

        buffer.update_clear(cache);

        ctx.draw_clear([0.13, 0.13, 0.13, 1.0]);

        let mut v = draw_sys.view(game_dim, camera);
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



pub trait Stuff{
    type N;
    fn sub(self,other:Self)->Self;
    fn div(self,other:Self::N)->Self;
}
impl Stuff for [f32;2]{
    type N=f32;
    fn sub(self,other:Self)->Self{
        [other[0]-self[0],other[1]-self[1]]
    }
    fn div(self,other:Self::N)->Self{
        [self[0]/other,self[1]/other]
    }
}