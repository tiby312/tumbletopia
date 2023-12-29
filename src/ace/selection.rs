use duckduckgeo::dists::grid::Grid;

use crate::movement::{
    movement_mesh::{SwingMove, SwingMoveRay},
    ComputeMovesRes, HexDir, MovementMesh,
};

use super::*;

pub enum SelectionType {
    Normal(selection::RegularSelection),
    Extra(selection::ComboContinueSelection),
}

#[derive(Clone)]
pub struct PossibleExtra {
    prev_move: moves::PartialMoveSigl,
    //prev_coord: GridCoord,
    prev_coord: UnitData,
}
impl PossibleExtra {
    pub fn new(prev_move: moves::PartialMoveSigl, prev_coord: UnitData) -> Self {
        PossibleExtra {
            prev_move,
            prev_coord,
        }
    }
    pub fn select(&self) -> ComboContinueSelection {
        ComboContinueSelection {
            extra: self.clone(),
            unit: self.prev_coord.clone(),
        }
    }
    pub fn prev_move(&self) -> &moves::PartialMoveSigl {
        &self.prev_move
    }
    pub fn coord(&self) -> GridCoord {
        self.prev_coord.position
    }
}

#[derive(Clone)]
pub struct ComboContinueSelection {
    extra: PossibleExtra,
    unit: UnitData,
}
#[derive(Debug)]
pub struct NoPathErr;
impl ComboContinueSelection {
    pub fn generate(&self, game: &GameViewMut) -> movement::MovementMesh {
        // self.extra
        // .prev_move
        // .unit
        // .to_cube()
        // .dist(&self.extra.prev_move.moveto.to_cube())
        // let foo = if self.extra.prev_coord == self.unit {
        //     Some(2)
        // } else {
        //     None
        // };

        generate_unit_possible_moves_inner(&self.unit.position, game, Some(self.extra.prev_move.unit))
    }
    pub async fn execute(
        &self,
        target_cell: GridCoord,
        mesh: movement::MovementMesh,
        game_view: &mut GameViewMut<'_, '_>,
        doop: &mut ace::WorkerManager<'_>,
        move_log: &mut MoveLog,
    ) -> Result<(), NoPathErr> {
        let unit = self.unit.position;

        let iii = moves::PartialMove::new(unit, mesh, target_cell, true);

        let iii = iii.execute_with_animation(game_view, doop, |_| {}).await;

        //TODO add this!

        move_log.push(moves::ActualMove::ExtraMove(
            self.extra.prev_move.clone(),
            iii.0,
        ));

        Ok(())
        // if let Some(_) = game_view.that_team.find_slow_mut(&target_cell) {
        //     let iii = moves::Invade::new(unit, mesh, target_cell);

        //     let iii = iii.execute_with_animation(game_view, doop, |_| {}).await;
        //     move_log.push(moves::ActualMove::ExtraMove(
        //         self.extra.prev_move.clone(),
        //         iii,
        //     ));
        // } else {
        //     let pm = moves::PartialMove::new(unit, mesh, target_cell);
        //     let jjj = pm
        //         .clone()
        //         .execute_with_animation(game_view, doop, |_| {})
        //         .await;

        //     let jjj = match jjj {
        //         (sigl, moves::ExtraMove::ExtraMove { unit }) => {
        //             //move_log.push(moves::ActualMove::NormalMove(sigl));
        //             move_log.push(moves::ActualMove::ExtraMove(
        //                 self.extra.prev_move.clone(),
        //                 sigl,
        //             ));
        //             Some(unit.position)
        //             //Some(selection::PossibleExtra::new(sigl, unit.clone()))
        //         }
        //         (sigl, moves::ExtraMove::FinishMoving) => {
        //             //move_log.push(moves::ActualMove::NormalMove(sigl));
        //             move_log.push(moves::ActualMove::ExtraMove(
        //                 self.extra.prev_move.clone(),
        //                 sigl,
        //             ));

        //             None
        //         }
        //     };

        //     if let Some(a) = jjj {
        //         let _ = game_view.this_team.find_take(&a);
        //     }
        // };

        // Ok(())
    }
    pub fn execute_no_animation(
        &self,
        target_cell: GridCoord,
        mesh: movement::MovementMesh,
        game_view: &mut GameViewMut<'_, '_>,
        move_log: &mut MoveLog,
    ) -> Result<(), NoPathErr> {
        let unit = self.unit.position;

        let iii = moves::Invade::new(unit, mesh, target_cell);

        let iii = iii.execute(game_view, |_| {});
        move_log.push(moves::ActualMove::ExtraMove(
            self.extra.prev_move.clone(),
            iii,
        ));

        Ok(())
        // if let Some(_) = game_view.that_team.find_slow_mut(&target_cell) {
        //     let iii = moves::Invade::new(unit, mesh, target_cell);

        //     let iii = iii.execute(game_view, |_| {});
        //     move_log.push(moves::ActualMove::ExtraMove(
        //         self.extra.prev_move.clone(),
        //         iii,
        //     ));
        // } else {
        //     let pm = moves::PartialMove::new(unit, mesh, target_cell);
        //     let jjj = pm.clone().execute(game_view, |_| {});
        //     let jjj = match jjj {
        //         (sigl, moves::ExtraMove::ExtraMove { unit }) => {
        //             //move_log.push(moves::ActualMove::NormalMove(sigl));
        //             move_log.push(moves::ActualMove::ExtraMove(
        //                 self.extra.prev_move.clone(),
        //                 sigl,
        //             ));
        //             Some(unit.position)
        //             //Some(selection::PossibleExtra::new(sigl, unit.clone()))
        //         }
        //         (sigl, moves::ExtraMove::FinishMoving) => {
        //             move_log.push(moves::ActualMove::ExtraMove(
        //                 self.extra.prev_move.clone(),
        //                 sigl,
        //             ));

        //             //move_log.push(moves::ActualMove::NormalMove(sigl));

        //             None
        //         }
        //     };

        //     if let Some(a) = jjj {
        //         let _ = game_view.this_team.find_take(&a);
        //     }
        // };

        // Ok(())
    }
}

