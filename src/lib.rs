use ace::GameWrapResponse;
//use ace::AnimationOptions;
use cgmath::{InnerSpace, Matrix4, Transform, Vector2};
use gloo::console::console_dbg;

use futures::{SinkExt, StreamExt};
use gloo::console::log;
use model::matrix::{self, MyMatrix};
use movement::bitfield::BitField;
use movement::GridCoord;
use serde::{Deserialize, Serialize};
use shogo::simple2d::{self, CtxWrap, ShaderSystem};
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

#[derive(Serialize, Deserialize, Debug, Clone)]
enum UiButton {
    ShowPopup(String),
    HidePopup,
}

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
pub struct Land {
    pub grass: BitField,
    pub snow: BitField,
}
impl Land {
    pub fn set_coord_false(&mut self, a: GridCoord) {
        if self.grass.is_coord_set(a) {
            self.grass.set_coord(a, false);
        } else if self.snow.is_coord_set(a) {
            self.snow.set_coord(a, false);
        } else {
            panic!("Invalid coord");
        }
    }
    pub fn is_coord_set(&self, a: GridCoord) -> bool {
        self.grass.is_coord_set(a) || self.snow.is_coord_set(a)
    }
}
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Environment {
    land: Land,
    forest: BitField,
    //powerup: BitField,
}

//Additionally removes need to special case animation.
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct GameState {
    factions: Factions,
    env: Environment,
    world: &'static board::MyWorld,
}

#[derive(Debug, Copy, Clone)]
pub enum GameOver {
    CatWon,
    DogWon,
    Tie,
}

impl GameState {
    pub fn game_is_over(&self) -> Option<GameOver> {
        let dog_stuck = 'foo: {
            for unit in self.factions.dogs.units.iter() {
                let mesh = self.generate_unit_possible_moves_inner(
                    &unit.position,
                    unit.typ,
                    ActiveTeam::Dogs,
                    None,
                );
                if !mesh.is_empty() {
                    break 'foo false;
                }
            }
            true
        };

        //dog can't move.

        let cat_stuck = 'foo: {
            for unit in self.factions.cats.units.iter() {
                let mesh = self.generate_unit_possible_moves_inner(
                    &unit.position,
                    unit.typ,
                    ActiveTeam::Cats,
                    None,
                );
                if !mesh.is_empty() {
                    break 'foo false;
                }
            }
            true
        };
        //cats can't move.

        match (dog_stuck, cat_stuck) {
            (true, true) | (false, false) => None,
            (true, false) => Some(GameOver::CatWon),
            (false, true) => Some(GameOver::DogWon),
        }
    }
}
#[wasm_bindgen]
pub async fn worker_entry() {
    console_error_panic_hook::set_once();

    console_dbg!("num tiles={}", hex::Cube::new(0, 0).range(4).count());

    let (mut wr, ss) = shogo::EngineWorker::new().await;
    let mut frame_timer = shogo::FrameTimer::new(60, ss);

    let render = EngineStuff::new(wr.canvas());

    loop {
        let sample_game = ace::share::load(ace::share::SAMPLE_GAME);

        let mut game = ace::game_init();

        let (game, mut r, w) = create_worker_render(&mut game);

        futures::join!(
            ace::main_logic(game, w),
            render.handle_render_loop(&mut r, &mut frame_timer, &mut wr)
        );
    }

    log!("Worker thread closin");
}

fn create_worker_render(
    game: &mut GameState,
) -> (&mut GameState, RenderManager, ace::WorkerManager) {
    let (command_sender, command_recv) = futures::channel::mpsc::channel(5);
    let (response_sender, response_recv) = futures::channel::mpsc::channel(5);
    let doop = ace::WorkerManager {
        game: game as *mut _,
        sender: command_sender,
        receiver: response_recv,
    };

    (
        game,
        RenderManager {
            response_sender,
            command_recv,
        },
        doop,
    )
}

struct RenderManager<'c> {
    response_sender: futures::channel::mpsc::Sender<GameWrapResponse<'c, ace::Response>>,
    command_recv: futures::channel::mpsc::Receiver<ace::GameWrap<'c, ace::Command>>,
}
pub struct EngineStuff {
    grid_matrix: grids::GridMatrix,
    models: Models<Foo<TextureGpu, ModelGpu>>,
    numm: Numm,
    ctx: CtxWrap,
    canvas: OffscreenCanvas,
}
impl EngineStuff {
    fn new(canvas: OffscreenCanvas) -> Self {
        let ctx = simple2d::ctx_wrap(&utils::get_context_webgl2_offscreen(&canvas));
        ctx.setup_alpha();

        let grid_matrix = grids::GridMatrix::new();
        let models = Models::new(&grid_matrix, &ctx);
        let numm = Numm::new(&ctx);

        EngineStuff {
            grid_matrix,
            models,
            numm,
            ctx,
            canvas,
        }
    }

