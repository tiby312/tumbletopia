use hex::HDir;

use super::*;

pub use spoke::SpokeInfo;
mod spoke {
    use super::*;
    #[derive(PartialEq, Eq, Copy, Clone, Debug)]
    pub struct SpokeInfo {
        pub data: [SpokeCell; 256],
    }

    // 3 bits for num white
    // 3 bits for num black
    // 2 bits left over

    //0
    //1
    //2
    //3
    //4
    //5
    //6

    //   5   5   5   5   5   5
    // |---|---|---|---|---|---|
    //
    //
    //

    #[derive(PartialEq, Eq, Copy, Clone, Debug)]
    pub struct SpokeCell {
        raw: [Team; 6],
        pub num_attack: [i64; 2],
    }

    pub struct SpokeTempInfo {
        data: [(i8, Option<unit::EndPoint>); 6],
    }

    impl SpokeInfo {
        pub fn new(game: &GameState, world: &MyWorld) -> Self {
            //tddt-t--dt---t-d-d-
            let mut spoke_info = SpokeInfo {
                data: [SpokeCell {
                    raw: [Team::Neutral; 6],
                    num_attack: [0; 2],
                }; 256],
            };

            //Update spoke info
            for index in world.get_game_cells().inner.iter_ones() {
                for (i, (_, rest)) in game.factions.iter_end_points(world, index).enumerate() {
                    let v = if let Some(e) = rest {
                        e.piece.team
                    } else {
                        Team::Neutral
                    };
                    spoke_info.set(index, HDir::from(i as u8), v);
                    debug_assert_eq!(v, spoke_info.get(index, HDir::from(i as u8)));
                }
            }

            spoke_info
        }

        pub fn process_move_better(
            &mut self,
            a: NormalMove,
            team: Team,
            world: &board::MyWorld,
            game: &GameState,
        ) -> SpokeTempInfo {
            if a.coord.0 == hex::PASS_MOVE_INDEX {
                return SpokeTempInfo {
                    data: std::array::from_fn(|_| (0, None)),
                };
            }
            let index = a.coord.0;
            debug_assert!(
                world.get_game_cells().inner[index as usize],
                "uhoh {:?}",
                world.format(&a.coord)
            );
            let mut it = hex::HDir::all().map(move |dd| {
                let (dis, it) = unit::ray(Axial::from_index(&index), dd, world);

                for (d, index2) in it.enumerate() {
                    debug_assert!(index != index2 as usize);
                    self.set(index2 as usize, dd.rotate_180(), team);

                    match game.factions.get_cell_inner(index2 as usize) {
                        &unit::GameCell::Piece(piece) => {
                            self.set(index, dd, piece.team);

                            return (
                                d as i8 + 1,
                                Some(unit::EndPoint {
                                    index: index2 as usize,
                                    piece,
                                }),
                            );
                        }
                        unit::GameCell::Empty => {}
                    }
                }
                self.set(index, dd, Team::Neutral);
                (dis, None)
            });

            SpokeTempInfo {
                data: std::array::from_fn(|_| it.next().unwrap()),
            }
        }

        pub fn undo_move(
            &mut self,
            a: NormalMove,
            effect: &move_build::NormalMoveEffect,
            _team: Team,
            _world: &board::MyWorld,
            _game: &GameState,
            spoke_temp: SpokeTempInfo,
        ) {
            if a.coord.0 == hex::PASS_MOVE_INDEX {
                return;
            }
            let index = a.coord.0;

            let arr = &spoke_temp.data;

            for (hexdir, (dis, rest)) in HDir::all().zip(arr.iter()) {
                let st = if let &Some(unit::EndPoint { .. }) = rest {
                    1
                } else {
                    0
                };

                let stride = board::STRIDES[hexdir as usize] as isize;

                let mut index2: isize = index as isize;

                let oppt = if let Some((_, t2)) = effect.destroyed_unit {
                    t2
                } else {
                    if let (_, Some(end)) = &arr[hexdir.rotate_180() as usize] {
                        end.piece.team
                    } else {
                        Team::Neutral
                    }
                };

                for _ in 0..*dis - 1 + st {
                    index2 += stride;
                    self.set(index2 as usize, hexdir.rotate_180(), oppt);
                }
            }
        }

        fn set(&mut self, index: usize, dir: HDir, new_team: Team) {
            let cc = &mut self.data[index];

            let curr_team = cc.raw[dir as usize];

            if new_team == curr_team {
                return;
            }

            if new_team != Team::Neutral {
                cc.num_attack[new_team] += 1;
            }

            if curr_team != Team::Neutral {
                cc.num_attack[curr_team] -= 1;
            }
            cc.raw[dir as usize] = new_team;
        }

        pub fn get(&self, index: usize, dir: HDir) -> Team {
            self.data[index].raw[dir as usize]
        }

        pub fn get_num_attack(&self, index: usize) -> &[i64; 2] {
            let foo = &self.data[index];
            &foo.num_attack
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MoveType {
    Capture,
    Reinforce,
    Fresh,
    Suicidal,
}

impl MoveType {
    pub fn is_suicidal(&self) -> bool {
        if let MoveType::Suicidal = self {
            true
        } else {
            false
        }
    }
}

impl hex::HexDraw for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, radius: i8) -> Result<(), std::fmt::Error> {
        Axial::from_index(self).fmt(f, radius)
    }
}

//Signifies a normal move of a [1,6] stack.
#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord)]
pub struct Coordinate(pub usize);

impl std::ops::Deref for Coordinate {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Default for Coordinate {
    fn default() -> Self {
        Self(Default::default())
    }
}

// #[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord)]
// pub struct GeneralMove {
//     coord: Coordinate,
//     typ: PieceType,
// }

// impl hex::HexDraw for GeneralMove {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>, radius: i8) -> Result<(), std::fmt::Error> {
//         self.coord.fmt(f, radius)
//     }
// }
