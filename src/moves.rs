use super::*;

#[derive(Debug)]
pub enum ActualMove {
    Invade(Invade),
    NormalMove(PartialMove),
    ExtraMove(PartialMove, Invade),
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

pub enum ExtraMove {
    ExtraMove { pos: GridCoord },
    FinishMoving,
}

use inner_partial::InnerPartialMove;
mod inner_partial {

    //https://users.rust-lang.org/t/macro-to-dry-sync-and-async-code/67556
    macro_rules! resolve_inner_movement_impl {
        ($args:expr, $($_await:tt)*) => {
            {
                let (start,path,doopa,game_view):(GridCoord,movement::Path,_,&mut GameViewMut<'_>)=$args;

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
            game_view: &mut GameViewMut<'_>,
            a: &mut Doopa2,
        ) {
            resolve_inner_movement_impl!((self.u, self.path, a, game_view),)
        }

        pub(super) async fn inner_execute_animate(
            self,
            game_view: &mut GameViewMut<'_>,
            a: &mut Doopa<'_, '_>,
        ) {
            resolve_inner_movement_impl!((self.u,self.path,a,game_view),.await)
        }
    }
}

pub use partial_move::PartialMove;
mod partial_move {
    use super::*;

    macro_rules! resolve_movement_impl {
        ($args:expr,$namey:ident, $($_await:tt)*) => {
            {
                let (selected_unit,path, doopa,mut game_view):(GridCoord,movement::Path,_,&mut GameViewMut<'_>)=$args;

                let target_cell=path.get_end_coord(selected_unit);


                InnerPartialMove::new(selected_unit,path).$namey(&mut game_view,doopa)$($_await)*;


                for n in target_cell.to_cube().neighbours() {
                    if let Some(f)=HandleSurround::new(n.to_axial()).$namey(&mut game_view.not(), doopa)$($_await)*{
                        game_view.that_team.find_take(&f).unwrap();
                    }
                }

                let k=HandleSurround::new(target_cell).$namey(&mut game_view, doopa)$($_await)*;


                if let Some(_) = k {
                    ExtraMove::ExtraMove{pos:target_cell}
                } else {
                    //Finish this players turn.
                    ExtraMove::FinishMoving
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

        pub(super) fn inner_execute_no_animate(
            self,
            game_view: &mut GameViewMut<'_>,
            a: &mut Doopa2,
        ) -> ExtraMove {
            resolve_movement_impl!(
                (self.selected_unit, self.path, a, game_view),
                inner_execute_no_animate,
            )
        }

        pub(super) async fn inner_execute_animate(
            self,
            game_view: &mut GameViewMut<'_>,
            a: &mut Doopa<'_, '_>,
        ) -> ExtraMove {
            resolve_movement_impl!((self.selected_unit, self.path, a, game_view),inner_execute_animate,.await)
        }

        pub fn execute(self, game_view: &mut GameViewMut<'_>) -> ExtraMove {
            self.inner_execute_no_animate(game_view, &mut Doopa2)
        }
        pub async fn execute_with_animation(
            self,
            game_view: &mut GameViewMut<'_>,
            data: &mut ace::WorkerManager<'_>,
        ) -> ExtraMove {
            self.inner_execute_animate(game_view, &mut Doopa::new(data))
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
                let (selected_unit,path,game_view,doopa):(GridCoord,Path,&mut GameViewMut<'_>,_)=$args;

                let target_coord=path.get_end_coord(selected_unit);

                let _target = game_view.that_team.find_take(&target_coord).unwrap();

                InnerPartialMove::new(selected_unit,path).$namey(game_view,doopa)$($_await)*;

                if let Some(f)=HandleSurround::new(target_coord).$namey(game_view,doopa)$($_await)*{
                    game_view.this_team.find_take(&f).unwrap();
                }
                for n in target_coord.to_cube().neighbours() {
                    if let Some(f)=HandleSurround::new(n.to_axial()).$namey(&mut game_view.not(),doopa)$($_await)*{
                        game_view.that_team.find_take(&f).unwrap();
                    }
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
            game_view: &mut GameViewMut<'_>,
            a: &mut Doopa2,
        ) {
            resolve_invade_impl!(
                (self.selected_unit, self.path, game_view, a),
                inner_execute_no_animate,
            )
        }

        pub(super) async fn inner_execute_animate(
            self,
            game_view: &mut GameViewMut<'_>,
            a: &mut Doopa<'_, '_>,
        ) {
            resolve_invade_impl!((self.selected_unit,self.path,game_view,a),inner_execute_animate,.await)
        }

        pub fn execute(self, game_view: &mut GameViewMut<'_>) {
            self.inner_execute_no_animate(game_view, &mut Doopa2)
        }
        pub async fn execute_with_animation(
            self,
            game_view: &mut GameViewMut<'_>,
            data: &mut ace::WorkerManager<'_>,
        ) {
            self.inner_execute_animate(game_view, &mut Doopa::new(data))
                .await
        }
    }
}

use surround::HandleSurround;

mod surround {
    use super::*;
    macro_rules! resolve_3_players_nearby_impl {
        ($args:expr, $($_await:tt)*) => {{
            let (n, doopa, game_view): (GridCoord, _, &mut GameViewMut<'_>) = $args;
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
            game_view: &mut GameViewMut<'_>,
            a: &mut Doopa2,
        ) -> Option<GridCoord> {
            resolve_3_players_nearby_impl!((self.cell, a, game_view),)
        }

        pub(super) async fn inner_execute_animate(
            self,
            game_view: &mut GameViewMut<'_>,
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
