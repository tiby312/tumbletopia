use gloo::console::log;

pub mod matrix;
use cgmath::Transform;
use gltf::image::Source;

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

pub struct ModelData {
    pub matrix: cgmath::Matrix4<f32>,
    pub positions: Vec<[f32; 3]>,
    pub indices: Option<Vec<u16>>,
    pub texture: Img,
    // pub texture:Vec<u8>,
    // pub texture_width:u32,
    // pub texture_height:u32,
    pub tex_coords: Vec<[f32; 2]>,
}

impl Doop {
    //TODO return a read only reference instead!
    pub fn gen(&self) -> ModelData {
        // TODO use this: https://www.nayuki.io/page/png-file-chunk-inspector
        let mut positions = Vec::new();
        let mut indices = Vec::new();
        let mut offset = 0;
        let mut tex_coords = Vec::new();

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

                    use image::GenericImageView;
                    let image =
                        image::load_from_memory_with_format(data, image::ImageFormat::Png).unwrap();
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
                    tex_coords.extend(t.into_f32())
                } else {
                    //if texture.is_some(){
                    panic!("no texture coords!");
                    //}
                };
                //log!(format!("pos:{:?}", &p));

                //log!(format!("ind:{:?}", &i));

                offset += p.len() as u16;
                positions.extend(p);

                indices.extend(i);
            }
        }

        let node = self.document.nodes().next().unwrap();

        let matrix: cgmath::Matrix4<f32> = node.transform().matrix().into();

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

        let positions = positions
            .into_iter()
            .map(|p| matrix.transform_point(p.into()).into())
            .collect();

        use cgmath::SquareMatrix;
        ModelData {
            matrix,
            texture,
            positions,
            indices: Some(indices),
            tex_coords,
        }
    }
}
