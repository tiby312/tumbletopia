use super::*;

#[derive(Debug)]
pub enum ActualMove {
    Invade(Invade),
    NormalMove(PartialMove),
    ExtraMove(PartialMove, Invade),
}

struct Doopa<'a, 'b, 'c> {
    data: &'c mut AwaitData<'a, 'b>,
}
impl<'a, 'b, 'c> Doopa<'a, 'b, 'c> {
    pub fn new(data: &'c mut AwaitData<'a, 'b>) -> Self {
        Doopa { data }
    }
    pub async fn wait_animation<W: UnwrapMe>(&mut self, m: W, team: ActiveTeam) -> W::Item {
        self.data.wait_animation(m, team).await
    }
}
struct Doopa2;
impl Doopa2 {
    pub fn wait_animation<W: UnwrapMe>(&mut self, m: W, _: ActiveTeam) -> W::Item {
        m.direct_unwrap()
    }
}

use crate::{ace::UnwrapMe, movement::Path};

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
                let (start,path,doopa,game_view):(GridCoord,movement::Path,_,&mut GameView<'_>)=$args;

                let this_unit = game_view
                .this_team
                .find_slow_mut(&start)
                .unwrap();

                let team=game_view.team;
                let _ = doopa
                    .wait_animation(ace::Movement::new(this_unit.clone(),path), team)
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
        pub(super) fn inner_execute_no_animate(self, game_view: &mut GameView<'_>, a: &mut Doopa2) {
            resolve_inner_movement_impl!((self.u, self.path, a, game_view),)
        }

        pub(super) async fn inner_execute_animate(
            self,
            game_view: &mut GameView<'_>,
            a: &mut Doopa<'_, '_, '_>,
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
                let (selected_unit,path, doopa,mut game_view):(GridCoord,movement::Path,_,&mut GameView<'_>)=$args;

                let target_cell=path.get_end_coord(selected_unit);


                InnerPartialMove::new(selected_unit,path).$namey(&mut game_view,doopa)$($_await)*;

                let k=HandleSurround::new(target_cell).$namey(&mut game_view, doopa)$($_await)*;


                //Need to add ourselves back so we can resolve and attacking groups
                //only to remove ourselves again later.
                let k = if let Some(k) = k {
                    let pp = k.position;
                    game_view.this_team.add(k);

                    for n in target_cell.to_cube().neighbours() {
                        let _=HandleSurround::new(n.to_axial()).$namey(&mut game_view.not(), doopa)$($_await)*;
                    }

                    Some(game_view.this_team.find_take(&pp).unwrap())
                } else {
                    for n in target_cell.to_cube().neighbours() {
                        let _=HandleSurround::new(n.to_axial()).$namey(&mut game_view.not(), doopa)$($_await)*;
                    }

                    None
                };

                if let Some(k) = k {
                    game_view.this_team.add(k);
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
            game_view: &mut GameView<'_>,
            a: &mut Doopa2,
        ) -> ExtraMove {
            resolve_movement_impl!(
                (self.selected_unit, self.path, a, game_view),
                inner_execute_no_animate,
            )
        }

        pub(super) async fn inner_execute_animate(
            self,
            game_view: &mut GameView<'_>,
            a: &mut Doopa<'_, '_, '_>,
        ) -> ExtraMove {
            resolve_movement_impl!((self.selected_unit, self.path, a, game_view),inner_execute_animate,.await)
        }

        pub fn execute(self, game_view: &mut GameView<'_>) -> ExtraMove {
            self.inner_execute_no_animate(game_view, &mut Doopa2)
        }
        pub async fn execute_with_animation(
            self,
            game_view: &mut GameView<'_>,
            data: &mut AwaitData<'_, '_>,
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
                let (selected_unit,path,game_view,doopa):(GridCoord,Path,&mut GameView<'_>,_)=$args;

                let target_coord=path.get_end_coord(selected_unit);

                let _target = game_view.that_team.find_take(&target_coord).unwrap();

                InnerPartialMove::new(selected_unit,path).$namey(game_view,doopa)$($_await)*;

                HandleSurround::new(target_coord).$namey(game_view,doopa)$($_await)*;
                for n in target_coord.to_cube().neighbours() {
                    HandleSurround::new(n.to_axial()).$namey(&mut game_view.not(),doopa)$($_await)*;
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
        pub(super) fn inner_execute_no_animate(self, game_view: &mut GameView<'_>, a: &mut Doopa2) {
            resolve_invade_impl!(
                (self.selected_unit, self.path, game_view, a),
                inner_execute_no_animate,
            )
        }

        pub(super) async fn inner_execute_animate(
            self,
            game_view: &mut GameView<'_>,
            a: &mut Doopa<'_, '_, '_>,
        ) {
            resolve_invade_impl!((self.selected_unit,self.path,game_view,a),inner_execute_animate,.await)
        }

        pub fn execute(self, game_view: &mut GameView<'_>) {
            self.inner_execute_no_animate(game_view, &mut Doopa2)
        }
        pub async fn execute_with_animation(
            self,
            game_view: &mut GameView<'_>,
            data: &mut AwaitData<'_, '_>,
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
            let (n, doopa, game_view): (GridCoord, _, &mut GameView<'_>) = $args;
            let team=game_view.team;
            let n = n.to_cube();
            if let Some(unit_pos) = game_view
                        .this_team
                        .warriors
                        .find_ext_mut(&n.to_axial())
            {

            let unit_pos = unit_pos.0.position;

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
                        .wait_animation(ace::Attack::new(them.clone(), unit_pos.clone()), team.not())
                        $($_await)*;

                    // game_view.that_team.add(them);
                    // game_view.this_team.add(this_unit);
                }
                Some(game_view.this_team.find_take(&unit_pos).unwrap())
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
            game_view: &mut GameView<'_>,
            a: &mut Doopa2,
        ) -> Option<UnitData> {
            resolve_3_players_nearby_impl!((self.cell, a, game_view),)
        }

        pub(super) async fn inner_execute_animate(
            self,
            game_view: &mut GameView<'_>,
            a: &mut Doopa<'_, '_, '_>,
        ) -> Option<UnitData> {
            resolve_3_players_nearby_impl!((self.cell,a,game_view),.await)
        }

        
    }
}
