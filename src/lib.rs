use std::collections::BTreeSet;

//use ace::AnimationOptions;
use cgmath::{InnerSpace, Matrix4, Transform, Vector2};
use gloo::console::console;
use gloo::console::console_dbg;

use futures::{SinkExt, StreamExt};
use gloo::console::log;
use model::matrix::{self, MyMatrix};
use movement::bitfield::BitField;
use movement::GridCoord;
use serde::{Deserialize, Serialize};
use shogo::simple2d::{self, ShaderSystem};
use shogo::utils;
use wasm_bindgen::prelude::*;
pub mod animation;
pub mod dom;
pub mod moves;
//pub mod gameplay;
pub mod board;
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
    direction: &'a MyModel,
    col: &'a [UnitData],
}

impl<'a> WarriorDraw<'a> {
    fn new(
        col: &'a [UnitData],
        model: &'a MyModel,
        drop_shadow: &'a MyModel,
        direction: &'a MyModel,
    ) -> Self {
        Self {
            model,
            drop_shadow,
            col,
            direction,
        }
    }
    fn draw(&self, gg: &grids::GridMatrix, draw_sys: &mut ShaderSystem, matrix: &Matrix4<f32>) {
        //let grey = self.typ == Type::Para;
        //TODO don't loop in this function!!!
        for cc in self.col.iter().filter(|a| a.typ != Type::Archer) {
            let pos = gg.hex_axial_to_world(&cc.position);

            // let pos: [f32; 2] = gg.to_world_topleft(cc.position.0.into()).into();

            let t = matrix::translation(pos[0], pos[1], 0.0);
            //let s = matrix::scale(1.0, 1.0, 1.0);

            //let r=rotate_by_dir(cc.direction,gg.spacing());

            let m = matrix.chain(t.clone()).generate();
            let mut v = draw_sys.view(m.as_ref());

            self.model.draw_ext(
                &mut v, false, /*  !cc.selectable(game)  */
                false, false, true,
            );

            let m = matrix
                .chain(t)
                //.chain(rotate_by_dir(cc.direction, gg.spacing()))
                .generate();
            let mut v = draw_sys.view(m.as_ref());

            // self.direction.draw_ext(
            //     &mut v, false, /*  !cc.selectable(game)  */
            //     false, false, true,
            // );
        }
    }

