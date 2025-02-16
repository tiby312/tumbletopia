use cgmath::Transform;
use cgmath::{InnerSpace, Vector2};
use serde::Deserialize;
use serde::Serialize;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Touches {
    pub all: [(i32, f32, f32); 4],
    pub count: usize,
}
impl Touches {
    //TODO return reference
    pub fn get_pos(&self, a: i32) -> Option<[f32; 2]> {
        self.all
            .iter()
            .take(self.count)
            .find(|&b| b.0 == a)
            .map(|a| [a.1, a.2])
    }
    pub fn select_lowest_touch(&self) -> Option<i32> {
        self.all
            .iter()
            .take(self.count)
            .min_by_key(|a| a.0)
            .map(|a| a.0)
    }
    pub fn select_lowest_touch_excluding(&self, b: i32) -> Option<i32> {
        self.all
            .iter()
            .take(self.count)
            .filter(|a| a.0 != b)
            .min_by_key(|a| a.0)
            .map(|a| a.0)
    }
}

enum Foo {
    OneTouchActive {
        touch_id: i32,
    },
    TwoTouchActive {
        rot: RotDelta,
        zoom: ZoomDelta,
        first_touch_id: i32,
        second_touch_id: i32,
    },
    MouseActive {
        _canvas_pos: [f32; 2],
    },
    None,
}

#[derive(Copy, Clone)]
struct RotDelta {
    starting_rot: f32,
    current_rot: f32,
}

impl RotDelta {
    fn compute(&self) -> f32 {
        self.starting_rot - self.current_rot
    }
    fn update(&mut self, a: f32) {
        self.current_rot = a;
    }
}
#[derive(Copy, Clone)]
struct ZoomDelta {
    starting_distance: f32,
    current_distance: f32,
}

impl ZoomDelta {
    fn compute(&self) -> f32 {
        self.starting_distance - self.current_distance
    }
    fn update(&mut self, a: f32) {
        self.current_distance = a;
    }
}

pub struct TouchController {
    inner: ScrollController,
    foo: Foo,
    persistent_zoom: f32,
    persistent_rot: f32,
}

fn compute_middle(touches: &Touches, first: i32, second: i32) -> (f32, [f32; 2], f32) {
    let first_pos: Vector2<f32> = touches.get_pos(first).unwrap().into();
    let second_pos: Vector2<f32> = touches.get_pos(second).unwrap().into();
    let offset = second_pos - first_pos;
    let rot = offset.x.atan2(offset.y);
    let dis = offset.magnitude();
    let middle = first_pos + offset / 2.0;
    (dis, middle.into(), rot)
}
const TOUCH_RAD: f32 = 10.0;
impl TouchController {
    pub fn new(camera: Vector2<f32>) -> Self {
        let inner = ScrollController::new(camera);
        TouchController {
            inner,
            foo: Foo::None,
            persistent_zoom: 0.0,
            persistent_rot: 0.0,
        }
    }

    pub fn on_mouse_down(&mut self, canvas_pos: [f32; 2]) {
        match self.foo {
            Foo::None => {
                self.inner.handle_mouse_down(canvas_pos);
                self.foo = Foo::MouseActive {
                    _canvas_pos: canvas_pos,
                }
            }
            _ => {}
        }
    }
    pub fn on_mouse_up(&mut self) -> MouseUp {
        match self.foo {
            Foo::MouseActive { .. } => {
                self.foo = Foo::None;
                self.inner.handle_mouse_up()
            }
            _ => MouseUp::NoSelect,
        }
    }
    pub fn on_mouse_move(
        &mut self,
        pos: [f32; 2],
        view_projection: &cgmath::Matrix4<f32>,
        dim: [f32; 2],
    ) {
        match self.foo {
            Foo::MouseActive { .. } => {
                self.inner
                    .handle_mouse_move(TOUCH_RAD, pos, view_projection, dim);
            }
            _ => {}
        }
    }

