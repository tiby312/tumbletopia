use super::*;

//TODO use this?
pub trait MoveCost {
    fn foop(&self, a: GridCoord, a: MoveUnit) -> MoveUnit;

    fn chain<B: MoveCost>(self, other: B) -> Chain<Self, B>
    where
        Self: Sized,
    {
        Chain { a: self, b: other }
    }
}

pub struct Chain<A, B> {
    a: A,
    b: B,
}
impl<A: MoveCost, B: MoveCost> MoveCost for Chain<A, B> {
    fn foop(&self, g: GridCoord, z: MoveUnit) -> MoveUnit {
        let a = self.a.foop(g, z);
        self.b.foop(g, a)
    }
}



pub trait MoveStrat{
    fn process(&self,a:MoveUnit)->MoveUnit;
}

impl<F:Fn(MoveUnit)->MoveUnit> MoveStrat for F{
    fn process(&self,a:MoveUnit)->MoveUnit {
        (self)(a)
    }
}


pub struct TerrainCollection<F> {
    pub pos: Vec<GridCoord>,
    pub func: F,
}
impl<F> TerrainCollection<F> {
    pub fn find_mut(&mut self, a: &GridCoord) -> Option<&mut GridCoord> {
        self.pos.iter_mut().find(|b| *b == a)
    }
    pub fn foo(&self) -> TerrainCollectionFoo<F> {
        TerrainCollectionFoo {
            a: &self.pos,
            func: &self.func,
        }
    }
}

pub struct TerrainCollectionFoo<'a, F> {
    a: &'a [GridCoord],
    func: &'a F,
}
impl<'a, F: MoveStrat> MoveCost for TerrainCollectionFoo<'a, F> {
    fn foop(&self, g: GridCoord, z: MoveUnit) -> MoveUnit {
        if self.a.contains(&g) {
            self.func.process(z)
        } else {
            z
        }
    }
}

pub struct Grass;
impl MoveCost for Grass {
    fn foop(&self, _: GridCoord, z: MoveUnit) -> MoveUnit {
        z
    }
}

//There is a base 2 cost movement everywhere.

//roads subtract 1

//mountains add cost