    fn draw_shadow(
        &self,
        gg: &grids::GridMatrix,
        draw_sys: &mut ShaderSystem,
        matrix: &Matrix4<f32>,
    ) {
        for a in self
            .col
            .iter()
            .filter(|a| a.typ != Type::Archer)
            .map(|a| &a.position)
        {
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
        for ccat in self.col.iter().filter(|a| a.typ != Type::Archer) {
            let pos: [f32; 2] = gg.hex_axial_to_world(&ccat.position).into();

            let t = matrix::translation(pos[0], pos[1] + 20.0, 20.0);

            let jj = view_proj.chain(t).generate();
            let jj: &[f32; 16] = jj.as_ref();
            let tt = matrix::translation(jj[12], jj[13], jj[14]);
            let new_proj = proj.clone().chain(tt);

            let s = matrix::scale(5.0, 5.0, 5.0);
            let m = new_proj.chain(s).generate();

            let nn = health_numbers.get_number(ccat.typ.type_index() as i8);
            let mut v = draw_sys.view(m.as_ref());
            nn.draw_ext(&mut v, false, false, true, false);

            //nn.draw(ccat.health,&ctx,&text_texture,&mut draw_sys,&m);
        }
    }
}

type MyModel = model_parse::Foo<model_parse::TextureGpu, model_parse::ModelGpu>;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Factions {
    pub dogs: Tribe,
    pub cats: Tribe,
}
impl Factions {
    fn relative_mut(&mut self, team: ActiveTeam) -> FactionRelative<&mut Tribe> {
        match team {
            ActiveTeam::Cats => FactionRelative {
                this_team: &mut self.cats,
                that_team: &mut self.dogs,
            },
            ActiveTeam::Dogs => FactionRelative {
                this_team: &mut self.dogs,
                that_team: &mut self.cats,
            },
        }
    }
    fn relative(&self, team: ActiveTeam) -> FactionRelative<&Tribe> {
        match team {
            ActiveTeam::Cats => FactionRelative {
                this_team: &self.cats,
                that_team: &self.dogs,
            },
            ActiveTeam::Dogs => FactionRelative {
                this_team: &self.dogs,
                that_team: &self.cats,
            },
        }
    }
}
pub struct FactionRelative<T> {
    pub this_team: T,
    pub that_team: T,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Environment {
    land: BitField,
    forest: BitField,
    powerup: BitField,
    world: board::World,
}

//Additionally removes need to special case animation.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct GameState {
    factions: Factions,
    env: Environment,
}

#[wasm_bindgen]
pub async fn worker_entry() {
    console_error_panic_hook::set_once();

    console_dbg!("num tiles={}", hex::Cube::new(0, 0).range(4).count());

    let (mut w, ss) = shogo::EngineWorker::new().await;
    let mut frame_timer = shogo::FrameTimer::new(60, ss);

    let canvas = w.canvas();
    let ctx = simple2d::ctx_wrap(&utils::get_context_webgl2_offscreen(&canvas));

    let mut draw_sys = ctx.shader_system();
    let mut counter: f32 = 0.0;
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

    let cats: smallvec::SmallVec<[UnitData; 6]> = smallvec::smallvec![
        //UnitData::new(GridCoord([-4, 4]), Type::King, HexDir { dir: 5 }),
        // UnitData::new(
        //     GridCoord([2, -1]),
        //     Type::Spotter { clockwise: true },
        //     HexDir { dir: 2 }
        // ),
        // UnitData::new(
        //     GridCoord([-3, -1]),
        //     Type::Spotter { clockwise: false },
        //     HexDir { dir: 2 }
        // ),
        //UnitData::new(GridCoord([-2, 1]), Type::Archer, HexDir { dir: 5 }),
        // UnitData::new(GridCoord([-3, 1]), Type::Archer, HexDir { dir: 5 }),
        UnitData::new(GridCoord([-3, 3]), Type::Foot),
        UnitData::new(GridCoord([-3, 2]), Type::Ship),
        UnitData::new(GridCoord([-2, 3]), Type::Ship),
    ];

    //player
    let dogs = smallvec::smallvec![
        //UnitData::new(GridCoord([4, -4]), Type::King, HexDir { dir: 2 }),
        // UnitData::new(
        //     GridCoord([1, -2]),
        //     Type::Spotter { clockwise: true },
        //     HexDir { dir: 2 }
        // ),
        // UnitData::new(
        //     GridCoord([2, -2]),
        //     Type::Spotter { clockwise: false },
        //     HexDir { dir: 2 }
        // ),
        UnitData::new(GridCoord([3, -3]), Type::Foot),
        UnitData::new(GridCoord([2, -3]), Type::Ship),
        UnitData::new(GridCoord([3, -2]), Type::Ship),
        // UnitData::new(GridCoord([1, -2]), Type::Rook, HexDir { dir: 2 }),
        // UnitData::new(GridCoord([1, -3]), Type::Rook, HexDir { dir: 2 }),
        // UnitData::new(GridCoord([1, -3]), Type::Warrior, HexDir { dir: 2 }),
        // UnitData::new(GridCoord([3, -1]), Type::Warrior, HexDir { dir: 2 }),
    ];

    let mut ggame = GameState {
        factions: Factions {
            dogs: Tribe { units: dogs },
            cats: Tribe { units: cats },
        },
        env: Environment {
            land: BitField::from_iter([GridCoord([3, -3]), GridCoord([-3, 3])]),
            forest: BitField::from_iter([]),
            // powerup: vec![
            //     GridCoord([0, -3]),
            //     GridCoord([3, 0]),
            //     GridCoord([-3, 0]),
            //     GridCoord([0, 3]),
            // ],
            powerup: BitField::from_iter([]),
            world: board::World::new(),
        },
    };

    let _roads = terrain::TerrainCollection {
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

    let mountain = quick_load(MOUNTAIN_GLB, 1, None);

    let water = quick_load(WATER_GLB, RESIZE, None);

    let grass = quick_load(GRASS_GLB, RESIZE, None);

    let select_model = quick_load(SELECT_GLB, 1, None);

    let attack_model = quick_load(ATTACK_GLB, 1, None);

    //let arrow_model = quick_load(ARROW_GLB, 1, None);

    let direction_model = quick_load(DIRECTION_GLB, 1, None);

    //let friendly_model = quick_load(FRIENDLY_GLB, 1, None);

    let text_texture = {
        let ascii_tex = model::load_texture_from_data(include_bytes!("../assets/ascii5.png"));

        model_parse::TextureGpu::new(&ctx, &ascii_tex)
    };

    let health_numbers = NumberTextManager::new(&ctx, &text_texture);

    let (command_sender, mut command_recv) = futures::channel::mpsc::channel(5);
    let (mut response_sender, response_recv) = futures::channel::mpsc::channel(5);

    let main_logic = async {
        ace::main_logic(command_sender, response_recv, &mut ggame).await;
    };

    let mut mouse_mouse = [0.0; 2];
    let render_thread = async {
        while let Some(ace::GameWrap {
            game: ggame,
            data: command,
            team,
        }) = command_recv.next().await
        {
            let mut command = command.process(&grid_matrix);
            //let game_view = ggame.view(team);

            let (cat_for_draw, dog_for_draw) = {
                let (this, that) = if let ace::ProcessedCommand::Animate(a) = &command {
                    match a.data() {
                        animation::AnimationCommand::Movement { unit, .. } => {
                            let a: Vec<_> = ggame
                                .factions
                                .relative(team)
                                .this_team
                                .units
                                .iter()
                                .cloned()
                                .filter(|a| a.position != unit.position)
                                .collect();
                            let b: Vec<_> = ggame
                                .factions
                                .relative(team)
                                .that_team
                                .units
                                .iter()
                                .cloned()
                                .collect();
                            (a, b)
                        }
                        animation::AnimationCommand::Attack { attacker, defender } => {
                            let a = ggame
                                .factions
                                .relative(team)
                                .this_team
                                .units
                                .iter()
                                .cloned()
                                .filter(|k| k.position != attacker.position)
                                .collect();
                            let b = ggame
                                .factions
                                .relative(team)
                                .that_team
                                .units
                                .iter()
                                .cloned()
                                .filter(|k| k.position != defender.position)
                                .collect();
                            (a, b)
                        }
                    }
                } else {
                    let a = ggame
                        .factions
                        .relative(team)
                        .this_team
                        .units
                        .iter()
                        .cloned()
                        .collect();
                    let b = ggame
                        .factions
                        .relative(team)
                        .that_team
                        .units
                        .iter()
                        .cloned()
                        .collect();
                    (a, b)
                };

                if team == ActiveTeam::Cats {
                    (this, that)
                } else {
                    (that, this)
                }
            };

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
                    ace::ProcessedCommand::Animate(a) => {
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
                    ace::ProcessedCommand::GetMouseInput(_) => {
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
                    ace::ProcessedCommand::Nothing => {}
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

                //TODO don't render where land is?
                for c in ggame.env.world.iter_cells() {
                    let pos = grid_matrix.hex_axial_to_world(&c.to_axial());

                    //let pos = a.calc_pos();
                    let t = matrix::translation(pos[0], pos[1], -10.0);
                    let s = matrix::scale(1.0, 1.0, 1.0);
                    let m = matrix.chain(t).chain(s).generate();
                    let mut v = draw_sys.view(m.as_ref());

                    water.draw(&mut v);
                }

                for c in ggame.env.powerup.iter_mesh(GridCoord([0; 2])) {
                    let pos = grid_matrix.hex_axial_to_world(&c);

                    //let pos = a.calc_pos();
                    let t = matrix::translation(pos[0], pos[1], -10.0);
                    let s = matrix::scale(1.0, 1.0, 1.0);
                    let m = matrix.chain(t).chain(s).generate();
                    let mut v = draw_sys.view(m.as_ref());

                    attack_model.draw(&mut v);
                }

                for c in ggame.env.land.iter_mesh(GridCoord([0; 2])) {
                    let pos = grid_matrix.hex_axial_to_world(&c);

                    //let pos = a.calc_pos();
                    let t = matrix::translation(pos[0], pos[1], -10.0);
                    let s = matrix::scale(1.0, 1.0, 1.0);
                    let m = matrix.chain(t).chain(s).generate();
                    let mut v = draw_sys.view(m.as_ref());

                    grass.draw(&mut v);
                }

                for c in ggame.env.forest.iter_mesh(GridCoord([0; 2])) {
                    let pos = grid_matrix.hex_axial_to_world(&c);

                    //let pos = a.calc_pos();
                    let t = matrix::translation(pos[0], pos[1], 0.0);
                    let s = matrix::scale(1.0, 1.0, 1.0);
                    let m = matrix.chain(t).chain(s).generate();
                    let mut v = draw_sys.view(m.as_ref());

                    mountain.draw(&mut v);
                }
                disable_depth(&ctx, || {
                    if let ace::ProcessedCommand::GetMouseInput(a) = &command {
                        let (a, &greyscale) = match a {
                            MousePrompt::Selection { selection, grey } => (selection, grey),
                            MousePrompt::None => return,
                        };

                        //if let Some(a) = testo.get_selection() {
                        match a {
                            CellSelection::MoveSelection(point, mesh) => {
                                for a in mesh.iter_mesh(*point) {
                                    let pos: [f32; 2] = grid_matrix.hex_axial_to_world(&a).into();
                                    let t = matrix::translation(pos[0], pos[1], 0.0);

                                    let m = matrix.chain(t).generate();

                                    let mut v = draw_sys.view(m.as_ref());

                                    select_model.draw_ext(&mut v, greyscale, false, false, false);

                                    //select_model.draw(&mut v);
                                }
                                // for a in mesh.iter_attackable_normal(*point) {
                                //     let pos: [f32; 2] = grid_matrix.hex_axial_to_world(&a).into();
                                //     let t = matrix::translation(pos[0], pos[1], 0.0);

                                //     let m = matrix.chain(t).generate();

                                //     let mut v = draw_sys.view(m.as_ref());

                                //     attack_model.draw_ext(&mut v, greyscale, false, false, false);

                                //     //select_model.draw(&mut v);
                                // }

                                // counter += 0.02;
                                // for (dir, a) in mesh.iter_swing_mesh(*point) {
                                //     let pos: [f32; 2] = grid_matrix.hex_axial_to_world(&a).into();
                                //     let t = matrix::translation(pos[0], pos[1], 0.0);

                                //     let r = rotate_by_dir(dir, grid_matrix.spacing());

                                //     let m = matrix.chain(t).chain(r).generate();

                                //     let mut v = draw_sys.view(m.as_ref());

                                //     arrow_model.draw_ext(&mut v, greyscale, false, false, false);

                                //     //select_model.draw(&mut v);
                                // }
                            }
                            CellSelection::BuildSelection(_) => {}
                        }
                    }

                    //for a in a.iter_coords() {

                    //select_model.draw(&mut v);
                    //}
                });

                {
                    let cat_draw =
                        WarriorDraw::new(&cat_for_draw, &cat, &drop_shadow, &direction_model);
                    let dog_draw =
                        WarriorDraw::new(&dog_for_draw, &dog, &drop_shadow, &direction_model);

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

                {
                    //TODO loop instead
                    let cat_draw =
                        WarriorDraw::new(&cat_for_draw, &cat, &drop_shadow, &direction_model);
                    let dog_draw =
                        WarriorDraw::new(&dog_for_draw, &dog, &drop_shadow, &direction_model);
                    cat_draw.draw(&grid_matrix, &mut draw_sys, &matrix);
                    dog_draw.draw(&grid_matrix, &mut draw_sys, &matrix);
                }

                {
                    let cat_draw =
                        WarriorDraw::new(&cat_for_draw, &cat, &drop_shadow, &direction_model);
                    let dog_draw =
                        WarriorDraw::new(&dog_for_draw, &dog, &drop_shadow, &direction_model);
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

                if let ace::ProcessedCommand::Animate(a) = &command {
                    let (this_draw, that_draw) = match team {
                        ActiveTeam::Cats => (&cat, &dog),
                        ActiveTeam::Dogs => (&dog, &cat),
                    };

                    let (pos, ty) = a.calc_pos();

                    let (a, b) = match ty {
                        animation::AnimationCommand::Movement { unit, .. } => {
                            ((this_draw, unit), None)
                        }
                        animation::AnimationCommand::Attack { attacker, defender } => {
                            ((this_draw, attacker), Some((that_draw, defender)))
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

    log!("Worker thread closin");
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

use crate::ace::{ActiveTeam, MousePrompt, Pototo};
use crate::movement::HexDir;
//use crate::gameplay::GameStepper;
use crate::movement::MoveUnit;
use crate::terrain::MoveCost;

const SELECT_GLB: &'static [u8] = include_bytes!("../assets/select_model.glb");
const DROP_SHADOW_GLB: &'static [u8] = include_bytes!("../assets/drop_shadow.glb");
//const ROAD_GLB: &'static [u8] = include_bytes!("../assets/road.glb");
const MOUNTAIN_GLB: &'static [u8] = include_bytes!("../assets/mountain.glb");

const ATTACK_GLB: &'static [u8] = include_bytes!("../assets/attack.glb");

const ARROW_GLB: &'static [u8] = include_bytes!("../assets/arrow.glb");

//const FRIENDLY_GLB: &'static [u8] = include_bytes!("../assets/friendly-select.glb");

// const SHADED_GLB: &'static [u8] = include_bytes!("../assets/shaded.glb");
// const KEY_GLB: &'static [u8] = include_bytes!("../assets/key.glb");
// const PERSON_GLB: &'static [u8] = include_bytes!("../assets/person-v1.glb");
const CAT_GLB: &'static [u8] = include_bytes!("../assets/donut.glb");
const DOG_GLB: &'static [u8] = include_bytes!("../assets/cat_final.glb");

const GRASS_GLB: &'static [u8] = include_bytes!("../assets/hex-grass.glb");
const WATER_GLB: &'static [u8] = include_bytes!("../assets/water.glb");

const DIRECTION_GLB: &'static [u8] = include_bytes!("../assets/direction.glb");

pub struct NumberTextManager<'a> {
    pub numbers: Vec<model_parse::ModelGpu>,
    pub texture: &'a model_parse::TextureGpu,
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

    pub fn get_number(
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

fn rotate_by_dir(dir: HexDir, spacing: f32) -> impl MyMatrix {
    use matrix::Inverse;
    let mm = ((dir.dir + 1) % 6) as f32 / 6.0;

    let zrot = matrix::z_rotation(-mm * (std::f32::consts::TAU));
    let tt = matrix::translation(-spacing / 2.0, -spacing / 2.0, 0.0);
    let tt2 = tt.clone().inverse();

    let r = tt2.chain(zrot).chain(tt);
    r
}