    pub fn on_new_touch(&mut self, touches: &Touches) {
        match self.foo {
            Foo::OneTouchActive { touch_id } => {
                if let Some(second_touch_id) = touches.select_lowest_touch_excluding(touch_id) {
                    assert!(touches.get_pos(touch_id).is_some());
                    assert!(touches.get_pos(second_touch_id).is_some());

                    let (dis, middle, rot) = compute_middle(touches, touch_id, second_touch_id);

                    //we don't want to propogate this click to the user.
                    let _ = self.inner.handle_mouse_up();
                    self.inner.handle_mouse_down(middle);

                    self.foo = Foo::TwoTouchActive {
                        zoom: ZoomDelta {
                            starting_distance: dis,
                            current_distance: dis,
                        },
                        rot: RotDelta {
                            starting_rot: rot,
                            current_rot: rot,
                        },
                        first_touch_id: touch_id,
                        second_touch_id,
                    }
                } else {
                    //TODO what to do here???
                    //This case rarely happens
                }
            }
            Foo::TwoTouchActive { .. } => {
                //ignore new touches. do nothing.
            }
            Foo::None => {
                //Guarenteed to exist because this function is called on new touch.
                let touch_id = touches.select_lowest_touch().unwrap();

                let mouse = touches.get_pos(touch_id).unwrap();
                self.inner.handle_mouse_down(mouse);
                //found one touch!
                //select one touch id.
                //find position
                self.foo = Foo::OneTouchActive { touch_id };
            }
            Foo::MouseActive { .. } => {
                //ignore touch mouse is active
            }
        }
    }

    pub fn on_touch_move(
        &mut self,
        touches: &Touches,
        view_projection: &cgmath::Matrix4<f32>,
        dim: [f32; 2],
    ) {
        match self.foo {
            Foo::OneTouchActive { touch_id } => {
                let mouse = touches.get_pos(touch_id).unwrap();
                self.inner
                    .handle_mouse_move(TOUCH_RAD, mouse, view_projection, dim);
            }
            Foo::TwoTouchActive {
                mut zoom,
                mut rot,
                first_touch_id,
                second_touch_id,
            } => {
                assert!(touches.get_pos(first_touch_id).is_some());
                assert!(touches.get_pos(second_touch_id).is_some());

                let (dis, middle, r) = compute_middle(touches, first_touch_id, second_touch_id);
                self.inner
                    .handle_mouse_move(0.0, middle, view_projection, dim);
                zoom.update(dis);
                rot.update(r);
                self.foo = Foo::TwoTouchActive {
                    zoom,
                    rot,
                    first_touch_id,
                    second_touch_id,
                }
            }
            Foo::None => {
                //A touch moved that we don't care about.
            }
            Foo::MouseActive { .. } => {
                //ignore touch mouse active
            }
        }
    }

    #[must_use]
    pub fn on_touch_up(&mut self, touches: &Touches) -> MouseUp {
        match self.foo {
            Foo::OneTouchActive { touch_id } => {
                if touches.get_pos(touch_id).is_none() {
                    self.foo = Foo::None;
                    self.inner.handle_mouse_up()
                } else {
                    MouseUp::NoSelect
                }
            }
            Foo::TwoTouchActive {
                zoom,
                rot,
                first_touch_id,
                second_touch_id,
            } => {
                let a = touches.get_pos(first_touch_id);
                let b = touches.get_pos(second_touch_id);

                match (a, b) {
                    (None, None) => {
                        //two touches got removed simultaneously
                        //don't propograte. otherwise it would click in the middle of both touches.
                        let _ = self.inner.handle_mouse_up();
                        self.foo = Foo::None;
                        self.persistent_rot += rot.compute();
                        self.persistent_zoom += zoom.compute();
                        MouseUp::NoSelect
                    }
                    (None, Some(pos)) => {
                        let _ = self.inner.handle_mouse_up();
                        self.inner.handle_mouse_down(pos);
                        // self.foo = Foo::OneTouchActive {
                        //     touch_id: second_touch_id,
                        // };
                        self.foo = Foo::None;
                        self.persistent_rot += rot.compute();
                        self.persistent_zoom += zoom.compute();
                        MouseUp::NoSelect
                    }
                    (Some(pos), None) => {
                        let _ = self.inner.handle_mouse_up();
                        self.inner.handle_mouse_down(pos);
                        // self.foo = Foo::OneTouchActive {
                        //     touch_id: first_touch_id,
                        // };
                        self.foo = Foo::None;
                        self.persistent_rot += rot.compute();
                        self.persistent_zoom += zoom.compute();
                        MouseUp::NoSelect
                    }
                    (Some(_), Some(_)) => {
                        //A touch we don't care about went up.
                        MouseUp::NoSelect
                    }
                }
            }
            Foo::None => {
                //Touch up for a touch we don't care about.
                MouseUp::NoSelect
            }
            Foo::MouseActive { .. } => {
                //ignore touch mosue active
                MouseUp::NoSelect
            }
        }
    }