#[derive(Clone)]
pub struct RegularSelection {
    pub unit: UnitData,
}

impl RegularSelection {
    pub fn new(a: &UnitData) -> Self {
        RegularSelection { unit: a.clone() }
    }
    // fn get_path_from_move(
    //     &self,
    //     target_cell: GridCoord,
    //     game: &GameViewMut,
    // ) -> Result<movement::Path, NoPathErr> {
    //     //Reconstruct possible paths with path information this time.
    //     let ss = generate_unit_possible_moves_inner(&self.unit, game, &None);

    //     let path_iter = ss.path(target_cell.sub(&self.unit.position));

    //     //TODO return iterator instead?
    //     let mut p = movement::Path::new();
    //     for a in path_iter {
    //         p.add(a);
    //     }
    //     Ok(p)
    // }
    pub fn generate(&self, game: &GameViewMut) -> movement::MovementMesh {
        generate_unit_possible_moves_inner(&self.unit.position, game, None)
    }

    pub async fn execute(
        &self,
        target_cell: GridCoord,
        mesh: movement::MovementMesh,
        game_view: &mut GameViewMut<'_, '_>,
        doop: &mut ace::WorkerManager<'_>,
        move_log: &mut MoveLog,
    ) -> Result<Option<selection::PossibleExtra>, NoPathErr> {
        //let path = self.get_path_from_move(target_cell, game_view)?;
        let unit = self.unit.position;

        let iii = moves::PartialMove::new(unit, mesh, target_cell, false);

        let iii = iii.execute_with_animation(game_view, doop, |_| {}).await;

        //move_log.push(moves::ActualMove::NormalMove(iii));

        Ok(match iii {
            (sigl, moves::ExtraMove::ExtraMove { unit }) => {
                Some(selection::PossibleExtra::new(sigl, unit.clone()))
            }
            (sigl, moves::ExtraMove::FinishMoving) => {
                move_log.push(moves::ActualMove::NormalMove(sigl));
                None
            }
        })

        //TODO here

        // let e = if let Some(_) = game_view.that_team.find_slow_mut(&target_cell) {
        //     let iii = moves::Invade::new(unit, mesh, target_cell);

        //     let iii = iii.execute_with_animation(game_view, doop, |_| {}).await;

        //     move_log.push(moves::ActualMove::NormalMove(iii));

        //     None
        // } else {
        //     let pm = moves::PartialMove::new(unit, mesh, target_cell);
        //     let jjj = pm
        //         .clone()
        //         .execute_with_animation(game_view, doop, |_| {})
        //         .await;

        //     match jjj {
        //         (sigl, moves::ExtraMove::ExtraMove { unit }) => {
        //             Some(selection::PossibleExtra::new(sigl, unit.clone()))
        //         }
        //         (sigl, moves::ExtraMove::FinishMoving) => {
        //             move_log.push(moves::ActualMove::NormalMove(sigl));
        //             None
        //         }
        //     }
        // };
    }
    pub fn execute_no_animation(
        &self,
        target_cell: GridCoord,
        mesh: movement::MovementMesh,
        game_view: &mut GameViewMut<'_, '_>,
        move_log: &mut MoveLog,
    ) -> Result<Option<selection::PossibleExtra>, NoPathErr> {
        //let path = self.get_path_from_move(target_cell, game_view)?;
        let unit = self.unit.position;

        let iii = moves::PartialMove::new(unit, mesh, target_cell, false);

        let iii = iii.execute(game_view, |_| {});

        //move_log.push(moves::ActualMove::NormalMove(iii));

        Ok(match iii {
            (sigl, moves::ExtraMove::ExtraMove { unit }) => {
                Some(selection::PossibleExtra::new(sigl, unit.clone()))
            }
            (sigl, moves::ExtraMove::FinishMoving) => {
                move_log.push(moves::ActualMove::NormalMove(sigl));
                None
            }
        })

        // let iii = moves::Invade::new(unit, mesh, target_cell);

        // let iii = iii.execute(game_view, |_| {});

        // move_log.push(moves::ActualMove::NormalMove(iii));

        // Ok(None)

        // let e = if let Some(_) = game_view.that_team.find_slow_mut(&target_cell) {
        //     let iii = moves::Invade::new(unit, mesh, target_cell);

        //     let iii = iii.execute(game_view, |_| {});

        //     move_log.push(moves::ActualMove::NormalMove(iii));
        //     None
        // } else {
        //     let pm = moves::PartialMove::new(unit, mesh, target_cell);
        //     let jjj = pm.clone().execute(game_view, |_| {});

        //     match jjj {
        //         (sigl, moves::ExtraMove::ExtraMove { unit }) => {
        //             Some(selection::PossibleExtra::new(sigl, unit.clone()))
        //         }
        //         (sigl, moves::ExtraMove::FinishMoving) => {
        //             move_log.push(moves::ActualMove::NormalMove(sigl));
        //             None
        //         }
        //     }
        // };

        // Ok(e)
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
    // pub fn add_invade(&mut self, i: moves::InvadeSigl) {}
    // pub fn add_movement(&mut self, a: moves::MovementSigl) {}
}

pub fn has_restricted_movement(unit: &UnitData, game: &GameView) -> bool {
    // let restricted_movement = if let Some(_) = unit
    //     .position
    //     .to_cube()
    //     .ring(1)
    //     .map(|s| game.that_team.find_slow(&s.to_axial()).is_some())
    //     .find(|a| *a)
    // {
    //     true
    // } else {
    match unit.typ {
        Type::Warrior { .. } => false,
        Type::King => true,
        Type::Archer => false,
        Type::Catapault => true,
        Type::Catapault => true,
        _ => todo!(),
    }
    // };
    // restricted_movement
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

pub const WARRIOR_STEERING: [(GridCoord, Steering, Attackable, StopsIter, ResetIter); 6] = {
    let f1 = GridCoord([0, 0]).advance(HexDir { dir: 0 });
    let f2 = GridCoord([0, 0]).advance(HexDir { dir: 1 });
    let f3 = GridCoord([0, 0]).advance(HexDir { dir: 2 });

    let f4 = GridCoord([0, 0]).advance(HexDir { dir: 3 });
    let f5 = GridCoord([0, 0]).advance(HexDir { dir: 4 });
    let f6 = GridCoord([0, 0]).advance(HexDir { dir: 5 });

    [
        (
            f1,
            Steering::None,
            Attackable::No,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f2,
            Steering::None,
            Attackable::No,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f3,
            Steering::None,
            Attackable::No,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f4,
            Steering::None,
            Attackable::No,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f5,
            Steering::None,
            Attackable::No,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f6,
            Steering::None,
            Attackable::No,
            StopsIter::No,
            ResetIter::No,
        ),
    ]
};

pub const WARRIOR_STEERING_ATTACKABLE: [(GridCoord, Steering, Attackable, StopsIter, ResetIter);
    6] = {
    let f1 = GridCoord([0, 0]).advance(HexDir { dir: 0 });
    let f2 = GridCoord([0, 0]).advance(HexDir { dir: 1 });
    let f3 = GridCoord([0, 0]).advance(HexDir { dir: 2 });

    let f4 = GridCoord([0, 0]).advance(HexDir { dir: 3 });
    let f5 = GridCoord([0, 0]).advance(HexDir { dir: 4 });
    let f6 = GridCoord([0, 0]).advance(HexDir { dir: 5 });

    [
        (
            f1,
            Steering::None,
            Attackable::Yes,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f2,
            Steering::None,
            Attackable::Yes,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f3,
            Steering::None,
            Attackable::Yes,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f4,
            Steering::None,
            Attackable::Yes,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f5,
            Steering::None,
            Attackable::Yes,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f6,
            Steering::None,
            Attackable::Yes,
            StopsIter::No,
            ResetIter::No,
        ),
    ]
};

pub const WARRIOR_STEERING_OLD2: [(GridCoord, Steering, Attackable, StopsIter, ResetIter); 6] = {
    let f1 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_left());
    let f2 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_right());
    let f3 = GridCoord([0, 0]).advance(HexDir { dir: 0 });

