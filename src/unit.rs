use super::*;

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct UnitData {
    pub position: Axial,
    pub typ: Type,
    pub has_powerup: bool,
}

#[derive(Debug, Clone)]
pub enum CellSelection {
    MoveSelection(
        Axial,
        mesh::small_mesh::SmallMesh,
        Option<ace::selection::HaveMoved>,
    ),
    BuildSelection(Axial),
}
impl Default for CellSelection {
    fn default() -> Self {
        CellSelection::BuildSelection(Axial::default())
    }
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

#[derive(Default, Eq, PartialEq, Hash, Clone, Debug)]
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

    pub fn find_slow_mut<'a>(&'a mut self, a: &Axial) -> Option<&'a mut UnitData> {
        self.units.iter_mut().find(|b| &b.position == a)
    }
}
