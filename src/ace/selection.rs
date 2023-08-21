use crate::movement::MovementMesh;

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
        let foo = if self.extra.prev_coord == self.unit {
            Some(2)
        } else {
            None
        };

        generate_unit_possible_moves_inner(&self.unit, game, foo)
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

        if let Some(_) = game_view.that_team.find_slow_mut(&target_cell) {
            let iii = moves::Invade::new(unit, mesh, target_cell);

            let iii = iii.execute_with_animation(game_view, doop, |_| {}).await;
            move_log.push(moves::ActualMove::ExtraMove(
                self.extra.prev_move.clone(),
                iii,
            ));
        } else {
            let pm = moves::PartialMove::new(unit, mesh, target_cell);
            let jjj = pm
                .clone()
                .execute_with_animation(game_view, doop, |_| {})
                .await;

            let jjj = match jjj {
                (sigl, moves::ExtraMove::ExtraMove { unit }) => {
                    //move_log.push(moves::ActualMove::NormalMove(sigl));
                    move_log.push(moves::ActualMove::ExtraMove(
                        self.extra.prev_move.clone(),
                        sigl,
                    ));
                    Some(unit.position)
                    //Some(selection::PossibleExtra::new(sigl, unit.clone()))
                }
                (sigl, moves::ExtraMove::FinishMoving) => {
                    //move_log.push(moves::ActualMove::NormalMove(sigl));
                    move_log.push(moves::ActualMove::ExtraMove(
                        self.extra.prev_move.clone(),
                        sigl,
                    ));

                    None
                }
            };

            if let Some(a) = jjj {
                let _ = game_view.this_team.find_take(&a);
            }
        };

        Ok(())
    }
    pub fn execute_no_animation(
        &self,
        target_cell: GridCoord,
        mesh: movement::MovementMesh,
        game_view: &mut GameViewMut<'_, '_>,
        move_log: &mut MoveLog,
    ) -> Result<(), NoPathErr> {
        let unit = self.unit.position;

        if let Some(_) = game_view.that_team.find_slow_mut(&target_cell) {
            let iii = moves::Invade::new(unit, mesh, target_cell);

            let iii = iii.execute(game_view, |_| {});
            move_log.push(moves::ActualMove::ExtraMove(
                self.extra.prev_move.clone(),
                iii,
            ));
        } else {
            let pm = moves::PartialMove::new(unit, mesh, target_cell);
            let jjj = pm.clone().execute(game_view, |_| {});
            let jjj = match jjj {
                (sigl, moves::ExtraMove::ExtraMove { unit }) => {
                    //move_log.push(moves::ActualMove::NormalMove(sigl));
                    move_log.push(moves::ActualMove::ExtraMove(
                        self.extra.prev_move.clone(),
                        sigl,
                    ));
                    Some(unit.position)
                    //Some(selection::PossibleExtra::new(sigl, unit.clone()))
                }
                (sigl, moves::ExtraMove::FinishMoving) => {
                    move_log.push(moves::ActualMove::ExtraMove(
                        self.extra.prev_move.clone(),
                        sigl,
                    ));

                    //move_log.push(moves::ActualMove::NormalMove(sigl));

                    None
                }
            };

            if let Some(a) = jjj {
                let _ = game_view.this_team.find_take(&a);
            }
        };

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
        generate_unit_possible_moves_inner(&self.unit, game, None)
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

        let e = if let Some(_) = game_view.that_team.find_slow_mut(&target_cell) {
            let iii = moves::Invade::new(unit, mesh, target_cell);

            let iii = iii.execute_with_animation(game_view, doop, |_| {}).await;

            move_log.push(moves::ActualMove::NormalMove(iii));

            None
        } else {
            let pm = moves::PartialMove::new(unit, mesh, target_cell);
            let jjj = pm
                .clone()
                .execute_with_animation(game_view, doop, |_| {})
                .await;

            match jjj {
                (sigl, moves::ExtraMove::ExtraMove { unit }) => {
                    Some(selection::PossibleExtra::new(sigl, unit.clone()))
                }
                (sigl, moves::ExtraMove::FinishMoving) => {
                    move_log.push(moves::ActualMove::NormalMove(sigl));
                    None
                }
            }
        };

        Ok(e)
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

        let e = if let Some(_) = game_view.that_team.find_slow_mut(&target_cell) {
            let iii = moves::Invade::new(unit, mesh, target_cell);

            let iii = iii.execute(game_view, |_| {});

            move_log.push(moves::ActualMove::NormalMove(iii));
            None
        } else {
            let pm = moves::PartialMove::new(unit, mesh, target_cell);
            let jjj = pm.clone().execute(game_view, |_| {});

            match jjj {
                (sigl, moves::ExtraMove::ExtraMove { unit }) => {
                    Some(selection::PossibleExtra::new(sigl, unit.clone()))
                }
                (sigl, moves::ExtraMove::FinishMoving) => {
                    move_log.push(moves::ActualMove::NormalMove(sigl));
                    None
                }
            }
        };

        Ok(e)
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

fn generate_unit_possible_moves_inner(
    unit: &UnitData,
    game: &GameViewMut,
    extra_attack: Option<i16>,
) -> movement::MovementMesh {
    // If there is an enemy near by restrict movement.

    let restricted_movement = if let Some(_) = unit
        .position
        .to_cube()
        .ring(1)
        .map(|s| game.that_team.find_slow(&s.to_axial()).is_some())
        .find(|a| *a)
    {
        true
    } else {
        match unit.typ {
            Type::Warrior => false,
            Type::Para => true,
            _ => todo!(),
        }
    };

    let mm = if let Some(extra_attack_range) = extra_attack {
        let mut m = MovementMesh::new();

        let f = game
            .world
            .filter()
            .and(
                game.that_team
                    .filter_type(Type::Warrior)
                    .and(game.that_team.filter())
                    .not(),
            )
            .and(game.this_team.filter().not());
        for a in unit.position.to_cube().ring(extra_attack_range) {
            let a = a.to_axial();
            if let movement::FilterRes::Accept = f.filter(&a) {
                m.add(a.sub(&unit.position));
            }
        }
        m

        // movement::compute_moves2(
        //     unit.position,
        //     &game.world.filter().and(game.that_team.filter()),
        //     &movement::NoFilter,
        //     restricted_movement,
        //     false,
        // )
        // movement::compute_moves(
        //     &movement::WarriorMovement,
        //     ,
        //     &movement::NoFilter,
        //     &terrain::Grass,
        //     unit.position,
        //     MoveUnit(1),
        //     false,
        //     ph,
        // )
    } else {
        movement::compute_moves2(
            unit.position,
            &game
                .world
                .filter()
                .and(
                    game.that_team
                        .filter_type(Type::Warrior)
                        .and(game.that_team.filter())
                        .not(),
                )
                .and(game.this_team.filter().not()),
            &game.this_team.filter(),
            restricted_movement,
            true,
        )
    };
    mm
}