    let f4 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_left().rotate60_left());
    let f5 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_right().rotate60_right());
    let f6 = GridCoord([0, 0]).advance(
        HexDir { dir: 0 }
            .rotate60_right()
            .rotate60_right()
            .rotate60_right(),
    );

    [
        (
            f1,
            Steering::None,
            Attackable::Yes,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f2,
            Steering::None,
            Attackable::Yes,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f3,
            Steering::None,
            Attackable::No,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f4,
            Steering::None,
            Attackable::No,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f5,
            Steering::None,
            Attackable::No,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f6,
            Steering::None,
            Attackable::No,
            StopsIter::No,
            ResetIter::No,
        ),
    ]
};

pub const WARRIOR_STEERING_OLD: [(GridCoord, Steering, Attackable, StopsIter, ResetIter); 3] = {
    let f1 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_left());
    let f2 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_right());
    let f3 = GridCoord([0, 0]).advance(HexDir { dir: 0 });
    [
        (
            f1,
            Steering::Left,
            Attackable::Yes,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f2,
            Steering::Right,
            Attackable::Yes,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f3,
            Steering::None,
            Attackable::No,
            StopsIter::No,
            ResetIter::No,
        ),
    ]
};

pub const LANCER_STEERING: [(GridCoord, Steering, Attackable, StopsIter, ResetIter); 5] = {
    let f1 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_left());
    let f2 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_right());
    let f3 = GridCoord([0, 0]).advance(HexDir { dir: 0 });

    let f4 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_right().rotate60_right());
    let f4 = f2.add(f4);

    let f5 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_left().rotate60_left());
    let f5 = f1.add(f5);

    [
        (
            f1,
            Steering::Left,
            Attackable::Yes,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f5,
            Steering::LeftLeft,
            Attackable::Yes,
            StopsIter::Yes,
            ResetIter::No,
        ),
        (
            f2,
            Steering::Right,
            Attackable::Yes,
            StopsIter::No,
            ResetIter::Yes,
        ),
        (
            f4,
            Steering::RightRight,
            Attackable::Yes,
            StopsIter::Yes,
            ResetIter::Yes,
        ),
        (
            f3,
            Steering::None,
            Attackable::No,
            StopsIter::No,
            ResetIter::Yes,
        ),
    ]
};
pub const ARCHER_STEERING: [(GridCoord, Steering, Attackable, StopsIter, ResetIter); 4] = {
    let f1 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_left());
    let f2 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_right());

    let f3 = GridCoord([0, 0]).advance(HexDir { dir: 0 });
    let f4 = GridCoord([0, 0])
        .advance(HexDir { dir: 0 })
        .advance(HexDir { dir: 0 });

    [
        (
            f1,
            Steering::Left,
            Attackable::Yes,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f2,
            Steering::Right,
            Attackable::Yes,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f3,
            Steering::None,
            Attackable::No,
            StopsIter::Yes,
            ResetIter::No,
        ),
        (
            f4,
            Steering::None,
            Attackable::Yes,
            StopsIter::Yes,
            ResetIter::No,
        ),
    ]
};