    async fn handle_render_loop(
        &self,
        rm: &mut RenderManager<'_>,
        frame_timer: &mut shogo::FrameTimer<
            MEvent,
            futures::channel::mpsc::UnboundedReceiver<MEvent>,
        >,
        engine_worker: &mut shogo::EngineWorker<MEvent, UiButton>,
    ) {
        let e = self;
        let response_sender = &mut rm.response_sender;
        let command_recv = &mut rm.command_recv;
        let ctx = &e.ctx;
        let canvas = &e.canvas;
        let grid_matrix = &e.grid_matrix;
        let models = &e.models;
        let numm = &e.numm;

        let mut draw_sys = ctx.shader_system();

        let gl_width = canvas.width();
        let gl_height = canvas.height();
        ctx.viewport(0, 0, gl_width as i32, gl_height as i32);
        let mut viewport = [canvas.width() as f32, canvas.height() as f32];

        let mut scroll_manager = scroll::TouchController::new([0., 0.].into());

        use cgmath::SquareMatrix;
        let mut last_matrix = cgmath::Matrix4::identity();

        let drop_shadow = &models.drop_shadow;
        let dog = &models.dog;
        let cat = &models.cat;
        let mountain = &models.mountain;
        let water = &models.water;
        let grass = &models.grass;
        let snow = &models.snow;
        let select_model = &models.select_model;

        while let Some(ace::GameWrap { game, data, team }) = command_recv.next().await {
            //First lets process the command. Break it down
            //into pieces that this thread understands.
            let mut get_mouse_input = None;
            let mut unit_animation = None;
            let mut terrain_animation = None;
            let mut poking = 0;

            let (mut cat_for_draw, mut dog_for_draw) = (
                game.factions.cats.units.clone().into_vec(),
                game.factions.dogs.units.clone().into_vec(),
            );
            match data {
                ace::Command::Animate(ak) => match ak {
                    animation::AnimationCommand::Movement {
                        unit,
                        mesh,
                        walls,
                        end,
                    } => {
                        if team == ActiveTeam::Cats {
                            cat_for_draw.retain(|k| k.position != unit.position);
                        } else {
                            dog_for_draw.retain(|k| k.position != unit.position);
                        }
                        let it = animation::movement(unit.position, mesh, walls, end, grid_matrix);
                        
                        unit_animation = Some((Vector2::new(0.0, 0.0), it,unit));
                    }
                    animation::AnimationCommand::Terrain { pos, terrain_type } => {
                        let it = animation::terrain_create();
                        terrain_animation = Some((0.0, it,pos,terrain_type));
                    }
                },
                ace::Command::GetMouseInput(kk) => {
                    get_mouse_input = Some(kk);
                }
                ace::Command::Nothing => {}
                ace::Command::Popup(str) => {
                    if str.is_empty() {
                        engine_worker.post_message(UiButton::HidePopup);
                    } else {
                        engine_worker.post_message(UiButton::ShowPopup(str));
                    }

                    response_sender
                        .send(ace::GameWrapResponse {
                            game,
                            data: ace::Response::Ack,
                        })
                        .await
                        .unwrap();
                    continue;
                }
                ace::Command::Poke => {
                    poking = 3;
                }
            };

            'render_loop: loop {
                if poking == 1 {
                    console_dbg!("we poked!");
                    response_sender
                        .send(ace::GameWrapResponse {
                            game,
                            data: ace::Response::Ack,
                        })
                        .await
                        .unwrap();
                    break 'render_loop;
                }
                poking = 0.max(poking - 1);

                let mut on_select = false;
                let mut end_turn = false;

                let res = frame_timer.next().await;

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
                            scroll_manager.on_new_touch(touches);
                        }
                        MEvent::TouchEnd { touches } => {
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
                            scroll_manager.on_mouse_move([*x, *y], &last_matrix, viewport);
                        }
                        MEvent::EndTurn => {
                            end_turn = true;
                        }
                        MEvent::CanvasMouseDown { x, y } => {
                            scroll_manager.on_mouse_down([*x, *y]);
                        }
                        MEvent::ButtonClick => {}
                        MEvent::ShutdownClick => break 'render_loop,
                    }
                }

                let proj = projection::projection(viewport).generate();
                let view_proj = projection::view_matrix(
                    scroll_manager.camera(),
                    scroll_manager.zoom(),
                    scroll_manager.rot(),
                );

                let my_matrix = proj.chain(view_proj).generate();

                last_matrix = my_matrix;

                let mouse_world =
                    scroll::mouse_to_world(scroll_manager.cursor_canvas(), &my_matrix, viewport);

                if let Some(_) = &mut get_mouse_input {
                    if end_turn {
                        response_sender
                            .send(ace::GameWrapResponse {
                                game,
                                data: ace::Response::Mouse(
                                    get_mouse_input.take().unwrap(),
                                    Pototo::EndTurn,
                                ),
                            })
                            .await
                            .unwrap();
                        break 'render_loop;
                    } else if on_select {
                        let mouse: GridCoord = grid_matrix.center_world_to_hex(mouse_world.into());
                        log!(format!("pos:{:?}", mouse));

                        response_sender
                            .send(ace::GameWrapResponse {
                                game,
                                data: ace::Response::Mouse(
                                    get_mouse_input.take().unwrap(),
                                    Pototo::Normal(mouse),
                                ),
                            })
                            .await
                            .unwrap();
                        break 'render_loop;
                    }
                }

                if let Some((z, a,_,_)) = &mut terrain_animation {
                    if let Some(zpos) = a.next() {
                        *z = zpos;
                    } else {
                        response_sender
                            .send(ace::GameWrapResponse {
                                game,
                                data: ace::Response::AnimationFinish,
                            })
                            .await
                            .unwrap();
                        break 'render_loop;
                    }
                }
                if let Some((lpos, a,_)) = &mut unit_animation {
                    if let Some(pos) = a.next() {
                        *lpos = pos;
                    } else {
                        response_sender
                            .send(ace::GameWrapResponse {
                                game,
                                data: ace::Response::AnimationFinish,
                            })
                            .await
                            .unwrap();
                        break 'render_loop;
                    }
                }

                scroll_manager.step();

                let ggame = &game;

                ctx.draw_clear([0.0, 0.0, 0.0, 0.0]);

                draw_something_grid(
                    ggame.world.get_game_cells().iter_mesh(GridCoord::zero()),
                    grid_matrix,
                    &mut draw_sys,
                    &water,
                    &my_matrix,
                    -10.0,
                );

                pub const LAND_OFFSET: f32 = -10.0;
                pub const MOUNTAIN_OFFSET: f32 = 0.0;

                for c in ggame.env.land.snow.iter_mesh(GridCoord::zero()) {
                    let pos = grid_matrix.hex_axial_to_world(&c);
                    let t = matrix::translation(pos.x, pos.y, LAND_OFFSET);
                    let m = my_matrix.chain(t).generate();
                    draw_sys.view(&m).draw_a_thing(snow);
                }

                for c in ggame.env.land.grass.iter_mesh(GridCoord::zero()) {
                    let pos = grid_matrix.hex_axial_to_world(&c);
                    let t = matrix::translation(pos.x, pos.y, LAND_OFFSET);
                    let m = my_matrix.chain(t).generate();
                    draw_sys.view(&m).draw_a_thing(grass);
                }

                for c in ggame.env.forest.iter_mesh(GridCoord::zero()) {
                    let pos = grid_matrix.hex_axial_to_world(&c);

                    let t = matrix::translation(pos.x, pos.y, MOUNTAIN_OFFSET);
                    let m = my_matrix.chain(t).generate();
                    draw_sys.view(&m).draw_a_thing(mountain);
                }

                if let Some((zpos, a,gpos,k)) = &mut terrain_animation {
                    
                    let texture = match k {
                        animation::TerrainType::Snow => snow,
                        animation::TerrainType::Grass => grass,
                        animation::TerrainType::Mountain => mountain,
                    };

                    let diff = match k {
                        animation::TerrainType::Snow => LAND_OFFSET,
                        animation::TerrainType::Grass => LAND_OFFSET,
                        animation::TerrainType::Mountain => MOUNTAIN_OFFSET,
                    };

                    let gpos = *gpos;

                    let pos = grid_matrix.hex_axial_to_world(&gpos);

                    let t = matrix::translation(pos.x, pos.y, diff + *zpos);
                    let m = my_matrix.chain(t).generate();
                    draw_sys.view(&m).draw_a_thing(texture);
                }

                if let Some(a) = &get_mouse_input {
                    match a {
                        MousePrompt::Selection { selection, grey } => match selection {
                            CellSelection::MoveSelection(point, mesh) => {
                                let _d = DepthDisabler::new(&ctx);

                                for a in mesh.iter_mesh(*point) {
                                    let pos = grid_matrix.hex_axial_to_world(&a);
                                    let t = matrix::translation(pos.x, pos.y, 0.0);
                                    let m = my_matrix.chain(t).generate();

                                    draw_sys.view(&m).draw_a_thing_ext(
                                        select_model,
                                        *grey,
                                        false,
                                        false,
                                        false,
                                    );
                                }
                            }
                            CellSelection::BuildSelection(_) => {}
                        },
                        MousePrompt::None => {}
                    };
                }

                let d = DepthDisabler::new(&ctx);

                draw_something_grid(
                    cat_for_draw
                        .iter()
                        .map(|x| x.position)
                        .chain(dog_for_draw.iter().map(|x| x.position)),
                    grid_matrix,
                    &mut draw_sys,
                    drop_shadow,
                    &my_matrix,
                    1.0,
                );

                drop(d);

                draw_something_grid(
                    cat_for_draw.iter().map(|x| x.position),
                    grid_matrix,
                    &mut draw_sys,
                    &cat,
                    &my_matrix,
                    0.0,
                );

                draw_something_grid(
                    dog_for_draw.iter().map(|x| x.position),
                    grid_matrix,
                    &mut draw_sys,
                    &dog,
                    &my_matrix,
                    0.0,
                );

                if let Some((pos, a,unit)) = &mut unit_animation {
                    let this_draw = match team {
                        ActiveTeam::Cats => &cat,
                        ActiveTeam::Dogs => &dog,
                    };

                    //This is a unit animation
                    let a = (this_draw, unit);

                    let d = DepthDisabler::new(&ctx);

                    let m = my_matrix
                        .chain(matrix::translation(pos.x, pos.y, 1.0))
                        .generate();

                    draw_sys.view(&m).draw_a_thing(drop_shadow);
                    drop(d);

                    let m = my_matrix
                        .chain(matrix::translation(pos.x, pos.y, 0.0))
                        .chain(matrix::scale(1.0, 1.0, 1.0))
                        .generate();

                    draw_sys.view(&m).draw_a_thing(*a.0);
                }

                let d = DepthDisabler::new(&ctx);

                draw_health_text(
                    cat_for_draw
                        .iter()
                        .map(|x| (x.position, x.typ.type_index() as i8))
                        .chain(
                            dog_for_draw
                                .iter()
                                .map(|x| (x.position, x.typ.type_index() as i8)),
                        ),
                    &grid_matrix,
                    &numm.health_numbers,
                    &view_proj,
                    &proj,
                    &mut draw_sys,
                    &numm.text_texture,
                );
                drop(d);

                ctx.flush();
            }
        }
    }
}

