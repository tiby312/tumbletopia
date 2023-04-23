use ace::AnimationOptions;
use axgeom::vec2same;
use cgmath::{InnerSpace, Matrix4, Transform, Vector2};

use futures::{SinkExt, StreamExt};
use gloo::console::log;
use grids::GridMatrix;
use model::matrix::{self, MyMatrix};
use movement::GridCoord;
use serde::{Deserialize, Serialize};
use shogo::simple2d::{self, ShaderSystem};
use shogo::utils;
use wasm_bindgen::prelude::*;
pub mod animation;
pub mod dom;
//pub mod gameplay;
pub mod grids;
pub mod model_parse;
pub mod movement;
pub mod projection;
pub mod scroll;
pub mod terrain;
pub mod util;
use dom::MEvent;
use projection::*;
pub mod ace;
pub mod hex;
pub mod unit;

use unit::*;

//pub mod state;
//pub mod logic;
pub const RESIZE: usize = 10;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
enum UiButton {
    ShowRoadUi,
    NoUi,
}

pub struct WarriorDraw<'a> {
    model: &'a MyModel,
    drop_shadow: &'a MyModel,
    col: &'a UnitCollection<UnitData>,
}
impl<'a> WarriorDraw<'a> {
    fn new(
        col: &'a UnitCollection<UnitData>,
        model: &'a MyModel,
        drop_shadow: &'a MyModel,
    ) -> Self {
        Self {
            model,
            drop_shadow,
            col,
        }
    }
    fn draw(&self, gg: &grids::GridMatrix, draw_sys: &mut ShaderSystem, matrix: &Matrix4<f32>) {
        for cc in self.col.elem.iter() {
            let pos = gg.hex_axial_to_world(&cc.position);

            // let pos: [f32; 2] = gg.to_world_topleft(cc.position.0.into()).into();

            let t = matrix::translation(pos[0], pos[1], 0.0);
            let s = matrix::scale(1.0, 1.0, 1.0);
            let m = matrix.chain(t).chain(s).generate();
            let mut v = draw_sys.view(m.as_ref());

            self.model.draw_ext(
                &mut v,
                !cc.selectable, /*  !cc.selectable(game)  */
                false,
                false,
                true,
            );
        }
    }

    fn draw_shadow(
        &self,
        gg: &grids::GridMatrix,
        draw_sys: &mut ShaderSystem,
        matrix: &Matrix4<f32>,
    ) {
        for a in self.col.elem.iter().map(|a| &a.position) {
            let pos: [f32; 2] = gg.hex_axial_to_world(a).into();
            let t = matrix::translation(pos[0], pos[1], 1.0);

            let m = matrix.chain(t).generate();

            let mut v = draw_sys.view(m.as_ref());
            self.drop_shadow.draw(&mut v);
        }
    }

    fn draw_health_text(
        &self,
        gg: &grids::GridMatrix,
        health_numbers: &NumberTextManager,
        view_proj: &Matrix4<f32>,
        proj: &Matrix4<f32>,
        draw_sys: &mut ShaderSystem,
    ) {
        //draw text
        for ccat in self.col.elem.iter() {
            let pos: [f32; 2] = gg.hex_axial_to_world(&ccat.position).into();

            let t = matrix::translation(pos[0], pos[1] + 20.0, 20.0);

            let jj = view_proj.chain(t).generate();
            let jj: &[f32; 16] = jj.as_ref();
            let tt = matrix::translation(jj[12], jj[13], jj[14]);
            let new_proj = proj.clone().chain(tt);

            let s = matrix::scale(5.0, 5.0, 5.0);
            let m = new_proj.chain(s).generate();

            let nn = health_numbers.get_number(ccat.health);
            let mut v = draw_sys.view(m.as_ref());
            nn.draw_ext(&mut v, false, false, true, false);

            //nn.draw(ccat.health,&ctx,&text_texture,&mut draw_sys,&m);
        }

        // for ccat in self.col.elem.iter() {
        //     let pos: [f32; 2] = gg.hex_axial_to_world(&ccat.position).into();

        //     let t = matrix::translation(pos[0] + 20.0, pos[1], 20.0);

        //     let jj = view_proj.chain(t).generate();
        //     let jj: &[f32; 16] = jj.as_ref();
        //     let tt = matrix::translation(jj[12], jj[13], jj[14]);
        //     let new_proj = proj.clone().chain(tt);

        //     let s = matrix::scale(5.0, 5.0, 5.0);
        //     let m = new_proj.chain(s).generate();

        //     let nn = health_numbers.get_number(ccat.stamina.0);
        //     let mut v = draw_sys.view(m.as_ref());
        //     nn.draw_ext(&mut v, false, false, true, false);

        //     //nn.draw(ccat.health,&ctx,&text_texture,&mut draw_sys,&m);
        // }
    }
}

