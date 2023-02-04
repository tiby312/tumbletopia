use axgeom::vec2same;
use cgmath::{InnerSpace, Transform, Vector2, Matrix4};
use duckduckgeo::grid::Grid2D;
use gloo::console::log;
use model::matrix::{self, MyMatrix};
use movement::GridCoord;
use serde::{Deserialize, Serialize};
use shogo::simple2d::{self, ShaderSystem};
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

pub const RESIZE: usize = 6;



// //TODO use htis!!!
// pub struct MyComp<T>{
//     a:Vec<Option<T>>
// }
// impl<T> MyComp<T>{
//     fn new_elem(&mut self,a:T)->usize{
//         //TODO look for a destroyed element.
//         let b=self.a.len();
//         self.a.push(Some(a));
//         b
//     }
//     fn destroy_elem(&mut self,i:usize){
//         self.a[i]=None;
//     }
//     fn get_mut(&mut self,i:usize)->Option<&mut T>{
//         self.a[i].as_mut()
//     }

//     fn get_two_mut(&mut self,a:usize,b:usize)->Option<(&mut T,&mut T)>{
//         let (first,second)=self.a.split_at_mut(a);
//         todo!();
//     }
// }



#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
enum UiButton {
    ShowRoadUi,
    NoUi,
}

pub struct UnitCollection<'a,T: HasPos>{
    elem:Vec<T>,
    model:&'a MyModel,
    drop_shadow:&'a MyModel,
    
}

impl<'a> UnitCollection<'a,Warrior> {
    fn draw(&self,gg:&grids::GridMatrix,draw_sys:&mut ShaderSystem,matrix:&ViewProjection){
        for cc in self.elem.iter() {
            let pos: [f32; 2] = gg.to_world_topleft(cc.position.0.into()).into();

            let t = matrix::translation(pos[0], pos[1], 20.0);
            let s = matrix::scale(1.0, 1.0, 1.0);
            let m = matrix.chain(t).chain(s).generate();
            let mut v = draw_sys.view(m.as_ref());

            self.model.draw_ext(&mut v, !cc.is_selectable(), false, false);
        }
    }

    fn draw_shadow(&self,gg:&grids::GridMatrix,draw_sys:&mut ShaderSystem,matrix:&ViewProjection){


        for &GridCoord(a) in self.elem.iter().map(|a| &a.position) {
            let pos: [f32; 2] = gg.to_world_topleft(a.into()).into();
            let t = matrix::translation(pos[0], pos[1], 1.0);

            let m = matrix.chain(t).generate();

            let mut v = draw_sys.view(m.as_ref());
            self.drop_shadow.draw(&mut v);
        }


    }

    // fn draw_text(&self,gg:&grids::GridMatrix,draw_sys:&mut ShaderSystem,proj:&matrix::Perspective,view_proj:&Matrix4<f32>){

    //         //draw text
    //         for ccat in self.elem.iter() {
    //             let pos: [f32; 2] = gg.to_world_topleft(ccat.position.0.into()).into();

    //             let t = matrix::translation(pos[0], pos[1] + 20.0, 20.0);

    //             let jj = view_proj.chain(t).generate();
    //             let jj: &[f32; 16] = jj.as_ref();
    //             let tt = matrix::translation(jj[12], jj[13], jj[14]);
    //             let new_proj = proj.clone().chain(tt);

    //             let s = matrix::scale(5.0, 5.0, 5.0);
    //             //let r=matrix::z_rotation(std::f32::consts::PI/4.0);
    //             let m = new_proj.chain(s).generate();

    //             // let m=matrix.chain(tt).generate();

    //             let mut v = draw_sys.view(m.as_ref());

    //             //TODO optimize
    //             let data = string_to_coords(&format!("{}", ccat.health));
    //             //TODO only sent to gpu on text update
    //             let m = model_parse::ModelGpu::new(&ctx, &data);
    //             model_parse::Foo {
    //                 texture: &text_texture,
    //                 model: &m,
    //             }
    //             .draw_ext(&mut v, false, false, true);
    //         }
    // }
}

impl<'a,T: HasPos> UnitCollection<'a,T> {
 