// pub const LANCER_STEERING: [(GridCoord, Steering, Attackable, StopsIter, ResetIter); 4] = {
//     let f1 = GridCoord([0, 0]).advance(HexDir { dir: 0 });
//     let f2 = f1.advance(HexDir { dir: 0 }.rotate60_left());
//     let f3 = f1.advance(HexDir { dir: 0 }.rotate60_right());

//     [
//         (
//             f1,
//             Steering::None,
//             Attackable::Yes,
//             StopsIter::Yes,
//             ResetIter::No,
//         ),
//         (
//             f2,
//             Steering::Left,
//             Attackable::Yes,
//             StopsIter::Yes,
//             ResetIter::No,
//         ),
//         (
//             f1,
//             Steering::None,
//             Attackable::Yes,
//             StopsIter::Yes,
//             ResetIter::Yes,
//         ),
//         (
//             f3,
//             Steering::Right,
//             Attackable::Yes,
//             StopsIter::Yes,
//             ResetIter::No,
//         ),
//     ]
// };

pub const CATAPAULT_STEERING: [(GridCoord, Steering, Attackable, StopsIter, ResetIter); 5] = {
    let ff1 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_left().rotate60_left());
    let ff2 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_right().rotate60_right());
    let f3 = GridCoord([0, 0]).advance(HexDir { dir: 0 });
    let f4 = GridCoord([0, 0])
        .advance(HexDir { dir: 0 })
        .advance(HexDir { dir: 0 });
    let f5 = GridCoord([0, 0])
        .advance(HexDir { dir: 0 })
        .advance(HexDir { dir: 0 })
        .advance(HexDir { dir: 0 });

    [
        (
            ff1,
            Steering::Right,
            Attackable::No,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            ff2,
            Steering::Left,
            Attackable::No,
            StopsIter::No,
            ResetIter::No,
        ),
        (
            f3,
            Steering::None,
            Attackable::Yes,
            StopsIter::Yes,
            ResetIter::No,
        ),
        (
            f4,
            Steering::None,
            Attackable::Yes,
            StopsIter::Yes,
            ResetIter::No,
        ),
        (
            f5,
            Steering::None,
            Attackable::Yes,
            StopsIter::Yes,
            ResetIter::No,
        ),
    ]
};