type MyModel = model_parse::Foo<model_parse::TextureGpu, model_parse::ModelGpu>;

// pub struct RestOfUnits<'a, T> {
//     first: &'a mut [T],
//     second: &'a mut [T],
// }

// fn get_unit<'a>(
//     a: &'a mut UnitCollection<Warrior>,
//     coord: &GridCoord,
// ) -> (&'a mut Warrior, RestOfUnits<'a, Warrior>) {
//     let (unit, _) = a
//         .elem
//         .iter()
//         .enumerate()
//         .find(|(_, b)| b.get_pos() == coord)
//         .unwrap();

//     let (first, mid, second) = split_at_mid_mut(&mut a.elem, unit);
//     (mid, RestOfUnits { first, second })
// }

// fn split_at_mid_mut<T>(a: &mut [T], ind: usize) -> (&mut [T], &mut T, &mut [T]) {
//     let (left, right) = a.split_at_mut(ind);
//     let (mid, rest) = right.split_first_mut().unwrap();
//     (left, mid, rest)
// }

//TODO store actual world pos? Less calculation each iteration.
//Additionally removes need to special case animation.
#[derive(Debug)]
pub struct Game {
    dogs: Tribe,
    cats: Tribe,
}

#[wasm_bindgen]
pub async fn worker_entry() {
    console_error_panic_hook::set_once();

    let (mut w, ss) = shogo::EngineWorker::new().await;
    let mut frame_timer = shogo::FrameTimer::new(60, ss);

    let canvas = w.canvas();
    let ctx = simple2d::ctx_wrap(&utils::get_context_webgl2_offscreen(&canvas));

    let mut draw_sys = ctx.shader_system();

    //TODO get rid of this somehow.
    //these values are incorrect.
    //they are set correctly after resize is called on startup.
    let gl_width = canvas.width(); // as f32*1.6;
    let gl_height = canvas.height(); // as f32*1.6;
    ctx.viewport(0, 0, gl_width as i32, gl_height as i32);
    let mut viewport = [canvas.width() as f32, canvas.height() as f32];

    ctx.setup_alpha();

    //TODO delete
    //let gg = grids::GridMatrix::new();

    let mut scroll_manager = scroll::TouchController::new([0., 0.].into());

    let dogs = vec![
        UnitCollection::new(vec![
            UnitData::new(GridCoord([1, -2])),
            // UnitData::new(GridCoord([1, -1])),
            //UnitData::new(GridCoord([2, -1])),
        ]),
        UnitCollection::new(vec![
        // UnitData::new(GridCoord([2, -2]))
        ]),
        UnitCollection::new(vec![
        UnitData::new(GridCoord([3, -3]))
        ]),
        UnitCollection::new(vec![
            UnitData::new(GridCoord([3, 0])),
            // UnitData::new(GridCoord([0, -3])),
        ]),
    ];

    let cats = vec![
        UnitCollection::new(vec![
            UnitData::new(GridCoord([-2, 1])),
            //  UnitData::new(GridCoord([-1, 1])),
            // UnitData::new(GridCoord([-1, 2])),
        ]),
        UnitCollection::new(vec![
        // UnitData::new(GridCoord([-2, 2])),
        // UnitData::new(GridCoord([-3, 3]))

        ]),
        UnitCollection::new(vec![
        UnitData::new(GridCoord([-3, 3]))
        ]),
        UnitCollection::new(vec![
            UnitData::new(GridCoord([0, 3])),
            // UnitData::new(GridCoord([-3, 0])),
        ]),
    ];

    let mut ggame = Game {
        dogs: Tribe { warriors: dogs },
        cats: Tribe { warriors: cats },
    };

    let roads = terrain::TerrainCollection {
        pos: vec![],
        func: |a: MoveUnit| MoveUnit(a.0 / 2),
    };

    use cgmath::SquareMatrix;
    let mut last_matrix = cgmath::Matrix4::identity();

    let grid_matrix = grids::GridMatrix::new();
    let quick_load = |name, res, alpha| {
        let (data, t) = model::load_glb(name).gen_ext(grid_matrix.spacing(), res, alpha);

        log!(format!("texture:{:?}", (t.width, t.height)));
        model_parse::Foo {
            texture: model_parse::TextureGpu::new(&ctx, &t),
            model: model_parse::ModelGpu::new(&ctx, &data),
        }
    };

    let drop_shadow = quick_load(DROP_SHADOW_GLB, 1, Some(0.5));

    let dog = quick_load(DOG_GLB, RESIZE, None);

    let cat = quick_load(CAT_GLB, RESIZE, None);

    let road = quick_load(ROAD_GLB, 1, None);

    let grass = quick_load(GRASS_GLB, RESIZE, None);

    let select_model = quick_load(SELECT_GLB, 1, None);

    let attack_model = quick_load(ATTACK_GLB, 1, None);

    let friendly_model = quick_load(FRIENDLY_GLB, 1, None);

    let text_texture = {
        let ascii_tex = model::load_texture_from_data(include_bytes!("../assets/ascii5.png"));

        model_parse::TextureGpu::new(&ctx, &ascii_tex)
    };

    let health_numbers = NumberTextManager::new(&ctx, &text_texture);

    let (command_sender, mut command_recv) = futures::channel::mpsc::channel(5);
    let (mut response_sender, response_recv) = futures::channel::mpsc::channel(5);

    let main_logic = async {
        ace::main_logic(command_sender, response_recv, &mut ggame, &grid_matrix).await;
    };

    let mut mouse_mouse = [0.0; 2];
    let render_thread = async {
        loop {
            let ace::GameWrap {
                game: ggame,
                data: mut command,
                team,
            } = command_recv.next().await.unwrap();

            'outer: loop {
                let mut on_select = false;

                let res = frame_timer.next().await;

                let mut end_turn = false;
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
                            scroll_manager.on_touch_move(touches, &last_matrix, viewport);
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
                            mouse_mouse = [*x, *y];
                            scroll_manager.on_mouse_move([*x, *y], &last_matrix, viewport);
                        }
                        MEvent::EndTurn => {
                            end_turn = true;
                        }
                        MEvent::CanvasMouseDown { x, y } => {
                            scroll_manager.on_mouse_down([*x, *y]);
                        }
                        MEvent::ButtonClick => {}
                        MEvent::ShutdownClick => break 'outer,
                    }
                }

                let proj = projection::projection(viewport).generate();
                let view_proj = projection::view_matrix(
                    scroll_manager.camera(),
                    scroll_manager.zoom(),
                    scroll_manager.rot(),
                );

                let matrix = proj.chain(view_proj).generate();

                last_matrix = matrix;

                //TODO don't compute every frame?.
                let mouse_world =
                    scroll::mouse_to_world(scroll_manager.cursor_canvas(), &matrix, viewport);

                match &mut command {
                    ace::Command::Animate(a) => {
                        if let Some(_) = a.animate_step() {
                        } else {
                            let a = command.take_animation();
                            response_sender
                                .send(ace::GameWrapResponse {
                                    game: ggame,
                                    data: ace::Response::AnimationFinish(a),
                                })
                                .await
                                .unwrap();
                            break 'outer;
                        }
                    }
                    ace::Command::GetMouseInput(_) => {
                        if end_turn {
                            response_sender
                                .send(ace::GameWrapResponse {
                                    game: ggame,
                                    data: ace::Response::Mouse(
                                        command.take_cell(),
                                        Pototo::EndTurn,
                                    ),
                                })
                                .await
                                .unwrap();
                            break 'outer;
                        } else if on_select {
                            let mouse: GridCoord =
                                grid_matrix.center_world_to_hex(mouse_world.into());
                            log!(format!("pos:{:?}", mouse));

                            response_sender
                                .send(ace::GameWrapResponse {
                                    game: ggame,
                                    data: ace::Response::Mouse(
                                        command.take_cell(),
                                        Pototo::Normal(mouse),
                                    ),
                                })
                                .await
                                .unwrap();
                            break 'outer;
                        }
                    }
                    ace::Command::Nothing => {}
                }

                // {
                //     //Advance state machine.
                //     let mouse = on_select.then_some(mouse_world);
                //     let [this_team, that_team] =
                //         state::team_view([&mut ggame.cats, &mut ggame.dogs], ggame.team);

                //     let mut jj = state::Stuff {
                //         team: &mut ggame.team,
                //         this_team,
                //         that_team,
                //         grid_matrix: &ggame.grid_matrix,
                //         mouse,
                //         end_turn,
                //     };
                //     testo.step(&mut jj);
                // }

                scroll_manager.step();

                use matrix::*;

                //Drawing below doesnt need mutable reference.
                //TODO move drawing to a function?
                let ggame = &ggame;

                ctx.draw_clear([0.0, 0.0, 0.0, 0.0]);

                let [vvx, vvy] = get_world_rect(&matrix, &grid_matrix);

                //

                for c in grid_matrix.world() {
                    let pos = grid_matrix.hex_axial_to_world(&c.to_axial());

                    //let pos = a.calc_pos();
                    let t = matrix::translation(pos[0], pos[1], -10.0);
                    let s = matrix::scale(1.0, 1.0, 1.0);
                    let m = matrix.chain(t).chain(s).generate();
                    let mut v = draw_sys.view(m.as_ref());

                    grass.draw(&mut v);
                }

                disable_depth(&ctx, || {
                    if let ace::Command::GetMouseInput(a) = &command {
                        let (a, greyscale) = match a {
                            MousePrompt::Friendly(c) => (c, false),
                            MousePrompt::Enemy(c) => (c, true),
                            MousePrompt::None => return,
                        };

                        //if let Some(a) = testo.get_selection() {
                        match a {
                            CellSelection::MoveSelection(a, friendly, attack) => {
                                for a in a.iter_coords() {
                                    let pos: [f32; 2] = grid_matrix.hex_axial_to_world(a).into();
                                    let t = matrix::translation(pos[0], pos[1], 0.0);

                                    let m = matrix.chain(t).generate();

                                    let mut v = draw_sys.view(m.as_ref());

                                    select_model.draw_ext(&mut v, greyscale, false, false, false);

                                    //select_model.draw(&mut v);
                                }

                                for a in attack.iter() {
                                    let pos: [f32; 2] = grid_matrix.hex_axial_to_world(a).into();
                                    let t = matrix::translation(pos[0], pos[1], 0.0);

                                    let m = matrix.chain(t).generate();

                                    let mut v = draw_sys.view(m.as_ref());
                                    //attack_model.draw(&mut v);
                                    attack_model.draw_ext(&mut v, greyscale, false, false, false);
                                }

                                for a in friendly.iter() {
                                    let pos: [f32; 2] = grid_matrix.hex_axial_to_world(a).into();
                                    let t = matrix::translation(pos[0], pos[1], 0.0);

                                    let m = matrix.chain(t).generate();

                                    let mut v = draw_sys.view(m.as_ref());
                                    //attack_model.draw(&mut v);
                                    friendly_model.draw_ext(&mut v, greyscale, false, false, false);
                                }
                            }
                            CellSelection::BuildSelection(_) => {}
                        }
                    }

                    // { TEST MOUSE
                    //     let mouse_mouse= scroll::mouse_to_world(mouse_mouse, &matrix, viewport);

                    //     let a: GridCoord =grid_matrix.center_world_to_hex(mouse_mouse.into());

                    //     let pos: [f32; 2] = grid_matrix.hex_axial_to_world(&a).into();
                    //     let t = matrix::translation(pos[0], pos[1], 3.0);

                    //     let m = matrix.chain(t).generate();

                    //     let mut v = draw_sys.view(m.as_ref());
                    //     road.draw(&mut v);
                    // }
                    //for a in roads.pos.iter() {
                    // let a: GridCoord =grid_matrix.center_world_to_hex(mouse_world.into());

                    // let pos: [f32; 2] = grid_matrix.hex_axial_to_world(&a).into();
                    // let t = matrix::translation(pos[0], pos[1], 3.0);

                    // let m = matrix.chain(t).generate();

                    // let mut v = draw_sys.view(m.as_ref());
                    // road.draw(&mut v);
                    //}
                });

                for i in 0..4 {
                    let cat_draw = WarriorDraw::new(&ggame.cats.warriors[i], &cat, &drop_shadow);
                    let dog_draw = WarriorDraw::new(&ggame.dogs.warriors[i], &dog, &drop_shadow);

                    disable_depth(&ctx, || {
                        //draw dropshadow
                        cat_draw.draw_shadow(&grid_matrix, &mut draw_sys, &matrix);
                        dog_draw.draw_shadow(&grid_matrix, &mut draw_sys, &matrix);

                        //TODO finish this!!!!
                        // if let ace::Command::Animate(a) = &command {
                        //     let (pos,ty) = a.calc_pos();
                        //     let t = matrix::translation(pos[0], pos[1], 1.0);

                        //     let m = matrix.chain(t).generate();

                        //     let mut v = draw_sys.view(m.as_ref());
                        //     drop_shadow.draw(&mut v);
                        // }
                    });
                }

                for i in 0..4 {
                    let cat_draw = WarriorDraw::new(&ggame.cats.warriors[i], &cat, &drop_shadow);
                    let dog_draw = WarriorDraw::new(&ggame.dogs.warriors[i], &dog, &drop_shadow);
                    cat_draw.draw(&grid_matrix, &mut draw_sys, &matrix);
                    dog_draw.draw(&grid_matrix, &mut draw_sys, &matrix);
                }

                for i in 0..4 {
                    let cat_draw = WarriorDraw::new(&ggame.cats.warriors[i], &cat, &drop_shadow);
                    let dog_draw = WarriorDraw::new(&ggame.dogs.warriors[i], &dog, &drop_shadow);
                    disable_depth(&ctx, || {
                        cat_draw.draw_health_text(
                            &grid_matrix,
                            &health_numbers,
                            &view_proj,
                            &proj,
                            &mut draw_sys,
                        );
                        dog_draw.draw_health_text(
                            &grid_matrix,
                            &health_numbers,
                            &view_proj,
                            &proj,
                            &mut draw_sys,
                        );
                    });
                }

                if let ace::Command::Animate(a) = &command {
                    let this_draw = if team == 0 { &cat } else { &dog };
                    let that_draw = if team == 1 { &cat } else { &dog };

                    let (pos, ty) = a.calc_pos();

                    let (a, b) = match ty {
                        AnimationOptions::Heal([a, b]) => ((this_draw, a), Some((this_draw, b))),
                        AnimationOptions::Movement(m) => ((this_draw, m), None),
                        AnimationOptions::Attack([a, b]) => ((this_draw, a), Some((that_draw, b))),
                        AnimationOptions::CounterAttack([a, b]) => {
                            ((that_draw, b), Some((this_draw, a)))
                        }
                    };

                    disable_depth(&ctx, || {
                        let t = matrix::translation(pos[0], pos[1], 1.0);

                        let m = matrix.chain(t).generate();

                        let mut v = draw_sys.view(m.as_ref());
                        drop_shadow.draw(&mut v);

                        if let Some((_, b)) = b {
                            let pos: [f32; 2] = grid_matrix.hex_axial_to_world(&b.position).into();
                            let t = matrix::translation(pos[0], pos[1], 1.0);

                            let m = matrix.chain(t).generate();

                            let mut v = draw_sys.view(m.as_ref());
                            drop_shadow.draw(&mut v);
                        }
                    });

                    let t = matrix::translation(pos[0], pos[1], 0.0);
                    let s = matrix::scale(1.0, 1.0, 1.0);
                    let m = matrix.chain(t).chain(s).generate();
                    let mut v = draw_sys.view(m.as_ref());
                    a.0.draw(&mut v);

                    if let Some((a, b)) = b {
                        let pos: [f32; 2] = grid_matrix.hex_axial_to_world(&b.position).into();

                        let t = matrix::translation(pos[0], pos[1], 0.0);
                        let s = matrix::scale(1.0, 1.0, 1.0);
                        let m = matrix.chain(t).chain(s).generate();
                        let mut v = draw_sys.view(m.as_ref());
                        a.draw(&mut v);
                    }
                }

                ctx.flush();
            }
        }
    };

    //futures::pin_mut!(main_logic);
    //futures::pin_mut!(render_thread);

    futures::join!(main_logic, render_thread);

    w.post_message(UiButton::NoUi);

    log!("worker thread closing");
}

