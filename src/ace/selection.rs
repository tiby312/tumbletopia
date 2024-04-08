use super::*;

#[derive(Clone, Debug)]
pub struct HaveMoved {
    pub the_move: move_build::MovePhase,
    pub effect: move_build::MoveEffect,
}

//This is for saving/loading.
#[derive(Deserialize, Serialize)]
pub struct JustMoveLog {
    pub seed: board::WorldSeed,
    pub inner: Vec<moves::ActualMove>,
}
impl JustMoveLog {
    // pub fn deserialize(buffer: Vec<u8>) -> JustMoveLog {
    //     use byteorder::{BigEndian, ReadBytesExt};
    //     use std::io::Cursor;
    //     let mut rdr = Cursor::new(buffer);
    //     let ver = rdr.read_u32::<BigEndian>().unwrap();
    //     assert_eq!(ver, 0);
    //     let num = rdr.read_u32::<BigEndian>().unwrap();

    //     let mut ret = vec![];
    //     for _ in 0..num {
    //         let vals = [
    //             rdr.read_i16::<BigEndian>().unwrap(),
    //             rdr.read_i16::<BigEndian>().unwrap(),
    //             rdr.read_i16::<BigEndian>().unwrap(),
    //             rdr.read_i16::<BigEndian>().unwrap(),
    //             rdr.read_i16::<BigEndian>().unwrap(),
    //             rdr.read_i16::<BigEndian>().unwrap(),
    //         ];

    //         ret.push(moves::ActualMove {
    //             original: Axial {
    //                 q: vals[0],
    //                 r: vals[1],
    //             },
    //             moveto: Axial {
    //                 q: vals[2],
    //                 r: vals[3],
    //             },
    //             attackto: Axial {
    //                 q: vals[4],
    //                 r: vals[5],
    //             },
    //         });
    //     }
    //     JustMoveLog { inner: ret }
    // }
    // pub fn serialize(&self) -> Vec<u8> {
    //     let o = &self.inner;
    //     use byteorder::{BigEndian, WriteBytesExt};

    //     let mut wtr = vec![];

    //     let version = 0;
    //     wtr.write_u32::<BigEndian>(version).unwrap();

    //     wtr.write_u32::<BigEndian>(o.len().try_into().unwrap())
    //         .unwrap();

    //     for a in o.iter() {
    //         wtr.write_i16::<BigEndian>(a.original.q).unwrap();
    //         wtr.write_i16::<BigEndian>(a.original.r).unwrap();
    //         wtr.write_i16::<BigEndian>(a.moveto.q).unwrap();
    //         wtr.write_i16::<BigEndian>(a.moveto.r).unwrap();
    //         wtr.write_i16::<BigEndian>(a.attackto.q).unwrap();
    //         wtr.write_i16::<BigEndian>(a.attackto.r).unwrap();
    //     }
    //     wtr
    // }
}

//Need to keep effect so you can undo all the way to the start.
pub struct MoveHistory {
    pub inner: Vec<(moves::ActualMove, move_build::CombinedEffect)>,
}
// impl Default for MoveHistory {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl Default for MoveHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveHistory {
    pub fn new() -> Self {
        MoveHistory { inner: vec![] }
    }
    pub fn into_just_move(self, seed: board::WorldSeed) -> JustMoveLog {
        JustMoveLog {
            seed,
            inner: self.inner.into_iter().map(|a| a.0).collect(),
        }
    }

    pub fn push(&mut self, o: (moves::ActualMove, move_build::CombinedEffect)) {
        self.inner.push(o);
    }

    pub async fn replay(&self, _doop: &mut WorkerManager, _kk: &mut GameState) {
        todo!();
        // let mut ii = ActiveTeam::Dogs.iter();
        // for (team_index, m) in (&mut ii).zip(self.inner.iter()) {
        //     m.execute_move_ani(kk, team_index, doop).await;
        // }
        // assert!(kk.game_is_over(ii.next().unwrap()).is_some());
    }
}

// #[derive(Copy, Clone, Debug)]
// pub enum Steering {
//     Left,
//     Right,
//     LeftLeft,
//     RightRight,
//     None,
// }

// #[derive(Copy, Clone, Debug)]
// pub enum Attackable {
//     Yes,
//     No,
// }