fn draw_something_grid(
    f: impl IntoIterator<Item = GridCoord>,
    grid_matrix: &grids::GridMatrix,
    draw_sys: &mut ShaderSystem,
    texture: &Foo<TextureGpu, ModelGpu>,
    m: &Matrix4<f32>,
    height: f32,
) {
    for a in f.into_iter() {
        let pos = grid_matrix.hex_axial_to_world(&a);

        let m = m
            .chain(matrix::translation(pos.x, pos.y, height))
            .generate();

        draw_sys.view(&m).draw_a_thing(texture);
    }
}
fn draw_health_text(
    f: impl IntoIterator<Item = (GridCoord, i8)>,

    gg: &grids::GridMatrix,
    health_numbers: &NumberTextManager,
    view_proj: &Matrix4<f32>,
    proj: &Matrix4<f32>,
    draw_sys: &mut ShaderSystem,
    text_texture: &TextureGpu,
) {
    //draw text
    for (ccat, ii) in f {
        let pos = gg.hex_axial_to_world(&ccat);

        let t = matrix::translation(pos.x, pos.y + 20.0, 20.0);

        let jj = view_proj.chain(t).generate();
        let jj: &[f32; 16] = jj.as_ref();
        let tt = matrix::translation(jj[12], jj[13], jj[14]);
        let new_proj = proj.clone().chain(tt);

        let s = matrix::scale(5.0, 5.0, 5.0);
        let m = new_proj.chain(s).generate();

        let nn = health_numbers.get_number(ii, text_texture);
        draw_sys
            .view(&m)
            .draw_a_thing_ext(&nn, false, false, true, false);
    }
}

