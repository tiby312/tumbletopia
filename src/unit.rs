use super::*;

#[derive(Serialize, Deserialize, Default, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Factions {
    pub dogs: Tribe,
    pub cats: Tribe,
}
impl Factions {
    pub fn has_a_set(&self, coord: Axial) -> bool {
        self.dogs.units.is_set(coord) || self.cats.units.is_set(coord)
    }

    // pub fn get_unit_mut(&mut self, team: ActiveTeam, coord: Axial) -> &mut UnitData {
    //     self.relative_mut(team)
    //         .this_team
    //         .find_slow_mut(&coord)
    //         .unwrap()
    // }
    // pub fn get_unit(&self, team: ActiveTeam, coord: Axial) -> &UnitData {
    //     self.relative(team).this_team.find_slow(&coord).unwrap()
    // }
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
#[derive(Serialize, Deserialize, Hash, Debug, Copy, Clone, Eq, PartialEq)]
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
impl FactionRelative<&mut Tribe> {
    pub fn has_a_set(&self, coord: Axial) -> bool {
        self.this_team.units.is_set(coord) || self.that_team.units.is_set(coord)
    }
}
impl FactionRelative<&Tribe> {
    pub fn has_a_set(&self, coord: Axial) -> bool {
        self.this_team.units.is_set(coord) || self.that_team.units.is_set(coord)
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, Hash, Eq, PartialEq)]
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
#[derive(Serialize, Deserialize, Default, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Environment {
    pub terrain: Terrain,
    pub fog: BitField,
    pub powerups: Vec<Axial>,
}

//Additionally removes need to special case animation.
#[derive(Serialize, Deserialize, Default, Clone, Debug, Hash, Eq, PartialEq)]
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
    pub fn hash_me(&self) -> u64 {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::hash::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
    pub fn game_is_over(&self, world: &board::MyWorld, team: ActiveTeam) -> Option<GameOver> {
        let this_team_stuck = 'foo: {
            for unit in self.factions.relative(team).this_team.units.iter_mesh() {
                let mesh = self.generate_possible_moves_movement(world, &unit, team);
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

// #[derive(Eq, PartialEq, Hash, Debug, Clone, Ord, PartialOrd)]
// pub struct UnitData {
//     pub position: Axial,
//     pub typ: Type,
// }

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

// #[derive(Hash, Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
// pub enum Type {
//     Warrior,
//     Archer,
// }

// impl Type {
//     pub fn is_warrior(&self) -> bool {
//         if let Type::Warrior { .. } = self {
//             true
//         } else {
//             false
//         }
//     }
//     pub fn is_archer(&self) -> bool {
//         if let Type::Archer = self {
//             true
//         } else {
//             false
//         }
//     }
//     pub fn type_index(&self) -> usize {
//         let a = self;
//         match a {
//             Type::Warrior { .. } => 0,
//             Type::Archer => 1,
//         }
//     }
// }

#[derive(Serialize, Deserialize, Default, Eq, PartialEq, Hash, Clone)]
pub struct Tribe {
    pub units: BitField,
}

impl std::fmt::Debug for Tribe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "tribe:[")?;
        for pos in self.units.iter_mesh() {
            write!(f, "{:?},", pos)?;
        }
        write!(f, "{}", "]")
    }
}
