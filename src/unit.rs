use crate::{
    ace::{ActiveTeam, UnwrapMe},
    movement::FilterRes,
};

use super::*;

#[derive(Debug)]
pub struct UnitData {
    pub position: GridCoord,
}

impl WarriorType<&UnitData> {
    //TODO use this instead of gridcoord when you know the type!!!!!
    pub fn slim(&self) -> WarriorType<GridCoord> {
        WarriorType {
            inner: self.inner.position,
            val: self.val,
        }
    }
}
impl UnitData {
    pub fn new(position: GridCoord) -> Self {
        UnitData { position }
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
        self.tribe
            .warriors
            .iter()
            .map(|a| a.filter().filter(b))
            .fold(FilterRes::Stop, |a, b| a.or(b))
    }
}

impl<T> std::borrow::Borrow<T> for WarriorType<T> {
    fn borrow(&self) -> &T {
        &self.inner
    }
}
impl<T> std::borrow::BorrowMut<T> for WarriorType<T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct WarriorType<T> {
    pub inner: T,
    pub val: Type,
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

impl<'a> WarriorType<&'a mut UnitData> {
    pub fn as_ref(&self) -> WarriorType<&UnitData> {
        WarriorType {
            inner: self.inner,
            val: self.val,
        }
    }
    pub fn to_ref(self) -> WarriorType<&'a UnitData> {
        let val = self.val;
        WarriorType {
            inner: self.inner,
            val,
        }
    }
}

impl WarriorType<UnitData> {
    //TODO use this instead of gridcoord when you know the type!!!!!
    pub fn as_ref(&self) -> WarriorType<&UnitData> {
        WarriorType {
            inner: &self.inner,
            val: self.val,
        }
    }
}

impl<T> std::ops::Deref for WarriorType<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for WarriorType<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct AwaitData<'a, 'b> {
    doop: &'b mut ace::Doop<'a>,
    team_index: ActiveTeam,
}
impl<'a, 'b> AwaitData<'a, 'b> {
    pub fn new(doop: &'b mut ace::Doop<'a>, team_index: ActiveTeam) -> Self {
        AwaitData { doop, team_index }
    }

    pub async fn resolve_movement(
        &mut self,
        start: WarriorType<UnitData>,
        path: movement::Path,
    ) -> WarriorType<UnitData> {
        let an = animation::AnimationCommand::Movement { unit: start, path };
        let mut start = self.wait_animation(an, ace::Movement).await;

        start.position = path.get_end_coord(start.position);

        start
    }

    pub async fn resolve_group_attack(
        &mut self,
        n: hex::Cube,
        game_view: &mut GameView<'_>,
    ) -> Option<WarriorType<UnitData>> {
        let that_team = &mut game_view.that_team;
        let this_team = &mut game_view.this_team;
        let Some(k)=that_team.find_slow(&n.to_axial()) else{
            return None;
        };

        let nearby_friendlies: Vec<_> = n
            .neighbours()
            .filter(|a| this_team.find_slow(&a.to_axial()).is_some())
            .collect();
        if nearby_friendlies.len() >= 3 {
            let mut enemy = Some(that_team.lookup_take(k.slim()));

            //TODO add animation
            //kill this unit!!!
            for a in nearby_friendlies {
                let f = this_team.find_slow(&a.to_axial()).unwrap();
                let f = this_team.lookup_take(f.slim());

                let _tt = enemy.as_ref().unwrap().position;

                let an = animation::AnimationCommand::Attack {
                    attacker: f,
                    defender: enemy.take().unwrap(),
                };
                let [this_unit, target] = self.wait_animation(an, ace::Attack).await;

                //this_unit.resting = 1;
                this_team.add(this_unit);
                enemy = Some(target);
            }

            enemy
        } else {
            None
        }
    }
    pub async fn resolve_attack(
        &mut self,
        this_unit: WarriorType<UnitData>,
        target: WarriorType<UnitData>,
        support_attack: bool,
        _path: &movement::Path,
    ) -> Pair {
        let _move_on_kill = match (this_unit.val, target.val) {
            (Type::Rook, _) => false,
            (Type::Warrior, _) => true,
            (Type::Para, _) => false,
            _ => {
                todo!()
            }
        };

        assert!(!support_attack);

        let path = movement::Path::new();
        let m = this_unit.position.dir_to(&target.position);
        let path = path.add(m).unwrap();

        let an = animation::AnimationCommand::Movement {
            unit: this_unit,
            path,
        };

        let mut this_unit = self.wait_animation(an, ace::Movement).await;

        //todo kill target animate
        this_unit.position = target.position;
        return Pair(Some(this_unit), None);
    }

    pub async fn wait_animation<K: UnwrapMe>(
        &mut self,
        an: animation::AnimationCommand,
        wrapper: K,
    ) -> K::Item {
        let aa = self.doop.wait_animation(an, self.team_index).await;
        wrapper.unwrapme(aa.into_data())
    }
}

pub struct Pair(
    pub Option<WarriorType<UnitData>>,
    pub Option<WarriorType<UnitData>>,
);

#[derive(Debug)]
pub struct Tribe {
    pub warriors: Vec<UnitCollection<UnitData>>,
}
impl Tribe {
    pub fn lookup(&self, a: WarriorType<GridCoord>) -> WarriorType<&UnitData> {
        self.warriors[a.val.type_index()]
            .find(&a.inner)
            .map(|b| WarriorType {
                inner: b,
                val: a.val,
            })
            .unwrap()
    }
    pub fn lookup_mut(&mut self, a: &WarriorType<GridCoord>) -> WarriorType<&mut UnitData> {
        self.warriors[a.val.type_index()]
            .find_mut(&a.inner)
            .map(|b| WarriorType {
                inner: b,
                val: a.val,
            })
            .unwrap()
    }
    pub fn lookup_take(&mut self, a: WarriorType<GridCoord>) -> WarriorType<UnitData> {
        Some(self.warriors[a.val.type_index()].remove(&a.inner))
            .map(|b| WarriorType {
                inner: b,
                val: a.val,
            })
            .unwrap()
    }

    pub fn add(&mut self, a: WarriorType<UnitData>) {
        self.warriors[a.val.type_index()].elem.push(a.inner);
    }

    pub fn find_slow(&self, a: &GridCoord) -> Option<WarriorType<&UnitData>> {
        for (c, o) in self.warriors.iter().enumerate() {
            if let Some(k) = o.find(a) {
                return Some(WarriorType {
                    inner: k,
                    val: Type::type_index_inverse(c),
                });
            }
        }

        None
    }

    pub fn find_slow_mut(&mut self, a: &GridCoord) -> Option<WarriorType<&mut UnitData>> {
        for (c, o) in self.warriors.iter_mut().enumerate() {
            if let Some(k) = o.find_mut(a) {
                return Some(WarriorType {
                    inner: k,
                    val: Type::type_index_inverse(c),
                });
            }
        }

        None
    }

    pub fn filter(&self) -> TribeFilter {
        TribeFilter { tribe: self }
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
