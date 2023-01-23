use super::*;

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
pub struct Animation {
    points: std::vec::IntoIter<Vector2<f32>>,
    doop: Doop,
    curr: f32,
}
impl Animation {
    pub fn new(start: &GridCoord, path: &movement::Path, v: &grids::GridMatrix) -> Self {
        let first: [f32; 2] = v.to_world_topleft(start.0.into()).into();
        let first = first.into();

        let mut points = vec![first];
        let mut cc = *start;
        for m in path.get_moves() {
            let a = m.to_relative();
            cc.0[0] += a.0[0];
            cc.0[1] += a.0[1];
            let k: [f32; 2] = v.to_world_topleft(cc.0.into()).into();
            points.push(k.into());
        }

        let mut points = points.into_iter();
        let next: Vector2<f32> = points.next().unwrap();

        Animation {
            points,
            doop: Doop::new(first, next),
            curr: 0.0,
        }
    }
    pub fn animate_step(&mut self) -> Option<[f32; 2]> {
        let tt=0.1;
        self.curr += (self.doop.distance_to_next()-self.curr)*tt;

        if self.curr > self.doop.distance_to_next()-tt {
            let Some(new_next)=self.points.next() else{
                        return None;
                    };

            let extra=self.curr-self.doop.distance_to_next();

            let new_curr = self.doop.next;

            self.doop = Doop::new(new_curr, new_next);
            self.curr = extra;
        }
        
        Some(self.doop.lerp(self.curr).into())
    }
}
