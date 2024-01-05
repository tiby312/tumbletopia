use super::*;

//TODO use this.
//signifies a move as well as the context in which the move can be played.
pub struct AMove {
    a: ActualMove,
    game_state: &'static GameState,
    selection: movement::MovementMesh,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum ActualMove {
    NormalMove(PartialMoveSigl),
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
                    Ok(ActualMove::NormalMove(PartialMoveSigl { unit, moveto }))
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
            ActualMove::NormalMove(i) => {
                let a = i.unit.0;
                let b = i.moveto.0;
                write!(f, "N{}:{}:{}:{},", a[0], a[1], b[0], b[1])?;
            }
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
struct Doopa2;
impl Doopa2 {
    pub fn wait_animation<W: UnwrapMe>(&mut self, m: W, _: ActiveTeam) -> W::Item {
        m.direct_unwrap()
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
mod partial_move {
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

    //TODO the mesh is only needed for animation purposes.
    //inefficient to calculate for ai
    #[derive(Clone, Debug)]
    pub struct PartialMove {
        pub selected_unit: GridCoord,
        pub typ: Type,
        pub mesh: MovementMesh,
        pub end: GridCoord,
        pub is_extra: bool,
    }

    impl PartialMove {
        pub fn execute<'b>(
            self,
            game_view: &'b mut GameViewMut<'_, '_>
        ) -> (PartialMoveSigl, ExtraMove<&'b mut UnitData>) {
            let mut a = Doopa2;

            let is_extra = self.is_extra;
            let selected_unit = self.selected_unit;
            let target_cell = self.end;
            let mesh = self.mesh;
            let doopa = &mut a;
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
                let _ =
                    doopa.wait_animation(Movement::new(this_unit.clone(), mesh, walls, end), team);

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
            data: &mut ace::WorkerManager<'_>
        ) -> (PartialMoveSigl, ExtraMove<&'b mut UnitData>) {
            let mut a = Doopa::new(data);

            let is_extra = self.is_extra;
            let selected_unit = self.selected_unit;
            let target_cell = self.end;
            let mesh = self.mesh;
            let doopa = &mut a;
            let typ = self.typ;
            //let (selected_unit,typ,mesh,target_cell, doopa,game_view,func,is_extra):(GridCoord,Type,MovementMesh,GridCoord,_,&mut GameViewMut<'_,'_>,_,_)=$args;

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
                let _ = doopa
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

struct Attack {
    attacker: UnitData,
    defender: UnitData,
}
impl Attack {
    pub fn new(attacker: UnitData, defender: UnitData) -> Self {
        Attack { attacker, defender }
    }
}
impl UnwrapMe for Attack {
    type Item = [UnitData; 2];
    fn direct_unwrap(self) -> Self::Item {
        [self.attacker, self.defender]
    }
    fn into_command(self) -> animation::AnimationCommand {
        animation::AnimationCommand::Attack {
            attacker: self.attacker,
            defender: self.defender,
        }
    }
    fn unwrapme(a: animation::AnimationCommand) -> Self::Item {
        let animation::AnimationCommand::Attack { attacker, defender } = a else {
            unreachable!()
        };
        [attacker, defender]
    }
}
