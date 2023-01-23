use axgeom::vec2same;
use cgmath::{InnerSpace, Transform, Vector2};
use gloo::console::log;
use model::matrix;
use movement::GridCoord;
use serde::{Deserialize, Serialize};
use shogo::simple2d::{self};
use shogo::utils;
use wasm_bindgen::prelude::*;
pub mod animation;
pub mod dom;
pub mod grids;
pub mod model_parse;
pub mod movement;
pub mod projection;
pub mod scroll;
pub mod terrain;
pub mod util;
use dom::MEvent;
use projection::*;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
enum UiButton {
    ShowRoadUi,
    NoUi,
}

pub struct UnitCollection(Vec<GridCoord>);
impl UnitCollection {
    fn find_mut(&mut self, a: &GridCoord) -> Option<&mut GridCoord> {
        self.0.iter_mut().find(|b| *b == a)
    }
    fn filter(&self) -> UnitCollectionFilter {
        UnitCollectionFilter { a: &self.0 }
    }
}

pub struct UnitCollectionFilter<'a> {
    a: &'a [GridCoord],
}
impl<'a> movement::Filter for UnitCollectionFilter<'a> {
    fn filter(&self, a: &GridCoord) -> bool {
        !self.a.contains(a)
    }
}

enum CellSelection {
    MoveSelection(movement::PossibleMoves),
    BuildSelection(GridCoord),
}

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

    let road = {
        let data = model::load_glb(ROAD_GLB).gen_ext(gg.spacing());
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

    let mut cats = UnitCollection(vec![
        GridCoord([2, 2]),
        GridCoord([5, 5]),
        GridCoord([6, 6]),
        GridCoord([7, 7]),
        GridCoord([3, 1]),
    ]);

    let mut roads = terrain::TerrainCollection {
        pos: vec![],
        func: |a: MoveUnit| MoveUnit(a.0 / 2),
    };

    // let mut roads=terrain::TerrainCollection{
    //     pos:vec!(),
    //     func:|a:MoveUnit|MoveUnit(a.0+10)
    // };

    let mut selected_cell: Option<CellSelection> = None;
    let mut animation = None;
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
                MEvent::ButtonClick => match selected_cell {
                    Some(CellSelection::BuildSelection(g)) => {
                        log!("adding to roads!!!!!");
                        roads.pos.push(g);
                        selected_cell = None;
                    }
                    _ => {
                        panic!("Received button push when we did not ask for it!")
                    }
                },
                MEvent::ShutdownClick => break 'outer,
            }
        }

        let mouse_world = scroll::mouse_to_world(scroll_manager.cursor_canvas(), matrix);

        if on_select {
            let cell: GridCoord = GridCoord(gg.to_grid((mouse_world).into()).into());

            if let Some(ss) = &mut selected_cell {
                match ss {
                    CellSelection::MoveSelection(ss) => {
                        if movement::contains_coord(ss.iter_coords(), &cell) {
                            animation = Some(animation::Animation::new(
                                ss.start(),
                                ss.get_path(cell).unwrap(),
                                &gg,
                            ));
                            let c = cats.find_mut(ss.start()).unwrap();
                            *c = cell;
                        }
                        selected_cell = None;
                    }
                    CellSelection::BuildSelection(_) => {
                        //do nothing? we are waiting on user to push a button.
                    }
                }
            } else {
                if cats.0.contains(&cell) {
                    let oo = movement::PossibleMoves::new(
                        &movement::WarriorMovement,
                        &gg.filter().chain(cats.filter()),
                        &terrain::Grass.chain(roads.foo()),
                        cell,
                        MoveUnit(3),
                    );
                    selected_cell = Some(CellSelection::MoveSelection(oo));
                } else {
                    selected_cell = Some(CellSelection::BuildSelection(cell));
                    //activate the build options for that terrain
                    w.post_message(UiButton::ShowRoadUi);
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
                match a {
                    CellSelection::MoveSelection(a) => {
                        for GridCoord(a) in a.iter_coords() {
                            let pos: [f32; 2] = gg.to_world_topleft(a.into()).into();
                            let t = matrix::translation(pos[0], pos[1], 0.0);

                            let m = matrix.chain(t).generate();

                            let mut v = draw_sys.view(m.as_ref());
                            select_model.draw(&mut v);
                        }
                    }
                    CellSelection::BuildSelection(_) => {}
                }
            }

            for GridCoord(a) in roads.pos.iter() {
                let pos: [f32; 2] = gg.to_world_topleft(a.into()).into();
                let t = matrix::translation(pos[0], pos[1], 3.0);

                let m = matrix.chain(t).generate();

                let mut v = draw_sys.view(m.as_ref());
                road.draw(&mut v);
            }

            ctx.enable(WebGl2RenderingContext::DEPTH_TEST);
            ctx.enable(WebGl2RenderingContext::CULL_FACE);
        }

        {
            //draw dropshadow
            ctx.disable(WebGl2RenderingContext::DEPTH_TEST);
            ctx.disable(WebGl2RenderingContext::CULL_FACE);

            for &GridCoord(a) in cats.0.iter() {
                let pos: [f32; 2] = gg.to_world_topleft(a.into()).into();
                let t = matrix::translation(pos[0], pos[1], 1.0);

                let m = matrix.chain(t).generate();

                let mut v = draw_sys.view(m.as_ref());
                drop_shadow.draw(&mut v);
            }

            ctx.enable(WebGl2RenderingContext::DEPTH_TEST);
            ctx.enable(WebGl2RenderingContext::CULL_FACE);
        }

        for &GridCoord(a) in cats.0.iter() {
            let pos: [f32; 2] = gg.to_world_topleft(a.into()).into();

            let t = matrix::translation(pos[0], pos[1], 20.0);
            let s = matrix::scale(1.0, 1.0, 1.0);
            let m = matrix.chain(t).chain(s).generate();
            let mut v = draw_sys.view(m.as_ref());
            cat.draw(&mut v);
        }

        if let Some(a) = &mut animation {
            if let Some(pos) = a.animate_step(5.0) {
                let t = matrix::translation(pos[0], pos[1], 20.0);
                let s = matrix::scale(1.0, 1.0, 1.0);
                let m = matrix.chain(t).chain(s).generate();
                let mut v = draw_sys.view(m.as_ref());
                cat.draw(&mut v);
            } else {
                animation = None;
            };
        }

        ctx.flush();
    }

    w.post_message(UiButton::NoUi);

    log!("worker thread closing");
}

use web_sys::WebGl2RenderingContext;

use crate::movement::{Filter, MoveUnit};
use crate::terrain::MoveCost;

const SELECT_GLB: &'static [u8] = include_bytes!("../assets/select_model.glb");
const DROP_SHADOW_GLB: &'static [u8] = include_bytes!("../assets/drop_shadow.glb");
const ROAD_GLB: &'static [u8] = include_bytes!("../assets/road.glb");

// const SHADED_GLB: &'static [u8] = include_bytes!("../assets/shaded.glb");
// const KEY_GLB: &'static [u8] = include_bytes!("../assets/key.glb");
// const PERSON_GLB: &'static [u8] = include_bytes!("../assets/person-v1.glb");
const CAT_GLB: &'static [u8] = include_bytes!("../assets/tiger2.glb");
const GRASS_GLB: &'static [u8] = include_bytes!("../assets/grass.glb");
