use axgeom::vec2same;
use cgmath::{InnerSpace, Transform, Vector2};
use gloo::console::log;
use model::matrix::{self, MyMatrix};
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

pub struct UnitCollection<T: HasPos>(Vec<T>);
impl<T: HasPos> UnitCollection<T> {
    fn remove(&mut self, a: &GridCoord) -> T {
        let (i, _) = self
            .0
            .iter()
            .enumerate()
            .find(|(_, b)| b.get_pos() == a)
            .unwrap();
        self.0.swap_remove(i)
    }
    fn find_mut(&mut self, a: &GridCoord) -> Option<&mut T> {
        self.0.iter_mut().find(|b| b.get_pos() == a)
    }
    fn find(&self, a: &GridCoord) -> Option<&T> {
        self.0.iter().find(|b| b.get_pos() == a)
    }
    fn filter(&self) -> UnitCollectionFilter<T> {
        UnitCollectionFilter { a: &self.0 }
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

impl HasPos for Cat {
    fn get_pos(&self) -> &GridCoord {
        &self.position
    }
}

pub struct Cat {
    position: GridCoord,
    move_deficit: MoveUnit,
    moved: bool,
}
impl Cat {
    fn new(position: GridCoord) -> Self {
        Cat {
            position,
            move_deficit: MoveUnit(0),
            moved: false,
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

    let attack_model = {
        let data = model::load_glb(ATTACK_GLB).gen_ext(gg.spacing());

        model_parse::ModelGpu::new(&ctx, &data)
    };


    let text_model = {
        let ascii_tex=model::load_texture_from_data(include_bytes!("../assets/ascii.png"));
        let data=string_to_coords(ascii_tex,"abcdefg");
    
        model_parse::ModelGpu::new(&ctx, &data)
    };

    

    let mut cats = UnitCollection(vec![
        Cat::new(GridCoord([2, 2])),
        Cat::new(GridCoord([5, 5])),
        Cat::new(GridCoord([6, 6])),
        Cat::new(GridCoord([7, 7])),
        Cat::new(GridCoord([3, 1])),
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
                MEvent::EndTurn => {
                    for a in cats.0.iter_mut() {
                        a.moved = false;
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

        let mouse_world = scroll::mouse_to_world(scroll_manager.cursor_canvas(), matrix);

        if animation.is_some() {
            on_select = false;
        }
        if on_select {
            let cell: GridCoord = GridCoord(gg.to_grid((mouse_world).into()).into());

            if let Some(ss) = &mut selected_cell {
                match ss {
                    CellSelection::MoveSelection(ss, attack) => {
                        if movement::contains_coord(ss.iter_coords(), &cell) {
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
                    let mm = if cat.moved {
                        MoveUnit(0)
                    } else {
                        //MoveUnit(2-1) vs MoveUnit(4-1) vs MoveUnit(6-1)
                        MoveUnit(6 - 1)
                    };

                    let mm = movement::PossibleMoves::new(
                        &movement::WarriorMovement,
                        &gg.filter().chain(cats.filter()),
                        &terrain::Grass.chain(roads.foo()),
                        cat.position,
                        mm,
                    );
                    log!(format!("deficit:{:?}", cat.move_deficit.0));

                    let attack_range = 3;
                    let aa = (attack_range);
                    let attack = movement::PossibleMoves::new(
                        &movement::WarriorMovement,
                        &gg.filter(),
                        &terrain::Grass,
                        cat.position,
                        MoveUnit(aa),
                    );

                    selected_cell = Some(CellSelection::MoveSelection(mm, attack));
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

            for &GridCoord(a) in cats.0.iter().map(|a| &a.position) {
                let pos: [f32; 2] = gg.to_world_topleft(a.into()).into();
                let t = matrix::translation(pos[0], pos[1], 1.0);

                let m = matrix.chain(t).generate();

                let mut v = draw_sys.view(m.as_ref());
                //text_model.draw_ext(&mut v,false,true);
                drop_shadow.draw(&mut v);
            }

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
                cats.0.push(a.into_data());
                animation = None;
            };
        }

        for cc in cats.0.iter() {
            let pos: [f32; 2] = gg.to_world_topleft(cc.position.0.into()).into();

            let t = matrix::translation(pos[0], pos[1], 20.0);
            let s = matrix::scale(1.0, 1.0, 1.0);
            let m = matrix.chain(t).chain(s).generate();
            let mut v = draw_sys.view(m.as_ref());

            cat.draw_ext(&mut v, cc.moved,false);
            //text_model.draw_ext(&mut v, cc.moved);
            
        }

        {
            ctx.disable(WebGl2RenderingContext::DEPTH_TEST);
            ctx.disable(WebGl2RenderingContext::CULL_FACE);

            //draw text
            for &GridCoord(a) in cats.0.iter().map(|a| &a.position) {
                let pos: [f32; 2] = gg.to_world_topleft(a.into()).into();

                
                let proj=projection::projection(viewport);
                //fn camera(camera: [f32; 2], zoom: f32, rot: f32) -> impl matrix::MyMatrix + matrix::Inverse {
                let view_proj=projection::camera(scroll_manager.camera(),
                    scroll_manager.zoom(),
                    scroll_manager.rot()).inverse();
                let t = matrix::translation(pos[0], pos[1], 20.0);
                
                let jj=view_proj.chain(t).generate();
                let jj:&[f32;16]=jj.as_ref();
                let tt=matrix::translation(jj[12],jj[13],jj[14]);
                let new_proj=proj.chain(tt);

                let s = matrix::scale(5.0,5.0,5.0);
                
                let m = new_proj.chain(s).generate();


                // let m=matrix.chain(tt).generate();

                let mut v = draw_sys.view(m.as_ref());
                text_model.draw_ext(&mut v,false,true);
                //drop_shadow.draw(&mut v);
            }

            ctx.enable(WebGl2RenderingContext::DEPTH_TEST);
            ctx.enable(WebGl2RenderingContext::CULL_FACE);
        }

        ctx.flush();
    }

    w.post_message(UiButton::NoUi);

    log!("worker thread closing");
}




//TODO just use reference???
fn string_to_coords(im:model::Img,st:&str)->model::ModelData{
    



    let mut tex_coords=vec!();
    let mut counter=0.0;
    let dd=20.0;
    let mut positions=vec!();


    let mut inds=vec!();
    for (_, a) in st.chars().enumerate() {
        let ascii = a as u8;
        let index=(ascii-32) as u16;

        //log!(format!("aaaa:{:?}",index));
        let x = (index % 16) as f32/16.;
        let y = ((index / 16)) as f32/14.;
        
        
        let x1=x;
        let x2=x1+1.0/16.0;
        
        let y1=y;
        let y2=y+1.0/14.0;

        let a=[
            [x1,y1],
            [x2,y1],
            [x1,y2],
            [x2,y2]
        ];

        tex_coords.extend(a);

        let iii=[
            0u16,1,2,2,1,3
            
        ].map(|a|positions.len() as u16+a);
       

        let xx1=counter;
        let xx2=counter+dd;
        let yy1=dd;
        let yy2=0.0;

        let zz=0.0;
        let y=[
            [xx1,yy1,zz],
            [xx2,yy1,zz],
            [xx1,yy2,zz],
            [xx2,yy2,zz]
        ];


        positions.extend(y);
        
        inds.extend(iii);

        assert!(ascii >= 32);
        counter += dd;
    }

    let normals=positions.iter().map(|_|[0.0,0.0,1.0]).collect();

    let cc=1.0/dd;
    let mm=matrix::scale(cc,cc,cc).generate();

    let positions=positions.into_iter().map(|a|mm.transform_point(a.into()).into()).collect();

    model::ModelData{
        positions,
        tex_coords,
        indices:Some(inds),
        texture:im,
        normals,
        matrix:mm

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
const GRASS_GLB: &'static [u8] = include_bytes!("../assets/grass.glb");
