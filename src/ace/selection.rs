use crate::movement::{ComputeMovesRes, HexDir, MovementMesh};

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

        generate_unit_possible_moves_inner(&self.unit, game, Some(self.extra.prev_move.unit))
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

        let iii = moves::Invade::new(unit, mesh, target_cell);

        let iii = iii.execute_with_animation(game_view, doop, |_| {}).await;
        move_log.push(moves::ActualMove::ExtraMove(
            self.extra.prev_move.clone(),
            iii,
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

        let iii = moves::Invade::new(unit, mesh, target_cell);

        let iii = iii.execute_with_animation(game_view, doop, |_| {}).await;

        move_log.push(moves::ActualMove::NormalMove(iii));

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

        Ok(None)
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
        let iii = moves::Invade::new(unit, mesh, target_cell);

        let iii = iii.execute(game_view, |_| {});

        move_log.push(moves::ActualMove::NormalMove(iii));

        Ok(None)

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
        Type::Warrior => false,
        Type::Para => true,
        Type::Rook => false,
        Type::Mage => true,
        _ => todo!(),
    }
    // };
    // restricted_movement
}
#[derive(Debug)]
pub enum Steering {
    Left,
    Right,
    None,
}

#[derive(Debug)]
pub enum Attackable {
    Yes,
    No,
}

pub const WARRIOR_STEERING: [(GridCoord, Steering, Attackable); 3] = {
    let f1 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_left());
    let f2 = GridCoord([0, 0]).advance(HexDir { dir: 0 }.rotate60_right());
    let f3 = GridCoord([0, 0]).advance(HexDir { dir: 0 });
    [
        (f1, Steering::Left, Attackable::Yes),
        (f2, Steering::Right, Attackable::Yes),
        (f3, Steering::None, Attackable::No),
    ]
};

