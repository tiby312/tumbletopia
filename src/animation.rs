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
        mesh: movement::MovementMesh,
        walls: movement::movement_mesh::Mesh,
        end: GridCoord,
    },
    Terrain {
        pos: GridCoord,
        terrain_type: TerrainType,
    },
}

#[derive(Debug, Clone)]
enum TerrainType {
    Snow,
    Grass,
    Mountain,
}

pub fn movement(
    start: GridCoord,
    path: movement::MovementMesh,
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

//TODO replace box with existental type
pub struct Animation<T> {
    it: Box<dyn Iterator<Item = Vector2<f32>>>,
    data: T,
}
use std::fmt;
impl<T> fmt::Debug for Animation<T> {
    fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}
impl<T> Animation<T> {
    pub fn new(it: impl Iterator<Item = Vector2<f32>> + 'static, data: T) -> Self {
        Self {
            it: Box::new(it),
            data,
        }
    }
    pub fn animate_step(&mut self) -> Option<cgmath::Vector2<f32>> {
        if let Some(x) = self.it.next() {
            Some(x.into())
        } else {
            None
        }
    }

    pub fn data(&self) -> &T {
        &self.data
    }
    pub fn into_data(self) -> T {
        self.data
    }
}
