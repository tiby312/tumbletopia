use mesh::small_mesh::SmallMesh;

use super::*;

#[derive(Serialize, Deserialize, Default, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Factions {
    pub cells: Tribe,
}
impl Factions {
    // pub fn has_a_set(&self, coord: Axial) -> bool {
    //     self.black.is_set(coord) || self.white.is_set(coord)
    // }

    // pub fn has_a_set_type(&self, coord: Axial) -> Option<UnitType> {
    //     if let Some(a) = self.black.try_get_type(coord) {
    //         return Some(a);
    //     }

    //     self.white.try_get_type(coord)
    // }

    // pub fn get_unit_mut(&mut self, team: ActiveTeam, coord: Axial) -> &mut UnitData {
    //     self.relative_mut(team)
    //         .this_team
    //         .find_slow_mut(&coord)
    //         .unwrap()
    // }
    // pub fn get_unit(&self, team: ActiveTeam, coord: Axial) -> &UnitData {
    //     self.relative(team).this_team.find_slow(&coord).unwrap()
    // }
    // pub fn relative_mut(&mut self, team: ActiveTeam) -> FactionRelative<&mut Tribe> {
    //     match team {
    //         ActiveTeam::White => FactionRelative {
    //             this_team: &mut self.white,
    //             that_team: &mut self.black,
    //         },
    //         ActiveTeam::Black => FactionRelative {
    //             this_team: &mut self.black,
    //             that_team: &mut self.white,
    //         },
    //     }
    // }
    // pub fn relative(&self, team: ActiveTeam) -> FactionRelative<&Tribe> {
    //     match team {
    //         ActiveTeam::White => FactionRelative {
    //             this_team: &self.white,
    //             that_team: &self.black,
    //         },
    //         ActiveTeam::Black => FactionRelative {
    //             this_team: &self.black,
    //             that_team: &self.white,
    //         },
    //     }
    // }
}

#[must_use]
#[derive(Serialize, Deserialize, Hash, Ord, PartialOrd, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ActiveTeam {
    White = 0,
    Black = 1,
}
impl ActiveTeam {
    pub fn iter(&self) -> impl Iterator<Item = Self> {
        [*self, self.not()].into_iter().cycle()
    }
    pub fn not(&self) -> Self {
        match self {
            ActiveTeam::White => ActiveTeam::Black,
            ActiveTeam::Black => ActiveTeam::White,
        }
    }
}

// pub struct FactionRelative<T> {
//     pub this_team: T,
//     pub that_team: T,
// }
// impl FactionRelative<&mut Tribe> {
//     pub fn has_a_set(&self, coord: Axial) -> bool {
//         self.this_team.is_set(coord) || self.that_team.is_set(coord)
//     }
// }
// impl FactionRelative<&Tribe> {
//     pub fn has_a_set(&self, coord: Axial) -> bool {
//         self.this_team.is_set(coord) || self.that_team.is_set(coord)
//     }
// }

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
    WhiteWon,
    BlackWon,
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
        // let this_team_stuck = 'foo: {
        //     for unit in self.factions.relative(team).this_team.iter_mesh() {
        //         let mesh = self.generate_possible_moves_movement(world, &unit, team);
        //         if !mesh.is_empty() {
        //             break 'foo false;
        //         }
        //     }
        //     true
        // };

        // if this_team_stuck {
        //     match team {
        //         ActiveTeam::White => Some(GameOver::BlackWon),
        //         ActiveTeam::Black => Some(GameOver::WhiteWon),
        //     }
        // } else {
        //     None
        // }
        None
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

#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq, Hash, Clone)]
pub struct Tribe {
    pub cells: [SmallMesh; 3],
    pub team: SmallMesh,
}

impl Tribe {
    pub fn new() -> Tribe {
        Tribe {
            cells: [0; 3].map(|_| SmallMesh::new()),
            team: SmallMesh::new(),
        }
    }

    pub fn remove(&mut self, a: Axial) {
        self.cells[0].set_coord(a,false);
        self.cells[1].set_coord(a,false);
        self.cells[2].set_coord(a,false);
        self.team.set_coord(a,false);
    }

    pub fn get_cell(&self, a: Axial) -> Option<(usize, ActiveTeam)> {
        let bit0 = self.cells[0].is_set(a) as usize;
        let bit1 = self.cells[1].is_set(a) as usize;
        let bit2 = self.cells[2].is_set(a) as usize;

        let val = bit0 | bit1 << 1 | bit2 << 2;
        assert!(val <= 6);

        if val == 0 {
            return None;
        }
        let team = if self.team.is_set(a) {
            ActiveTeam::White
        } else {
            ActiveTeam::Black
        };
        Some((val, team))
    }

    pub fn add_cell(&mut self, a: Axial, stack: usize, team: ActiveTeam) {
        assert!(stack <= 6);
        let bit2 = ((stack >> 2) & 1) != 0;
        let bit1 = ((stack >> 1) & 1) != 0;
        let bit0 = ((stack >> 0) & 1) != 0;

        
        self.cells[0].set_coord(a, bit0);
        self.cells[1].set_coord(a, bit1);
        self.cells[2].set_coord(a, bit2);

        if let ActiveTeam::White = team {
            self.team.set_coord(a, true)
        } else {
            self.team.set_coord(a, false)
        }
    }
}

#[derive(PartialOrd, Ord, Eq, PartialEq, Copy, Clone, Debug)]
pub enum UnitType {
    Mouse,
    Rabbit,
}

// impl std::fmt::Debug for Tribe {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", "tribe:[")?;
//         for pos in self.warrior.iter_mesh() {
//             write!(f, "{:?},", pos)?;
//         }
//         write!(f, "{}", "]")
//     }
// }
