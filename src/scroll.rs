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
    cursor_canvas: Vec2<f32>,
    //world coord
    camera: Vec2<f32>,
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
    pub fn world_cursor(&self) -> Vec2<f32> {
        //get cursor in world coordinates relative to origin.
        let cursor = self.camera_data.canvas_to_world(self.cursor_canvas);

        //log!(format!("{:?}",(self.camera,cursor)));
        //get abosolute position by adding it to where the camera is
        -self.camera + cursor
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
