use axgeom::vec2same;
use cgmath::Transform;
use gloo::console::log;
use model::matrix;
use serde::{Deserialize, Serialize};
use shogo::simple2d::{self};
use shogo::utils;
use wasm_bindgen::prelude::*;
pub mod dom;
mod model_parse;
mod projection;
mod util;
use dom::MEvent;
use projection::*;

mod grids;
mod movement;
mod scroll;

// enum SelectState<'a>{
//     Nothing,
//     CatSelected{
//         cat_pos:[i16;2],
//         allowable_squares:&'a mut Vec<[i16;2]>,
//     }
// }

// //Handles different states within one turn only!!!
// pub struct Player<'a>{
//     state:SelectState<'a>
// }
// impl<'a> Player<'a>{
//     fn handle_mouse_select(&mut self,grid:[i16;2]){
//         match self.state{
//             SelectState::Nothing=>{

//             },
//             SelectState::CatSelected(pos)=>{

//             }
//         }
//     }
//     fn draw(&mut self){
//         match self.state{
//             SelectState::Nothing=>{

//             },
//             SelectState::CatSelected(pos)=>{

//             }
//         }
//     }
// }

// struct World{

// }

// pub trait SelectedItem{
//     fn handle_select(&mut self,grid:[i16;2],world:&mut World){

//     }
//     fn draw(&self){

//     }
// }

#[wasm_bindgen]
pub async fn worker_entry() {
    let (mut w, ss) = shogo::EngineWorker::new().await;
    let mut frame_timer = shogo::FrameTimer::new(60, ss);

    let canvas = w.canvas();
    let ctx = simple2d::ctx_wrap(&utils::get_context_webgl2_offscreen(&canvas));

    let mut draw_sys = ctx.shader_system();
    let mut buffer = ctx.buffer_dynamic();
    let cache = &mut vec![];

    //TODO get rid of this somehow.
    //these values are incorrect.
    //they are set correctly after resize is called on startup.
    let gl_width = canvas.width(); // as f32*1.6;
    let gl_height = canvas.height(); // as f32*1.6;
    ctx.viewport(0, 0, gl_width as i32, gl_height as i32);
    let mut viewport = [canvas.width() as f32, canvas.height() as f32];

    ctx.setup_alpha();

    let gg = grids::GridMatrix::new();

    let mut scroll_manager = scroll::TouchController::new([0., 0.].into());

    let drop_shadow = {
        let data = model::load_glb(DROP_SHADOW_GLB).gen_ext(gg.spacing());
        model_parse::ModelGpu::new(&ctx, &data)
    };

    let cat = {
        let data = model::load_glb(CAT_GLB).gen_ext(gg.spacing());
        model_parse::ModelGpu::new(&ctx, &data)
    };

    let grass = {
        let data = model::load_glb(GRASS_GLB).gen_ext(gg.spacing());

        model_parse::ModelGpu::new(&ctx, &data)
    };

    let select_model = {
        let data = model::load_glb(SELECT_GLB).gen_ext(gg.spacing());

        model_parse::ModelGpu::new(&ctx, &data)
    };

    let mut cats = vec![[2; 2]];

    let mut selected_cell:Option<movement::PossibleMoves> = None;

    'outer: loop {
        let mut on_select = false;
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
                        on_select = true;
                    }
                }
                MEvent::CanvasMouseLeave => {
                    log!("mouse leaving!");
                    let _ = scroll_manager.on_mouse_up();
                }
                MEvent::CanvasMouseUp => {
                    if let scroll::MouseUp::Select = scroll_manager.on_mouse_up() {
                        on_select = true;
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
                MEvent::ButtonClick => {}
                MEvent::ShutdownClick => break 'outer,
            }
        }

        let mouse_world = scroll::mouse_to_world(scroll_manager.cursor_canvas(), matrix);

        if on_select {
            let cell: [i16; 2] = gg.to_grid((mouse_world).into()).into();

            if let Some(gg)=selected_cell{
                if gg.contains_coord(&GridCoord(cell)){
                    let c=cats.iter_mut().find(|a|**a==gg.start().0).unwrap();
                    *c=cell;
                }
                selected_cell=None;

            }else{    
                
                if cats.contains(&cell){

                    let gg = movement::PossibleMoves::new::<movement::WarriorMovement>(
                        GridCoord(cell),
                        MoveUnit(2),
                    );
                    selected_cell = Some(gg);
                }
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

        let [vvx, vvy] = get_world_rect(matrix, &gg);

        for a in (vvx[0]..vvx[1])
            .skip_while(|&a| a < 0)
            .take_while(|&a| a < gg.num_rows())
        {
            //both should be skip
            for b in (vvy[0]..vvy[1])
                .skip_while(|&a| a < 0)
                .take_while(|&a| a < gg.num_rows())
            {
                use matrix::*;
                let x1 = gg.spacing() * a as f32;
                let y1 = gg.spacing() * b as f32;
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
            ctx.disable(WebGl2RenderingContext::DEPTH_TEST);
            ctx.disable(WebGl2RenderingContext::CULL_FACE);

            if let Some(a) = &selected_cell {
                for GridCoord(a) in a.iter_coords() {
                    let pos: [f32; 2] = gg.to_world_topleft(a.into()).into();
                    let t = matrix::translation(pos[0], pos[1], 0.0);

                    let m = matrix.chain(t).generate();

                    let mut v = draw_sys.view(m.as_ref());
                    select_model.draw(&mut v);
                }
            }
            ctx.enable(WebGl2RenderingContext::DEPTH_TEST);
            ctx.enable(WebGl2RenderingContext::CULL_FACE);
        }

        {
            //draw dropshadow
            ctx.disable(WebGl2RenderingContext::DEPTH_TEST);
            ctx.disable(WebGl2RenderingContext::CULL_FACE);

            for &a in cats.iter() {
                let pos: [f32; 2] = gg.to_world_topleft(a.into()).into();
                let t = matrix::translation(pos[0], pos[1], 1.0);

                let m = matrix.chain(t).generate();

                let mut v = draw_sys.view(m.as_ref());
                drop_shadow.draw(&mut v);
            }

            ctx.enable(WebGl2RenderingContext::DEPTH_TEST);
            ctx.enable(WebGl2RenderingContext::CULL_FACE);
        }

        for a in cats.iter() {
            let pos: [f32; 2] = gg.to_world_topleft(a.into()).into();

            let t = matrix::translation(pos[0], pos[1], 20.0);
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

use web_sys::WebGl2RenderingContext;

use crate::movement::{GridCoord, MoveUnit};

const SELECT_GLB: &'static [u8] = include_bytes!("../assets/select_model.glb");
const DROP_SHADOW_GLB: &'static [u8] = include_bytes!("../assets/drop_shadow.glb");
// const SHADED_GLB: &'static [u8] = include_bytes!("../assets/shaded.glb");
// const KEY_GLB: &'static [u8] = include_bytes!("../assets/key.glb");
// const PERSON_GLB: &'static [u8] = include_bytes!("../assets/person-v1.glb");
const CAT_GLB: &'static [u8] = include_bytes!("../assets/tiger2.glb");
const GRASS_GLB: &'static [u8] = include_bytes!("../assets/grass.glb");
