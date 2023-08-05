use super::*;

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

//https://users.rust-lang.org/t/macro-to-dry-sync-and-async-code/67556
macro_rules! resolve_movement_impl {
    ($args:expr, $($_await:tt)*) => {
        {
            let (start,path,mut doopa,game_view):(UnitData,movement::Path,_,&mut GameView<'_>)=$args;
            let team=game_view.team;
            let mut start = doopa
                .wait_animation(ace::Movement::new(start,path), team)
                $($_await)*;

            start.position = path.get_end_coord(start.position);
            start
        }
    }
}

macro_rules! resolve_invade_impl {
    ($args:expr, $($_await:tt)*) => {
        {
            let (selected_unit,target_coord,game_view,doopa):(GridCoord,GridCoord,&mut GameView<'_>,_)=$args;
            let team=game_view.team;
            let this_unit = game_view.this_team.find_take(&selected_unit).unwrap();

            let _target = game_view.that_team.find_take(&target_coord).unwrap();

            let path = movement::Path::new();
            let m = this_unit.position.dir_to(&target_coord);
            let path = path.add(m).unwrap();

            let mut this_unit=resolve_movement_impl!((this_unit,path,doopa,game_view),$($_await)*);

            this_unit.position = target_coord;

            game_view.this_team.add(this_unit);




        }
    }
}

macro_rules! resolve_3_players_nearby_impl {
    ($args:expr, $($_await:tt)*) => {{
        let (n, mut doopa, game_view): (GridCoord, _, &mut GameView<'_>) = $args;
        let team=game_view.team;
        let n = n.to_cube();
        let Some(unit_pos) = game_view
                    .this_team
                    .warriors
                    .find_ext_mut(&n.to_axial()) else{
                        return None;
                    };

        let unit_pos = unit_pos.0.position;

        let nearby_enemies: Vec<_> = n
            .neighbours()
            .filter(|a| game_view.that_team.find_slow(&a.to_axial()).is_some())
            .map(|a| a.to_axial())
            .collect();
        if nearby_enemies.len() >= 3 {
            // let mut this_unit=game.this_team
            //     .find_take(&unit_pos).unwrap();

            for n in nearby_enemies {
                let unit_pos = game_view.this_team.find_take(&unit_pos).unwrap();
                let them = game_view.that_team.find_take(&n).unwrap();

                let [them, this_unit] = doopa
                    .wait_animation(ace::Attack::new(them, unit_pos), team.not())
                    $($_await)*;

                game_view.that_team.add(them);
                game_view.this_team.add(this_unit);
            }
            Some(game_view.this_team.find_take(&unit_pos).unwrap())
        } else {
            None
        }
    }};
}

use crate::{ace::UnwrapMe, movement::Path};

pub struct PartialMove {
    selected_unit: UnitData,
    path: Path,
}

impl PartialMove {
    pub fn new(a: UnitData, path: Path) -> Self {
        PartialMove {
            selected_unit: a,
            path,
        }
    }
    pub fn execute(self, game_view: &mut GameView<'_>) -> UnitData {
        resolve_movement_impl!((self.selected_unit, self.path, Doopa2, game_view),)
    }
    pub async fn execute_with_animation(
        self,
        game_view: &mut GameView<'_>,
        data: &mut AwaitData<'_, '_>,
    ) -> UnitData {
        resolve_movement_impl!((self.selected_unit,self.path,Doopa::new(data),game_view),.await)
    }
}

pub struct Attack {
    selected_unit: GridCoord,
    target_coord: GridCoord,
}

impl Attack {
    pub fn new(a: GridCoord, b: GridCoord) -> Self {
        Attack {
            selected_unit: a,
            target_coord: b,
        }
    }
    pub fn execute(self, relative_game_view: &mut GameView<'_>) {
        resolve_invade_impl!((
            self.selected_unit,
            self.target_coord,
            relative_game_view,
            Doopa2
        ),)
    }
    pub async fn execute_with_animation(
        self,
        relative_game_view: &mut GameView<'_>,
        data: &mut AwaitData<'_, '_>,
    ) {
        resolve_invade_impl!((self.selected_unit,self.target_coord,relative_game_view,Doopa::new(data)),.await)
    }
}

pub struct HandleSurround {
    cell: GridCoord,
}
impl HandleSurround {
    pub fn new(cell: GridCoord) -> Self {
        Self { cell }
    }

    pub fn execute(self, game_view: &mut GameView<'_>) -> Option<UnitData> {
        resolve_3_players_nearby_impl!((self.cell, Doopa2, game_view),)
    }

    pub async fn execute_with_animation(
        self,
        game_view: &mut GameView<'_>,
        data: &mut AwaitData<'_, '_>,
    ) -> Option<UnitData> {
        resolve_3_players_nearby_impl!((self.cell,Doopa::new(data),game_view),.await)
    }
}
