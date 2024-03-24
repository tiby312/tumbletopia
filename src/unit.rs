
use super::*;

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct UnitData {
    pub position: Axial,
    pub typ: Type,
    pub has_powerup: bool,
}

// impl UnitData {
//     pub fn new(position: GridCoord, typ: Type) -> Self {
//         UnitData { position, typ }
//     }
// }

#[derive(Debug, Clone)]
pub enum CellSelection {
    MoveSelection(Axial, movement::movement_mesh::SmallMesh),
    BuildSelection(Axial),
}

#[derive(Hash, Debug, Clone, Copy, Eq, PartialEq)]
pub enum Type {
    Warrior { powerup: bool },
    Archer,
}

impl Type {
    pub fn is_warrior(&self) -> bool {
        if let Type::Warrior { .. } = self {
            true
        } else {
            false
        }
    }
    pub fn is_archer(&self) -> bool {
        if let Type::Archer = self {
            true
        } else {
            false
        }
    }
    pub fn type_index(&self) -> usize {
        let a = self;
        match a {
            Type::Warrior { .. } => 0,
            Type::Archer => 1,
        }
    }
}

impl std::ops::Deref for Tribe {
    type Target = [UnitData];

    fn deref(&self) -> &Self::Target {
        &self.units
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Tribe {
    pub units: Vec<UnitData>,
}
impl Tribe {
    pub fn add(&mut self, a: UnitData) {
        self.units.push(a);
    }

    #[must_use]
    pub fn find_take(&mut self, a: &Axial) -> Option<UnitData> {
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

    pub fn find_slow(&self, a: &Axial) -> Option<&UnitData> {
        self.units.iter().find(|b| &b.position == a)
    }

    pub fn find_slow_mut<'a, 'b>(&'a mut self, a: &'b Axial) -> Option<&'a mut UnitData> {
        self.units.iter_mut().find(|b| &b.position == a)
    }

    pub fn filter(&self) -> UnitCollectionFilter<UnitData> {
        UnitCollectionFilter { a: &self.units }
    }

    // pub fn filter_type(&self, ty: Type) -> UnitCollectionFilterType<UnitData> {
    //     UnitCollectionFilterType { a: &self.units, ty }
    // }
}

pub struct UnitCollectionFilter<'a, T> {
    a: &'a [T],
}
// impl<'a> movement::Filter for UnitCollectionFilter<'a, UnitData> {
//     fn filter(&self, b: &Axial) -> FilterRes {
//         FilterRes::from_bool(self.a.iter().find(|a| a.get_pos() == b).is_some())
//     }
// }

pub trait HasPos {
    fn get_pos(&self) -> &Axial;
}
impl HasPos for Axial {
    fn get_pos(&self) -> &Axial {
        self
    }
}

impl HasPos for UnitData {
    fn get_pos(&self) -> &Axial {
        &self.position
    }
}
