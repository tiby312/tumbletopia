use axgeom::{vec2, vec2same};
use cgmath::Transform;
use gloo::console::log;
use model::matrix;
use serde::{Deserialize, Serialize};
use shogo::simple2d::{self};
use shogo::{simple2d::DynamicBuffer, utils};
use wasm_bindgen::prelude::*;
pub mod dom;
mod model_parse;
mod projection;
mod util;
use dom::MEvent;
use duckduckgeo::grid::{self, GridViewPort};
use projection::*;

mod scroll;

const COLORS: &[[f32; 4]] = &[
    [1.0, 0.0, 0.0, 0.5],
    [0.0, 1.0, 0.0, 0.5],
    [0.0, 0.0, 1.0, 0.5],
];

#[wasm_bindgen]
pub async fn worker_entry() {
    let (mut w, ss) = shogo::EngineWorker::new().await;
    let mut frame_timer = shogo::FrameTimer::new(60, ss);

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

    // let checkers = {
    //     let mut positions = Vec::new();
    //     let mut k = simple2d::shapes(&mut positions);
    //     for x in 0..grid_width {
    //         let offset = if x % 2 == 0 {
    //             0..grid_width
    //         } else {
    //             1..grid_width - 1
    //         };
    //         for y in offset.step_by(2) {
    //             k.rect(
    //                 simple2d::Rect {
    //                     x: x as f32 * grid_viewport.spacing,
    //                     y: y as f32 * grid_viewport.spacing,
    //                     w: grid_viewport.spacing,
    //                     h: grid_viewport.spacing,
    //                 },
    //                 -1.0,
    //             );
    //         }
    //     }
    //     let j = (0..positions.len()).map(|_| [0.0; 2]).collect();
    //     let normals = (0..positions.len()).map(|_| [0.0,0.0,1.0]).collect();

    //     model::ModelData {
    //         matrix: cgmath::Matrix4::identity(),
    //         positions,
    //         normals,
    //         indices: None,
    //         texture: model::single_tex(),
    //         tex_coords: j,
    //     }
    // };

    //let checkers_gpu = ModelGpu::new(&ctx, &checkers);

    //let checker = ctx.buffer_static_clear(cache);

    
    let mut scroll_manager = scroll::TouchController::new([0., 0.].into());

    let drop_shadow = {
        let data = model::load_glb(DROP_SHADOW_GLB).gen_ext(grid_viewport.spacing);
        //log!(format!("grass:{:?}",(&data.positions.len(),&data.normals.len(),&data.indices.as_ref().map(|o|o.len()))));

        model_parse::ModelGpu::new(&ctx, &data)
    };

    let cat = {
        let data = model::load_glb(CAT_GLB).gen_ext(grid_viewport.spacing);
        model_parse::ModelGpu::new(&ctx, &data)
    };

    let grass = {
        let data = model::load_glb(GRASS_GLB).gen_ext(grid_viewport.spacing);

        model_parse::ModelGpu::new(&ctx, &data)
    };


    let mut cats=vec!([2;2]);

    'outer: loop {
        let mut j = false;
        let res = frame_timer.next().await;

        let matrix = view_projection(
            scroll_manager.camera(),
            viewport,
            scroll_manager.zoom(),
            scroll_manager.rot(),
        );

        for e in res {
            match e {
                MEvent::Resize {
                    canvasx: _canvasx,
                    canvasy: _canvasy,
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
                MEvent::TouchMove { touches } => {
                    scroll_manager.on_touch_move(touches, matrix);
                }
                MEvent::TouchDown { touches } => {
                    //log!(format!("touch down:{:?}",touches));
                    scroll_manager.on_new_touch(touches);
                }
                MEvent::TouchEnd { touches } => {
                    //log!(format!("touch end:{:?}",touches));
                    if let scroll::MouseUp::Select = scroll_manager.on_touch_up(&touches) {
                        j = true;
                    }
                }
                MEvent::CanvasMouseLeave => {
                    log!("mouse leaving!");
                    let _ = scroll_manager.on_mouse_up();
                }
                MEvent::CanvasMouseUp => {
                    if let scroll::MouseUp::Select = scroll_manager.on_mouse_up() {
                        j = true;
                    }
                }
                MEvent::CanvasMouseMove { x, y } => {
                    //log!(format!("{:?}",(x,y)));

                    scroll_manager.on_mouse_move([*x, *y], matrix);
                }
                MEvent::CanvasMouseDown { x, y } => {
                    //log!(format!("{:?}",(x,y)));

                    scroll_manager.on_mouse_down([*x, *y]);
                }
                MEvent::ButtonClick => {
                    let _ = color_iter.next();
                }
                MEvent::ShutdownClick => break 'outer,
            }
        }

        let mouse_world = scroll::mouse_to_world(scroll_manager.cursor_canvas(), matrix);

        if j {
            let val:[i16;2]=grid_viewport.to_grid((mouse_world).into()).into();
            if !cats.contains(&val){
                cats.push(val);
            }else{
                cats.retain(|a|a!=&val);
            }

        }
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

        let [vvx, vvy] = get_world_rect(matrix, &grid_viewport);

        

        for a in (vvx[0]..vvx[1])
            .skip_while(|&a| a < 0)
            .take_while(|&a| a < grid_width)
        {
            //both should be skip
            for b in (vvy[0]..vvy[1])
                .skip_while(|&a| a < 0)
                .take_while(|&a| a < grid_width)
            {
                use matrix::*;
                let x1 = grid_viewport.spacing * a as f32;
                let y1 = grid_viewport.spacing * b as f32;
                let s = 0.99;
                let mm = matrix
                    .chain(translation(x1, y1, -1.0))
                    .chain(scale(s, s, s))
                    .generate();

                let mut v = draw_sys.view(mm.as_ref());
                grass.draw(&mut v);
            }
        }

        {
            //draw dropshadow
            ctx.disable(WebGl2RenderingContext::DEPTH_TEST);
            ctx.disable(WebGl2RenderingContext::CULL_FACE);

            for a in cats.iter(){
                //let t = matrix::translation(a[0] as f32, a[1] as f32, 0.0);
                
                let mm=projection::grid_to_world_center().generate();
                let pos:[f32;3]=mm.transform_vector([a[0] as f32,a[1] as f32,1.0].into()).into();

                //let pos:[f32;2]=grid_viewport.to_world_center(a.into()).into();
                let t = matrix::translation(pos[0], pos[1] , 1.0);
            
                //let j = grid_viewport.spacing / 2.0;
                //let s = matrix::scale(1.0, 1.0, 1.0);
                //let m = matrix.chain(t).chain(s).generate();
                let m=matrix.chain(t).generate();
                
                let mut v = draw_sys.view(m.as_ref());
                drop_shadow.draw(&mut v);

            }

            ctx.enable(WebGl2RenderingContext::DEPTH_TEST);
            ctx.enable(WebGl2RenderingContext::CULL_FACE);
        }

        for a in cats.iter(){
            let pos:[f32;2]=grid_viewport.to_world_center(a.into()).into();
        
            let j = grid_viewport.spacing / 2.0;
            let t = matrix::translation(pos[0] - j, pos[1] - j, 20.0);
            let s = matrix::scale(1.0, 1.0, 1.0);
            let m = matrix.chain(t).chain(s).generate();
            let mut v = draw_sys.view(m.as_ref());
            cat.draw(&mut v);
        }

        ctx.flush();
    }

    w.post_message(());

    log!("worker thread closing");
}

use shogo::simple2d::Vertex;
use web_sys::WebGl2RenderingContext;


const DROP_SHADOW_GLB: &'static [u8] = include_bytes!("../assets/drop_shadow.glb");
// const SHADED_GLB: &'static [u8] = include_bytes!("../assets/shaded.glb");
// const KEY_GLB: &'static [u8] = include_bytes!("../assets/key.glb");
// const PERSON_GLB: &'static [u8] = include_bytes!("../assets/person-v1.glb");
const CAT_GLB: &'static [u8] = include_bytes!("../assets/tiger2.glb");
const GRASS_GLB: &'static [u8] = include_bytes!("../assets/grass.glb");
