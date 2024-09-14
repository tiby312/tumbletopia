use super::*;

#[derive(Clone, Debug)]
pub struct HaveMoved {
    pub the_move: ActualMove,
    pub effect: move_build::MoveEffect,
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
