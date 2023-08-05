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



macro_rules! resolve_movement_impl {
    ($args:expr,$namey:ident, $($_await:tt)*) => {
        {
            let (selected_unit,path,mut doopa,mut game_view):(GridCoord,movement::Path,_,&mut GameView<'_>)=$args;

            let target_cell=path.get_end_coord(selected_unit);

            let this_unit = game_view
            .this_team
            .find_take(&selected_unit)
            .unwrap();

            //let this_unit=resolve_inner_movement_impl!((this_unit,path,&mut doopa,&mut game_view),$($_await)*);
            
            let this_unit=InnerPartialMove::new(this_unit,path).$namey(&mut game_view,&mut doopa)$($_await)*;

            



            game_view.this_team.add(this_unit);

            let k=resolve_3_players_nearby_impl!((target_cell,&mut doopa,&mut game_view),$($_await)*);

            //Need to add ourselves back so we can resolve and attacking groups
            //only to remove ourselves again later.
            let k = if let Some(k) = k {
                let pp = k.position;
                game_view.this_team.add(k);

                for n in target_cell.to_cube().neighbours() {
                    let _=resolve_3_players_nearby_impl!((n.to_axial(),&mut doopa,&mut game_view.not()),$($_await)*);
                }

                Some(game_view.this_team.find_take(&pp).unwrap())
            } else {
                for n in target_cell.to_cube().neighbours() {
                    let _=resolve_3_players_nearby_impl!((n.to_axial(),&mut doopa,&mut game_view.not()),$($_await)*);
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

//https://users.rust-lang.org/t/macro-to-dry-sync-and-async-code/67556
macro_rules! resolve_inner_movement_impl {
    ($args:expr, $($_await:tt)*) => {
        {
            let (start,path,doopa,game_view):(UnitData,movement::Path,_,&mut GameView<'_>)=$args;
            let team=game_view.team;
            let mut start = doopa
                .wait_animation(ace::Movement::new(start,path), team)
                $($_await)*;

            start.position = path.get_end_coord(start.position);

            let start:UnitData=start;
            start
        }
    }
}

macro_rules! resolve_invade_impl {
    ($args:expr, $($_await:tt)*) => {
        {
            let (selected_unit,target_coord,game_view,mut doopa):(GridCoord,GridCoord,&mut GameView<'_>,_)=$args;
            let this_unit = game_view.this_team.find_take(&selected_unit).unwrap();

            let _target = game_view.that_team.find_take(&target_coord).unwrap();

            let path = movement::Path::new();
            let m = this_unit.position.dir_to(&target_coord);
            let path = path.add(m).unwrap();

            let mut this_unit=resolve_inner_movement_impl!((this_unit,path,&mut doopa,game_view),$($_await)*);

            this_unit.position = target_coord;

            game_view.this_team.add(this_unit);


            resolve_3_players_nearby_impl!((target_coord,&mut doopa,game_view),$($_await)*);


            for n in target_coord.to_cube().neighbours() {
                resolve_3_players_nearby_impl!((n.to_axial(),&mut doopa,&mut game_view.not()),$($_await)*);
            }



        }
    }
}


mod testy{

    macro_rules! hay {
        ($chicken:expr,$args:ident) => {{
            $chicken.$args();
        }}
    }

    struct Chicken;
    impl Chicken{
        fn no_animate(&self){

        }
        fn animate(&self){

        }
    }
    fn testy(){
        let c=Chicken;
        hay!(c,no_animate);
        hay!(c,animate);
    }
}

macro_rules! resolve_3_players_nearby_impl {
    ($args:expr, $($_await:tt)*) => {{
        let (n, mut doopa, game_view): (GridCoord, _, &mut GameView<'_>) = $args;
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
    } else{
        None
    }
    }};
}

use crate::{ace::UnwrapMe, movement::Path};

pub enum ExtraMove {
    ExtraMove { pos: GridCoord },
    FinishMoving,
}



pub struct InnerPartialMove{
    u:UnitData,
    path:Path
}
impl InnerPartialMove {
    pub fn new(a:UnitData, path: Path) -> Self {
        InnerPartialMove {
            u: a,
            path,
        }
    }
    fn inner_execute_no_animate(self, game_view: &mut GameView<'_>,a:&mut Doopa2) -> UnitData {
        resolve_inner_movement_impl!((self.u, self.path, a, game_view),)
    }

    async fn inner_execute_animate(self, game_view: &mut GameView<'_>,a:&mut Doopa<'_,'_,'_>) -> UnitData {
        resolve_inner_movement_impl!((self.u,self.path,a,game_view),.await)
    }

    pub fn execute(self, game_view: &mut GameView<'_>) -> UnitData {
        self.inner_execute_no_animate(game_view, &mut Doopa2)
    }
    pub async fn execute_with_animation(
        self,
        game_view: &mut GameView<'_>,
        data: &mut AwaitData<'_, '_>,
    ) -> UnitData {
        self.inner_execute_animate(game_view, &mut Doopa::new(data)).await
    }
}




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

    fn inner_execute_no_animate(self, game_view: &mut GameView<'_>,a:&mut Doopa2) -> ExtraMove {
        resolve_movement_impl!((self.selected_unit, self.path, a, game_view),inner_execute_no_animate,)
    }

    async fn inner_execute_animate(self, game_view: &mut GameView<'_>,a:&mut Doopa<'_,'_,'_>) -> ExtraMove {
        resolve_movement_impl!((self.selected_unit, self.path, a, game_view),inner_execute_animate,.await)
    }
    

    pub fn execute(self, game_view: &mut GameView<'_>) -> ExtraMove {
        self.inner_execute_no_animate(game_view,&mut Doopa2)
    }
    pub async fn execute_with_animation(
        self,
        game_view: &mut GameView<'_>,
        data: &mut AwaitData<'_, '_>,
    ) -> ExtraMove {
        self.inner_execute_animate(game_view,&mut Doopa::new(data)).await
    }
}

pub struct Invade {
    selected_unit: GridCoord,
    target_coord: GridCoord,
}

impl Invade {
    pub fn new(a: GridCoord, b: GridCoord) -> Self {
        Invade {
            selected_unit: a,
            target_coord: b,
        }
    }
    pub fn execute(self, relative_game_view: &mut GameView<'_>) {
        resolve_invade_impl!((
            self.selected_unit,
            self.target_coord,
            relative_game_view,
            &mut Doopa2
        ),)
    }
    pub async fn execute_with_animation(
        self,
        relative_game_view: &mut GameView<'_>,
        data: &mut AwaitData<'_, '_>,
    ) {
        resolve_invade_impl!((self.selected_unit,self.target_coord,relative_game_view,&mut Doopa::new(data)),.await)
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
        resolve_3_players_nearby_impl!((self.cell, &mut Doopa2, game_view),)
    }

    pub async fn execute_with_animation(
        self,
        game_view: &mut GameView<'_>,
        data: &mut AwaitData<'_, '_>,
    ) -> Option<UnitData> {
        resolve_3_players_nearby_impl!((self.cell,&mut Doopa::new(data),game_view),.await)
    }
}
