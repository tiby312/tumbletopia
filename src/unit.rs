use crate::{
    ace::{ActiveTeam, UnwrapMe},
    movement::FilterRes,
};

use super::*;

#[derive(Debug, Clone)]
pub struct UnitData {
    pub position: GridCoord,
    pub typ: Type,
}
impl UnitData {
    pub fn new(position: GridCoord, typ: Type) -> Self {
        UnitData { position, typ }
    }
}

#[derive(Debug, Clone)]
pub enum CellSelection {
    MoveSelection(movement::PossibleMoves2<()>),
    BuildSelection(GridCoord),
}

pub struct TribeFilter<'a> {
    tribe: &'a Tribe,
}
impl<'a> movement::Filter for TribeFilter<'a> {
    fn filter(&self, b: &GridCoord) -> FilterRes {
        self.tribe.warriors.filter().filter(b)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Type {
    Warrior,
    Para,
    Rook,
    Mage,
}

impl Type {
    pub fn type_index(&self) -> usize {
        let a = self;
        match a {
            Type::Warrior => 0,
            Type::Para => 1,
            Type::Rook => 2,
            Type::Mage => 3,
        }
    }

    pub fn type_index_inverse(a: usize) -> Type {
        match a {
            0 => Type::Warrior,
            1 => Type::Para,
            2 => Type::Rook,
            3 => Type::Mage,
            _ => unreachable!(),
        }
    }
}

//https://users.rust-lang.org/t/macro-to-dry-sync-and-async-code/67556
macro_rules! resolve_movement {
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

macro_rules! resolve_invade {
    ($args:expr, $($_await:tt)*) => {
        {
            let (selected_unit,target_coord,game_view,doopa):(GridCoord,GridCoord,&mut GameView<'_>,_)=$args;
            let team=game_view.team;
            let this_unit = game_view.this_team.find_take(&selected_unit).unwrap();

            let _target = game_view.that_team.find_take(&target_coord).unwrap();

            let path = movement::Path::new();
            let m = this_unit.position.dir_to(&target_coord);
            let path = path.add(m).unwrap();

            let mut this_unit=resolve_movement!((this_unit,path,doopa,game_view),$($_await)*);

            this_unit.position = target_coord;

            game_view.this_team.add(this_unit);




        }
    }
}

macro_rules! resolve_3_players_nearby {
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

pub struct Doopa<'a, 'b, 'c> {
    data: &'c mut AwaitData<'a, 'b>,
}
impl<'a, 'b, 'c> Doopa<'a, 'b, 'c> {
    pub async fn wait_animation<W: UnwrapMe>(&mut self, m: W, team: ActiveTeam) -> W::Item {
        self.data.wait_animation(m, team).await
    }
}
pub struct Doopa2;
impl Doopa2 {
    pub fn wait_animation<W: UnwrapMe>(&mut self, m: W, _: ActiveTeam) -> W::Item {
        m.direct_unwrap()
    }
}

impl GameView<'_> {
    pub fn resolve_movement_no_animate(
        &mut self,
        start: UnitData,
        path: movement::Path,
    ) -> UnitData {
        resolve_movement!((start, path, Doopa2, self),)
    }
    pub fn resolve_surrounded_no_animate(&mut self, n: hex::Cube) -> Option<UnitData> {
        resolve_3_players_nearby!((n.to_axial(), Doopa2, self),)
    }
    pub async fn resolve_invade_no_animate(
        &mut self,
        selected_unit: GridCoord,
        target_coord: GridCoord,
    ) {
        resolve_invade!((selected_unit, target_coord, self, Doopa2),);
    }
}
pub struct AwaitData<'a, 'b> {
    doop: &'b mut ace::Doop<'a>,
}
impl<'a, 'b> AwaitData<'a, 'b> {
    pub fn new(doop: &'b mut ace::Doop<'a>) -> Self {
        AwaitData { doop }
    }

    pub async fn resolve_movement(
        &mut self,
        start: UnitData,
        path: movement::Path,
        game: &mut GameView<'_>,
    ) -> UnitData {
        resolve_movement!((start,path,Doopa{data:self},game),.await)
    }

    pub async fn resolve_surrounded(
        &mut self,
        n: hex::Cube,
        game_view: &mut GameView<'_>,
    ) -> Option<UnitData> {
        resolve_3_players_nearby!((n.to_axial(),Doopa{data:self},game_view),.await)
    }

    pub async fn resolve_invade(
        &mut self,
        selected_unit: GridCoord,
        target_coord: GridCoord,
        relative_game_view: &mut GameView<'_>,
    ) {
        resolve_invade!((selected_unit,target_coord,relative_game_view,Doopa{data:self}),.await);
    }

    pub async fn wait_animation<K: UnwrapMe>(
        &mut self,
        wrapper: K,
        team_index: ActiveTeam,
    ) -> K::Item {
        let an = wrapper.into_command();
        let aa = self.doop.wait_animation(an, team_index).await;
        K::unwrapme(aa.into_data())
    }
}

pub struct Move {
    selected_unit: GridCoord,
    target_coord: GridCoord,
    extra: GridCoord,
}

impl MoveT for Move {
    fn execute(self, relative_game_view: &mut GameView<'_>) {}
}

pub struct Attack {
    selected_unit: GridCoord,
    target_coord: GridCoord,
}
pub trait MoveT {
    fn execute(self, relative_game_view: &mut GameView<'_>);
}
impl Attack {
    pub fn new(a: GridCoord, b: GridCoord) -> Self {
        Attack {
            selected_unit: a,
            target_coord: b,
        }
    }
}
impl MoveT for Attack {
    fn execute(self, relative_game_view: &mut GameView<'_>) {
        let target = relative_game_view
            .that_team
            .find_take(&self.target_coord)
            .unwrap();
        let mut this_unit = relative_game_view
            .this_team
            .find_slow_mut(&self.selected_unit)
            .unwrap();
        this_unit.position = target.position;
    }
}

#[derive(Debug)]
pub struct Tribe {
    pub warriors: UnitCollection<UnitData>,
}
impl Tribe {
    pub fn add(&mut self, a: UnitData) {
        self.warriors.elem.push(a);
    }

    pub fn find_take(&mut self, a: &GridCoord) -> Option<UnitData> {
        if let Some((i, _)) = self
            .warriors
            .elem
            .iter()
            .enumerate()
            .find(|(_, b)| &b.position == a)
        {
            Some(self.warriors.elem.remove(i))
        } else {
            None
        }
    }

    pub fn find_slow(&self, a: &GridCoord) -> Option<&UnitData> {
        self.warriors.elem.iter().find(|b| &b.position == a)
    }

    pub fn find_slow_mut<'a, 'b>(&'a mut self, a: &'b GridCoord) -> Option<&'a mut UnitData> {
        self.warriors.elem.iter_mut().find(|b| &b.position == a)
    }

    pub fn filter(&self) -> TribeFilter {
        TribeFilter { tribe: self }
    }
}

pub struct Rest<'a, T> {
    left: &'a mut [T],
    right: &'a mut [T],
}
impl<'a, T> Rest<'a, T> {
    pub fn into_iter(self) -> impl Iterator<Item = &'a mut T> {
        self.left.into_iter().chain(self.right.into_iter())
    }
}

//TODO sort this by x and then y axis!!!!!!!
#[derive(Debug)]
pub struct UnitCollection<T: HasPos> {
    pub elem: Vec<T>,
}

impl<T: HasPos> UnitCollection<T> {
    pub fn new(elem: Vec<T>) -> Self {
        UnitCollection { elem }
    }
    pub fn remove(&mut self, a: &GridCoord) -> T {
        let (i, _) = self
            .elem
            .iter()
            .enumerate()
            .find(|(_, b)| b.get_pos() == a)
            .unwrap();
        self.elem.swap_remove(i)
    }

    pub fn find_mut(&mut self, a: &GridCoord) -> Option<&mut T> {
        self.elem.iter_mut().find(|b| b.get_pos() == a)
    }

    pub fn find_ext_mut(&mut self, a: &GridCoord) -> Option<(&mut T, Rest<T>)> {
        let (i, _) = self
            .elem
            .iter()
            .enumerate()
            .find(|(_, b)| b.get_pos() == a)?;
        let (left, rest) = self.elem.split_at_mut(i);
        let (foo, right) = rest.split_first_mut().unwrap();
        Some((foo, Rest { left, right }))
    }

    pub fn find(&self, a: &GridCoord) -> Option<&T> {
        self.elem.iter().find(|b| b.get_pos() == a)
    }
    pub fn filter(&self) -> UnitCollectionFilter<T> {
        UnitCollectionFilter { a: &self.elem }
    }
}

pub struct SingleFilter<'a> {
    a: &'a GridCoord,
}
impl<'a> movement::Filter for SingleFilter<'a> {
    fn filter(&self, a: &GridCoord) -> FilterRes {
        FilterRes::from_bool(self.a != a)
    }
}

pub struct UnitCollectionFilter<'a, T> {
    a: &'a [T],
}
impl<'a> movement::Filter for UnitCollectionFilter<'a, UnitData> {
    fn filter(&self, b: &GridCoord) -> FilterRes {
        FilterRes::from_bool(self.a.iter().find(|a| a.get_pos() == b).is_some())
    }
}

pub trait HasPos {
    fn get_pos(&self) -> &GridCoord;
}
impl HasPos for GridCoord {
    fn get_pos(&self) -> &GridCoord {
        self
    }
}

impl HasPos for UnitData {
    fn get_pos(&self) -> &GridCoord {
        &self.position
    }
}
