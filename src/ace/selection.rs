use super::*;

pub enum SelectionType {
    Normal(selection::PossibleMovesNormal),
    Extra(selection::PossibleExtraMove),
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
    pub fn select(&self, a: &UnitData) -> PossibleExtraMove {
        PossibleExtraMove {
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
pub struct PossibleExtraMove {
    extra: PossibleExtra,
    unit: UnitData,
}

impl PossibleExtraMove {
    pub fn get_path_from_move(&self, target_cell: GridCoord, game: &GameViewMut) -> movement::Path {
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
            .unwrap();

        *path
    }
    pub fn generate(&self, game: &GameViewMut) -> movement::PossibleMoves2<()> {
        generate_unit_possible_moves_inner(
            &self.unit,
            game,
            &Some((self.extra.prev_move.clone(), self.extra.prev_coord)),
            NoPath,
        )
    }
}

pub struct PossibleMovesNormal {
    unit: UnitData,
}

impl PossibleMovesNormal {
    pub fn new(a: &UnitData) -> Self {
        PossibleMovesNormal { unit: a.clone() }
    }
    pub fn get_path_from_move(&self, target_cell: GridCoord, game: &GameViewMut) -> movement::Path {
        //Reconstruct possible paths with path information this time.
        let ss = generate_unit_possible_moves_inner(&self.unit, game, &None, movement::WithPath);

        let path = ss
            .moves
            .iter()
            .find(|a| a.target == target_cell)
            .map(|a| &a.path)
            .unwrap();

        *path
    }
    pub fn generate(&self, game: &GameViewMut) -> movement::PossibleMoves2<()> {
        generate_unit_possible_moves_inner(&self.unit, game, &None, NoPath)
    }
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
