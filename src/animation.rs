use super::*;

// #[derive(Debug, Clone)]
// struct Doop {
//     current: Vector2<f32>,
//     next: Vector2<f32>,
//     dir: Vector2<f32>,
//     distance_to_next: f32,
// }

// impl Doop {
//     fn new(current: Vector2<f32>, next: Vector2<f32>) -> Self {
//         let distance_to_next = (next - current).magnitude();
//         let dir = (next - current).normalize();
//         Doop {
//             current,
//             next,
//             dir,
//             distance_to_next,
//         }
//     }
//     fn distance_to_next(&self) -> f32 {
//         self.distance_to_next
//     }
//     fn lerp(&self, val: f32) -> Vector2<f32> {
//         self.current + self.dir * val
//     }
// }

pub struct Interpolate {
    curr: f32,
    target: f32,
    tt: f32,
    max: f32,
}
impl Iterator for Interpolate {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        self.curr += ((self.target - self.curr) * self.tt).min(self.max);

        if self.curr > self.target - self.tt {
            return None;
        }
        Some(self.curr)
    }
}




fn doop(
    start: GridCoord,
    path: movement::Path,
    v: &grids::GridMatrix,
) -> impl Iterator<Item = Vector2<f32>> {
    let v = v.clone();
    let mut counter = v.hex_axial_to_world(&start);
    let mut cc = start;
    path.into_moves().flat_map(move |m| {
        let a = m.to_relative();
        cc.0[0] += a.0[0];
        cc.0[1] += a.0[1];
        let k = v.hex_axial_to_world(&cc);
        let dis = (k - counter).magnitude();
        let dir = (k - counter).normalize();
        let old = counter;
        counter = k;

        Interpolate {
            curr: 0.0,
            target: dis,
            tt: 0.2,
            max: 4.0,
        }
        .map(move |val| old + dir * val)
    })
}

//TODO replace box with existental type
pub struct Animation<T> {
    it: Box<dyn Iterator<Item = Vector2<f32>>>,
    data: T,
    last: Option<Vector2<f32>>,
}
use std::fmt;
impl<T> fmt::Debug for Animation<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}
impl<T> Animation<T> {
    pub fn new(start: GridCoord, path: &movement::Path, v: &grids::GridMatrix, data: T) -> Self {
        let it = doop(start, path.clone(), v);
        Self {
            it:Box::new(it),
            data,
            last: None,
        }
    }
    pub fn animate_step(&mut self) -> Option<[f32; 2]> {
        if let Some(x) = self.it.next() {
            self.last = Some(x);
            Some(x.into())
        } else {
            None
        }
    }
    pub fn into_data(self) -> T {
        self.data
    }
    pub fn calc_pos(&self) -> [f32; 2] {
        self.last.unwrap().into()
    }
}

// #[derive(Debug, Clone)]
// pub struct Animation2<T> {
//     points: std::vec::IntoIter<Vector2<f32>>,
//     doop: Doop,
//     curr: f32,
//     data: T,
// }
// impl<T> Animation2<T> {
//     pub fn new(start: GridCoord, path: &movement::Path, v: &grids::GridMatrix, data: T) -> Self {
//         let first: [f32; 2] = v.hex_axial_to_world(&start).into();
//         let first = first.into();

//         let mut points = vec![first];
//         let mut cc = start;
//         for m in path.get_moves() {
//             let a = m.to_relative();
//             cc.0[0] += a.0[0];
//             cc.0[1] += a.0[1];
//             let k: [f32; 2] = v.hex_axial_to_world(&cc).into();
//             points.push(k.into());
//         }

//         let mut points = points.into_iter();
//         let next: Vector2<f32> = points.next().unwrap();

//         Animation2 {
//             data,
//             points,
//             doop: Doop::new(first, next),
//             curr: 0.0,
//         }
//     }
//     pub fn calc_pos(&self) -> [f32; 2] {
//         self.doop.lerp(self.curr).into()
//     }
//     pub fn animate_step(&mut self) -> Option<[f32; 2]> {
//         let tt = 0.2;
//         let max_tween = 4.0;
//         self.curr += ((self.doop.distance_to_next() - self.curr) * tt).min(max_tween);

//         if self.curr > self.doop.distance_to_next() - tt {
//             let Some(new_next)=self.points.next() else{
//                         return None;
//                     };

//             let extra = self.curr - self.doop.distance_to_next();

//             let new_curr = self.doop.next;

//             self.doop = Doop::new(new_curr, new_next);
//             self.curr = extra;
//         }

//         Some(self.doop.lerp(self.curr).into())
//     }
//     pub fn into_data(self) -> T {
//         self.data
//     }
// }
