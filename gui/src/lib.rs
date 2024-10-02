use gloo::console::console_dbg;
use gloo::console::log;

use hex::*;
use model::matrix::MyMatrix;
use shader_sys::ShaderSystem;

pub mod animation;
pub mod dom;
pub mod model_parse;
pub mod projection;
pub mod scroll;
pub mod shader_sys;
pub mod worker;
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

use web_sys::WebGl2RenderingContext;

use model_parse::{Foo, ModelGpu, TextureGpu};

//purple #6d32e7
//orange #ff8100

//TODO move this to a seperate crate
pub struct Models<T> {
    pub select_model: T,
    pub drop_shadow: T,
    pub token_black: T,
    pub token_white: T,
    pub snow: T,
    pub token_neutral: T,
    pub water: T,
    pub land: T,
}

impl Models<Foo<TextureGpu, ModelGpu>> {
    pub fn new(grid_matrix: &hex::HexConverter, shader: &ShaderSystem) -> Self {
        let quick_load = |name, res, alpha| {
            let (data, t) = model::load_glb(name).gen_ext(grid_matrix.spacing(), res, alpha);

            log!(format!("texture:{:?}", (t.width, t.height)));

            model_parse::Foo {
                texture: model_parse::TextureGpu::new(&shader.ctx, &t),
                model: model_parse::ModelGpu::new(shader, &data),
                width: data
                    .positions
                    .iter()
                    .map(|x| x[0])
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap(),
                height: data
                    .positions
                    .iter()
                    .map(|x| x[2])
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap(),
            }
        };

        pub const RESIZE: usize = 10;

        Models {
            select_model: quick_load(include_bytes!("../../assets/hex-select.glb"), 1, None),
            drop_shadow: quick_load(include_bytes!("../../assets/drop_shadow.glb"), 1, Some(0.5)),
            // fog: quick_load(include_bytes!("../assets/fog.glb"), RESIZE, None),
            // attack: quick_load(include_bytes!("../assets/attack.glb"), 1, None),
            // white_mouse: quick_load(include_bytes!("../assets/white_mouse.glb"), RESIZE, None),
            // black_mouse: quick_load(include_bytes!("../assets/black_mouse.glb"), RESIZE, None),
            // white_rabbit: quick_load(include_bytes!("../assets/white_rabbit.glb"), RESIZE, None),
            // black_rabbit: quick_load(include_bytes!("../assets/black_rabbit.glb"), RESIZE, None),
            token_black: quick_load(include_bytes!("../../assets/hex-black.glb"), RESIZE, None),
            token_white: quick_load(include_bytes!("../../assets/hex-white.glb"), RESIZE, None),
            token_neutral: quick_load(include_bytes!("../../assets/hex-neutral.glb"), RESIZE, None),
            // direction: quick_load(include_bytes!("../assets/direction.glb"), 1, None),
            snow: quick_load(include_bytes!("../../assets/ice.glb"), RESIZE, None),
            water: quick_load(include_bytes!("../../assets/water.glb"), RESIZE, None),
            land: quick_load(include_bytes!("../../assets/hex-grass.glb"), RESIZE, None),
        }
    }
}

#[must_use]
pub struct BatchBuilder<'a, I> {
    sys: &'a mut ShaderSystem,
    ff: I,
    lighting: bool,
    grey: bool,
}
impl<I: Iterator<Item = K>, K: MyMatrix> BatchBuilder<'_, I> {
    pub fn build(&mut self, texture: &Foo<TextureGpu, ModelGpu>, my_matrix: &[f32; 16]) {
        let mmatrix: Vec<[f32; 16]> = (&mut self.ff)
            .map(|x| {
                //let my_matrix:&Matrix4<f32>=my_matrix.into();
                //let x = my_matrix.chain(x).generate();
                //let x: &[f32; 16] = x.as_ref();
                let x = x.generate();
                let x: &[f32; 16] = x.as_ref();
                *x
            })
            .collect();

        // let uworlds: Vec<[f32; 16]> = (&mut self.ff)
        //     .map(|x| {
        //         let x = x.generate();
        //         let x: &[f32; 16] = x.as_ref();
        //         *x
        //     })
        //     .collect();

        // let mmatrix: Vec<_> = uworlds
        //     .iter()
        //     .map(|x| {
        //         let x: &Matrix4<f32> = x.into();
        //         let my_matrix: &Matrix4<f32> = my_matrix.into();
        //         let x = my_matrix.chain(*x).generate();
        //         let x: &[f32; 16] = x.as_ref();
        //         *x
        //     })
        //     .collect();

        //if !uworlds.is_empty() {

        self.sys.draw(
            &texture.model.res,
            &texture.texture.texture,
            &mmatrix,
            self.grey,
            false,
            self.lighting,
            my_matrix,
        );
        //}
    }
    pub fn grey(&mut self, grey: bool) -> &mut Self {
        self.grey = grey;
        self
    }
    pub fn no_lighting(&mut self) -> &mut Self {
        self.lighting = false;
        self
    }
}
impl Doop for ShaderSystem {
    fn batch<K: MyMatrix, I>(&mut self, ff: I) -> BatchBuilder<'_, I::IntoIter>
    where
        I: IntoIterator<Item = K>,
    {
        BatchBuilder {
            sys: self,
            ff: ff.into_iter(),
            lighting: true,
            grey: false,
        }
    }
}

