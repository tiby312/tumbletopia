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

// #[must_use]
// pub struct MoveSelector<'a>{
//     game:&'a mut GameRun
// }
// impl<'a> MoveSelector<'a>{
//     pub fn select(self,a:GridCoord){
//         todo!()
//     }
// }
// pub struct GameRun{

// }
// impl GameRun{
//     fn get_moves(&mut self,grid:GridCoord)->(MoveSelector,Vec<GridCoord>){
//         todo!()
//     }

// }

pub struct AwaitData<'a, 'b> {
    doop: &'b mut ace::Doop<'a>,
    team_index: ActiveTeam,
}
impl<'a, 'b> AwaitData<'a, 'b> {
    pub fn new(doop: &'b mut ace::Doop<'a>, team_index: ActiveTeam) -> Self {
        AwaitData { doop, team_index }
    }

    pub async fn resolve_movement(&mut self, start: UnitData, path: movement::Path) -> UnitData {
        let an = animation::AnimationCommand::Movement { unit: start, path };
        let mut start = self
            .wait_animation(an, ace::Movement, self.team_index)
            .await;

        start.position = path.get_end_coord(start.position);

        start
    }

    pub async fn resolve_surrounded(
        &mut self,
        n: hex::Cube,
        game_view: &mut GameView<'_>,
    ) -> Option<UnitData> {
        if let Some((unit_pos, b)) = game_view.resolve_surrounded_logic(n.to_axial()) {
            let u = unit_pos.position;
            game_view.this_team.add(unit_pos);
            for n in b {
                self.animate_attack(u, n, game_view).await;
            }
            Some(game_view.this_team.find_take(&u).unwrap())
        } else {
            None
        }
    }

    pub async fn resolve_invade(
        &mut self,
        selected_unit: GridCoord,
        target_coord: GridCoord,
        relative_game_view: &mut GameView<'_>,
    ) {
        self.animate_invade(selected_unit, target_coord, relative_game_view)
            .await;
        let _ = relative_game_view.resolve_invade_logic(selected_unit, target_coord);
        
    }

    pub async fn wait_animation<K: UnwrapMe>(
        &mut self,
        an: animation::AnimationCommand,
        wrapper: K,
        team_index: ActiveTeam,
    ) -> K::Item {
        let aa = self.doop.wait_animation(an, team_index).await;
        wrapper.unwrapme(aa.into_data())
    }

    pub async fn animate_attack(
        &mut self,
        unit_pos: GridCoord,
        n: GridCoord,
        game_view: &mut GameView<'_>,
    ) {
        let unit_pos = game_view.this_team.find_take(&unit_pos).unwrap();
        let them = game_view.that_team.find_take(&n).unwrap();

        let an = animation::AnimationCommand::Attack {
            attacker: them,
            defender: unit_pos,
        };
        let [them, this_unit] = self
            .wait_animation(an, ace::Attack, self.team_index.not())
            .await;

        game_view.that_team.add(them);
        game_view.this_team.add(this_unit);
    }

    pub async fn animate_invade(
        &mut self,
        selected_unit: GridCoord,
        target_coord: GridCoord,
        relative_game_view: &mut GameView<'_>,
    ) {
        let this_unit = relative_game_view
            .this_team
            .find_take(&selected_unit)
            .unwrap();

        let path = movement::Path::new();
        let m = this_unit.position.dir_to(&target_coord);
        let path = path.add(m).unwrap();

        let an = animation::AnimationCommand::Movement {
            unit: this_unit,
            path,
        };

        let this_unit = self
            .wait_animation(an, ace::Movement, self.team_index)
            .await;

        relative_game_view.this_team.add(this_unit);
    }
}

// pub struct AttackAnimator {
//     attack: Attack,
// }
// impl AttackAnimator {
//     pub fn new(a: GridCoord, b: GridCoord) -> Self {
//         AttackAnimator {
//             attack: Attack::new(a, b),
//         }
//     }
//     pub async fn animate(
//         self,
//         await_data: &mut AwaitData<'_, '_>,
//         relative_game_view: &mut GameView<'_>,
//     ) -> Attack {
//         let target = relative_game_view
//             .that_team
//             .find_take(&self.attack.target_coord)
//             .unwrap();
//         let this_unit = relative_game_view
//             .this_team
//             .find_take(&self.attack.selected_unit)
//             .unwrap();

//         let path = movement::Path::new();
//         let m = this_unit.position.dir_to(&target.position);
//         let path = path.add(m).unwrap();

//         let an = animation::AnimationCommand::Movement {
//             unit: this_unit,
//             path,
//         };

//         let this_unit = await_data
//             .wait_animation(an, ace::Movement, await_data.team_index)
//             .await;

//         //todo kill target animate
//         //this_unit.position = target.position;
//         relative_game_view.that_team.add(target);
//         relative_game_view.this_team.add(this_unit);
//         self.attack
//     }
// }

// pub struct Attack {
//     selected_unit: GridCoord,
//     target_coord: GridCoord,
// }
// impl Attack {
//     pub fn new(a: GridCoord, b: GridCoord) -> Self {
//         Attack {
//             selected_unit: a,
//             target_coord: b,
//         }
//     }

//     pub fn execute(self, relative_game_view: &mut GameView<'_>) {
//         let target = relative_game_view
//             .that_team
//             .find_take(&self.target_coord)
//             .unwrap();
//         let mut this_unit = relative_game_view
//             .this_team
//             .find_slow_mut(&self.selected_unit)
//             .unwrap();
//         this_unit.position = target.position;
//     }
// }

// pub trait MoveTrait{
//     fn execute(&self){

//     }
//     fn animate(&self,game_view:&mut GameView<'_>)->Box<dyn std::future::Future<Output=()>>;
// }

// pub enum Move{
//     Attack{
//         selected_unit:WarriorType<GridCoord>,
//         target_coord:WarriorType<GridCoord>
//     },
//     Move{
//         selected_unit:WarriorType<GridCoord>,
//         target_coord:WarriorType<GridCoord>
//     }
// }

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
