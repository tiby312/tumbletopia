use crate::{
    ace::{generate_unit_possible_moves2, AnimationWrapper, Doop, UnwrapMe},
    animation::Animation,
    movement::{Filter, NoFilter},
};

use super::*;

#[derive(Debug)]
pub struct UnitData {
    pub position: GridCoord,
    pub stamina: MoveUnit,
    pub attacked: bool,
    pub health: i8,
    pub selectable: bool,
}

impl WarriorType<&UnitData> {
    pub fn block_counter(&self) -> bool {
        match self.val {
            Type::Warrior => true,
            _ => false,
        }
    }

    // TODO use
    // Attack again if first attack was dodged
    pub fn pierce_attack(&self) -> bool {
        match self.val {
            Type::Warrior => true,
            _ => false,
        }
    }

    pub fn get_movement_data(&self) -> i8 {
        let a = self;
        match a.val {
            Type::Warrior => 2,
            Type::Rook => 1,
            Type::Mage => 1,
            Type::Archer => 1,
        }
    }

    pub fn get_friendly_data(&self) -> impl Iterator<Item = GridCoord> {
        let first = if let Type::Mage = self.val {
            Some(self.position.to_cube().ring(2))
        } else {
            None
        };

        first.into_iter().flatten().map(|x| x.to_axial())
    }

    //TODO additionally return a animation??.
    pub fn get_attack_data(&self, ff: impl Filter + Copy) -> impl Iterator<Item = GridCoord> {
        let a = self;
        let first = if let Type::Warrior = a.val {
            Some(
                a.position
                    .to_cube()
                    .ring(1)
                    .filter(move |o| ff.filter(&o.to_axial())),
            )
        } else {
            None
        };

        let second = if let Type::Rook = a.val {
            Some(a.position.to_cube().rays(1, 10, ff))
        } else {
            None
        };

        let third = if let Type::Archer = a.val {
            Some(
                a.position
                    .to_cube()
                    .range(2)
                    .filter(move |o| ff.filter(&o.to_axial())),
            )
        } else {
            None
        };
        first
            .into_iter()
            .flatten()
            .chain(second.into_iter().flatten())
            .chain(third.into_iter().flatten())
            .map(|x| x.to_axial())
    }

    //TODO use this instead of gridcoord when you know the type!!!!!
    pub fn slim(&self) -> WarriorType<GridCoord> {
        WarriorType {
            inner: self.inner.position,
            val: self.val,
        }
    }
    pub fn calculate_selectable(
        &self,
        this_team: &Tribe,
        that_team: &Tribe,
        grid_matrix: &GridMatrix,
    ) -> bool {
        let s = self; //this_team.lookup(*self);
        let pos = ace::generate_unit_possible_moves2(&s, this_team, that_team, grid_matrix);

        //check if there are enemies in range.
        let enemy_in_range = {
            let (_, att) = match &pos {
                CellSelection::MoveSelection(ss, _, att) => (ss, att),
                _ => unreachable!(),
            };

            let mut found = false;
            for a in att.iter() {
                if let Some(_) = that_team.find_slow(a) {
                    found = true;
                    break;
                }
            }
            found
        };

        //TODO move this and the above into an high-level "Has possible moves function"
        let has_stamina_to_move = s.stamina.0 > 0; //0f??? TODO
        let ret = enemy_in_range && !s.attacked || has_stamina_to_move && !s.attacked;
        ret
    }
}
impl UnitData {
    //TODO replace with has possible moves
    // fn selectable(&self) -> bool {
    //     self.has_possible_moves()
    //     //!self.attacked || self.stamina.0 > 0
    // }

    // fn can_attack(&self) -> bool {
    //     !self.attacked
    // }