fn disable_depth(ctx: &WebGl2RenderingContext, func: impl FnOnce()) {
    ctx.disable(WebGl2RenderingContext::DEPTH_TEST);
    ctx.disable(WebGl2RenderingContext::CULL_FACE);

    func();

    ctx.enable(WebGl2RenderingContext::DEPTH_TEST);
    ctx.enable(WebGl2RenderingContext::CULL_FACE);
}

//TODO just use reference???
fn string_to_coords<'a>(st: &str) -> model::ModelData {
    let num_rows = 16;
    let num_columns = 16;

    let mut tex_coords = vec![];
    let mut counter = 0.0;
    let dd = 20.0;
    let mut positions = vec![];

    let mut inds = vec![];
    for (_, a) in st.chars().enumerate() {
        let ascii = a as u8;
        let index = (ascii - 0/*32*/) as u16;

        //log!(format!("aaaa:{:?}",index));
        let x = (index % num_rows) as f32 / num_rows as f32;
        let y = (index / num_rows) as f32 / num_columns as f32;

        let x1 = x;
        let x2 = x1 + 1.0 / num_rows as f32;

        let y1 = y;
        let y2 = y + 1.0 / num_columns as f32;

        let a = [[x1, y1], [x2, y1], [x1, y2], [x2, y2]];

        tex_coords.extend(a);

        let iii = [0u16, 1, 2, 2, 1, 3].map(|a| positions.len() as u16 + a);

        let xx1 = counter;
        let xx2 = counter + dd;
        let yy1 = dd;
        let yy2 = 0.0;

        let zz = 0.0;
        let y = [
            [xx1, yy1, zz],
            [xx2, yy1, zz],
            [xx1, yy2, zz],
            [xx2, yy2, zz],
        ];

        positions.extend(y);

        inds.extend(iii);

        assert!(ascii >= 32);
        counter += dd;
    }

    let normals = positions.iter().map(|_| [0.0, 0.0, 1.0]).collect();

    let cc = 1.0 / dd;
    let mm = matrix::scale(cc, cc, cc).generate();

    let positions = positions
        .into_iter()
        .map(|a| mm.transform_point(a.into()).into())
        .collect();

    model::ModelData {
        positions,
        tex_coords,
        indices: Some(inds),
        normals,
        matrix: mm,
    }
}

