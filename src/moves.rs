use super::*;

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum ActualMove {
    ExtraMove(PartialMoveSigl, PartialMoveSigl),
    SkipTurn,
    GameEnd(GameEnding),
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum GameEnding {
    Win(ActiveTeam),
    Draw,
}

pub fn from_foo(input: &str) -> Result<Vec<ActualMove>, std::fmt::Error> {
    input
        .split(",")
        .filter(|a| *a != "")
        .map(|a| {
            dbg!(&a);
            let mut s = a.chars();

            match s.next().ok_or(std::fmt::Error)? {
                'N' => {
                    let s = s.as_str();
                    let mut k = s.split(":").map(|a| a.parse::<i16>());

                    let mut foo = || {
                        k.next()
                            .ok_or(std::fmt::Error)?
                            .map_err(|_| std::fmt::Error)
                    };

                    let unit = GridCoord([foo()?, foo()?]);
                    let moveto = GridCoord([foo()?, foo()?]);
                    todo!();
                    //Ok(ActualMove::NormalMove(PartialMoveSigl { unit, moveto }))
                }
                'E' => {
                    let s = s.as_str();
                    let mut k = s.split(":").map(|a| a.parse::<i16>());
                    let mut foo = || {
                        k.next()
                            .ok_or(std::fmt::Error)?
                            .map_err(|_| std::fmt::Error)
                    };
                    let unit = GridCoord([foo()?, foo()?]);
                    let moveto = GridCoord([foo()?, foo()?]);

                    let unit2 = GridCoord([foo()?, foo()?]);
                    let moveto2 = GridCoord([foo()?, foo()?]);
                    Ok(ActualMove::ExtraMove(
                        PartialMoveSigl { unit, moveto },
                        PartialMoveSigl {
                            unit: unit2,
                            moveto: moveto2,
                        },
                    ))
                }
                // 'I' => {
                //     let s = s.as_str();
                //     let mut k = s.split(":").map(|a| a.parse::<i16>());
                //     let mut foo = || {
                //         k.next()
                //             .ok_or(std::fmt::Error)?
                //             .map_err(|_| std::fmt::Error)
                //     };

                //     let unit = GridCoord([foo()?, foo()?]);
                //     let moveto = GridCoord([foo()?, foo()?]);
                //     Ok(ActualMove::Invade(InvadeSigl { unit, moveto }))
                // }
                //'S' => Ok(ActualMove::SkipTurn),
                'F' => {
                    let c = s.next().ok_or(std::fmt::Error)?;
                    Ok(ActualMove::GameEnd(match c {
                        'W' => GameEnding::Win(ActiveTeam::Cats),
                        'B' => GameEnding::Win(ActiveTeam::Dogs),
                        'D' => GameEnding::Draw,
                        _ => return Err(std::fmt::Error),
                    }))
                }
                _ => Err(std::fmt::Error),
            }
        })
        .collect()
}

pub fn to_foo(a: &[ActualMove], mut f: impl std::fmt::Write) -> std::fmt::Result {
    for a in a.iter() {
        match a {
            // ActualMove::Invade(i) => {
            //     let a = i.unit.0;
            //     let b = i.moveto.0;
            //     write!(f, "I{}:{}:{}:{},", a[0], a[1], b[0], b[1])?;
            // }
            // ActualMove::NormalMove(i) => {
            //     let a = i.unit.0;
            //     let b = i.moveto.0;
            //     write!(f, "N{}:{}:{}:{},", a[0], a[1], b[0], b[1])?;
            // }
            ActualMove::ExtraMove(i, j) => {
                let a = i.unit.0;
                let b = i.moveto.0;
                let c = j.unit.0;
                let d = j.moveto.0;
                write!(
                    f,
                    "E{}:{}:{}:{}:{}:{}:{}:{},",
                    a[0], a[1], b[0], b[1], c[0], c[1], d[0], d[1]
                )?;
            }
            ActualMove::SkipTurn => {
                write!(f, "S,")?;
            }
            ActualMove::GameEnd(g) => {
                let w = match g {
                    GameEnding::Win(ActiveTeam::Cats) => "W",
                    GameEnding::Win(ActiveTeam::Dogs) => "B",
                    GameEnding::Draw => "D",
                };

                write!(f, "F{}", w)?;
            }
        }
    }
    Ok(())
}

struct Doopa<'a, 'b> {
    data: &'a mut ace::WorkerManager<'b>,
}
impl<'a, 'b> Doopa<'a, 'b> {
    pub fn new(data: &'a mut ace::WorkerManager<'b>) -> Self {
        Doopa { data }
    }
    pub async fn wait_animation<W: UnwrapMe>(&mut self, m: W, team: ActiveTeam) -> W::Item {
        let an = m.into_command();
        let aa = self.data.wait_animation(an, team).await;
        W::unwrapme(aa.into_data())
    }
}

use crate::movement::{movement_mesh::Mesh, MovementMesh};

pub enum ExtraMove<T> {
    ExtraMove { unit: T },
    FinishMoving,
}

#[derive(Debug, Clone)]
pub struct InvadeSigl {
    pub unit: GridCoord,
    pub moveto: GridCoord,
}

#[derive(Debug, Clone)]
pub struct MovementSigl {
    pub unit: GridCoord,
    pub moveto: GridCoord,
}
#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct PartialMoveSigl {
    pub unit: GridCoord,
    pub moveto: GridCoord,
}
impl PartialMoveSigl {
    pub fn to_movement(&self) -> MovementSigl {
        MovementSigl {
            unit: self.unit,
            moveto: self.moveto,
        }
    }
}

