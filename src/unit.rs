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

impl GameView<'_> {
    // pub fn resolve_movement_no_animate(
    //     &mut self,
    //     start: UnitData,
    //     path: movement::Path,
    // ) -> UnitData {
    //     resolve_movement_impl!((start, path, Doopa2, self),)
    // }
    // pub fn resolve_surrounded_no_animate(&mut self, n: hex::Cube) -> Option<UnitData> {
    //     resolve_3_players_nearby_impl!((n.to_axial(), Doopa2, self),)
    // }
    // pub async fn resolve_invade_no_animate(
    //     &mut self,
    //     selected_unit: GridCoord,
    //     target_coord: GridCoord,
    // ) {
    //     resolve_invade_impl!((selected_unit, target_coord, self, Doopa2),);
    // }
}
pub struct AwaitData<'a, 'b> {
    doop: &'b mut ace::Doop<'a>,
}
impl<'a, 'b> AwaitData<'a, 'b> {
    pub fn new(doop: &'b mut ace::Doop<'a>) -> Self {
        AwaitData { doop }
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

    pub fn filter_type(&self, ty: Type) -> UnitCollectionFilterType<T> {
        UnitCollectionFilterType { a: &self.elem, ty }
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

pub struct UnitCollectionFilterType<'a, T> {
    a: &'a [T],
    ty: Type,
}
impl<'a> movement::Filter for UnitCollectionFilterType<'a, UnitData> {
    fn filter(&self, b: &GridCoord) -> FilterRes {
        FilterRes::from_bool(
            self.a
                .iter()
                .find(|a| a.position == *b && a.typ == self.ty)
                .is_some(),
        )
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
