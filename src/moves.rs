use super::*;

#[derive(Debug, Clone)]
pub enum ActualMove {
    NormalMove(PartialMoveSigl),
    ExtraMove(PartialMoveSigl, PartialMoveSigl),
    SkipTurn,
    GameEnd(GameEnding),
}

#[derive(Debug, Clone)]
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
                'S' => Ok(ActualMove::SkipTurn),
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

use crate::movement::Path;

pub enum ExtraMove<T> {
    ExtraMove { unit: T },
    FinishMoving,
}

use inner_partial::InnerPartialMove;
mod inner_partial {

    //https://users.rust-lang.org/t/macro-to-dry-sync-and-async-code/67556
    macro_rules! resolve_inner_movement_impl {
        ($args:expr, $($_await:tt)*) => {
            {
                let (start,path,doopa,game_view):(GridCoord,movement::Path,_,&mut GameViewMut<'_,'_>)=$args;

                let this_unit = game_view
                .this_team
                .find_slow_mut(&start)
                .unwrap();

                let team=game_view.team;
                let _ = doopa
                    .wait_animation(Movement::new(this_unit.clone(),path), team)
                    $($_await)*;

                this_unit.position= path.get_end_coord(this_unit.position);

            }
        }
    }

    use super::*;
    pub struct InnerPartialMove {
        u: GridCoord,
        path: Path,
    }
    impl InnerPartialMove {
        pub fn new(a: GridCoord, path: Path) -> Self {
            InnerPartialMove { u: a, path }
        }
        pub(super) fn inner_execute_no_animate(
            self,
            game_view: &mut GameViewMut<'_, '_>,
            a: &mut Doopa2,
        ) {
            resolve_inner_movement_impl!((self.u, self.path, a, game_view),)
        }

        pub(super) async fn inner_execute_animate(
            self,
            game_view: &mut GameViewMut<'_, '_>,
            a: &mut Doopa<'_, '_>,
        ) {
            resolve_inner_movement_impl!((self.u,self.path,a,game_view),.await)
        }
    }
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
#[derive(Debug, Clone)]
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

    macro_rules! resolve_movement_impl {
        ($args:expr,$namey:ident, $($_await:tt)*) => {
            {
                let (selected_unit,path, doopa,mut game_view,mut func):(GridCoord,movement::Path,_,&mut GameViewMut<'_,'_>,_)=$args;


                let target_cell=path.get_end_coord(selected_unit);


                InnerPartialMove::new(selected_unit,path).$namey(&mut game_view,doopa)$($_await)*;


                for n in target_cell.to_cube().neighbours() {
                    if let Some(f)=HandleSurround::new(n.to_axial()).$namey(&mut game_view.not(), doopa)$($_await)*{
                        func(game_view.that_team.find_take(&f).unwrap());
                    }
                }

                let k=HandleSurround::new(target_cell).$namey(&mut game_view, doopa)$($_await)*;


                let sigl=PartialMoveSigl{unit:selected_unit,moveto:target_cell};

                let unit=game_view.this_team.find_slow_mut(&target_cell).unwrap();
                if let Some(_) = k {
                    (sigl,ExtraMove::ExtraMove{unit})
                } else {
                    //Finish this players turn.
                    (sigl,ExtraMove::FinishMoving)
                }
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct PartialMove {
        selected_unit: GridCoord,
        path: Path,
    }

    impl PartialMove {
        pub fn new(a: GridCoord, path: Path) -> Self {
            PartialMove {
                selected_unit: a,
                path,
            }
        }

        pub(super) fn inner_execute_no_animate<'b>(
            self,
            game_view: &'b mut GameViewMut,
            a: &mut Doopa2,
            func: impl FnMut(UnitData),
        ) -> (PartialMoveSigl, ExtraMove<&'b mut UnitData>) {
            resolve_movement_impl!(
                (self.selected_unit, self.path, a, game_view, func),
                inner_execute_no_animate,
            )
        }

        pub(super) async fn inner_execute_animate<'b>(
            self,
            game_view: &'b mut GameViewMut<'_, '_>,
            a: &mut Doopa<'_, '_>,
            func: impl FnMut(UnitData),
        ) -> (PartialMoveSigl, ExtraMove<&'b mut UnitData>) {
            resolve_movement_impl!((self.selected_unit, self.path, a, game_view,func),inner_execute_animate,.await)
        }

        pub fn execute<'b>(
            self,
            game_view: &'b mut GameViewMut<'_, '_>,
            func: impl FnMut(UnitData),
        ) -> (PartialMoveSigl, ExtraMove<&'b mut UnitData>) {
            self.inner_execute_no_animate(game_view, &mut Doopa2, func)
        }
        pub async fn execute_with_animation<'b>(
            self,
            game_view: &'b mut GameViewMut<'_, '_>,
            data: &mut ace::WorkerManager<'_>,
            func: impl FnMut(UnitData),
        ) -> (PartialMoveSigl, ExtraMove<&'b mut UnitData>) {
            self.inner_execute_animate(game_view, &mut Doopa::new(data), func)
                .await
        }
    }
}

pub use invade::Invade;
mod invade {
    use super::*;

