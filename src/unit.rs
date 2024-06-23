use std::num::NonZeroUsize;

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
    pub fn game_is_over(&self, world: &board::MyWorld) -> Option<GameOver> {
        if self.factions.white.get(UnitType::King).count_ones(..) == 0 {
            return Some(GameOver::BlackWon);
        }
        if self.factions.black.get(UnitType::King).count_ones(..) == 0 {
            return Some(GameOver::WhiteWon);
        }
        None

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
    pub fields: [BitField; 7],
    // pub bishop: BitField,
    // pub knight: BitField,
    //pub pawn: BitField,
}

impl Tribe {
    pub fn new() -> Tribe {
        Tribe {
            fields: [(); 7].map(|_| BitField::new()),
        }
    }

    pub fn all_alloc(&self) -> BitField {
        let mut j = BitField::new();
        for a in self.fields.iter() {
            j.union_with(&a);
        }

        // j.union_with(&self.bishop);
        // j.union_with(&self.knight);
        // j.union_with(&self.pawn);

        j
    }
    pub fn count_ones(&self) -> usize {
        self.fields.iter().fold(0, |acc, a| acc + a.count_ones(..))
    }

    pub fn iter_mesh(&self) -> impl Iterator<Item = Axial> + '_ {
        self.fields.iter().map(|x| x.iter_mesh()).flatten()
    }
    pub fn is_set(&self, a: Axial) -> bool {
        self.fields.iter().fold(false, |acc, x| acc || x.is_set(a))
    }

    pub fn move_unit(&mut self, a: Axial, b: Axial) {
        for arr in self.fields.iter_mut() {
            if arr.is_set(a) {
                arr.set_coord(a, false);
                arr.set_coord(b, true);
                return;
            }
        }

        unreachable!("Can't move")
    }

    pub fn get_mut(&mut self, a: UnitType) -> &mut BitField {
        &mut self.fields[a as usize]
    }

    pub fn get(&self, a: UnitType) -> &BitField {
        &self.fields[a as usize]
    }
    pub fn clear(&mut self, a: Axial) -> UnitType {
        for (i, arr) in self.fields.iter_mut().enumerate() {
            if arr.is_set(a) {
                arr.set_coord(a, false);
                return UnitType::from_int(i);
            }
        }

        unreachable!("coord isnt set in first place.")
    }
    pub fn try_get_type(&self, a: Axial) -> Option<UnitType> {
        for (i, arr) in self.fields.iter().enumerate() {
            if arr.is_set(a) {
                return Some(UnitType::from_int(i));
            }
        }

        return None;
    }

    pub fn get_type(&self, a: Axial) -> UnitType {
        self.try_get_type(a)
            .expect("Could not find unit at position")
    }
}

#[derive(PartialOrd, Ord, Eq, PartialEq, Copy, Clone, Debug)]
pub enum UnitType {
    Book1 = 0,
    Book2 = 1,
    Knight1 = 2,
    Knight2 = 3,
    Pawn = 4,
    King = 5,
    Rook = 6,
}
impl UnitType {
    pub fn from_int(a: usize) -> UnitType {
        use UnitType::*;
        match a {
            0 => Book1,
            1 => Book2,
            2 => Knight1,
            3 => Knight2,
            4 => Pawn,
            5 => King,
            6 => Rook,
            _ => unreachable!(),
        }
    }
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