pub fn generate_unit_possible_moves_inner(
    unit: &GridCoord,
    game: &GameViewMut,
    extra_attack_prev_coord: Option<GridCoord>,
) -> movement::MovementMesh {
    let unit=*unit;
    let mut mesh = movement::MovementMesh::new(vec![]);

    let cond = |a: GridCoord| {
        let is_world_cell = game.world.filter().filter(&a).to_bool();
        a != unit && is_world_cell && game.land.iter().find(|&&b| a == b).is_none()
        //&& game.this_team.find_slow(&a).is_none()
        //&& game.that_team.find_slow(&a).is_none()
    };
    let cond2 = |a: GridCoord| {
        game.this_team.find_slow(&a).is_none() && game.that_team.find_slow(&a).is_none()
    };

    //TODO don't do this most of the time. ai doesnt care. used just for animation
    for a in unit.to_cube().range(2) {
        let a = a.to_axial();

        if game.land.iter().find(|&&b| a == b).is_some() {
            mesh.add_wall(a.sub(&unit));
        }
    }


    for (_, a) in unit.to_cube().ring(1) {
        let a = a.to_axial();

        if cond(a) {
            if cond2(a) {
                mesh.add_normal_cell(a.sub(&unit), false);
            }
            if extra_attack_prev_coord.is_none() {
                for (_, b) in a.to_cube().ring(1) {
                    let b = b.to_axial();
                    //TODO inefficient
                    if cond(b) {
                        if cond2(b) {
                            mesh.add_normal_cell(b.sub(&unit), false);
                        }
                    }
                }
            }
        }
    }

    mesh
}