// #[derive(Copy, Clone, Debug)]
// pub enum StopsIter {
//     Yes,
//     No,
// }

// #[derive(Copy, Clone, Debug)]
// pub enum ResetIter {
//     Yes,
//     No,
// }

// pub const WARRIOR_STEERING: [(GridCoord, Steering, Attackable, StopsIter, ResetIter); 6] = {
//     let f1 = GridCoord([0, 0]).advance(HexDir { dir: 0 });
//     let f2 = GridCoord([0, 0]).advance(HexDir { dir: 1 });
//     let f3 = GridCoord([0, 0]).advance(HexDir { dir: 2 });

//     let f4 = GridCoord([0, 0]).advance(HexDir { dir: 3 });
//     let f5 = GridCoord([0, 0]).advance(HexDir { dir: 4 });
//     let f6 = GridCoord([0, 0]).advance(HexDir { dir: 5 });

//     [
//         (
//             f1,
//             Steering::None,
//             Attackable::No,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f2,
//             Steering::None,
//             Attackable::No,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f3,
//             Steering::None,
//             Attackable::No,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f4,
//             Steering::None,
//             Attackable::No,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f5,
//             Steering::None,
//             Attackable::No,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f6,
//             Steering::None,
//             Attackable::No,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//     ]
// };

// pub const WARRIOR_STEERING_ATTACKABLE: [(GridCoord, Steering, Attackable, StopsIter, ResetIter);
//     6] = {
//     let f1 = GridCoord([0, 0]).advance(HexDir { dir: 0 });
//     let f2 = GridCoord([0, 0]).advance(HexDir { dir: 1 });
//     let f3 = GridCoord([0, 0]).advance(HexDir { dir: 2 });

//     let f4 = GridCoord([0, 0]).advance(HexDir { dir: 3 });
//     let f5 = GridCoord([0, 0]).advance(HexDir { dir: 4 });
//     let f6 = GridCoord([0, 0]).advance(HexDir { dir: 5 });

//     [
//         (
//             f1,
//             Steering::None,
//             Attackable::Yes,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f2,
//             Steering::None,
//             Attackable::Yes,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f3,
//             Steering::None,
//             Attackable::Yes,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f4,
//             Steering::None,
//             Attackable::Yes,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f5,
//             Steering::None,
//             Attackable::Yes,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f6,
//             Steering::None,
//             Attackable::Yes,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//     ]
// };

// pub const WARRIOR_STEERING_OLD2: [(GridCoord, Steering, Attackable, StopsIter, ResetIter); 6] = {
//     let f1 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_left());
//     let f2 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_right());
//     let f3 = GridCoord([0, 0]).advance(HexDir { dir: 0 });

//     let f4 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_left().rotate60_left());
//     let f5 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_right().rotate60_right());
//     let f6 = GridCoord([0, 0]).advance(
//         HexDir { dir: 0 }
//             .rotate60_right()
//             .rotate60_right()
//             .rotate60_right(),
//     );

//     [
//         (
//             f1,
//             Steering::None,
//             Attackable::Yes,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f2,
//             Steering::None,
//             Attackable::Yes,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f3,
//             Steering::None,
//             Attackable::No,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f4,
//             Steering::None,
//             Attackable::No,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f5,
//             Steering::None,
//             Attackable::No,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f6,
//             Steering::None,
//             Attackable::No,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//     ]
// };

// pub const WARRIOR_STEERING_OLD: [(GridCoord, Steering, Attackable, StopsIter, ResetIter); 3] = {
//     let f1 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_left());
//     let f2 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_right());
//     let f3 = GridCoord([0, 0]).advance(HexDir { dir: 0 });
//     [
//         (
//             f1,
//             Steering::Left,
//             Attackable::Yes,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f2,
//             Steering::Right,
//             Attackable::Yes,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f3,
//             Steering::None,
//             Attackable::No,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//     ]
// };

// pub const LANCER_STEERING: [(GridCoord, Steering, Attackable, StopsIter, ResetIter); 5] = {
//     let f1 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_left());
//     let f2 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_right());
//     let f3 = GridCoord([0, 0]).advance(HexDir { dir: 0 });

