use crate::movement::FilterRes;

use super::*;

#[derive(Eq, PartialEq, Hash, Debug, Clone, Default)]
pub struct UnitData {
    pub position: GridCoord,
    pub typ: Type,
    pub direction: movement::HexDir,
}

impl UnitData {
    pub fn new(position: GridCoord, typ: Type, direction: movement::HexDir) -> Self {
        UnitData {
            position,
            typ,
            direction,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CellSelection {
    MoveSelection(GridCoord, movement::MovementMesh),
    BuildSelection(GridCoord),
}

#[derive(Hash, Default, Debug, Clone, Copy, Eq, PartialEq)]
pub enum Type {
    #[default]
    Warrior,
    King,
    Archer,
    Catapault,
    Lancer,
    Spotter,
}

impl Type {
    pub fn type_index(&self) -> usize {
        let a = self;
        match a {
            Type::Warrior => 0,
            Type::King => 1,
            Type::Archer => 2,
            Type::Catapault => 3,
            Type::Lancer => 4,
            Type::Spotter => 5,
        }
    }

    pub fn type_index_inverse(a: usize) -> Type {
        match a {
            0 => Type::Warrior,
            1 => Type::King,
            2 => Type::Archer,
            3 => Type::Catapault,
            4 => Type::Lancer,
            5 => Type::Spotter,
            _ => unreachable!(),
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Tribe {
    pub units: smallvec::SmallVec<[UnitData; 6]>,
}
impl Tribe {
    pub fn add(&mut self, a: UnitData) {
        self.units.push(a);
    }

    #[must_use]
    pub fn find_take(&mut self, a: &GridCoord) -> Option<UnitData> {
        if let Some((i, _)) = self
            .units
            .iter()
            .enumerate()
            .find(|(_, b)| &b.position == a)
        {
            Some(self.units.remove(i))
        } else {
            None
        }
    }

    pub fn find_slow(&self, a: &GridCoord) -> Option<&UnitData> {
        self.units.iter().find(|b| &b.position == a)
    }

    pub fn find_slow_mut<'a, 'b>(&'a mut self, a: &'b GridCoord) -> Option<&'a mut UnitData> {
        self.units.iter_mut().find(|b| &b.position == a)
    }

    pub fn filter(&self) -> UnitCollectionFilter<UnitData> {
        UnitCollectionFilter { a: &self.units }
    }

    pub fn filter_type(&self, ty: Type) -> UnitCollectionFilterType<UnitData> {
        UnitCollectionFilterType { a: &self.units, ty }
    }
}

pub struct Rest<'a, T> {
    left: &'a mut [T],
    right: &'a mut [T],
}
impl<'a, T> Rest<'a, T> {
    pub fn into_iter(self) -> impl Iterator<Item = &'a mut T> {
        self.left.into_iter().chain(self.right.into_iter())
    }
}

//TODO sort this by x and then y axis!!!!!!!
#[derive(Debug)]
pub struct UnitCollection<T: HasPos> {
    pub elem: Vec<T>,
}

impl<T: HasPos> UnitCollection<T> {
    pub fn new(elem: Vec<T>) -> Self {
        UnitCollection { elem }
    }
    pub fn remove(&mut self, a: &GridCoord) -> T {
        let (i, _) = self
            .elem
            .iter()
            .enumerate()
            .find(|(_, b)| b.get_pos() == a)
            .unwrap();
        self.elem.swap_remove(i)
    }

    pub fn find_mut(&mut self, a: &GridCoord) -> Option<&mut T> {
        self.elem.iter_mut().find(|b| b.get_pos() == a)
    }

    pub fn find_ext_mut(&mut self, a: &GridCoord) -> Option<(&mut T, Rest<T>)> {
        let (i, _) = self
            .elem
            .iter()
            .enumerate()
            .find(|(_, b)| b.get_pos() == a)?;
        let (left, rest) = self.elem.split_at_mut(i);
        let (foo, right) = rest.split_first_mut().unwrap();
        Some((foo, Rest { left, right }))
    }

    pub fn find(&self, a: &GridCoord) -> Option<&T> {
        self.elem.iter().find(|b| b.get_pos() == a)
    }
}

pub struct SingleFilter<'a> {
    a: &'a GridCoord,
}
impl<'a> movement::Filter for SingleFilter<'a> {
    fn filter(&self, a: &GridCoord) -> FilterRes {
        FilterRes::from_bool(self.a != a)
    }
}

pub struct UnitCollectionFilterType<'a, T> {
    a: &'a [T],
    ty: Type,
}
impl<'a> movement::Filter for UnitCollectionFilterType<'a, UnitData> {
    fn filter(&self, b: &GridCoord) -> FilterRes {
        FilterRes::from_bool(
            self.a
                .iter()
                .find(|a| a.position == *b && a.typ == self.ty)
                .is_some(),
        )
    }
}

pub struct UnitCollectionFilter<'a, T> {
    a: &'a [T],
}
impl<'a> movement::Filter for UnitCollectionFilter<'a, UnitData> {
    fn filter(&self, b: &GridCoord) -> FilterRes {
        FilterRes::from_bool(self.a.iter().find(|a| a.get_pos() == b).is_some())
    }
}

pub trait HasPos {
    fn get_pos(&self) -> &GridCoord;
}
impl HasPos for GridCoord {
    fn get_pos(&self) -> &GridCoord {
        self
    }
}

impl HasPos for UnitData {
    fn get_pos(&self) -> &GridCoord {
        &self.position
    }
}