pub trait Doop {
    fn batch<K: MyMatrix, I>(&mut self, ff: I) -> BatchBuilder<'_, I::IntoIter>
    where
        I: IntoIterator<Item = K>;
}

// fn draw_health_text(
//     f: impl IntoIterator<Item = (Axial, i8)>,

//     gg: &grids::HexConverter,
//     health_numbers: &NumberTextManager,
//     view_proj: &Matrix4<f32>,
//     proj: &Matrix4<f32>,
//     draw_sys: &mut ShaderSystem,
//     text_texture: &TextureGpu,
// ) {
//     //draw text
//     for (ccat, ii) in f {
//         let pos = gg.hex_axial_to_world(&ccat);

//         let t = matrix::translation(pos.x, pos.y + 20.0, 20.0);

//         let jj = view_proj.chain(t).generate();
//         let jj: &[f32; 16] = jj.as_ref();
//         let tt = matrix::translation(jj[12], jj[13], jj[14]);
//         let new_proj = (*proj).chain(tt);

//         let s = matrix::scale(5.0, 5.0, 5.0);
//         let m = new_proj.chain(s).generate();

//         let nn = health_numbers.get_number(ii, text_texture);
//         draw_sys
//             .view(&m)
//             .draw_a_thing_ext(&nn, false, false, true, false);
//     }
// }

// fn string_to_coords<'a>(st: &str) -> model::ModelData {
//     let num_rows = 16;
//     let num_columns = 16;

//     let mut tex_coords = vec![];
//     let mut counter = 0.0;
//     let dd = 20.0;
//     let mut positions = vec![];

//     let mut inds = vec![];
//     for (_, a) in st.chars().enumerate() {
//         let ascii = a as u8;
//         let index = ascii as u16;

//         let x = (index % num_rows) as f32 / num_rows as f32;
//         let y = (index / num_rows) as f32 / num_columns as f32;

//         let x1 = x;
//         let x2 = x1 + 1.0 / num_rows as f32;

//         let y1 = y;
//         let y2 = y + 1.0 / num_columns as f32;

//         let a = [[x1, y1], [x2, y1], [x1, y2], [x2, y2]];

//         tex_coords.extend(a);

//         let iii = [0u16, 1, 2, 2, 1, 3].map(|a| positions.len() as u16 + a);

//         let xx1 = counter;
//         let xx2 = counter + dd;
//         let yy1 = dd;
//         let yy2 = 0.0;

//         let zz = 0.0;
//         let y = [
//             [xx1, yy1, zz],
//             [xx2, yy1, zz],
//             [xx1, yy2, zz],
//             [xx2, yy2, zz],
//         ];

//         positions.extend(y);

//         inds.extend(iii);

//         assert!(ascii >= 32);
//         counter += dd;
//     }

//     let normals = positions.iter().map(|_| [0.0, 0.0, 1.0]).collect();

//     let cc = 1.0 / dd;
//     let mm = matrix::scale(cc, cc, cc).generate();

//     let positions = positions
//         .into_iter()
//         .map(|a| mm.transform_point(a.into()).into())
//         .collect();

//     model::ModelData {
//         positions,
//         tex_coords,
//         indices: Some(inds),
//         normals,
//         matrix: mm,
//     }
// }