pub struct DepthDisabler<'a> {
    ctx: &'a WebGl2RenderingContext,
}
impl<'a> Drop for DepthDisabler<'a> {
    fn drop(&mut self) {
        self.ctx.enable(WebGl2RenderingContext::DEPTH_TEST);
        self.ctx.enable(WebGl2RenderingContext::CULL_FACE);
    }
}
impl<'a> DepthDisabler<'a> {
    pub fn new(ctx: &'a WebGl2RenderingContext) -> DepthDisabler<'a> {
        ctx.disable(WebGl2RenderingContext::DEPTH_TEST);
        ctx.disable(WebGl2RenderingContext::CULL_FACE);

        DepthDisabler { ctx }
    }
}


use web_sys::{OffscreenCanvas, WebGl2RenderingContext};

use crate::ace::{ActiveTeam, MousePrompt, Pototo};
use crate::model_parse::{Foo, ModelGpu, TextureGpu};
//use crate::gameplay::GameStepper;
use crate::movement::MoveUnit;

pub struct Models<T> {
    select_model: T,
    drop_shadow: T,
    mountain: T,
    attack: T,
    cat: T,
    dog: T,
    grass: T,
    snow: T,
    water: T,
    direction: T,
}

impl Models<Foo<TextureGpu, ModelGpu>> {
    pub fn new(grid_matrix: &grids::GridMatrix, ctx: &WebGl2RenderingContext) -> Self {
        const ASSETS: &[(&'static [u8], usize, Option<f64>)] = &[
            (include_bytes!("../assets/select_model.glb"), 1, None),
            (include_bytes!("../assets/drop_shadow.glb"), 1, Some(0.5)),
            (include_bytes!("../assets/mountain.glb"), RESIZE, None),
            (include_bytes!("../assets/attack.glb"), 1, None),
            (include_bytes!("../assets/donut.glb"), RESIZE, None),
            (include_bytes!("../assets/cat_final.glb"), RESIZE, None),
            (include_bytes!("../assets/hex-grass.glb"), RESIZE, None),
            (include_bytes!("../assets/snow.glb"), RESIZE, None),
            (include_bytes!("../assets/water.glb"), RESIZE, None),
            (include_bytes!("../assets/direction.glb"), 1, None),
        ];
        let quick_load = |name, res, alpha| {
            let (data, t) = model::load_glb(name).gen_ext(grid_matrix.spacing(), res, alpha);

            log!(format!("texture:{:?}", (t.width, t.height)));
            model_parse::Foo {
                texture: model_parse::TextureGpu::new(&ctx, &t),
                model: model_parse::ModelGpu::new(&ctx, &data),
            }
        };

        let qq = |num| {
            let (b, c, d) = ASSETS[num];
            quick_load(b, c, d)
        };

        Models {
            select_model: qq(0),
            drop_shadow: qq(1),
            mountain: qq(2),
            attack: qq(3),
            cat: qq(4),
            dog: qq(5),
            grass: qq(6),
            snow: qq(7),
            water: qq(8),
            direction: qq(9),
        }
    }
}

pub struct Numm {
    text_texture: TextureGpu,
    health_numbers: NumberTextManager,
}
impl Numm {
    pub fn new(ctx: &WebGl2RenderingContext) -> Self {
        let text_texture = {
            let ascii_tex = model::load_texture_from_data(include_bytes!("../assets/ascii5.png"));

            model_parse::TextureGpu::new(&ctx, &ascii_tex)
        };

        let health_numbers = NumberTextManager::new(&ctx);

        Numm {
            text_texture,
            health_numbers,
        }
    }
}

pub struct NumberTextManager {
    pub numbers: Vec<model_parse::ModelGpu>,
}
impl NumberTextManager {
    fn new(ctx: &WebGl2RenderingContext) -> Self {
        let range = -10..=10;
        fn generate_number(number: i8, ctx: &WebGl2RenderingContext) -> model_parse::ModelGpu {
            let data = string_to_coords(&format!("{}", number));
            model_parse::ModelGpu::new(ctx, &data)
        }

        let numbers = range.into_iter().map(|i| generate_number(i, ctx)).collect();
        Self { numbers }
    }

    pub fn get_number<'b>(
        &self,
        num: i8,
        texture: &'b model_parse::TextureGpu,
    ) -> model_parse::Foo<&'b model_parse::TextureGpu, &model_parse::ModelGpu> {
        let gpu = &self.numbers[(num + 10) as usize];

        model_parse::Foo {
            texture,
            model: gpu,
        }
    }
}
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
