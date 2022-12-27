#[derive(PartialEq, Debug)]
enum Scrollin {
    MouseDown { anchor: Vec2<f32> },
    Scrolling { anchor: Vec2<f32> },
    NotScrolling,
}
use super::*;
pub struct ScrollController {
    mouse_pos_canvas: Vec2<f32>,
    camera: Vec2<f32>,
    camera_velocity: Vec2<f32>,
    world_cursor: Vec2<f32>,
    scrolling: Scrollin,
}
impl ScrollController {
    pub fn new(mouse_pos_canvas: Vec2<f32>) -> Self {
        ScrollController {
            mouse_pos_canvas,
            camera: mouse_pos_canvas,
            camera_velocity: vec2same(0.0),
            world_cursor: vec2same(0.0),
            scrolling: Scrollin::NotScrolling,
        }
    }
    pub fn world_cursor(&self) -> &Vec2<f32> {
        &self.world_cursor
    }
    pub fn camera_pos(&self) -> &Vec2<f32> {
        &self.camera
    }

    pub fn handle_mouse_move(&mut self, mouse: [f32; 2]) {
        self.mouse_pos_canvas = mouse.into();
        self.world_cursor = self.camera + self.mouse_pos_canvas.into();

        match self.scrolling {
            Scrollin::MouseDown { anchor } | Scrollin::Scrolling { anchor } => {
                let curr = self.mouse_pos_canvas;
                let anchor: Vec2<_> = anchor.into();
                self.camera_velocity += (curr - anchor) * 0.2;
                self.scrolling = Scrollin::Scrolling {
                    anchor: self.mouse_pos_canvas,
                }
            }
            _ => {}
        }
    }
    pub fn handle_mouse_down(&mut self) {
        self.scrolling = Scrollin::MouseDown {
            anchor: self.mouse_pos_canvas,
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
        self.world_cursor = -self.camera + self.mouse_pos_canvas.into();
        //log!(format!("{:?}",&scrolling));

        {
            self.camera += self.camera_velocity;
            self.camera_velocity *= 0.9;
        }
    }
}
