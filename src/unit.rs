use mesh::small_mesh::SmallMesh;

use super::*;

// #[derive(Serialize, Deserialize, Default, Clone, Debug, Hash, Eq, PartialEq)]
// pub struct Factions {
//     pub cells: Tribe,
// }
// impl Factions {
//     // pub fn has_a_set(&self, coord: Axial) -> bool {
//     //     self.black.is_set(coord) || self.white.is_set(coord)
//     // }

//     // pub fn has_a_set_type(&self, coord: Axial) -> Option<UnitType> {
//     //     if let Some(a) = self.black.try_get_type(coord) {
//     //         return Some(a);
//     //     }

//     //     self.white.try_get_type(coord)
//     // }

//     // pub fn get_unit_mut(&mut self, team: ActiveTeam, coord: Axial) -> &mut UnitData {
//     //     self.relative_mut(team)
//     //         .this_team
//     //         .find_slow_mut(&coord)
//     //         .unwrap()
//     // }
//     // pub fn get_unit(&self, team: ActiveTeam, coord: Axial) -> &UnitData {
//     //     self.relative(team).this_team.find_slow(&coord).unwrap()
//     // }
//     // pub fn relative_mut(&mut self, team: ActiveTeam) -> FactionRelative<&mut Tribe> {
//     //     match team {
//     //         ActiveTeam::White => FactionRelative {
//     //             this_team: &mut self.white,
//     //             that_team: &mut self.black,
//     //         },
//     //         ActiveTeam::Black => FactionRelative {
//     //             this_team: &mut self.black,
//     //             that_team: &mut self.white,
//     //         },
//     //     }
//     // }
//     // pub fn relative(&self, team: ActiveTeam) -> FactionRelative<&Tribe> {
//     //     match team {
//     //         ActiveTeam::White => FactionRelative {
//     //             this_team: &self.white,
//     //             that_team: &self.black,
//     //         },
//     //         ActiveTeam::Black => FactionRelative {
//     //             this_team: &self.black,
//     //             that_team: &self.white,
//     //         },
//     //     }
//     // }
// }

#[must_use]
#[derive(Serialize, Deserialize, Hash, Ord, PartialOrd, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ActiveTeam {
    White = 0,
    Black = 1,
    Neutral = 2,
}
impl ActiveTeam {
    pub fn is_white(&self) -> bool {
        if let ActiveTeam::White = self {
            true
        } else {
            false
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = Self> {
        [*self, self.not()].into_iter().cycle()
    }
    pub fn not(&self) -> Self {
        match self {
            ActiveTeam::White => ActiveTeam::Black,
            ActiveTeam::Black => ActiveTeam::White,
            ActiveTeam::Neutral => ActiveTeam::Neutral,
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
    pub factions: Tribe,
    //pub env: Environment,
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



#[derive(Default,Clone)]
pub struct SpokeNode{
    spokes:[u8;6],
}
impl SpokeNode{
    pub fn has_piece_at_end(&self,dir:usize)->bool{
        self.spokes[dir] & (1<<7) !=0
    }

    pub fn distance(&self,dir:usize)->u8{
        self.spokes[dir] & !(1<<7)
    }
}


pub struct Spokes{
    inner:Vec<SpokeNode>
}
impl Spokes{
    pub fn generate(game:&GameState,world:&board::MyWorld)->Spokes{

        let mut s=Spokes{
            inner:vec![SpokeNode::default();256]
        };

        for unit in world.get_game_cells().iter_mesh(){

            let res=game.factions.iter_end_points(world, unit);

            let res=res.map(|(ax,foo)|{
                let mut val=ax.to_cube().dist(&unit.to_cube()) as u8;
                
                if foo.is_some(){
                    val|=1<<7;
                }
                val
            });

            s.get_spokes_mut(unit).spokes=res;


        }

        s
    }
    pub fn get_spokes(&self,a:Axial)->&SpokeNode{
        let ind=mesh::small_mesh::conv(a);
        &self.inner[ind]
    }
    pub fn get_spokes_mut(&mut self,a:Axial)->&mut SpokeNode{
        let ind=mesh::small_mesh::conv(a);
        &mut self.inner[ind]
    }
}



#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq, Hash, Clone)]
pub struct Tribe {
    pub cells: [SmallMesh; 3],
    pub team: SmallMesh,
}

impl Tribe {
    pub fn iter_end_points(
        &self,
        world: &board::MyWorld,
        unit: Axial,
    ) -> [(Axial, Option<(usize, ActiveTeam)>); 6] {
        let for_ray = |unit: Axial, dir: [i8; 3]| {
            unit.to_cube()
                .ray_from_vector(hex::Cube::from_arr(dir))
                .take_while(|k| {
                    let k = k.to_axial();
                    world.get_game_cells().is_set(k)
                })
                .map(|x| x.to_axial())
        };

        let iter_end_points = |unit: Axial| {
            hex::OFFSETS.map(|h| {
                let mut last_cell = (Axial::zero(), None);
                for k in for_ray(unit, h) {
                    last_cell.0 = k;

                    if let Some((a, b)) = self.get_cell(k) {
                        last_cell.1 = Some((a, b));

                        break;
                    }
                }
                last_cell
            })
        };

        iter_end_points(unit)
    }


    pub fn new() -> Tribe {
        Tribe {
            cells: [0; 3].map(|_| SmallMesh::new()),
            team: SmallMesh::new(),
        }
    }

    pub fn remove(&mut self, a: Axial) {
        self.cells[0].set_coord(a, false);
        self.cells[1].set_coord(a, false);
        self.cells[2].set_coord(a, false);
        self.team.set_coord(a, false);
    }

    pub fn get_cell(&self, a: Axial) -> Option<(usize, ActiveTeam)> {
        let bit0 = self.cells[0].is_set(a) as usize;
        let bit1 = self.cells[1].is_set(a) as usize;
        let bit2 = self.cells[2].is_set(a) as usize;

        let val = bit0 | bit1 << 1 | bit2 << 2;

        // if val == 6 {
        //     return Some((1, ActiveTeam::Neutral));
        // }
        if val == 7 {
            return Some((2, ActiveTeam::Neutral));
        }
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

    fn set_coord(&mut self, a: Axial, stack: usize) {
        assert!(stack <= 7);
        let bit2 = ((stack >> 2) & 1) != 0;
        let bit1 = ((stack >> 1) & 1) != 0;
        let bit0 = ((stack >> 0) & 1) != 0;

        self.cells[0].set_coord(a, bit0);
        self.cells[1].set_coord(a, bit1);
        self.cells[2].set_coord(a, bit2);
    }

    pub fn add_cell(&mut self, a: Axial, stack: usize, team: ActiveTeam) {
        match team {
            ActiveTeam::White => self.team.set_coord(a, true),
            ActiveTeam::Black => self.team.set_coord(a, false),
            ActiveTeam::Neutral => {
                let val = if stack == 2 { 7 } else { panic!("impossible") };

                self.set_coord(a, val);
                self.team.set_coord(a, false);
                return;
            }
        }
        self.set_coord(a, stack);
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