    fn new(elem:Vec<T>,model:&'a MyModel,drop_shadow:&'a MyModel)->Self{
        UnitCollection { elem,model,drop_shadow }
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

    fn find_mut(&mut self, a: &GridCoord) -> Option<&mut T> {
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

type MyModel=model_parse::Foo<model_parse::TextureGpu, model_parse::ModelGpu>;

pub struct Warrior {
    position: GridCoord,
    move_deficit: MoveUnit,
    moved: bool,
    attacked: bool,
    health: i8,
}

impl Warrior {
    fn is_selectable(&self) -> bool {
        !self.moved || !self.attacked
    }

    fn new(position: GridCoord) -> Self {
        Warrior {
            position,
            move_deficit: MoveUnit(0),
            moved: false,
            attacked: false,
            health: 10,
        }
    }
}

enum CellSelection {
    MoveSelection(movement::PossibleMoves, movement::PossibleMoves),
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

    let quick_load = |name| {
        let (data, t) = model::load_glb(name).gen_ext(gg.spacing(), RESIZE);
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

    let health_numbers=NumberTextManager::new(0..=10,&ctx,&text_texture);



    let mut dogs = UnitCollection::new(vec![
        Warrior::new(GridCoord([3, 3])),
        Warrior::new(GridCoord([4, 4])),
    ],&dog,&drop_shadow);

    let mut cats = UnitCollection::new(vec![
        Warrior::new(GridCoord([2, 2])),
        Warrior::new(GridCoord([5, 5])),
        Warrior::new(GridCoord([6, 6])),
        Warrior::new(GridCoord([7, 7])),
        Warrior::new(GridCoord([3, 1])),
    ],&cat,&drop_shadow);

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
                    let mm = view_projection(
                        scroll_manager.camera(),
                        viewport,
                        scroll_manager.zoom(),
                        scroll_manager.rot(),
                    );

                    scroll_manager.on_touch_move(touches, mm);
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
                    let mm = view_projection(
                        scroll_manager.camera(),
                        viewport,
                        scroll_manager.zoom(),
                        scroll_manager.rot(),
                    );

                    scroll_manager.on_mouse_move([*x, *y], mm);
                }
                MEvent::EndTurn => {
                    for a in cats.elem.iter_mut() {
                        a.moved = false;
                        a.attacked = false;
                    }
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

        let proj = projection::projection(viewport);
        let view_proj = projection::view_matrix(
            scroll_manager.camera(),
            scroll_manager.zoom(),
            scroll_manager.rot(),
        );
        //TODO simplify
        let matrix = view_projection(
            scroll_manager.camera(),
            viewport,
            scroll_manager.zoom(),
            scroll_manager.rot(),
        );

        let mouse_world = scroll::mouse_to_world(scroll_manager.cursor_canvas(), matrix);

        if animation.is_some() {
            on_select = false;
        }
        if on_select {
            let cell: GridCoord = GridCoord(gg.to_grid((mouse_world).into()).into());

            if let Some(ss) = &mut selected_cell {
                match ss {
                    CellSelection::MoveSelection(ss, attack) => {
                        let target_cat_pos = &cell;

                        let current_attack=cats.find_mut(ss.start()).unwrap().attacked;
                        
                        if !current_attack && movement::contains_coord(attack.iter_coords(), target_cat_pos)
                            && cats.find(target_cat_pos).is_some() 
                        {
                            let target_cat = cats.find_mut(target_cat_pos).unwrap();
                            target_cat.health -= 1;

                            let current_cat = cats.find_mut(ss.start()).unwrap();
                            current_cat.attacked = true;
                        } else if movement::contains_coord(ss.iter_coords(), &cell) {
                            let mut c = cats.remove(ss.start());
                            let (dd, aa) = ss.get_path_data(cell).unwrap();
                            c.position = cell;
                            c.move_deficit = *aa;
                            c.moved = true;
                            animation = Some(animation::Animation::new(ss.start(), dd, &gg, c));
                        }
                        selected_cell = None;
                    }
                    CellSelection::BuildSelection(_) => {
                        //do nothing? we are waiting on user to push a button.
                    }
                }
            } else {
                if let Some(cat) = cats.find(&cell) {
                    if cat.is_selectable() {
                        selected_cell = Some(get_cat_move_attack_matrix(
                            cat,
                            cats.filter(),
                            roads.foo(),
                            &gg,
                        ));
                    }
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
                    CellSelection::MoveSelection(a, attack) => {
                        for GridCoord(a) in a.iter_coords() {
                            let pos: [f32; 2] = gg.to_world_topleft(a.into()).into();
                            let t = matrix::translation(pos[0], pos[1], 0.0);

                            let m = matrix.chain(t).generate();

                            let mut v = draw_sys.view(m.as_ref());
                            select_model.draw(&mut v);
                        }

                        for GridCoord(a) in attack.iter_coords() {
                            let pos: [f32; 2] = gg.to_world_topleft(a.into()).into();
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

            cats.draw_shadow(&gg, &mut draw_sys, &matrix);
            

            if let Some(a) = &animation {
                let pos = a.calc_pos();
                let t = matrix::translation(pos[0], pos[1], 1.0);

                let m = matrix.chain(t).generate();

                let mut v = draw_sys.view(m.as_ref());
                drop_shadow.draw(&mut v);
            }

            ctx.enable(WebGl2RenderingContext::DEPTH_TEST);
            ctx.enable(WebGl2RenderingContext::CULL_FACE);
        }

        if let Some(mut a) = animation.take() {
            if let Some(pos) = a.animate_step() {
                let t = matrix::translation(pos[0], pos[1], 20.0);
                let s = matrix::scale(1.0, 1.0, 1.0);
                let m = matrix.chain(t).chain(s).generate();
                let mut v = draw_sys.view(m.as_ref());
                cat.draw(&mut v);
                animation = Some(a);
            } else {
                let cat = a.into_data();
                animation = None;

                selected_cell = Some(get_cat_move_attack_matrix(
                    &cat,
                    cats.filter(),
                    roads.foo(),
                    &gg,
                ));
                cats.elem.push(cat);
            };
        }


        cats.draw(&gg,&mut draw_sys,&matrix);




        
        {
            ctx.disable(WebGl2RenderingContext::DEPTH_TEST);
            ctx.disable(WebGl2RenderingContext::CULL_FACE);

            //draw text
            for ccat in cats.elem.iter() {
                let pos: [f32; 2] = gg.to_world_topleft(ccat.position.0.into()).into();

                let t = matrix::translation(pos[0], pos[1] + 20.0, 20.0);

                let jj = view_proj.chain(t).generate();
                let jj: &[f32; 16] = jj.as_ref();
                let tt = matrix::translation(jj[12], jj[13], jj[14]);
                let new_proj = proj.clone().chain(tt);

                let s = matrix::scale(5.0, 5.0, 5.0);
                let m = new_proj.chain(s).generate();

                let nn=health_numbers.get_number(ccat.health);


                let mut v = draw_sys.view(m.as_ref());

                nn.draw_ext(&mut v, false, false, true);
        

                //nn.draw(ccat.health,&ctx,&text_texture,&mut draw_sys,&m);
                
            }

            ctx.enable(WebGl2RenderingContext::DEPTH_TEST);
            ctx.enable(WebGl2RenderingContext::CULL_FACE);
        }

        ctx.flush();
    }

    w.post_message(UiButton::NoUi);

    log!("worker thread closing");
}

fn get_cat_move_attack_matrix(
    cat: &Warrior,
    cat_filter: impl Filter,
    roads: impl MoveCost,
    gg: &grids::GridMatrix,
) -> CellSelection {
    
    let mm = if cat.moved {
        MoveUnit(0)
    } else {
        MoveUnit(6 - 1)
    };

    let mm = movement::PossibleMoves::new(
        &movement::WarriorMovement,
        &gg.filter().chain(cat_filter),
        &terrain::Grass.chain(roads),
        cat.position,
        mm,
    );

    let attack_range = 2-1;
    let attack = movement::PossibleMoves::new(
        &movement::WarriorMovement,
        &gg.filter().chain(SingleFilter { a: cat.get_pos() }),
        &terrain::Grass,
        cat.position,
        MoveUnit(attack_range),
    );

    CellSelection::MoveSelection(mm, attack)
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




pub struct NumberTextManager<'a>{
    numbers:Vec<model_parse::ModelGpu>,
    texture:&'a model_parse::TextureGpu
}
impl<'a> NumberTextManager<'a>{

    fn new(range:impl IntoIterator<Item=i8>,ctx:&WebGl2RenderingContext,texture:&'a model_parse::TextureGpu)->Self{

        fn generate_number(number:i8,ctx:&WebGl2RenderingContext)->model_parse::ModelGpu{
            let data=string_to_coords(&format!("{}", number));
            model_parse::ModelGpu::new(ctx, &data)
        }

        let numbers=range.into_iter().map(|i|{
            generate_number(i,ctx)
        }).collect();
        Self { numbers,texture }
    }

    fn get_number(&self,num:i8)->model_parse::Foo<&model_parse::TextureGpu,&model_parse::ModelGpu>{
        let gpu=&self.numbers[num as usize];


        model_parse::Foo {
            texture: &self.texture,
            model: gpu,
        }

    }
}

