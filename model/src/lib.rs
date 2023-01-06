use gltf::image::Source;



pub struct Doop{
    document:gltf::Document,
    buffers:Vec<gltf::buffer::Data>,
    images:Vec<gltf::image::Data>
}

//TODO wouldnt it be amazing if this was a const function????
pub fn load_glb(bytes:&[u8])->Doop{
    //Use https://www.gltfeditor.com/ also
    //Use https://gltf.report/ to compress it to the binary format!!!!
    
    //TODO discard normal verticies if not used???


    let (document,buffers,images)=gltf::import_slice(bytes).unwrap();
    //log!(format!("{:?}", document));
    Doop { document, buffers, images }
}




pub struct ModelData<'a>{
    pub positions:Vec<[f32;3]>,
    pub indices:Option<Vec<u16>>,
    pub texture:&'a [u8],
    pub texture_width:u32,
    pub texture_height:u32,
    pub tex_coords:Vec<[f32;2]>
}


impl Doop{

    //TODO return a read only reference instead!
    pub fn gen(&self)->ModelData{
        // TODO use this: https://www.nayuki.io/page/png-file-chunk-inspector
        let mut positions=Vec::new();
        let mut indices=Vec::new();
        let mut offset=0;   
        let mut tex_coords=Vec::new();
        let texture=if let Some(texture)= self.document.textures().next(){

            //log!("found a texture!");
            let g_img = texture.source();
            
            let buffers = &self.buffers;
            match g_img.source() {
                Source::View { view, mime_type } => {
                    let parent_buffer_data = &buffers[view.buffer().index()].0;
                    let data = &parent_buffer_data[view.offset()..view.offset() + view.length()];
                    //log!(format!("{:?}",data));

                    //data is in png format, need to decode it to rgba pixels for webgl

                    use image::GenericImageView;
                    let image = image::load_from_memory_with_format(data, image::ImageFormat::Png).unwrap();
                    let width=image.width();
                    let height=image.height();

                    let rgba_image = image.to_rgba8();
                    let texture=rgba_image.into_raw();


                    (width,height,data)
                },
                _=>{panic!("not supported")}
            }
        }else{
            panic!("no texture!");
        };



        for mesh in self.document.meshes(){
            
            for p in mesh.primitives(){
                //only support triangles
                assert_eq!(p.mode(),gltf::mesh::Mode::Triangles);
                

                let reader = p.reader(|buffer| Some(&self.buffers[buffer.index()]));

                let p:Vec<_>=reader.read_positions().unwrap().collect();
                
                let i:Vec<_>=reader.read_indices().unwrap().into_u32().map(|x|offset+(x as u16)).collect();
                

                
                if let Some(t) = reader.read_tex_coords(0) {
                    tex_coords.extend(t.into_f32())
                } else {
                    //if texture.is_some(){
                        panic!("no texture coords!");
                    //}
                };
                //log!(format!("pos:{:?}", &p));

                //log!(format!("ind:{:?}", &i));
                
                offset+=p.len() as u16;
                positions.extend(p);

                indices.extend(i);


            } 
        };
        
        let (texture_width,texture_height,texture)=texture;
        ModelData { texture_width,texture_height,texture,positions,indices:Some(indices),tex_coords}
        
    }

}