use web_sys::WebGl2RenderingContext;

use crate::ace::{MousePrompt, Pototo};
//use crate::gameplay::GameStepper;
use crate::movement::MoveUnit;
use crate::terrain::MoveCost;

const SELECT_GLB: &'static [u8] = include_bytes!("../assets/select_model.glb");
const DROP_SHADOW_GLB: &'static [u8] = include_bytes!("../assets/drop_shadow.glb");
const ROAD_GLB: &'static [u8] = include_bytes!("../assets/road.glb");
const ATTACK_GLB: &'static [u8] = include_bytes!("../assets/attack.glb");
const FRIENDLY_GLB: &'static [u8] = include_bytes!("../assets/friendly-select.glb");

// const SHADED_GLB: &'static [u8] = include_bytes!("../assets/shaded.glb");
// const KEY_GLB: &'static [u8] = include_bytes!("../assets/key.glb");
// const PERSON_GLB: &'static [u8] = include_bytes!("../assets/person-v1.glb");
const CAT_GLB: &'static [u8] = include_bytes!("../assets/donut.glb");
const DOG_GLB: &'static [u8] = include_bytes!("../assets/cat_final.glb");

const GRASS_GLB: &'static [u8] = include_bytes!("../assets/hex-grass.glb");

pub struct NumberTextManager<'a> {
    numbers: Vec<model_parse::ModelGpu>,
    texture: &'a model_parse::TextureGpu,
}
impl<'a> NumberTextManager<'a> {
    fn new(ctx: &WebGl2RenderingContext, texture: &'a model_parse::TextureGpu) -> Self {
        let range = -10..=10;
        fn generate_number(number: i8, ctx: &WebGl2RenderingContext) -> model_parse::ModelGpu {
            let data = string_to_coords(&format!("{}", number));
            model_parse::ModelGpu::new(ctx, &data)
        }

        let numbers = range.into_iter().map(|i| generate_number(i, ctx)).collect();
        Self { numbers, texture }
    }

    fn get_number(
        &self,
        num: i8,
    ) -> model_parse::Foo<&model_parse::TextureGpu, &model_parse::ModelGpu> {
        let gpu = &self.numbers[(num + 10) as usize];

        model_parse::Foo {
            texture: &self.texture,
            model: gpu,
        }
    }
}
