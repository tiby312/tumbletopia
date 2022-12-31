#[derive(PartialEq, Debug)]
enum Scrollin {
    MouseDown {
        mouse_anchor: Vec2<f32>,
        camera_anchor: Vec2<f32>,
    },
    Scrolling {
        mouse_anchor: Vec2<f32>,
        camera_anchor: Vec2<f32>,
    },
    NotScrolling,
}
use super::*;
pub struct ScrollController {
    pub cursor_canvas: Vec2<f32>,
    //world coord
    pub camera: Vec2<f32>,
    last_camera: Vec2<f32>,

    scrolling: Scrollin,
    camera_data: CamData,
}

//angle of camera/zoom etc
struct CamData;

impl CamData {
    fn canvas_to_world(&self, a: Vec2<f32>) -> Vec2<f32> {
        a
    }
}

impl ScrollController {
    pub fn new(cursor_canvas: Vec2<f32>) -> Self {
        ScrollController {
            camera: vec2same(0.0),
            last_camera: vec2same(0.0),
            cursor_canvas,
            scrolling: Scrollin::NotScrolling,
            camera_data: CamData,
        }
    }
    pub fn world_cursor(&self, dim: &[f32; 2]) -> Vec2<f32> {
        //get cursor in world coordinates relative to origin.
        let cursor = self.camera_data.canvas_to_world(self.cursor_canvas);

        //log!(format!("{:?}",(self.camera,cursor)));
        //get abosolute position by adding it to where the camera is
        let point = -self.camera + cursor;

        let mut id = Mat4::identity();

        // let mut k=Mat4::identity();
        // Doop(&mut k).x_rotation(std::f32::consts::PI / 4.);
        // k.inverse().unwrap_throw();

        use webgl_matrix::prelude::*;
        use super::matrix::*;
        id.mul(&super::matrix::z_rotation(std::f32::consts::PI / 4.).generate());
        //z_rotation(std::f32::consts::PI / 4.);

        //translation(-dim[0] / 2., -dim[1] / 2., 0.0);

        let vec = [point.x, point.y, 0.0];
        let ans = id.mul_vector(&vec);

        vec2(ans[0], ans[1])
    }
    
    pub fn camera_pos(&self) -> &Vec2<f32> {
        &self.camera
    }

    pub fn handle_mouse_move(&mut self, mouse: [f32; 2]) {
        self.cursor_canvas = mouse.into();

        match self.scrolling {
            Scrollin::Scrolling {
                mouse_anchor,
                camera_anchor,
            } => {
                let offset = self.cursor_canvas - mouse_anchor;
                self.last_camera = self.camera;
                self.camera = camera_anchor + self.camera_data.canvas_to_world(offset);
            }
            Scrollin::MouseDown {
                mouse_anchor,
                camera_anchor,
            } => {
                self.scrolling = Scrollin::Scrolling {
                    mouse_anchor,
                    camera_anchor,
                }
            }
            _ => {}
        }
    }
    pub fn handle_mouse_down(&mut self, mouse: [f32; 2]) {
        self.cursor_canvas = mouse.into();

        self.scrolling = Scrollin::MouseDown {
            mouse_anchor: self.cursor_canvas,
            camera_anchor: self.camera,
        };
    }

    //Return true if a regular tap is detected.
    pub fn handle_mouse_up(&mut self) -> bool {
        match self.scrolling {
            Scrollin::MouseDown { .. } => {
                self.scrolling = Scrollin::NotScrolling;
                true
            }
            Scrollin::Scrolling { .. } => {
                self.scrolling = Scrollin::NotScrolling;
                false
            }
            Scrollin::NotScrolling => {
                panic!("not possible?")
            }
        }
    }
    pub fn step(&mut self) {
        match self.scrolling {
            Scrollin::Scrolling { .. } => {}
            _ => {
                let delta = self.camera - self.last_camera;
                self.last_camera = self.camera;
                self.camera += delta * 0.9;
            }
        }
    }
}
