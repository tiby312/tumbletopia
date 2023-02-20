use axgeom::vec2same;
use cgmath::{InnerSpace, Matrix4, Transform, Vector2};

use gloo::console::log;
use model::matrix::{self, MyMatrix};
use movement::GridCoord;
use serde::{Deserialize, Serialize};
use shogo::simple2d::{self, ShaderSystem};
use shogo::utils;
use wasm_bindgen::prelude::*;
pub mod animation;
pub mod dom;
pub mod gameplay;
pub mod grids;
pub mod model_parse;
pub mod movement;
pub mod projection;
pub mod scroll;
pub mod terrain;
pub mod util;
use dom::MEvent;
use projection::*;
pub mod state;
//pub mod logic;
pub const RESIZE: usize = 6;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
enum UiButton {
    ShowRoadUi,
    NoUi,
}

pub struct WarriorDraw<'a> {
    model: &'a MyModel,
    drop_shadow: &'a MyModel,
    col: &'a UnitCollection<Warrior>,
}
impl<'a> WarriorDraw<'a> {
    fn new(col: &'a UnitCollection<Warrior>, model: &'a MyModel, drop_shadow: &'a MyModel) -> Self {
        Self {
            model,
            drop_shadow,
            col,
        }
    }
    fn draw(&self, gg: &grids::GridMatrix, draw_sys: &mut ShaderSystem, matrix: &Matrix4<f32>) {
        for cc in self.col.elem.iter() {
            let pos: [f32; 2] = gg.to_world_topleft(cc.position.0.into()).into();

            let t = matrix::translation(pos[0], pos[1], 20.0);
            let s = matrix::scale(1.0, 1.0, 1.0);
            let m = matrix.chain(t).chain(s).generate();
            let mut v = draw_sys.view(m.as_ref());

            self.model
                .draw_ext(&mut v, !cc.is_selectable(), false, false);
        }
    }

    fn draw_shadow(
        &self,
        gg: &grids::GridMatrix,
        draw_sys: &mut ShaderSystem,
        matrix: &Matrix4<f32>,
    ) {
        for &GridCoord(a) in self.col.elem.iter().map(|a| &a.position) {
            let pos: [f32; 2] = gg.to_world_topleft(a.into()).into();
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
            let pos: [f32; 2] = gg.to_world_topleft(ccat.position.0.into()).into();

            let t = matrix::translation(pos[0], pos[1] + 20.0, 20.0);

            let jj = view_proj.chain(t).generate();
            let jj: &[f32; 16] = jj.as_ref();
            let tt = matrix::translation(jj[12], jj[13], jj[14]);
            let new_proj = proj.clone().chain(tt);

            let s = matrix::scale(5.0, 5.0, 5.0);
            let m = new_proj.chain(s).generate();

            let nn = health_numbers.get_number(ccat.health);
            let mut v = draw_sys.view(m.as_ref());
            nn.draw_ext(&mut v, false, false, true);

            //nn.draw(ccat.health,&ctx,&text_texture,&mut draw_sys,&m);
        }
    }
}

//TODO sort this by x and then y axis!!!!!!!
#[derive(Debug)]
pub struct UnitCollection<T: HasPos> {
    elem: Vec<T>,
}

impl<T: HasPos> UnitCollection<T> {
    fn new(elem: Vec<T>) -> Self {
        UnitCollection { elem }
    }
    fn remove(&mut self, a: &GridCoord) -> T {
        let (i, _) = self
            .elem
            .iter()
            .enumerate()
            .find(|(_, b)| b.get_pos() == a)
            .unwrap();
        self.elem.swap_remove(i)
    }

    pub fn find_mut(&mut self, a: &GridCoord) -> Option<&mut T> {
        self.elem.iter_mut().find(|b| b.get_pos() == a)
    }
    fn find(&self, a: &GridCoord) -> Option<&T> {
        self.elem.iter().find(|b| b.get_pos() == a)
    }
    fn filter(&self) -> UnitCollectionFilter<T> {
        UnitCollectionFilter { a: &self.elem }
    }
}

pub struct SingleFilter<'a> {
    a: &'a GridCoord,
}
impl<'a> movement::Filter for SingleFilter<'a> {
    fn filter(&self, a: &GridCoord) -> bool {
        self.a != a
    }
}

pub struct UnitCollectionFilter<'a, T> {
    a: &'a [T],
}
impl<'a, T: HasPos> movement::Filter for UnitCollectionFilter<'a, T> {
    fn filter(&self, b: &GridCoord) -> bool {
        self.a.iter().find(|a| a.get_pos() == b).is_none()
    }
}