    pub fn step(&mut self) {
        // used
        // https://www.gamedev.net/forums/topic/370644-angle-between-two-vectors/3442782/
        // not used
        // https://stackoverflow.com/questions/2708476/rotation-interpolation

        let r = self.persistent_rot;
        let v1 = Vector2::new(r.cos(), r.sin());
        let target = Vector2::new(1.0, 0.0);

        let perp_dot = -target.perp_dot(v1);
        let dot = target.dot(v1);
        let angle = perp_dot.atan2(dot);

        //let m=0.02;
        self.persistent_rot += angle * 0.05; //angle.clamp(-m,m);

        self.inner.step();
    }

    pub fn zoom(&self) -> f32 {
        let z = if let Foo::TwoTouchActive { zoom, .. } = &self.foo {
            zoom.compute()
        } else {
            0.0
        };
        (self.persistent_zoom + z) * 0.75
    }

    pub fn rot(&self) -> f32 {
        let z = if let Foo::TwoTouchActive { rot, .. } = &self.foo {
            rot.compute()
        } else {
            0.0
        };
        self.persistent_rot + z
    }

    //camera in world coordinates
    pub fn camera(&self) -> [f32; 2] {
        self.inner.camera()
    }

    pub fn cursor_canvas(&self) -> [f32; 2] {
        self.inner.cursor_canvas()
    }
}

pub enum MouseUp {
    /// This was a select
    Select,
    /// This was a scroll mouse up
    NoSelect,
}

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
pub struct ScrollController {
    //canvas coordinates
    cursor_canvas: Vector2<f32>,

    //world coord
    camera: Vector2<f32>,
    last_camera: Vector2<f32>,

    scrolling: Scrollin,
}

impl ScrollController {
    pub fn new(camera: Vector2<f32>) -> Self {
        ScrollController {
            camera,
            last_camera: camera,
            cursor_canvas: [0.0; 2].into(),
            scrolling: Scrollin::NotScrolling,
        }
    }

    pub fn camera(&self) -> [f32; 2] {
        [self.camera[0], self.camera[1]]
    }

    pub fn handle_mouse_move(
        &mut self,
        buffer_radius: f32,
        mouse: [f32; 2],
        view_projection: &cgmath::Matrix4<f32>,
        dim: [f32; 2],
    ) {
        self.cursor_canvas = mouse.into();

        match self.scrolling {
            Scrollin::Scrolling {
                mouse_anchor,
                camera_anchor,
            } => {
                let mouse_world1: Vector2<f32> =
                    mouse_to_world(self.cursor_canvas.into(), view_projection, dim).into();

                let mouse_world2: Vector2<f32> =
                    mouse_to_world(mouse_anchor.into(), view_projection, dim).into();

                let offset = mouse_world2 - mouse_world1;
                self.last_camera = self.camera;
                self.camera = camera_anchor + offset;
            }
            Scrollin::MouseDown {
                mouse_anchor,
                camera_anchor,
            } => {
                let a = self.cursor_canvas;
                let b = mouse_anchor;
                let offset = b - a;
                if offset.magnitude2() >= buffer_radius * buffer_radius {
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
            mouse_anchor: self.cursor_canvas,
            camera_anchor: self.camera,
        };
    }

    //Return true if a regular tap is detected.
    //TODO replace with enum!
    #[must_use]
    pub fn handle_mouse_up(&mut self) -> MouseUp {
        match self.scrolling {
            Scrollin::MouseDown { .. } => {
                self.scrolling = Scrollin::NotScrolling;
                MouseUp::Select
            }
            Scrollin::Scrolling { .. } => {
                self.scrolling = Scrollin::NotScrolling;
                MouseUp::NoSelect
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

    pub fn cursor_canvas(&self) -> [f32; 2] {
        self.cursor_canvas.into()
    }
}

//TODO don't do this every step, just when user clicks!!!
pub fn mouse_to_world(
    mouse: [f32; 2],
    view_projection: &cgmath::Matrix4<f32>,
    dim: [f32; 2],
) -> [f32; 2] {
    //generate some mouse points
    let clip_x = mouse[0] / dim[0] * 2. - 1.;
    let clip_y = mouse[1] / dim[1] * -2. + 1.;
    super::projection::clip_to_world([clip_x, clip_y], view_projection)
}

pub fn world_to_mouse(
    world: [f32; 2],
    dim: [f32; 2],
    view_projection: &cgmath::Matrix4<f32>,
) -> [f32; 2] {
    let p = view_projection.transform_point(cgmath::Point3 {
        x: world[0],
        y: world[1],
        z: 0.0,
    });
    let mouse_x = dim[0] * (p.x + 1.0) / 2.0;
    let mouse_y = dim[1] * (p.y - 1.0) / -2.0;
    [mouse_x, mouse_y]
}
