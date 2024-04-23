use super::*;

#[derive(Default, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Factions {
    pub dogs: Tribe,
    pub cats: Tribe,
}
impl Factions {
    pub fn contains(&self, coord: Axial) -> bool {
        self.dogs
            .iter()
            .chain(self.cats.iter())
            .map(|a| a.position)
            .any(|a| a == coord)
    }
    pub fn get_unit_mut(&mut self, team: ActiveTeam, coord: Axial) -> &mut UnitData {
        self.relative_mut(team)
            .this_team
            .find_slow_mut(&coord)
            .unwrap()
    }
    pub fn get_unit(&self, team: ActiveTeam, coord: Axial) -> &UnitData {
        self.relative(team).this_team.find_slow(&coord).unwrap()
    }
    pub fn relative_mut(&mut self, team: ActiveTeam) -> FactionRelative<&mut Tribe> {
        match team {
            ActiveTeam::Cats => FactionRelative {
                this_team: &mut self.cats,
                that_team: &mut self.dogs,
            },
            ActiveTeam::Dogs => FactionRelative {
                this_team: &mut self.dogs,
                that_team: &mut self.cats,
            },
        }
    }
    pub fn relative(&self, team: ActiveTeam) -> FactionRelative<&Tribe> {
        match team {
            ActiveTeam::Cats => FactionRelative {
                this_team: &self.cats,
                that_team: &self.dogs,
            },
            ActiveTeam::Dogs => FactionRelative {
                this_team: &self.dogs,
                that_team: &self.cats,
            },
        }
    }
}

#[must_use]
#[derive(Hash, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ActiveTeam {
    Cats = 0,
    Dogs = 1,
}
impl ActiveTeam {
    pub fn iter(&self) -> impl Iterator<Item = Self> {
        [*self, self.not()].into_iter().cycle()
    }
    pub fn not(&self) -> Self {
        match self {
            ActiveTeam::Cats => ActiveTeam::Dogs,
            ActiveTeam::Dogs => ActiveTeam::Cats,
        }
    }
}

pub struct FactionRelative<T> {
    pub this_team: T,
    pub that_team: T,
}

#[derive(Default, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Terrain {
    pub land: BitField,
    pub forest: BitField,
    pub mountain: BitField,
}
impl Terrain {
    pub fn is_set(&self, a: Axial) -> bool {
        self.land.is_set(a) || self.forest.is_set(a) || self.mountain.is_set(a)
    }
    pub fn gen_all_terrain(&self) -> BitField {
        let mut k = BitField::new();
        k.union_with(&self.land);
        k.union_with(&self.forest);
        k.union_with(&self.mountain);
        k
    }
}
#[derive(Default, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Environment {
    pub terrain: Terrain,
    pub fog: BitField,
    pub powerups: Vec<Axial>,
}

//Additionally removes need to special case animation.
#[derive(Default, Clone, Debug, Hash, Eq, PartialEq)]
pub struct GameState {
    pub factions: Factions,
    pub env: Environment,
}

#[must_use]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameOver {
    CatWon,
    DogWon,
    Tie,
}

impl GameState {
    pub fn game_is_over(&self, world: &board::MyWorld, team: ActiveTeam) -> Option<GameOver> {
        let this_team_stuck = 'foo: {
            for unit in self.factions.relative(team).this_team.units.iter() {
                let mesh =
                    self.generate_possible_moves_movement(world, &unit.position, unit.typ, team);
                if !mesh.is_empty() {
                    break 'foo false;
                }
            }
            true
        };

        if this_team_stuck {
            match team {
                ActiveTeam::Cats => Some(GameOver::DogWon),
                ActiveTeam::Dogs => Some(GameOver::CatWon),
            }
        } else {
            None
        }
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Clone, Ord, PartialOrd)]
pub struct UnitData {
    pub position: Axial,
    pub typ: Type,
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

#[derive(Hash, Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum Type {
    Warrior,
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