//     let f4 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_right().rotate60_right());
//     let f4 = f2.add(f4);

//     let f5 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_left().rotate60_left());
//     let f5 = f1.add(f5);

//     [
//         (
//             f1,
//             Steering::Left,
//             Attackable::Yes,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f5,
//             Steering::LeftLeft,
//             Attackable::Yes,
//             StopsIter::Yes,
//             ResetIter::No,
//         ),
//         (
//             f2,
//             Steering::Right,
//             Attackable::Yes,
//             StopsIter::No,
//             ResetIter::Yes,
//         ),
//         (
//             f4,
//             Steering::RightRight,
//             Attackable::Yes,
//             StopsIter::Yes,
//             ResetIter::Yes,
//         ),
//         (
//             f3,
//             Steering::None,
//             Attackable::No,
//             StopsIter::No,
//             ResetIter::Yes,
//         ),
//     ]
// };
// pub const ARCHER_STEERING: [(GridCoord, Steering, Attackable, StopsIter, ResetIter); 4] = {
//     let f1 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_left());
//     let f2 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_right());

//     let f3 = GridCoord([0, 0]).advance(HexDir { dir: 0 });
//     let f4 = GridCoord([0, 0])
//         .advance(HexDir { dir: 0 })
//         .advance(HexDir { dir: 0 });

//     [
//         (
//             f1,
//             Steering::Left,
//             Attackable::Yes,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f2,
//             Steering::Right,
//             Attackable::Yes,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f3,
//             Steering::None,
//             Attackable::No,
//             StopsIter::Yes,
//             ResetIter::No,
//         ),
//         (
//             f4,
//             Steering::None,
//             Attackable::Yes,
//             StopsIter::Yes,
//             ResetIter::No,
//         ),
//     ]
// };

// // pub const LANCER_STEERING: [(GridCoord, Steering, Attackable, StopsIter, ResetIter); 4] = {
// //     let f1 = GridCoord([0, 0]).advance(HexDir { dir: 0 });
// //     let f2 = f1.advance(HexDir { dir: 0 }.rotate60_left());
// //     let f3 = f1.advance(HexDir { dir: 0 }.rotate60_right());

// //     [
// //         (
// //             f1,
// //             Steering::None,
// //             Attackable::Yes,
// //             StopsIter::Yes,
// //             ResetIter::No,
// //         ),
// //         (
// //             f2,
// //             Steering::Left,
// //             Attackable::Yes,
// //             StopsIter::Yes,
// //             ResetIter::No,
// //         ),
// //         (
// //             f1,
// //             Steering::None,
// //             Attackable::Yes,
// //             StopsIter::Yes,
// //             ResetIter::Yes,
// //         ),
// //         (
// //             f3,
// //             Steering::Right,
// //             Attackable::Yes,
// //             StopsIter::Yes,
// //             ResetIter::No,
// //         ),
// //     ]
// // };

// pub const CATAPAULT_STEERING: [(GridCoord, Steering, Attackable, StopsIter, ResetIter); 5] = {
//     let ff1 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_left().rotate60_left());
//     let ff2 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_right().rotate60_right());
//     let f3 = GridCoord([0, 0]).advance(HexDir { dir: 0 });
//     let f4 = GridCoord([0, 0])
//         .advance(HexDir { dir: 0 })
//         .advance(HexDir { dir: 0 });
//     let f5 = GridCoord([0, 0])
//         .advance(HexDir { dir: 0 })
//         .advance(HexDir { dir: 0 })
//         .advance(HexDir { dir: 0 });

//     [
//         (
//             ff1,
//             Steering::Right,
//             Attackable::No,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             ff2,
//             Steering::Left,
//             Attackable::No,
//             StopsIter::No,
//             ResetIter::No,
//         ),
//         (
//             f3,
//             Steering::None,
//             Attackable::Yes,
//             StopsIter::Yes,
//             ResetIter::No,
//         ),
//         (
//             f4,
//             Steering::None,
//             Attackable::Yes,
//             StopsIter::Yes,
//             ResetIter::No,
//         ),
//         (
//             f5,
//             Steering::None,
//             Attackable::Yes,
//             StopsIter::Yes,
//             ResetIter::No,
//         ),
//     ]
// };