pub fn generate_unit_possible_moves_inner(
    unit: &UnitData,
    game: &GameViewMut,
    extra_attack_prev_coord: Option<GridCoord>,
) -> movement::MovementMesh {
    // If there is an enemy near by restrict movement.

    let restricted_movement = has_restricted_movement(unit, &game.into_const());

    let mm = if let Some(extra_attack_prev_coord) = extra_attack_prev_coord {
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

        let dir = unit.position.sub(&extra_attack_prev_coord).to_cube();
        let start = unit.position.to_cube();

        let positions = (0..4).map(|a| (0..a).fold(start, |acc, _| acc.add(dir)));
        //console_dbg!(dir, start, positions);

        for a in positions {
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
        if unit.typ == Type::Warrior {
            let mut mesh = movement::MovementMesh::new();

            let k = unit.direction;

            console_dbg!("START ROTATE", k);
            //let k=HexDir{dir: as u8};

            let m = WARRIOR_STEERING.map(|a| (a.0.to_cube().rotate_back(k), a.1, a.2));
            console_dbg!(WARRIOR_STEERING, m);

            for (rel_coord, _, attack) in m {
                let abs_coord = unit.position.add(rel_coord.to_axial());

                let mm = if let Attackable::No = attack {
                    if game.that_team.find_slow(&abs_coord).is_some() {
                        false
                    } else {
                        true
                    }
                } else {
                    true
                };

                let f1 = game.world.filter().filter(&abs_coord).to_bool();
                let f2 = game.this_team.filter().filter(&abs_coord).to_bool();

                if mm && f1 && !f2 {
                    mesh.add(rel_coord.to_axial());
                }
            }

            mesh
            // let rook_pos: Vec<_> = game
            //     .that_team
            //     .units
            //     .iter()
            //     .filter(|a| a.typ == Type::Rook)
            //     .map(|a| a.position)
            //     .collect();
            // let rook_pos = rook_pos
            //     .into_iter()
            //     .flat_map(|a| a.to_cube().neighbours().map(|a| a.to_axial()));
            // let rook_pos = rook_pos.filter(|a| game.that_team.find_slow(a).is_none());
            // let foo = movement::AcceptCoords::new(rook_pos.into_iter()).not();

            // movement::compute_moves22(unit.position, false, |pos| {
            //     let f1 = game.world.filter().filter(pos).to_bool();
            //     let f2 = foo.filter(pos).to_bool();
            //     let f3 = game.that_team.filter().filter(pos).to_bool();
            //     let f4 = game.this_team.filter().filter(pos).to_bool();

            //     if f1 && f2 && !f3 && !f4 {
            //         ComputeMovesRes::Add
            //     } else {
            //         if let Some(f) = game.that_team.find_slow(pos) {
            //             if f.typ == Type::Warrior || f.typ == Type::Para {
            //                 ComputeMovesRes::AddAndStop
            //             } else {
            //                 ComputeMovesRes::Stop
            //             }
            //         } else {
            //             ComputeMovesRes::Stop
            //         }
            //         // if game.that_team.filter().filter(pos).to_bool() {
            //         //     ComputeMovesRes::AddAndStop
            //         // } else {
            //         //     ComputeMovesRes::Stop
            //         // }
            //     }
            // })
        } else if unit.typ == Type::Rook {
            movement::compute_moves22(unit.position, false, |pos| {
                let f1 = game.world.filter().filter(pos).to_bool();
                let f3 = game.that_team.filter().filter(pos).to_bool();
                let f4 = game.this_team.filter().filter(pos).to_bool();

                if f1 && !f3 && !f4 {
                    ComputeMovesRes::Add
                } else {
                    // if let Some(k)=game.that_team.find_slow(pos){
                    //     if k.typ==Type::Rook{
                    //         ComputeMovesRes::Add
                    //     }else{
                    //         ComputeMovesRes::NoAddContinue
                    //     }
                    // }else{
                    //     ComputeMovesRes::NoAddContinue
                    // }

                    ComputeMovesRes::NoAddContinue
                    // if game.that_team.filter().filter(pos).to_bool(){
                    //     ComputeMovesRes::AddAndStop
                    // }else{
                    //     ComputeMovesRes::Stop
                    // }
                }
            })
        } else if unit.typ == Type::Para {
            movement::compute_moves2(
                unit.position,
                &game
                    .world
                    .filter()
                    //.and(foo)
                    .and(game.that_team.filter().not())
                    .and(game.this_team.filter().not()),
                &game.that_team.filter(),
                true,
                false, //true
            )
        } else {
            unreachable!()
        }
        // let rook_pos: Vec<_> = game
        //     .that_team
        //     .units
        //     .iter()
        //     .filter(|a| a.typ == Type::Rook)
        //     .map(|a| a.position)
        //     .collect();
        // let rook_pos = rook_pos
        //     .into_iter()
        //     .flat_map(|a| a.to_cube().neighbours().map(|a| a.to_axial()));
        // let rook_pos = rook_pos.filter(|a| game.that_team.find_slow(a).is_none());
        // let foo = movement::AcceptCoords::new(rook_pos.into_iter()).not();

        // let foo = if restricted_movement || unit.typ == Type::Mage {
        //     movement::Either::A(movement::NoFilter)
        // } else {
        //     movement::Either::B(foo)
        // };

        // movement::compute_moves2(
        //     unit.position,
        //     &game
        //         .world
        //         .filter()
        //         .and(foo)
        //         .and(game.that_team.filter().not())
        //         // .and(
        //         //     game.that_team
        //         //         .filter_type(Type::Warrior)
        //         //         .and(game.that_team.filter())
        //         //         .not(),
        //         // )
        //         .and(game.this_team.filter().not()),
        //     &game.that_team.filter(),
        //     // &game.this_team.filter().or(movement::AcceptCoords::new(
        //     //     board::water_border().map(|x| x.to_axial()),
        //     // )),
        //     restricted_movement,
        //     false, //true
        // )
    };
    mm
}
