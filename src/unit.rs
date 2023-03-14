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
    pub fn get_movement_data(&self) -> (i8, i8) {
        let a = self;
        match a.val {
            Type::Warrior => (1, 1),
            Type::Archer => (1, 1),
            Type::Mage => (1, 1),
            Type::Knight => (1, 1),
        }
    }

    //TODO additionally return a animation??.
    pub fn get_attack_data(&self) -> impl Iterator<Item = GridCoord> {
        let a = self;
        let first = if let Type::Warrior = a.val {
            Some(a.position.to_cube().range(1))
        } else {
            None
        };

        let second = if let Type::Archer = a.val {
            Some(a.position.to_cube().ring(2))
        } else {
            None
        };

        let third = if let Type::Knight = a.val {
            Some(a.position.to_cube().ring(3))
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
        let has_stamina_to_move = s.stamina.0 >= 0; //0f??? TODO

        let ret = enemy_in_range || has_stamina_to_move;
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
            health: 10,
            selectable: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CellSelection {
    MoveSelection(movement::PossibleMoves, (), Vec<GridCoord>),
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
    inner: T,
    val: Type,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Type {
    Warrior,
    Knight,
    Archer,
    Mage,
}

impl Type {
    fn type_index(&self) -> usize {
        let a = self;
        match a {
            Type::Warrior => 0,
            Type::Archer => 1,
            Type::Knight => 2,
            Type::Mage => 3,
        }
    }

    fn type_index_inverse(a: usize) -> Type {
        match a {
            0 => Type::Warrior,
            1 => Type::Archer,
            2 => Type::Knight,
            3 => Type::Mage,
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

        start
    }
    pub async fn resolve_attack(
        &mut self,
        mut this_unit: WarriorType<UnitData>,
        mut target: WarriorType<UnitData>,
    ) -> Pair {
        match (this_unit.val, target.val) {
            (Type::Warrior, Type::Warrior) => {
                let damage = 5;
                let counter_damage = 5;

                if target.health <= damage {
                    let path = movement::Path::new();
                    let m = this_unit.position.dir_to(&target.position);
                    let path = path.add(m).unwrap();

                    let it = animation::movement(this_unit.position, path, self.grid_matrix);
                    let aa = animation::Animation::new(
                        it,
                        AnimationOptions::Attack([this_unit, target]),
                    );
                    let aa = self.doop.wait_animation(aa, self.team_index).await;

                    let AnimationOptions::Attack([mut this_unit,target])=aa.into_data() else{
                        unreachable!();
                    };
                    //todo kill target animate
                    this_unit.position = target.position;
                    this_unit.attacked = true;
                    return Pair(Some(this_unit), None);
                } else {
                    target.health -= damage;

                    let it =
                        animation::attack(this_unit.position, target.position, self.grid_matrix);
                    let aa = animation::Animation::new(
                        it,
                        AnimationOptions::Attack([this_unit, target]),
                    );
                    let aa = self.doop.wait_animation(aa, self.team_index).await;
                    let AnimationOptions::Attack([mut this_unit,target])=aa.into_data() else{
                        unreachable!();
                    };

                    let it =
                        animation::attack(target.position, this_unit.position, self.grid_matrix);
                    let aa = animation::Animation::new(
                        it,
                        AnimationOptions::CounterAttack([this_unit, target]),
                    );
                    let aa = self.doop.wait_animation(aa, self.team_index).await;
                    let AnimationOptions::CounterAttack([mut this_unit,mut target])=aa.into_data() else{
                        unreachable!()
                    };

                    if this_unit.health <= counter_damage {
                        //todo self die animation.
                        Pair(None, Some(target))
                    } else {
                        //todo normal attack animation..
                        this_unit.attacked = true;
                        this_unit.health -= counter_damage;
                        Pair(Some(this_unit), Some(target))
                    }
                }
            }
            _ => {
                todo!()
            }
        }
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
        for a in self.warriors.iter_mut() {
            for b in a.elem.iter_mut() {
                if b.stamina.0 <= 10 - 1 {
                    b.stamina.0 += 1;
                }
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
