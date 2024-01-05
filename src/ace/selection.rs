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
        generate_unit_possible_moves_inner(&self.unit.position, self.unit.typ, game, true)
    }
    pub async fn execute(
        &self,
        target_cell: GridCoord,
        mesh: movement::MovementMesh,
        game_view: &mut GameViewMut<'_, '_>,
        doop: &mut ace::WorkerManager<'_>
    ) -> Result<(), NoPathErr> {
        let unit = self.unit.position;

        let iii = moves::PartialMove {
            selected_unit: unit,
            typ: self.unit.typ,
            end: target_cell,
            is_extra: true,
        };

        let _ = iii.execute_with_animation(game_view, doop, mesh).await;

        Ok(())
    }
    pub fn execute_no_animation(
        &self,
        target_cell: GridCoord,
        game_view: &mut GameViewMut<'_, '_>
    ) -> Result<(), NoPathErr> {
        let unit = self.unit.position;

        let iii = moves::PartialMove {
            selected_unit: unit,
            typ: self.unit.typ,
            end: target_cell,
            is_extra: true,
        };

        let _ = iii.execute(game_view);

        Ok(())
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

    pub fn generate(&self, game: &GameViewMut) -> movement::MovementMesh {
        generate_unit_possible_moves_inner(&self.unit.position, self.unit.typ, game, false)
    }

    pub async fn execute(
        &self,
        target_cell: GridCoord,
        mesh: movement::MovementMesh,
        game_view: &mut GameViewMut<'_, '_>,
        doop: &mut ace::WorkerManager<'_>,
    ) -> Result<Option<selection::PossibleExtra>, NoPathErr> {
        let unit = self.unit.position;

        let iii = moves::PartialMove {
            selected_unit: unit,
            typ: self.unit.typ,
            end: target_cell,
            is_extra: false,
        };

        let iii = iii.execute_with_animation(game_view, doop, mesh).await;

        Ok(match iii {
            (sigl, moves::ExtraMove::ExtraMove { unit }) => {
                Some(selection::PossibleExtra::new(sigl, unit.clone()))
            }
            (sigl, moves::ExtraMove::FinishMoving) => {
                unreachable!();
                //move_log.push(moves::ActualMove::NormalMove(sigl));
                None
            }
        })
    }
    pub fn execute_no_animation(
        &self,
        target_cell: GridCoord,
        game_view: &mut GameViewMut<'_, '_>,
    ) -> Result<Option<selection::PossibleExtra>, NoPathErr> {
        let unit = self.unit.position;

        let iii = moves::PartialMove {
            selected_unit: unit,
            typ: self.unit.typ,
            end: target_cell,
            is_extra: false,
        };

        let iii = iii.execute(game_view);

        Ok(match iii {
            (sigl, moves::ExtraMove::ExtraMove { unit }) => {
                Some(selection::PossibleExtra::new(sigl, unit.clone()))
            }
            (sigl, moves::ExtraMove::FinishMoving) => {
                unreachable!();
                //move_log.push(moves::ActualMove::NormalMove(sigl));
                None
            }
        })
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
    typ: Type,
    game: &GameViewMut,
    extra: bool,
) -> movement::MovementMesh {
    let unit = *unit;
    let mut mesh = movement::MovementMesh::new(vec![]);

    let cond = |a: GridCoord| {
        let cc = if typ == Type::Ship {
            game.land.iter().find(|&&b| a == b).is_none()
        } else if typ == Type::Foot {
            game.land.iter().find(|&&b| a == b).is_some()
                && game.forest.iter().find(|&&b| a == b).is_none()
        } else {
            unreachable!();
        };

        let is_world_cell = game.world.filter().filter(&a).to_bool();
        a != unit && is_world_cell && cc
    };
    let cond2 = |a: GridCoord| {
        game.this_team.find_slow(&a).is_none() && game.that_team.find_slow(&a).is_none()
    };

    for (_, a) in unit.to_cube().ring(1) {
        let a = a.to_axial();

        if cond(a) {
            if cond2(a) {
                mesh.add_normal_cell(a.sub(&unit));
            }
            if !extra {
                for (_, b) in a.to_cube().ring(1) {
                    let b = b.to_axial();
                    //TODO inefficient
                    if cond(b) {
                        if cond2(b) {
                            mesh.add_normal_cell(b.sub(&unit));
                        }
                    }
                }
            }
        }
    }

    mesh
}
