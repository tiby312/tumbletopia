use super::*;

#[derive(Clone)]
pub struct PossibleExtra {
    pub prev_move: moves::PartialMoveSigl,
    pub prev_coord: UnitData,
}
impl PossibleExtra {
    pub fn new(prev_move: moves::PartialMoveSigl, prev_coord: UnitData) -> Self {
        PossibleExtra {
            prev_move,
            prev_coord,
        }
    }

    // pub fn prev_move(&self) -> &moves::PartialMoveSigl {
    //     &self.prev_move
    // }
    pub fn coord(&self) -> GridCoord {
        self.prev_coord.position
    }
}

pub struct MoveLog {
    pub inner: Vec<moves::ActualMove>,
}
impl MoveLog {
    pub fn new() -> Self {
        MoveLog { inner: vec![] }
    }
    pub fn push(&mut self, o: moves::ActualMove) {
        self.inner.push(o);
    }

    pub async fn replay(&self, doop: &mut WorkerManager<'_>, kk: &mut GameState) {
        for (team_index, m) in ActiveTeam::Dogs.iter().zip(self.inner.iter()) {
            m.execute_move_ani(kk, team_index, doop).await;
        }
        assert!(kk.game_is_over().is_some());
    }
    pub fn deserialize(buffer: Vec<u8>) -> MoveLog {
        use byteorder::{BigEndian, ReadBytesExt};
        use std::io::Cursor;
        let mut rdr = Cursor::new(buffer);
        let ver = rdr.read_u32::<BigEndian>().unwrap();
        assert_eq!(ver, 0);
        let num = rdr.read_u32::<BigEndian>().unwrap();

        let mut ret = vec![];
        for _ in 0..num {
            let vals = [
                rdr.read_i16::<BigEndian>().unwrap(),
                rdr.read_i16::<BigEndian>().unwrap(),
                rdr.read_i16::<BigEndian>().unwrap(),
                rdr.read_i16::<BigEndian>().unwrap(),
                rdr.read_i16::<BigEndian>().unwrap(),
                rdr.read_i16::<BigEndian>().unwrap(),
            ];
            todo!()
            // ret.push(moves::ActualMove {
            //     unit: GridCoord([vals[0], vals[1]]),
            //     moveto: GridCoord([vals[2], vals[3]]),
            //     attackto: GridCoord([vals[4], vals[5]]),
            // })
        }
        MoveLog { inner: ret }
    }
    pub fn serialize(&self) -> Vec<u8> {
        let o = &self.inner;
        use byteorder::{BigEndian, WriteBytesExt};

        let mut wtr = vec![];

        let version = 0;
        wtr.write_u32::<BigEndian>(version).unwrap();

        wtr.write_u32::<BigEndian>(o.len().try_into().unwrap())
            .unwrap();

        for a in o.iter() {
            todo!();
            // wtr.write_i16::<BigEndian>(a.unit.0[0]).unwrap();
            // wtr.write_i16::<BigEndian>(a.unit.0[1]).unwrap();
            // wtr.write_i16::<BigEndian>(a.moveto.0[0]).unwrap();
            // wtr.write_i16::<BigEndian>(a.moveto.0[1]).unwrap();
            // wtr.write_i16::<BigEndian>(a.attackto.0[0]).unwrap();
            // wtr.write_i16::<BigEndian>(a.attackto.0[1]).unwrap();
        }
        wtr
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Steering {
    Left,
    Right,
    LeftLeft,
    RightRight,
    None,
}

#[derive(Copy, Clone, Debug)]
pub enum Attackable {
    Yes,
    No,
}

#[derive(Copy, Clone, Debug)]
pub enum StopsIter {
    Yes,
    No,
}

#[derive(Copy, Clone, Debug)]
pub enum ResetIter {
    Yes,
    No,
}

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