pub use partial_move::PartialMove;
pub mod partial_move {
    use crate::movement::Filter;

    use super::*;

    fn calculate_walls(
        position: GridCoord,
        typ: Type,
        land: &[GridCoord],
        forest: &[GridCoord],
    ) -> Mesh {
        let mut walls = Mesh::new();

        for a in position.to_cube().range(2) {
            let a = a.to_axial();
            //TODO this is duplicated logic in selection function???
            let cc = if typ == Type::Ship {
                land.iter().find(|&&b| a == b).is_some()
            } else {
                land.iter().find(|&&b| a == b).is_none()
                    || forest.iter().find(|&&b| a == b).is_some()
            };
            if cc {
                walls.add(a.sub(&position));
            }
        }

        walls
    }

    fn apply_normal_move(this_unit: &mut UnitData, target_cell: GridCoord) -> PartialMoveSigl {
        let orig = this_unit.position;
        this_unit.position = target_cell;

        let sigl = PartialMoveSigl {
            unit: orig,
            moveto: target_cell,
        };

        //let unit=game_view.this_team.find_slow_mut(&target_cell).unwrap();
        sigl
    }
    fn apply_extra_move(
        unit: GridCoord,
        typ: Type,
        target_cell: GridCoord,
        land: &mut Vec<GridCoord>,
        forest: &mut Vec<GridCoord>,
    ) -> PartialMoveSigl {
        let sigl = PartialMoveSigl {
            unit,
            moveto: target_cell,
        };

        if typ == Type::Ship {
            land.push(target_cell);
        } else if typ == Type::Foot {
            forest.push(target_cell);
        }
        sigl
    }

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

    pub fn for_all_moves_fast(mut state: GameState, team: ActiveTeam) -> Vec<moves::ActualMove> {
        let mut movs = Vec::new();
        for i in 0..state.view_mut(team).this_team.units.len() {
            let pos = state.view_mut(team).this_team.units[i].position;
            let typ = state.view_mut(team).this_team.units[i].typ;

            let mesh = generate_unit_possible_moves_inner(&pos, typ, &state.view_mut(team), false);
            for mm in mesh.iter_mesh(pos) {
                //Temporarily move the player in the game world.
                //We do this so that the mesh generated for extra is accurate.
                apply_normal_move(&mut state.view_mut(team).this_team.units[i], mm);

                let second_mesh =
                    generate_unit_possible_moves_inner(&mm, typ, &state.view_mut(team), true);

                for sm in second_mesh.iter_mesh(mm) {
                    //Don't both applying the extra move. just generate the sigl.
                    movs.push(moves::ActualMove::ExtraMove(
                        moves::PartialMoveSigl {
                            unit: pos,
                            moveto: mm,
                        },
                        moves::PartialMoveSigl {
                            unit: mm,
                            moveto: sm,
                        },
                    ))
                }
            }
            //revert it back.
            state.view_mut(team).this_team.units[i].position = pos;
        }
        movs
    }

    #[derive(Debug, Copy, Clone)]
    pub enum GameOver {
        CatWon,
        DogWon,
        Tie,
    }

    pub fn game_is_over(game: &mut GameState, team_index: ActiveTeam) -> Option<GameOver> {
        let game = game.view_mut(team_index);

        for unit in game.this_team.units.iter() {
            //TODO instead check iterator of all moves is empty???
            let mesh = moves::partial_move::generate_unit_possible_moves_inner(
                &unit.position,
                unit.typ,
                &game,
                false,
            );
            if mesh.iter_mesh(GridCoord([0; 2])).count() != 0 {
                return None;
            }
        }

        //console_dbg!("This team won:", team_index);
        if team_index == ActiveTeam::Cats {
            return Some(GameOver::DogWon);
        } else {
            return Some(GameOver::CatWon);
        }
    }