pub trait HasPos {
    fn get_pos(&self) -> &GridCoord;
}
impl HasPos for GridCoord {
    fn get_pos(&self) -> &GridCoord {
        self
    }
}

impl HasPos for Warrior {
    fn get_pos(&self) -> &GridCoord {
        &self.position
    }
}

type MyModel = model_parse::Foo<model_parse::TextureGpu, model_parse::ModelGpu>;

#[derive(Debug)]
pub struct Warrior {
    position: GridCoord,
    move_deficit: MoveUnit,
    moved: bool,
    health: i8,
}

impl Warrior {
    fn is_selectable(&self) -> bool {
        !self.moved
    }

    fn new(position: GridCoord) -> Self {
        Warrior {
            position,
            move_deficit: MoveUnit(0),
            moved: false,
            health: 10,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CellSelection {
    MoveSelection(movement::PossibleMoves, movement::PossibleMoves),
    BuildSelection(GridCoord),
}

pub struct TribeFilter<'a> {
    tribe: &'a Tribe,
}
impl<'a> movement::Filter for TribeFilter<'a> {
    fn filter(&self, b: &GridCoord) -> bool {
        self.tribe
            .warriors
            .iter()
            .map(|a| a.filter().filter(b))
            .fold(true, |a, b| a && b)
    }
}

impl<T> std::borrow::Borrow<T> for WarriorPointer<T> {
    fn borrow(&self) -> &T {
        &self.inner
    }
}
impl<T> std::borrow::BorrowMut<T> for WarriorPointer<T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

#[derive(Copy, Clone)]
pub struct WarriorPointer<T> {
    inner: T,
    val: usize,
}

impl WarriorPointer<&Warrior> {
    //TODO use this instead of gridcoord when you know the type!!!!!
    fn slim(&self) -> WarriorPointer<GridCoord> {
        WarriorPointer {
            inner: self.inner.position,
            val: self.val,
        }
    }
}
impl WarriorPointer<Warrior> {
    //TODO use this instead of gridcoord when you know the type!!!!!
    fn slim(&self) -> WarriorPointer<GridCoord> {
        WarriorPointer {
            inner: self.inner.position,
            val: self.val,
        }
    }
}

impl<T> std::ops::Deref for WarriorPointer<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for WarriorPointer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct Tribe {
    warriors: Vec<UnitCollection<Warrior>>,
}
impl Tribe {
    fn lookup(&self, a: WarriorPointer<GridCoord>) -> WarriorPointer<&Warrior> {
        self.warriors[a.val]
            .find(&a.inner)
            .map(|b| WarriorPointer {
                inner: b,
                val: a.val,
            })
            .unwrap()
    }
    fn lookup_mut(&mut self, a: &WarriorPointer<GridCoord>) -> WarriorPointer<&mut Warrior> {
        self.warriors[a.val]
            .find_mut(&a.inner)
            .map(|b| WarriorPointer {
                inner: b,
                val: a.val,
            })
            .unwrap()
    }
    fn lookup_take(&mut self, a: WarriorPointer<GridCoord>) -> WarriorPointer<Warrior> {
        Some(self.warriors[a.val].remove(&a.inner))
            .map(|b| WarriorPointer {
                inner: b,
                val: a.val,
            })
            .unwrap()
    }

    fn add(&mut self, a: WarriorPointer<Warrior>) {
        self.warriors[a.val].elem.push(a.inner);
    }

    // fn remove(&mut self, a: &GridCoord) -> WarriorPointer<Warrior> {
    //     WarriorPointer {
    //         inner: self.warriors.remove(a),
    //         val: 0,
    //     }
    // }

    // pub fn find_mut(&mut self, a: &GridCoord) -> Option<WarriorPointer<&mut Warrior>> {
    //     self.warriors
    //         .find_mut(a)
    //         .map(|a| WarriorPointer { inner: a, val: 0 })
    // }

    fn find2(&self, a: &GridCoord) -> Option<WarriorPointer<&Warrior>> {
        for (c, o) in self.warriors.iter().enumerate() {
            if let Some(k) = o.find(a) {
                return Some(WarriorPointer { inner: k, val: c });
            }
        }

        None
    }
    fn filter(&self) -> TribeFilter {
        TribeFilter { tribe: self }
    }

    fn reset(&mut self) {
        for a in self.warriors.iter_mut() {
            for b in a.elem.iter_mut() {
                b.moved = false;
            }
        }
    }
}

//TODO store actual world pos? Less calculation each iteration.
//Additionally removes need to special case animation.
pub struct Game {
    team: usize,
    grid_matrix: grids::GridMatrix,
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

    let dogs = UnitCollection::new(vec![
        Warrior::new(GridCoord([3, 3])),
        Warrior::new(GridCoord([4, 3])),
        Warrior::new(GridCoord([5, 3])),
        Warrior::new(GridCoord([6, 3])),
    ]);

    let cats = UnitCollection::new(vec![
        Warrior::new(GridCoord([3, 6])),
        Warrior::new(GridCoord([4, 6])),
        Warrior::new(GridCoord([5, 6])),
        Warrior::new(GridCoord([6, 6])),
    ]);

    let mut ggame = Game {
        team: 0,
        dogs: Tribe {
            warriors: vec![dogs],
        },
        cats: Tribe {
            warriors: vec![cats],
        },
        grid_matrix: grids::GridMatrix::new(),
    };

    let roads = terrain::TerrainCollection {
        pos: vec![],
        func: |a: MoveUnit| MoveUnit(a.0 / 2),
    };

    use cgmath::SquareMatrix;
    let mut last_matrix = cgmath::Matrix4::identity();

    let mut testo = state::create_state_machine();
    //log!(format!("size={:?}",std::mem::size_of_val(&testo)));

    let quick_load = |name| {
        let (data, t) = model::load_glb(name).gen_ext(ggame.grid_matrix.spacing(), RESIZE);
        model_parse::Foo {
            texture: model_parse::TextureGpu::new(&ctx, &t),
            model: model_parse::ModelGpu::new(&ctx, &data),
        }
    };

    let drop_shadow = quick_load(DROP_SHADOW_GLB);

    let dog = quick_load(DOG_GLB);

    let cat = quick_load(CAT_GLB);

    let road = quick_load(ROAD_GLB);

    let grass = quick_load(GRASS_GLB);

    let select_model = quick_load(SELECT_GLB);

    let attack_model = quick_load(ATTACK_GLB);

    let text_texture = {
        let ascii_tex = model::load_texture_from_data(include_bytes!("../assets/ascii5.png"));

        model_parse::TextureGpu::new(&ctx, &ascii_tex)
    };

    let health_numbers = NumberTextManager::new(0..=10, &ctx, &text_texture);

    'outer: loop {
        let mut on_select = false;

        let res = frame_timer.next().await;

        let mut reset = false;
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
                    scroll_manager.on_mouse_move([*x, *y], &last_matrix, viewport);
                }
                MEvent::EndTurn => {
                    reset = true;
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
        let mouse_world = scroll::mouse_to_world(scroll_manager.cursor_canvas(), &matrix, viewport);

        {
            //Advance state machine.
            let mouse = on_select.then_some(mouse_world);
            let [this_team, that_team] =
                state::team_view([&mut ggame.cats, &mut ggame.dogs], ggame.team);

            let mut jj = state::Stuff {
                team: &mut ggame.team,
                this_team,
                that_team,
                grid_matrix: &ggame.grid_matrix,
                mouse,
                reset,
            };
            testo.step(&mut jj);
        }

        scroll_manager.step();

        use matrix::*;

        //Drawing below doesnt need mutable reference.
        //TODO move drawing to a function?
        let ggame = &ggame;

        ctx.draw_clear([0.0, 0.0, 0.0, 0.0]);

        let [vvx, vvy] = get_world_rect(&matrix, &ggame.grid_matrix);

        for a in (vvx[0]..vvx[1])
            .skip_while(|&a| a < 0)
            .take_while(|&a| a < ggame.grid_matrix.num_rows())
        {
            //both should be skip
            for b in (vvy[0]..vvy[1])
                .skip_while(|&a| a < 0)
                .take_while(|&a| a < ggame.grid_matrix.num_rows())
            {
                use matrix::*;
                let x1 = ggame.grid_matrix.spacing() * a as f32;
                let y1 = ggame.grid_matrix.spacing() * b as f32;
                let s = 0.99;
                let mm = matrix
                    .chain(translation(x1, y1, -1.0))
                    .chain(scale(s, s, s))
                    .generate();

                let mut v = draw_sys.view(mm.as_ref());
                grass.draw(&mut v);
            }
        }

        let cat_draw = WarriorDraw::new(&ggame.cats.warriors[0], &cat, &drop_shadow);
        let dog_draw = WarriorDraw::new(&ggame.dogs.warriors[0], &dog, &drop_shadow);

        let animation_draw = if ggame.team == 0 { &cat } else { &dog };

        disable_depth(&ctx, || {
            if let Some(a) = testo.get_selection() {
                match a {
                    CellSelection::MoveSelection(a, attack) => {
                        for GridCoord(a) in a.iter_coords() {
                            let pos: [f32; 2] = ggame.grid_matrix.to_world_topleft(a.into()).into();
                            let t = matrix::translation(pos[0], pos[1], 0.0);

                            let m = matrix.chain(t).generate();

                            let mut v = draw_sys.view(m.as_ref());
                            select_model.draw(&mut v);
                        }

                        for GridCoord(a) in attack.iter_coords() {
                            let pos: [f32; 2] = ggame.grid_matrix.to_world_topleft(a.into()).into();
                            let t = matrix::translation(pos[0], pos[1], 0.0);

                            let m = matrix.chain(t).generate();

                            let mut v = draw_sys.view(m.as_ref());
                            attack_model.draw(&mut v);
                        }
                    }
                    CellSelection::BuildSelection(_) => {}
                }
            }

            for GridCoord(a) in roads.pos.iter() {
                let pos: [f32; 2] = ggame.grid_matrix.to_world_topleft(a.into()).into();
                let t = matrix::translation(pos[0], pos[1], 3.0);

                let m = matrix.chain(t).generate();

                let mut v = draw_sys.view(m.as_ref());
                road.draw(&mut v);
            }
        });

        disable_depth(&ctx, || {
            //draw dropshadow

            cat_draw.draw_shadow(&ggame.grid_matrix, &mut draw_sys, &matrix);
            dog_draw.draw_shadow(&ggame.grid_matrix, &mut draw_sys, &matrix);

            if let Some(a) = &testo.get_animation() {
                let pos = a.calc_pos();
                let t = matrix::translation(pos[0], pos[1], 1.0);

                let m = matrix.chain(t).generate();

                let mut v = draw_sys.view(m.as_ref());
                drop_shadow.draw(&mut v);
            }
        });

        if let Some(a) = &testo.get_animation() {
            let pos = a.calc_pos();
            let t = matrix::translation(pos[0], pos[1], 20.0);
            let s = matrix::scale(1.0, 1.0, 1.0);
            let m = matrix.chain(t).chain(s).generate();
            let mut v = draw_sys.view(m.as_ref());

            animation_draw.draw(&mut v);
        }

        cat_draw.draw(&ggame.grid_matrix, &mut draw_sys, &matrix);
        dog_draw.draw(&ggame.grid_matrix, &mut draw_sys, &matrix);

        disable_depth(&ctx, || {
            cat_draw.draw_health_text(
                &ggame.grid_matrix,
                &health_numbers,
                &view_proj,
                &proj,
                &mut draw_sys,
            );
            dog_draw.draw_health_text(
                &ggame.grid_matrix,
                &health_numbers,
                &view_proj,
                &proj,
                &mut draw_sys,
            );
        });

        ctx.flush();
    }

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

use crate::gameplay::GameStepper;
use crate::movement::{Filter, MoveUnit};
use crate::terrain::MoveCost;

const SELECT_GLB: &'static [u8] = include_bytes!("../assets/select_model.glb");
const DROP_SHADOW_GLB: &'static [u8] = include_bytes!("../assets/drop_shadow.glb");
const ROAD_GLB: &'static [u8] = include_bytes!("../assets/road.glb");
const ATTACK_GLB: &'static [u8] = include_bytes!("../assets/attack.glb");

// const SHADED_GLB: &'static [u8] = include_bytes!("../assets/shaded.glb");
// const KEY_GLB: &'static [u8] = include_bytes!("../assets/key.glb");
// const PERSON_GLB: &'static [u8] = include_bytes!("../assets/person-v1.glb");
const CAT_GLB: &'static [u8] = include_bytes!("../assets/tiger2.glb");
const DOG_GLB: &'static [u8] = include_bytes!("../assets/cat2.glb");

const GRASS_GLB: &'static [u8] = include_bytes!("../assets/grass.glb");

pub struct NumberTextManager<'a> {
    numbers: Vec<model_parse::ModelGpu>,
    texture: &'a model_parse::TextureGpu,
}
impl<'a> NumberTextManager<'a> {
    fn new(
        range: impl IntoIterator<Item = i8>,
        ctx: &WebGl2RenderingContext,
        texture: &'a model_parse::TextureGpu,
    ) -> Self {
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
        let gpu = &self.numbers[num as usize];

        model_parse::Foo {
            texture: &self.texture,
            model: gpu,
        }
    }
}
