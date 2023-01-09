use cgmath::{InnerSpace, Vector2};

#[derive(PartialEq, Debug)]
enum Scrollin {
    MouseDown {
        mouse_anchor: Vector2<f32>,
        camera_anchor: Vector2<f32>,
    },
    Scrolling {
        mouse_anchor: Vector2<f32>,
        camera_anchor: Vector2<f32>,
    },
    NotScrolling,
}
use super::*;
pub struct ScrollController {
    pub cursor_canvas: Vector2<f32>,
    //world coord
    camera: Vector2<f32>,
    last_camera: Vector2<f32>,

    scrolling: Scrollin,
}

impl ScrollController {
    pub fn new(camera: Vector2<f32>) -> Self {
        ScrollController {
            camera,
            last_camera: camera.into(),
            cursor_canvas: [0.0; 2].into(),
            scrolling: Scrollin::NotScrolling,
        }
    }

    pub fn camera(&self) -> [f32; 2] {
        [self.camera[0], self.camera[1]]
    }

    pub fn handle_mouse_move(&mut self, mouse: [f32; 2], viewport: [f32; 2]) {
        self.cursor_canvas = mouse.into();

        match self.scrolling {
            Scrollin::Scrolling {
                mouse_anchor,
                camera_anchor,
            } => {
                let mouse_world1: Vector2<f32> =
                    mouse_to_world(self.cursor_canvas.into(), camera_anchor.into(), viewport)
                        .into();

                let mouse_world2: Vector2<f32> =
                    mouse_to_world(mouse_anchor.into(), camera_anchor.into(), viewport).into();

                let offset = mouse_world2 - mouse_world1;
                self.last_camera = self.camera;
                self.camera = camera_anchor + offset;
            }
            Scrollin::MouseDown {
                mouse_anchor,
                camera_anchor,
            } => {
                let a = Vector2::from(self.cursor_canvas);
                let b = Vector2::from(mouse_anchor);
                let offset = b - a;
                if offset.magnitude2() > 10.0 * 10.0 {
                    self.scrolling = Scrollin::Scrolling {
                        mouse_anchor,
                        camera_anchor,
                    }
                }
            }
            _ => {}
        }
    }
    pub fn handle_mouse_down(&mut self, mouse: [f32; 2]) {
        self.cursor_canvas = mouse.into();

        self.scrolling = Scrollin::MouseDown {
            mouse_anchor: Vector2::from(self.cursor_canvas),
            camera_anchor: Vector2::from(self.camera),
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
                let delta = Vector2::from(self.camera) - Vector2::from(self.last_camera);
                self.last_camera = Vector2::from(self.camera);

                self.camera = (Vector2::from(self.camera) + delta * 0.9).into();
            }
        }
    }
}