    #[derive(Clone, Debug)]
    pub struct PartialMove {
        pub selected_unit: GridCoord,
        pub typ: Type,
        pub end: GridCoord,
        pub is_extra: bool,
    }

    impl PartialMove {
        pub fn execute<'b>(
            self,
            game_view: &'b mut GameViewMut<'_, '_>,
        ) -> (PartialMoveSigl, ExtraMove<&'b mut UnitData>) {
            let is_extra = self.is_extra;
            let selected_unit = self.selected_unit;
            let target_cell = self.end;
            let typ = self.typ;

            if !is_extra {
                let start = selected_unit;
                let this_unit = game_view.this_team.find_slow_mut(&start).unwrap();

                let sigl = apply_normal_move(this_unit, target_cell);

                (sigl, ExtraMove::ExtraMove { unit: this_unit })
            } else {
                let sigl = apply_extra_move(
                    selected_unit,
                    typ,
                    target_cell,
                    game_view.land,
                    game_view.forest,
                );

                (sigl, ExtraMove::FinishMoving)
            }
        }
        pub async fn execute_with_animation<'b>(
            self,
            game_view: &'b mut GameViewMut<'_, '_>,
            data: &mut ace::WorkerManager<'_>,
            mesh: MovementMesh,
        ) -> (PartialMoveSigl, ExtraMove<&'b mut UnitData>) {
            let is_extra = self.is_extra;
            let selected_unit = self.selected_unit;
            let target_cell = self.end;
            let typ = self.typ;

            if !is_extra {
                let start = selected_unit;
                let end = target_cell;
                let this_unit = game_view.this_team.find_slow_mut(&start).unwrap();

                let walls = calculate_walls(
                    this_unit.position,
                    this_unit.typ,
                    game_view.land,
                    game_view.forest,
                );

                let team = game_view.team;
                let _ = Doopa::new(data)
                    .wait_animation(Movement::new(this_unit.clone(), mesh, walls, end), team)
                    .await;

                let sigl = apply_normal_move(this_unit, target_cell);

                (sigl, ExtraMove::ExtraMove { unit: this_unit })
            } else {
                let sigl = apply_extra_move(
                    selected_unit,
                    typ,
                    target_cell,
                    game_view.land,
                    game_view.forest,
                );

                (sigl, ExtraMove::FinishMoving)
            }
        }
    }
}

trait UnwrapMe {
    type Item;

    fn direct_unwrap(self) -> Self::Item;
    fn into_command(self) -> animation::AnimationCommand;
    fn unwrapme(a: animation::AnimationCommand) -> Self::Item;
}
struct Movement {
    start: UnitData,
    mesh: MovementMesh,
    walls: Mesh,
    end: GridCoord,
}
impl Movement {
    pub fn new(start: UnitData, mesh: MovementMesh, walls: Mesh, end: GridCoord) -> Self {
        Movement {
            start,
            mesh,
            walls,
            end,
        }
    }
}
impl UnwrapMe for Movement {
    type Item = UnitData;

    fn direct_unwrap(mut self) -> Self::Item {
        // let last_dir = self
        //     .mesh
        //     .path(self.end.sub(&self.start.position))
        //     .last()
        //     .unwrap();

        //TODO is this right????
        self.start.position = self.end;
        self.start
    }
    fn into_command(self) -> animation::AnimationCommand {
        animation::AnimationCommand::Movement {
            unit: self.start,
            mesh: self.mesh,
            walls: self.walls,
            end: self.end,
        }
    }
    fn unwrapme(a: animation::AnimationCommand) -> Self::Item {
        let animation::AnimationCommand::Movement { unit, .. } = a else {
            unreachable!()
        };
        unit
    }
}

// struct Attack {
//     attacker: UnitData,
//     defender: UnitData,
// }
// impl Attack {
//     pub fn new(attacker: UnitData, defender: UnitData) -> Self {
//         Attack { attacker, defender }
//     }
// }
// impl UnwrapMe for Attack {
//     type Item = [UnitData; 2];
//     fn direct_unwrap(self) -> Self::Item {
//         [self.attacker, self.defender]
//     }
//     fn into_command(self) -> animation::AnimationCommand {
//         animation::AnimationCommand::Attack {
//             attacker: self.attacker,
//             defender: self.defender,
//         }
//     }
//     fn unwrapme(a: animation::AnimationCommand) -> Self::Item {
//         let animation::AnimationCommand::Attack { attacker, defender } = a else {
//             unreachable!()
//         };
//         [attacker, defender]
//     }
// }
