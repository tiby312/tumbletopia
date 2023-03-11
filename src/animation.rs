use super::*;

#[derive(Debug, Clone)]
struct Doop {
    current: Vector2<f32>,
    next: Vector2<f32>,
    dir: Vector2<f32>,
    distance_to_next: f32,
}

impl Doop {
    fn new(current: Vector2<f32>, next: Vector2<f32>) -> Self {
        let distance_to_next = (next - current).magnitude();
        let dir = (next - current).normalize();
        Doop {
            current,
            next,
            dir,
            distance_to_next,
        }
    }
    fn distance_to_next(&self) -> f32 {
        self.distance_to_next
    }
    fn lerp(&self, val: f32) -> Vector2<f32> {
        self.current + self.dir * val
    }
}
#[derive(Debug, Clone)]
pub struct Animation<T> {
    points: std::vec::IntoIter<Vector2<f32>>,
    doop: Doop,
    curr: f32,
    data: T,
}
impl<T> Animation<T> {
    pub fn new(start: GridCoord, path: &movement::Path, v: &grids::GridMatrix, data: T) -> Self {
        let first: [f32; 2] = v.hex_axial_to_world(&start).into();
        let first = first.into();

        let mut points = vec![first];
        let mut cc = start;
        for m in path.get_moves() {
            let a = m.to_relative();
            cc.0[0] += a.0[0];
            cc.0[1] += a.0[1];
            let k: [f32; 2] = v.hex_axial_to_world(&cc).into();
            points.push(k.into());
        }

        let mut points = points.into_iter();
        let next: Vector2<f32> = points.next().unwrap();

        Animation {
            data,
            points,
            doop: Doop::new(first, next),
            curr: 0.0,
        }
    }
    pub fn calc_pos(&self) -> [f32; 2] {
        self.doop.lerp(self.curr).into()
    }
    pub fn animate_step(&mut self) -> Option<[f32; 2]> {
        let tt = 0.2;
        let max_tween = 4.0;
        self.curr += ((self.doop.distance_to_next() - self.curr) * tt).min(max_tween);

        if self.curr > self.doop.distance_to_next() - tt {
            let Some(new_next)=self.points.next() else{
                        return None;
                    };

            let extra = self.curr - self.doop.distance_to_next();

            let new_curr = self.doop.next;

            self.doop = Doop::new(new_curr, new_next);
            self.curr = extra;
        }

        Some(self.doop.lerp(self.curr).into())
    }
    pub fn into_data(self) -> T {
        self.data
    }
}
