use crate::move_build::PushInfo;

use super::*;

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

        if (self.curr - self.target).abs() < self.tt {
            return None;
        }
        Some(self.curr)
    }
}

pub fn attack(
    start: Axial,
    target: Axial,
    v: &grids::HexConverter,
) -> impl Iterator<Item = Vector2<f32>> {
    let v = v.clone();
    let start = v.hex_axial_to_world(&start);
    let end = v.hex_axial_to_world(&target);
    let dir = (end - start).normalize();
    let dis = v.spacing() / 3.0;

    Interpolate {
        curr: 0.0,
        target: dis,
        tt: 0.2,
        max: 4.0,
    }
    .chain(Interpolate {
        curr: dis,
        target: 0.0,
        tt: 0.2,
        max: 4.0,
    })
    .map(move |val| start + dir * val)
}

#[derive(Debug, Clone)]
pub enum AnimationCommand {
    Movement {
        unit: Axial,
        end: Axial,
        path: mesh::MyPath,
        data: PushInfo,
    },
    Terrain {
        pos: Axial,
        terrain_type: TerrainType,
        dir: AnimationDirection,
    },
}

#[derive(Debug, Clone)]
pub enum AnimationDirection {
    Up,
    Down,
}

#[derive(Debug, Clone)]
pub enum TerrainType {
    Grass,
    Fog,
}

pub fn terrain_create(curr: f32, target: f32) -> impl Iterator<Item = f32> {
    Interpolate {
        curr,
        target,
        tt: 0.2,
        max: 4.0,
    }
}

// pub fn terrain_create_down() -> impl Iterator<Item = f32> {
//     Interpolate {
//         curr: 0.0,
//         target: -10.0,
//         tt: 0.2,
//         max: 4.0,
//     }
// }

pub fn land_delta(start: Axial, end: Axial, v: &grids::HexConverter) -> Vector2<f32> {
    let s = v.hex_axial_to_world(&start);
    let e = v.hex_axial_to_world(&end);
    e - s
}

impl mesh::MyPath {
    pub fn animation_iter(
        self,
        start: Axial,
        v: &grids::HexConverter,
    ) -> impl Iterator<Item = Vector2<f32>> {
        let v = v.clone();
        let mut counter = v.hex_axial_to_world(&start);
        let mut cc = start;

        self.into_iter().flat_map(move |m| {
            let a = m.to_relative();
            cc.q += a.q;
            cc.r += a.r;
            let k = v.hex_axial_to_world(&cc);
            let dis = (k - counter).magnitude();
            let dir = (k - counter).normalize();
            let old = counter;
            counter = k;

            animation::Interpolate {
                curr: 0.0,
                target: dis,
                tt: 0.2,
                max: 4.0,
            }
            .map(move |val| old + dir * val)
        })
    }
}

// pub fn movement(
//     start: Axial,
//     path: mesh::small_mesh::SmallMesh,
//     walls: mesh::small_mesh::SmallMesh,
//     end: Axial,
//     game: &GameState,
//     team: ActiveTeam,
//     world: &board::MyWorld,
//     v: &grids::HexConverter,
// ) -> impl Iterator<Item = Vector2<f32>> {
//     let v = v.clone();
//     let mut counter = v.hex_axial_to_world(&start);
//     let mut cc = start;

//     let capturing = game.factions.has_a_set(end);

//     mesh::path(&path, start, end, &walls, game, team, world, capturing)
//         .into_iter()
//         .flat_map(move |m| {
//             let a = m.to_relative();
//             cc.q += a.q;
//             cc.r += a.r;
//             let k = v.hex_axial_to_world(&cc);
//             let dis = (k - counter).magnitude();
//             let dir = (k - counter).normalize();
//             let old = counter;
//             counter = k;

//             Interpolate {
//                 curr: 0.0,
//                 target: dis,
//                 tt: 0.2,
//                 max: 4.0,
//             }
//             .map(move |val| old + dir * val)
//         })
// }
