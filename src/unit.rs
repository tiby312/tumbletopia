use super::*;

#[derive(Serialize, Deserialize, Default, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Factions {
    pub black: Tribe,
    pub white: Tribe,
}
impl Factions {
    pub fn has_a_set(&self, coord: Axial) -> bool {
        self.black.is_set(coord) || self.white.is_set(coord)
    }

    pub fn has_a_set_type(&self, coord: Axial) -> Option<UnitType> {
        if let Some(a) = self.black.try_get_type(coord) {
            return Some(a);
        }

        self.white.try_get_type(coord)
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
            ActiveTeam::White => FactionRelative {
                this_team: &mut self.white,
                that_team: &mut self.black,
            },
            ActiveTeam::Black => FactionRelative {
                this_team: &mut self.black,
                that_team: &mut self.white,
            },
        }
    }
    pub fn relative(&self, team: ActiveTeam) -> FactionRelative<&Tribe> {
        match team {
            ActiveTeam::White => FactionRelative {
                this_team: &self.white,
                that_team: &self.black,
            },
            ActiveTeam::Black => FactionRelative {
                this_team: &self.black,
                that_team: &self.white,
            },
        }
    }
}

#[must_use]
#[derive(Serialize, Deserialize, Hash, Debug, Copy, Clone, Eq, PartialEq)]
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

pub struct FactionRelative<T> {
    pub this_team: T,
    pub that_team: T,
}
impl FactionRelative<&mut Tribe> {
    pub fn has_a_set(&self, coord: Axial) -> bool {
        self.this_team.is_set(coord) || self.that_team.is_set(coord)
    }
}
impl FactionRelative<&Tribe> {
    pub fn has_a_set(&self, coord: Axial) -> bool {
        self.this_team.is_set(coord) || self.that_team.is_set(coord)
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
        let this_team_stuck = 'foo: {
            for unit in self.factions.relative(team).this_team.iter_mesh() {
                let mesh = self.generate_possible_moves_movement(world, &unit, team);
                if !mesh.is_empty() {
                    break 'foo false;
                }
            }
            true
        };

        if this_team_stuck {
            match team {
                ActiveTeam::White => Some(GameOver::BlackWon),
                ActiveTeam::Black => Some(GameOver::WhiteWon),
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

#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq, Hash, Clone)]
pub struct Tribe {
    pub rook: BitField,
    pub bishop: BitField,
    pub knight: BitField,
    pub pawn: BitField,
}

impl Tribe {
    pub fn all_alloc(&self) -> BitField {
        let mut j = self.rook.clone();
        j.union_with(&self.bishop);
        j.union_with(&self.knight);
        j.union_with(&self.pawn);

        j
    }
    pub fn count_ones(&self) -> usize {
        self.rook.count_ones(..)
            + self.bishop.count_ones(..)
            + self.knight.count_ones(..)
            + self.pawn.count_ones(..)
    }

    pub fn iter_mesh(&self) -> impl Iterator<Item = Axial> + '_ {
        self.rook
            .iter_mesh()
            .chain(self.bishop.iter_mesh())
            .chain(self.knight.iter_mesh())
            .chain(self.pawn.iter_mesh())
    }
    pub fn is_set(&self, a: Axial) -> bool {
        self.rook.is_set(a) || self.bishop.is_set(a) || self.knight.is_set(a) || self.pawn.is_set(a)
    }

    pub fn move_unit(&mut self, a: Axial, b: Axial) {
        if self.rook.is_set(a) {
            self.rook.set_coord(a, false);
            self.rook.set_coord(b, true);
            return;
        }
        if self.bishop.is_set(a) {
            self.bishop.set_coord(a, false);
            self.bishop.set_coord(b, true);
            return;
        }
        if self.knight.is_set(a) {
            self.knight.set_coord(a, false);
            self.knight.set_coord(b, true);
            return;
        }

        if self.pawn.is_set(a) {
            self.pawn.set_coord(a, false);
            self.pawn.set_coord(b, true);
            return;
        }
        unreachable!("Can't move")
    }

    pub fn get_mut(&mut self, a: UnitType) -> &mut BitField {
        match a {
            UnitType::Rook => &mut self.rook,
            UnitType::Bishop => &mut self.bishop,
            UnitType::Knight => &mut self.knight,
            UnitType::Pawn => &mut self.pawn,
        }
    }

    pub fn clear(&mut self, a: Axial) -> UnitType {
        if self.rook.is_set(a) {
            self.rook.set_coord(a, false);
            return UnitType::Rook;
        }
        if self.bishop.is_set(a) {
            self.bishop.set_coord(a, false);
            return UnitType::Bishop;
        }
        if self.knight.is_set(a) {
            self.knight.set_coord(a, false);
            return UnitType::Knight;
        }
        if self.pawn.is_set(a) {
            self.pawn.set_coord(a, false);
            return UnitType::Pawn;
        }
        unreachable!("coord isnt set in first place.")
    }
    pub fn try_get_type(&self, a: Axial) -> Option<UnitType> {
        if self.rook.is_set(a) {
            return Some(UnitType::Rook);
        }

        if self.bishop.is_set(a) {
            return Some(UnitType::Bishop);
        }

        if self.knight.is_set(a) {
            return Some(UnitType::Knight);
        }

        if self.pawn.is_set(a) {
            return Some(UnitType::Pawn);
        }
        return None;
    }
    pub fn get_type(&self, a: Axial) -> UnitType {
        if self.rook.is_set(a) {
            return UnitType::Rook;
        }

        if self.bishop.is_set(a) {
            return UnitType::Bishop;
        }

        if self.knight.is_set(a) {
            return UnitType::Knight;
        }

        if self.pawn.is_set(a) {
            return UnitType::Pawn;
        }
        unreachable!("Could not find unit at position");
    }
}

#[derive(PartialOrd, Ord, Eq, PartialEq, Copy, Clone, Debug)]
pub enum UnitType {
    Rook,
    Bishop,
    Knight,
    Pawn,
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