    macro_rules! resolve_invade_impl {
        ($args:expr,$namey:ident, $($_await:tt)*) => {
            {
                let (selected_unit,path,game_view,doopa,mut func):(GridCoord,Path,&mut GameViewMut<'_,'_>,_,_)=$args;

                let target_coord=path.get_end_coord(selected_unit);



                InnerPartialMove::new(selected_unit,path).$namey(game_view,doopa)$($_await)*;

                func(game_view.that_team.find_take(&target_coord).unwrap());



                if let Some(f)=HandleSurround::new(target_coord).$namey(game_view,doopa)$($_await)*{
                    func(game_view.this_team.find_take(&f).unwrap());
                }
                for n in target_coord.to_cube().neighbours() {
                    if let Some(f)=HandleSurround::new(n.to_axial()).$namey(&mut game_view.not(),doopa)$($_await)*{
                        func(game_view.that_team.find_take(&f).unwrap());
                    }
                }

                PartialMoveSigl{
                    unit:selected_unit,
                    moveto:target_coord
                }



            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct Invade {
        selected_unit: GridCoord,
        path: Path,
    }

    impl Invade {
        pub fn new(a: GridCoord, b: Path) -> Self {
            Invade {
                selected_unit: a,
                path: b,
            }
        }
        pub(super) fn inner_execute_no_animate(
            self,
            game_view: &mut GameViewMut<'_, '_>,
            a: &mut Doopa2,
            func: impl FnMut(UnitData),
        ) -> PartialMoveSigl {
            resolve_invade_impl!(
                (self.selected_unit, self.path, game_view, a, func),
                inner_execute_no_animate,
            )
        }

        pub(super) async fn inner_execute_animate(
            self,
            game_view: &mut GameViewMut<'_, '_>,
            a: &mut Doopa<'_, '_>,
            func: impl FnMut(UnitData),
        ) -> PartialMoveSigl {
            resolve_invade_impl!((self.selected_unit,self.path,game_view,a,func),inner_execute_animate,.await)
        }

        pub fn execute(
            self,
            game_view: &mut GameViewMut<'_, '_>,
            func: impl FnMut(UnitData),
        ) -> PartialMoveSigl {
            self.inner_execute_no_animate(game_view, &mut Doopa2, func)
        }
        pub async fn execute_with_animation(
            self,
            game_view: &mut GameViewMut<'_, '_>,
            data: &mut ace::WorkerManager<'_>,
            func: impl FnMut(UnitData),
        ) -> PartialMoveSigl {
            self.inner_execute_animate(game_view, &mut Doopa::new(data), func)
                .await
        }
    }
}

use surround::HandleSurround;

mod surround {
    use super::*;
    macro_rules! resolve_3_players_nearby_impl {
        ($args:expr, $($_await:tt)*) => {{
            let (n, doopa, game_view): (GridCoord, _, &mut GameViewMut<'_,'_>) = $args;
            let team=game_view.team;
            let n = n.to_cube();
            if let Some(unit_pos) = game_view
                        .this_team
                        .find_slow_mut(&n.to_axial())
            {

            let unit_pos = unit_pos.position;

            let nearby_enemies: Vec<_> = n
                .neighbours()
                .filter(|a| game_view.that_team.find_slow(&a.to_axial()).is_some())
                .map(|a| a.to_axial())
                .collect();
            if nearby_enemies.len() >= 3 {

                for n in nearby_enemies {
                    let unit_pos = game_view.this_team.find_slow_mut(&unit_pos).unwrap();
                    let them = game_view.that_team.find_slow_mut(&n).unwrap();

                    let _ = doopa
                        .wait_animation(Attack::new(them.clone(), unit_pos.clone()), team.not())
                        $($_await)*;

                    // game_view.that_team.add(them);
                    // game_view.this_team.add(this_unit);
                }
                Some(unit_pos)
            } else {
                None
            }
        } else{
            None
        }
        }};
    }

    pub struct HandleSurround {
        cell: GridCoord,
    }
    impl HandleSurround {
        pub fn new(cell: GridCoord) -> Self {
            Self { cell }
        }

        pub(super) fn inner_execute_no_animate(
            self,
            game_view: &mut GameViewMut<'_, '_>,
            a: &mut Doopa2,
        ) -> Option<GridCoord> {
            resolve_3_players_nearby_impl!((self.cell, a, game_view),)
        }

        pub(super) async fn inner_execute_animate(
            self,
            game_view: &mut GameViewMut<'_, '_>,
            a: &mut Doopa<'_, '_>,
        ) -> Option<GridCoord> {
            resolve_3_players_nearby_impl!((self.cell,a,game_view),.await)
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
    path: movement::Path,
}
impl Movement {
    pub fn new(start: UnitData, path: movement::Path) -> Self {
        Movement { start, path }
    }
}
impl UnwrapMe for Movement {
    type Item = UnitData;

    fn direct_unwrap(self) -> Self::Item {
        self.start
    }
    fn into_command(self) -> animation::AnimationCommand {
        animation::AnimationCommand::Movement {
            unit: self.start,
            path: self.path,
        }
    }
    fn unwrapme(a: animation::AnimationCommand) -> Self::Item {
        let animation::AnimationCommand::Movement{unit,..}=a else{
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
        let animation::AnimationCommand::Attack{attacker,defender}=a else{
            unreachable!()
        };
        [attacker, defender]
    }
}