    pub fn new(position: GridCoord) -> Self {
        UnitData {
            position,
            stamina: MoveUnit(0),
            attacked: false,
            health: 2,
            selectable: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CellSelection {
    MoveSelection(movement::PossibleMoves, Vec<GridCoord>, Vec<GridCoord>),
    BuildSelection(GridCoord),
}

pub struct TribeFilter<'a> {
    tribe: &'a Tribe,
}
impl<'a> movement::Filter for TribeFilter<'a> {
    fn filter(&self, b: &GridCoord) -> bool {
        self.tribe
            .warriors
            .iter()
            .map(|a| a.filter().filter(b))
            .fold(true, |a, b| a && b)
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
    Archer,
    Rook,
    Mage,
}

impl Type {
    pub fn type_index(&self) -> usize {
        let a = self;
        match a {
            Type::Warrior => 0,
            Type::Rook => 1,
            Type::Mage => 2,
            Type::Archer => 3,
        }
    }

    pub fn type_index_inverse(a: usize) -> Type {
        match a {
            0 => Type::Warrior,
            1 => Type::Rook,
            2 => Type::Mage,
            3 => Type::Archer,
            _ => unreachable!(),
        }
    }
}

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

pub struct Dash {
    pub has_dash: bool,
}

pub struct AwaitData<'a, 'b> {
    doop: &'b mut ace::Doop<'a>,
    grid_matrix: &'b GridMatrix,
    team_index: usize,
}
impl<'a, 'b> AwaitData<'a, 'b> {
    pub fn new(
        doop: &'b mut ace::Doop<'a>,
        grid_matrix: &'b GridMatrix,
        team_index: usize,
    ) -> Self {
        AwaitData {
            doop,
            grid_matrix,
            team_index,
        }
    }

    pub async fn resolve_movement(
        &mut self,
        start: WarriorType<UnitData>,
        path: &movement::Path,
    ) -> WarriorType<UnitData> {
        let it = animation::movement(start.position, path.clone(), self.grid_matrix);

        let aa = animation::Animation::new(it, AnimationOptions::Movement(start));

        let aa = self.doop.wait_animation(aa, self.team_index).await;

        let AnimationOptions::Movement(mut start)=aa.into_data()else{
            unreachable!()
        };

        start.stamina.0 -= path.total_cost().0;
        start.position = path.get_end_coord(start.position);

        // let has_dash = match start.val {
        //     Type::Archer => false,
        //     _ => {
        //         true
        //     }
        // };

        // if !has_dash{
        //     start.attacked=true;
        // }
        start.attacked = true;
        start
    }

    pub async fn resolve_heal(
        &mut self,
        mut this_unit: WarriorType<UnitData>,
        mut target: WarriorType<UnitData>,
    ) -> Pair {
        //TODO make the user not able to perform a no-op heal thus wasting a turn.
        target.health = (target.health + 1).min(2);

        this_unit.attacked = true;

        let it = animation::attack(this_unit.position, target.position, self.grid_matrix);
        let aa = animation::Animation::new(it, AnimationOptions::Heal([this_unit, target]));
        let aa = self.doop.wait_animation(aa, self.team_index).await;

        let AnimationOptions::Heal([this_unit,target])=aa.into_data() else{
            unreachable!();
        };

        Pair(Some(this_unit), Some(target))
    }
    pub async fn resolve_attack(
        &mut self,
        mut this_unit: WarriorType<UnitData>,
        mut target: WarriorType<UnitData>,
        support_attack: bool,
    ) -> Pair {
        //TODO store somewhere
        let damage = 1;

        let counter_damage = if let Some(_) = target
            .as_ref()
            .get_attack_data(NoFilter)
            .find(|&a| a == *this_unit.as_ref().slim())
        {
            Some(damage)
        } else {
            None
        };

        // let counter_damage = match (this_unit.val, target.val) {
        //     (Type::Warrior, Type::Rook) => None,
        //     (Type::Warrior, Type::Archer) => None,
        //     _ => counter_damage,
        // };

        // let counter_damage = if support_attack { None } else { counter_damage };

        let move_on_kill = match (this_unit.val, target.val) {
            (Type::Rook, _) => false,
            (Type::Warrior, _) => true,
            (Type::Archer, _) => false,
            _ => {
                todo!()
            }
        };

        target.health -= damage;
        this_unit.attacked = true;

        if target.health <= 0 {
            assert!(!support_attack);
            let this_unit = if move_on_kill {
                this_unit.health = (this_unit.health + 1).min(2);

                let path = movement::Path::new();
                let m = this_unit.position.dir_to(&target.position);
                let path = path.add(m).unwrap();

                let it = animation::movement(this_unit.position, path, self.grid_matrix);
                let aa = AnimationOptions::attack([this_unit, target]);
                let [mut this_unit, target] = self.wait_animation(it, aa).await;

                //todo kill target animate
                this_unit.position = target.position;
                this_unit
            } else {
                let it = animation::attack(this_unit.position, target.position, self.grid_matrix);
                let aa = AnimationOptions::attack([this_unit, target]);
                let [this_unit, _target] = self.wait_animation(it, aa).await;

                this_unit
            };
            return Pair(Some(this_unit), None);
        }

        let it = animation::attack(this_unit.position, target.position, self.grid_matrix);
        let aa = AnimationOptions::attack([this_unit, target]);
        let [this_unit, target] = self.wait_animation(it, aa).await;

        let Some(counter_damage)=counter_damage else{
            return  Pair(Some(this_unit), Some(target))
        };

        let it = animation::attack(target.position, this_unit.position, self.grid_matrix);
        let aa = AnimationOptions::counter_attack([this_unit, target]);
        let [mut this_unit, target] = self.wait_animation(it, aa).await;

        if !this_unit.as_ref().block_counter()
            || this_unit.as_ref().block_counter() && target.as_ref().pierce_attack()
        {
            this_unit.health -= counter_damage;
            if this_unit.health <= 0 {
                //todo self die animation.
                return Pair(None, Some(target));
            }
        }
        Pair(Some(this_unit), Some(target))
    }

    async fn wait_animation<'c, K: UnwrapMe, I: Iterator<Item = Vector2<f32>> + 'static>(
        &mut self,
        it: I,
        wrapper: AnimationWrapper<K>,
    ) -> K::Item {
        let aa = animation::Animation::new(it, wrapper.enu);
        let aa = self.doop.wait_animation(aa, self.team_index).await;
        wrapper.unwrapper.unwrapme(aa.into_data())
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
    pub fn other_units_in_range_of_target<'b>(
        &'b self,
        target: GridCoord,
        grid_matrix: &'b GridMatrix,
    ) -> impl Iterator<Item = WarriorType<&UnitData>> + 'b {
        self.warriors.iter().enumerate().flat_map(move |(i, o)| {
            o.elem.iter().filter_map(move |u| {
                let unit = WarriorType {
                    inner: u,
                    val: Type::type_index_inverse(i),
                };

                unit.get_attack_data(&self.filter().chain(grid_matrix.filter()))
                    .find(|&f| f == target)
                    .map(move |_| unit)
            })
        })
    }
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

    pub fn reset_attacked(&mut self) {
        for a in self.warriors.iter_mut() {
            for b in a.elem.iter_mut() {
                b.attacked = false;
                //Just set it selectable during other peoples turns so its not gray,
                //even though it is not selectable.
                b.selectable = true;
            }
        }
    }
    pub fn calculate_selectable_all(&mut self, that_team: &mut Tribe, grid_matrix: &GridMatrix) {
        let this_team = self;
        for i in 0..this_team.warriors.len() {
            let a = &this_team.warriors[i];
            for ii in 0..a.elem.len() {
                let a = &this_team.warriors[i];

                let b = &a.elem[ii];
                let b = WarriorType {
                    inner: b,
                    val: Type::type_index_inverse(i),
                };

                let vv = b.calculate_selectable(this_team, that_team, grid_matrix);

                this_team.warriors[i].elem[ii].selectable = vv;
                //b.selectable=vv;
            }
        }
    }
    pub fn replenish_stamina(&mut self) {
        for (i, val) in self.warriors.iter_mut().enumerate() {
            for unit in val.elem.iter_mut() {
                let mut k = WarriorType {
                    val: Type::type_index_inverse(i),
                    inner: unit,
                };
                k.stamina.0 = k.as_ref().get_movement_data();
            }
        }
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
    fn filter(&self, a: &GridCoord) -> bool {
        self.a != a
    }
}

pub struct UnitCollectionFilter<'a, T> {
    a: &'a [T],
}
impl<'a, T: HasPos> movement::Filter for UnitCollectionFilter<'a, T> {
    fn filter(&self, b: &GridCoord) -> bool {
        self.a.iter().find(|a| a.get_pos() == b).is_none()
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
