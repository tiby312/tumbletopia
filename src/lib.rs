use axgeom::{vec2, vec2same, Vec2};
use gloo::console::log;
use serde::{Deserialize, Serialize};
use shogo::{utils, simple2d::DynamicBuffer};
use wasm_bindgen::prelude::*;
use shogo::simple2d;

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
    let game_dim = [2000.0f32, 2000.0];
    let viewport = [canvas.width() as f32, canvas.height() as f32];

    let grid_width = 32;

    let grid_viewport = duckduckgeo::grid::GridViewPort {
        origin: vec2(0.0, 0.0),
        spacing: game_dim[0] / (grid_width as f32),
    };

    let mut k = simple2d::shapes(cache);
    for x in 0..grid_width {
        let offset = if x % 2 == 0 {
            0..grid_width
        } else {
            1..grid_width - 1
        };
        for y in offset.step_by(2) {
            k.rect(simple2d::Rect {
                x: x as f32 * grid_viewport.spacing,
                y: y as f32 * grid_viewport.spacing,
                w: grid_viewport.spacing,
                h: grid_viewport.spacing,
            });
        }
    }

    let checker = ctx.buffer_static_clear(cache);

    let mut grid_walls = grid::Grid2D::new(vec2same(grid_width));

    grid_walls.set(vec2(0,0),true);
    grid_walls.set(vec2(2,1),true);

    update_walls(&grid_viewport,cache,&mut walls,&grid_walls);

    let mut scroll_manager = scroll::ScrollController::new(vec2same(0.0));

    'outer: loop {
        for e in frame_timer.next().await {
            log!(format!("{:?}", e));
            match e {
                MEvent::CanvasMouseUp => {
                    if scroll_manager.handle_mouse_up() {
                        grid_walls.set(
                            grid_viewport.to_grid((scroll_manager.world_cursor(&game_dim)).into()),
                            true,
                        );
                        update_walls(&grid_viewport,cache,&mut walls,&grid_walls);
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

        
        //let world_cursor:[f32;2] = (scroll_manager.world_cursor(&game_dim)).into();

        //let cam=(scroll_manager.camera).into();
        let k=inverse_projection(viewport,(*scroll_manager.camera_pos()).into());
        let j:[f32;2]=scroll_manager.cursor_canvas.into();

        let a=canvas_to_clip(viewport);
        let res=a.mul_vector_left(&[j[0],j[1],0.0,1.0]);
        log!(format!("canvas clip:{:?}",res));
        
        let res=k.mul_vector_left(&[res[0],res[1],0.0,1.0]); 
        

        simple2d::shapes(cache)
            .rect(simple2d::Rect {
                x: res[0],
                y: res[1],
                w: grid_viewport.spacing,
                h: grid_viewport.spacing,
            });


        buffer.update_clear(cache);

        ctx.draw_clear([0.0, 0.0, 0.0, 0.0]);

        let matrix = projection(viewport, (*scroll_manager.camera_pos()).into());
        let mut v = draw_sys.view(&matrix);

        v.draw_triangles(&checker, &[0.3, 0.3, 0.3, 0.3]);

        v.draw_triangles(&walls, &[1.0, 0.5, 0.5, 1.0]);
        v.draw_triangles(&buffer, color_iter.peek().unwrap_throw());

        ctx.flush();
    }

    w.post_message(());

    log!("worker thread closing");
}


fn update_walls(grid_viewport:&duckduckgeo::grid::GridViewPort,cache:&mut Vec<[f32;2]>,buffer:&mut DynamicBuffer,grid_walls:&grid::Grid2D){
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



use webgl_matrix::prelude::*;



pub struct Doop<'a>(pub &'a mut [f32;16]);

impl<'a> Doop<'a>{
    pub fn scale(&mut self,x:f32,y:f32,z:f32)->&mut Self{
        self.0.mul(&[
            x, 0., 0., 0.,
             0., y, 0., 0.,
              0., 0., z, 0.,
               0., 0., 0., 1.0,
        ]);
        self
    }

    pub fn translation(&mut self,tx:f32,ty:f32,tz:f32)->&mut Self{
        self.0.mul(&[
            1., 0., 0., 0.,
             0., 1., 0., 0.,
              0., 0., 1., 0.,
               tx, ty, tz, 1.,
        ]);
        self
    }

    pub fn x_rotation(&mut self,angle_rad:f32)->&mut Self{
        let c = angle_rad.cos();
        let s = angle_rad.sin();

        self.0.mul(&
            [
                    1., 0., 0., 0.,
                    0., c, s, 0.,
                    0., -s, c, 0.,
                     0., 0., 0., 1.]);
        self
    }


    pub fn y_rotation(&mut self,angle_rad:f32)->&mut Self{
        let c = angle_rad.cos();
        let s = angle_rad.sin();

        self.0.mul(&
            [c, 0., -s, 0.,
             0., 1., 0., 0.,
              s, 0., c, 0.,
               0., 0., 0., 1.]);
        self
    }

    pub fn z_rotation(&mut self,angle_rad:f32)->&mut Self{
        let c = angle_rad.cos();
        let s = angle_rad.sin();

        self.0.mul(&
            [c, s, 0., 0.,
             -s, c, 0., 0.,
              0., 0., 1., 0.,
               0., 0., 0., 1.]);
        self
    }

    
}


fn canvas_to_clip(dim:[f32;2])->[f32;16]{

    
    let mut id=Mat4::identity();

    let mut doop=Doop(&mut id);
    
    doop.scale(2.0 / dim[0], -  2.0/dim[1], 0.0);
    doop.translation(-1.0,1.0,0.0);

    //TODO WHY THIS SCALE NEEDED?
    doop.scale(1.0,2.0,0.0);
    
    id
}





fn inverse_projection(dim: [f32; 2], offset: [f32; 2]) -> [f32; 16] {
    

    let mut id=Mat4::identity();

    let mut doop=Doop(&mut id);

    doop.
        translation(1.0,-1.0,0.0).    
        
        //WHY THIS SCALE NEEDED?  
        //scale(1.0,2.0,0.0).

        scale(dim[0]/2.0 , -dim[1]/2.0 , 0.0).
        translation(-( -dim[0] / 2. + offset[0]),-( -dim[1] / 2. + offset[1]), 0.0).
        x_rotation(-std::f32::consts::PI / 4.).
        z_rotation(-std::f32::consts::PI / 4.);
        
    id
}



//screenspace to clip space
fn projection(dim: [f32; 2], offset: [f32; 2]) -> [f32; 16] {
    

    let mut id=Mat4::identity();

    let mut doop=Doop(&mut id);

    doop.
        z_rotation(std::f32::consts::PI / 4.).
        x_rotation(std::f32::consts::PI / 4.).
        translation(-dim[0] / 2. + offset[0], -dim[1] / 2. + offset[1], 0.0).
        scale(2.0 / dim[0], -2.0 / dim[1], 0.0).
        translation(-1.0,1.0,0.0);

    id
}
