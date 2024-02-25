use crate::moves::UndoInformation;

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
    start: GridCoord,
    target: GridCoord,
    v: &grids::GridMatrix,
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
        unit: UnitData,
        mesh: movement::RelativeMesh,
        walls: movement::movement_mesh::Mesh,
        end: GridCoord,
        data: UndoInformation,
    },
    Terrain {
        pos: GridCoord,
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
    Mountain,
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

pub fn land_delta(start: GridCoord, end: GridCoord, v: &grids::GridMatrix) -> Vector2<f32> {
    let s = v.hex_axial_to_world(&start);
    let e = v.hex_axial_to_world(&end);
    e - s
}
pub fn movement(
    start: GridCoord,
    path: movement::RelativeMesh,
    walls: movement::movement_mesh::Mesh,
    end: GridCoord,
    v: &grids::GridMatrix,
) -> impl Iterator<Item = Vector2<f32>> {
    let v = v.clone();
    let mut counter = v.hex_axial_to_world(&start);
    let mut cc = start;
    path.path(end.sub(&start), &walls).flat_map(move |m| {
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
