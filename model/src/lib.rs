pub mod matrix;
use gltf::image::Source;
use image::imageops::FilterType;

#[macro_export]
macro_rules! mauga {
    ($a:expr)=>{
        ($a).generate()
    };
    ( $a:expr,$( $x:expr ),* ) => {
        {

            let mut a=$a;
            $(

                let k=$x;
                let a={
                    use $crate::matrix::MyMatrix;
                    a.chain(k)
                };

            )*
            a.generate()
        }
    };
}

#[derive(Debug)]
pub struct Doop {
    pub document: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images: Vec<gltf::image::Data>,
}

//TODO wouldnt it be amazing if this was a const function????
pub fn load_glb(bytes: &[u8]) -> Doop {
    //Use https://www.gltfeditor.com/ also
    //Use https://gltf.report/ to compress it to the binary format!!!!

    //TODO discard normal verticies if not used???

    let (document, buffers, images) = gltf::import_slice(bytes).unwrap();
    //log!(format!("{:?}", document));
    Doop {
        document,
        buffers,
        images,
    }
}

#[derive(Debug)]
pub struct Img {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

pub fn single_tex() -> Img {
    Img {
        width: 1,
        height: 1,
        data: vec![0, 0, 255, 255],
    }
}

#[derive(Debug)]
pub struct ModelData {
    pub matrix: glam::f32::Mat4,
    pub positions: Vec<[f32; 3]>,
    pub indices: Option<Vec<u16>>,
    pub normals: Vec<[f32; 3]>,
    pub tex_coords: Vec<[f32; 2]>,
}

impl Doop {
    pub fn gen_ext(&self, ss: f32, foo: usize, custom_alpha: Option<f64>) -> (ModelData, Img) {
        use matrix::*;
        use std::f32::consts::PI;
        let (mut m, tex) = self.gen(foo, custom_alpha);

        let v = ss;
        let s = matrix::translate(0.0, 0.0, 0.0)
            .chain(rotate_x(PI / 2.0))
            .chain(matrix::scale(v, v, v))
            .generate();

        for p in m.positions.iter_mut() {
            *p = s.transform_point3((*p).into()).into();
        }

        let kk = rotate_x(PI / 2.0).generate();

        for p in m.normals.iter_mut() {
            *p = kk.transform_point3((*p).into()).into();
        }

        // let s = matrix::translation(-v / 2.0, -v / 2.0, 0.0).generate();
        // for p in m.positions.iter_mut() {
        //     *p = s.transform_point((*p).into()).into();
        // }

        (m, tex)
    }
    //TODO return a read only reference instead!
    pub fn gen(&self, foo: usize, custom_alpha: Option<f64>) -> (ModelData, Img) {
        // TODO use this: https://www.nayuki.io/page/png-file-chunk-inspector
        let mut positions = Vec::new();
        let mut indices = Vec::new();
        let mut offset = 0;
        let mut tex_coords = Vec::new();
        let mut normals = Vec::new();

        let texture = if let Some(texture) = self.document.textures().next() {
            //log!("found a texture!");
            let g_img = texture.source();

            let buffers = &self.buffers;
            match g_img.source() {
                Source::View { view, .. } => {
                    let parent_buffer_data = &buffers[view.buffer().index()].0;
                    let data = &parent_buffer_data[view.offset()..view.offset() + view.length()];
                    //log!(format!("{:?}",data));

                    //data is in png format, need to decode it to rgba pixels for webgl

                    //use image::GenericImageView;
                    let image =
                        image::load_from_memory_with_format(data, image::ImageFormat::Png).unwrap();

                    //unpremultiply here?

                    let width = image.width();
                    let height = image.height();
                    //TODO pass as argument
                    let image =
                        image.resize(width * foo as u32, height * foo as u32, FilterType::Nearest);

                    let width = image.width();
                    let height = image.height();
                    let mut rgba_image = image.to_rgba8();

                    if let Some(custom_alpha) = custom_alpha {
                        for a in rgba_image.pixels_mut() {
                            let mut alpha = a.0[3] as f64 / 256.0;
                            alpha *= custom_alpha;
                            a.0[3] = (alpha * 256.0) as u8;
                        }
                    }

                    // greyscale
                    // for a in rgba_image.pixels_mut() {

                    //     let r=a.0[0] as f64 /256.0;
                    //     let g=a.0[1] as f64 /256.0;
                    //     let b=a.0[2] as f64 /256.0;

                    //     let coll =  0.299 * r + 0.587 * g + 0.114 * b;

                    //     a.0[0] = (coll * 256.0) as u8;
                    //     a.0[1] = (coll * 256.0) as u8;
                    //     a.0[2] = (coll * 256.0) as u8;

                    // }

                    let data = rgba_image.into_raw();

                    Img {
                        width,
                        height,
                        data,
                    }
                }
                _ => {
                    panic!("not supported")
                }
            }
        } else {
            panic!("no texture!");
        };

        for mesh in self.document.meshes() {
            for p in mesh.primitives() {
                //only support triangles
                assert_eq!(p.mode(), gltf::mesh::Mode::Triangles);

                let reader = p.reader(|buffer| Some(&self.buffers[buffer.index()]));

                let p: Vec<_> = reader.read_positions().unwrap().collect();

                let i: Vec<_> = reader
                    .read_indices()
                    .unwrap()
                    .into_u32()
                    .map(|x| offset + (x as u16))
                    .collect();

                if let Some(t) = reader.read_tex_coords(0) {
                    tex_coords.extend(t.into_f32().map(|[x, y]| [x, y]))
                } else {
                    //if texture.is_some(){
                    panic!("no texture coords!");
                    //}
                };

                if let Some(t) = reader.read_normals() {
                    normals.extend(t);
                } else {
                    panic!("no normals!");
                }
                //log!(format!("pos:{:?}", &p));

                //log!(format!("ind:{:?}", &i));

                offset += p.len() as u16;
                positions.extend(p);

                indices.extend(i);
            }
        }

        let node = self.document.nodes().next().unwrap();

        let matrix = glam::f32::Mat4::from_cols_array_2d(&node.transform().matrix());

        // log!(format!("mat:    {:?}",node.transform().matrix()));

        // let (t,r,s)=node.transform().decomposed();

        // let rot={
        //     let a:&cgmath::Quaternion<f32>=(&r).into();
        //     log!(format!("quart:    {:?}",r));
        //     let a=*a;
        //     let rot:cgmath::Matrix4<f32>=a.into();
        //     rot
        // };

        // let t={
        //     let a:&cgmath::Vector3<f32>=(&t).into();
        //     matrix::translation(a.x, a.y, a.z)
        // };

        // let s={
        //     let a:&cgmath::Vector3<f32>=(&s).into();
        //     matrix::scale(a.x, a.y, a.z)
        // };

        // use matrix::*;
        // let matrix=s.chain(t).chain(rot).generate();// rot.chain(t).chain(s).generate();
        //let cc = |x: &f32, y: &f32| x.partial_cmp(y).unwrap();

        // let min = [
        //     positions.iter().map(|x| x[0]).min_by(cc).unwrap(),
        //     positions.iter().map(|x| x[1]).min_by(cc).unwrap(),
        //     positions.iter().map(|x| x[2]).min_by(cc).unwrap(),
        // ];

        // let max = [
        //     positions.iter().map(|x| x[0]).max_by(cc).unwrap(),
        //     positions.iter().map(|x| x[1]).max_by(cc).unwrap(),
        //     positions.iter().map(|x| x[2]).max_by(cc).unwrap(),
        // ];

        let positions = positions
            .into_iter()
            .map(|p| matrix.transform_point3(p.into()).into())
            .collect();

        let normals = normals
            .into_iter()
            .map(|p| matrix.transform_point3(p.into()).into())
            .collect();

        (
            ModelData {
                normals,
                matrix,

                positions,
                indices: Some(indices),
                tex_coords,
            },
            texture,
        )
    }
}

pub fn load_texture_from_data(data: &[u8]) -> Img {
    //use image::GenericImageView;
    let image = image::load_from_memory_with_format(data, image::ImageFormat::Png).unwrap();
    let width = image.width();
    let height = image.height();

    let rgba_image = image.to_rgba8();
    let data = rgba_image.into_raw();

    Img {
        width,
        height,
        data,
    }
}
