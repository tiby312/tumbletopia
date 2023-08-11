use super::*;

pub enum SelectionType {
    Normal(selection::PossibleMovesNormal),
    Extra(selection::ComboContinueSelection),
}

#[derive(Clone)]
pub struct PossibleExtra {
    prev_move: moves::PartialMoveSigl,
    prev_coord: GridCoord,
}
impl PossibleExtra {
    pub fn new(prev_move: moves::PartialMoveSigl, prev_coord: GridCoord) -> Self {
        PossibleExtra {
            prev_move,
            prev_coord,
        }
    }
    pub fn select(&self, a: &UnitData) -> ComboContinueSelection {
        ComboContinueSelection {
            extra: self.clone(),
            unit: a.clone(),
        }
    }
    pub fn prev_move(&self) -> &moves::PartialMoveSigl {
        &self.prev_move
    }
    pub fn coord(&self) -> GridCoord {
        self.prev_coord
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
    fn get_path_from_move(
        &self,
        target_cell: GridCoord,
        game: &GameViewMut,
    ) -> Result<movement::Path, NoPathErr> {
        //Reconstruct possible paths with path information this time.
        let ss = generate_unit_possible_moves_inner(
            &self.unit,
            game,
            &Some((self.extra.prev_move.clone(), self.extra.prev_coord)),
            movement::WithPath,
        );

        let path = ss
            .moves
            .iter()
            .find(|a| a.target == target_cell)
            .map(|a| &a.path)
            .ok_or(NoPathErr)?;

        Ok(*path)
    }
    pub fn generate(&self, game: &GameViewMut) -> movement::PossibleMoves2<()> {
        generate_unit_possible_moves_inner(
            &self.unit,
            game,
            &Some((self.extra.prev_move.clone(), self.extra.prev_coord)),
            NoPath,
        )
    }
    pub async fn execute(
        &self,
        target_cell: GridCoord,
        game_view: &mut GameViewMut<'_>,
        doop: &mut ace::WorkerManager<'_>,
        move_log: &mut MoveLog,
    ) -> Result<(), NoPathErr> {
        let path = self.get_path_from_move(target_cell, game_view)?;
        let unit = self.unit.position;

        if let Some(_) = game_view.that_team.find_slow_mut(&target_cell) {
            let iii = moves::Invade::new(unit, path);

            let iii = iii.execute_with_animation(game_view, doop, |_| {}).await;

            move_log.push(moves::ActualMove::ExtraMove(
                self.extra.prev_move.clone(),
                iii,
            ));
        } else {
            unreachable!("Not possible!");
        };

        Ok(())
    }
}

pub struct PossibleMovesNormal {
    unit: UnitData,
}

impl PossibleMovesNormal {
    pub fn new(a: &UnitData) -> Self {
        PossibleMovesNormal { unit: a.clone() }
    }
    fn get_path_from_move(
        &self,
        target_cell: GridCoord,
        game: &GameViewMut,
    ) -> Result<movement::Path, NoPathErr> {
        //Reconstruct possible paths with path information this time.
        let ss = generate_unit_possible_moves_inner(&self.unit, game, &None, movement::WithPath);

        let path = ss
            .moves
            .iter()
            .find(|a| a.target == target_cell)
            .map(|a| &a.path)
            .ok_or(NoPathErr)?;

        Ok(*path)
    }
    pub fn generate(&self, game: &GameViewMut) -> movement::PossibleMoves2<()> {
        generate_unit_possible_moves_inner(&self.unit, game, &None, NoPath)
    }

    pub async fn execute(
        &self,
        target_cell: GridCoord,
        game_view: &mut GameViewMut<'_>,
        doop: &mut ace::WorkerManager<'_>,
        move_log: &mut MoveLog,
    ) -> Result<Option<selection::PossibleExtra>, NoPathErr> {
        let path = self.get_path_from_move(target_cell, game_view)?;
        let unit = self.unit.position;

        let e = if let Some(_) = game_view.that_team.find_slow_mut(&target_cell) {
            let iii = moves::Invade::new(unit, path);

            let iii = iii.execute_with_animation(game_view, doop, |_| {}).await;

            move_log.add_invade(iii);
            None
        } else {
            let pm = moves::PartialMove::new(unit, path);
            let jjj = pm
                .clone()
                .execute_with_animation(game_view, doop, |_| {})
                .await;

            match jjj {
                (sigl, moves::ExtraMove::ExtraMove { pos }) => {
                    Some(selection::PossibleExtra::new(sigl, pos))
                }
                (sigl, moves::ExtraMove::FinishMoving) => {
                    move_log.add_movement(sigl.to_movement());
                    None
                }
            }
        };

        Ok(e)
    }
}

pub struct MoveLog {
    inner: Vec<moves::ActualMove>,
}
impl MoveLog {
    pub fn new() -> Self {
        MoveLog { inner: vec![] }
    }
    pub fn push(&mut self, o: moves::ActualMove) {
        self.inner.push(o);
    }
    pub fn add_invade(&mut self, i: moves::InvadeSigl) {}
    pub fn add_movement(&mut self, a: moves::MovementSigl) {}
}

fn generate_unit_possible_moves_inner<P: movement::PathHave>(
    unit: &UnitData,
    game: &GameViewMut,
    extra_attack: &Option<(moves::PartialMoveSigl, GridCoord)>,
    ph: P,
) -> movement::PossibleMoves2<P::Foo> {
    // If there is an enemy near by restrict movement.

    let j = if let Some(_) = unit
        .position
        .to_cube()
        .ring(1)
        .map(|s| game.that_team.find_slow(&s.to_axial()).is_some())
        .find(|a| *a)
    {
        1
    } else {
        match unit.typ {
            Type::Warrior => 2,
            Type::Para => 1,
            _ => todo!(),
        }
    };

    let mm = MoveUnit(j);

    let mm = if let Some(_) = extra_attack
        .as_ref()
        .filter(|&(_, aaa)| *aaa == unit.position)
    {
        movement::compute_moves(
            &movement::WarriorMovement,
            &game.world.filter().and(game.that_team.filter()),
            &movement::NoFilter,
            &terrain::Grass,
            unit.position,
            MoveUnit(1),
            false,
            ph,
        )
    } else {
        movement::compute_moves(
            &movement::WarriorMovement,
            &game.world.filter().and(
                game.that_team
                    .filter_type(Type::Warrior)
                    .and(game.that_team.filter())
                    .not(),
            ),
            &game.this_team.filter().not(),
            &terrain::Grass,
            unit.position,
            mm,
            true,
            ph,
        )
    };
    mm
}
