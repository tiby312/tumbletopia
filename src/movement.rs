use crate::hex::HDir;

//pub use self::movement_mesh::RelativeMesh;

use super::*;

#[derive(Hash, Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
#[repr(transparent)]
pub struct GridCoord(pub [i16; 2]);
impl GridCoord {
    pub fn zero() -> GridCoord {
        GridCoord([0; 2])
    }
    pub fn dir_to(&self, other: &GridCoord) -> HDir {
        let mut offset = other.sub(self);

        offset.0[0] = offset.0[0].clamp(-1, 1);
        offset.0[1] = offset.0[1].clamp(-1, 1);

        // assert!(offset.0[0].abs() <= 1);
        // assert!(offset.0[1].abs() <= 1);
        let offset = offset.to_cube();

        hex::OFFSETS
            .iter()
            .enumerate()
            .find(|(_, x)| **x == offset.0)
            .map(|(i, _)| HDir::from(i as u8))
            .unwrap()
    }
    pub fn to_cube(self) -> hex::Cube {
        let a = self.0;
        hex::Cube([a[0], a[1], -a[0] - a[1]])
    }

    pub fn advance(self, m: HDir) -> GridCoord {
        self.add(m.to_relative())
    }
    pub fn back(self, m: HDir) -> GridCoord {
        self.sub(&m.to_relative())
    }
    pub fn sub(mut self, o: &GridCoord) -> Self {
        self.0[0] -= o.0[0];
        self.0[1] -= o.0[1];
        self
    }
    pub const fn add(mut self, o: GridCoord) -> Self {
        self.0[0] += o.0[0];
        self.0[1] += o.0[1];
        self
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MoveUnit(pub i8);
impl MoveUnit {
    pub fn add(self, a: MoveUnit) -> Self {
        MoveUnit(self.0 + a.0)
    }
    pub fn sub(self, a: MoveUnit) -> Self {
        MoveUnit(self.0 - a.0)
    }
}

impl<T: Filter> Filter for &T {
    fn filter(&self, a: &GridCoord) -> FilterRes {
        (**self).filter(a)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct NoFilter;

impl Filter for NoFilter {
    fn filter(&self, _: &GridCoord) -> FilterRes {
        FilterRes::from_bool(true)
    }
}

pub struct FilterThese<'a>(pub &'a [GridCoord]);

impl Filter for FilterThese<'_> {
    fn filter(&self, a: &GridCoord) -> FilterRes {
        FilterRes::from_bool(self.0.contains(a))
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum FilterRes {
    Accept,
    Stop,
}
impl FilterRes {
    pub fn to_bool(self) -> bool {
        match self {
            FilterRes::Accept => true,
            FilterRes::Stop => false,
        }
    }
    pub fn and(self, other: FilterRes) -> FilterRes {
        match (self, other) {
            (FilterRes::Accept, FilterRes::Accept) => FilterRes::Accept,
            (FilterRes::Accept, FilterRes::Stop) => FilterRes::Stop,
            (FilterRes::Stop, FilterRes::Accept) => FilterRes::Stop,
            (FilterRes::Stop, FilterRes::Stop) => FilterRes::Stop,
        }
    }

    pub fn or(self, other: FilterRes) -> FilterRes {
        match (self, other) {
            (FilterRes::Accept, FilterRes::Accept) => FilterRes::Accept,
            (FilterRes::Accept, FilterRes::Stop) => FilterRes::Accept,
            (FilterRes::Stop, FilterRes::Accept) => FilterRes::Accept,
            (FilterRes::Stop, FilterRes::Stop) => FilterRes::Stop,
        }
    }

    pub fn from_bool(val: bool) -> Self {
        if val {
            FilterRes::Accept
        } else {
            FilterRes::Stop
        }
    }
}

pub struct AcceptCoords<I> {
    coords: I,
}
impl<I: Iterator<Item = GridCoord> + Clone> AcceptCoords<I> {
    pub fn new(coords: I) -> Self {
        Self { coords }
    }
}
impl<I: Iterator<Item = GridCoord> + Clone> Filter for AcceptCoords<I> {
    fn filter(&self, a: &GridCoord) -> FilterRes {
        if self.coords.clone().any(|b| b == *a) {
            FilterRes::Accept
        } else {
            FilterRes::Stop
        }
    }
}

pub trait Filter {
    fn filter(&self, a: &GridCoord) -> FilterRes;
    fn and<K: Filter>(self, other: K) -> And<Self, K>
    where
        Self: Sized,
    {
        And { a: self, b: other }
    }
    fn or<K: Filter>(self, other: K) -> Or<Self, K>
    where
        Self: Sized,
    {
        Or { a: self, b: other }
    }

    fn not(self) -> NotFilter<Self>
    where
        Self: Sized,
    {
        NotFilter { filter: self }
    }
}
pub struct NotFilter<F> {
    filter: F,
}
impl<F: Filter> Filter for NotFilter<F> {
    fn filter(&self, a: &GridCoord) -> FilterRes {
        match self.filter.filter(a) {
            FilterRes::Accept => FilterRes::Stop,
            FilterRes::Stop => FilterRes::Accept,
        }
    }
}

pub enum Either<A, B> {
    A(A),
    B(B),
}
impl<A: Filter, B: Filter> Filter for Either<A, B> {
    fn filter(&self, a: &GridCoord) -> FilterRes {
        match self {
            Either::A(c) => c.filter(a),
            Either::B(c) => c.filter(a),
        }
    }
}

#[derive(Clone)]
pub struct Or<A, B> {
    a: A,
    b: B,
}
impl<A: Filter, B: Filter> Filter for Or<A, B> {
    fn filter(&self, a: &GridCoord) -> FilterRes {
        self.a.filter(a).or(self.b.filter(a))
    }
}
pub struct And<A, B> {
    a: A,
    b: B,
}
impl<A: Filter, B: Filter> Filter for And<A, B> {
    fn filter(&self, a: &GridCoord) -> FilterRes {
        self.a.filter(a).and(self.b.filter(a))
    }
}

pub fn contains_coord<I: Iterator<Item = GridCoord>>(mut it: I, b: GridCoord) -> bool {
    it.find(|a| *a == b).is_some()
}

pub mod movement_mesh {

    pub fn explore_outward_two() -> impl Iterator<Item = (HDir, [HDir; 3])> {
        HDir::all().map(move |dir| {
            let straight = dir;
            let left = dir.rotate60_left();
            let right = dir.rotate60_right();
            (dir, [straight, left, right])
        })
    }

    use crate::hex::HDir;
    // #[test]
    // fn test_path() {
    //     let k1 = GridCoord([1, -1]);
    //     let k2 = GridCoord([1, -2]);

    //     let mut mesh = RelativeMesh::new();
    //     mesh.add_normal_cell(k1);
    //     mesh.add_normal_cell(k2);

    //     let res: Vec<HDir> = mesh.path(GridCoord([1, -2]), &Mesh::new()).collect();
    //     dbg!(res);
    //     panic!();
    // }

    #[test]
    fn test_mesh() {
        //dbg!(generate_range(1).count());

        // let k = generate_range(2).collect::<Vec<_>>();
        // for a in k {
        //     println!("[{},{}],", a.0[0], a.0[1]);
        // }

        let k1 = GridCoord([2, 0]);
        let k2 = GridCoord([2, -2]);
        let k3 = GridCoord([-2, 1]);

        let mut mesh = SmallMesh::new();
        mesh.add(k1);
        mesh.add(k2);
        mesh.add(k3);

        assert!(mesh.is_set(k1));
        assert!(mesh.is_set(k2));
        assert!(mesh.is_set(k3));
        assert!(!mesh.is_set(GridCoord([-2, 2])));

        let res: Vec<_> = mesh.iter_mesh(GridCoord([0; 2])).collect();

        assert_eq!(
            res,
            vec!(GridCoord([-2, 1]), GridCoord([2, -2]), GridCoord([2, 0]))
        )
    }

    #[derive(PartialOrd, Ord, PartialEq, Eq, Debug, Clone)]
    pub struct SmallMesh {
        pub inner: u128,
    }

    impl SmallMesh {
        pub fn new() -> SmallMesh {
            SmallMesh { inner: 0 }
        }
        pub fn from_iter(it: impl Iterator<Item = GridCoord>) -> SmallMesh {
            let mut m = SmallMesh::new();
            for a in it {
                m.add(a);
            }
            m
        }
        pub fn validate_rel(a: GridCoord) {
            let x = a.0[0];
            let y = a.0[1];

            assert!(x <= 6 && x >= -6);
            assert!(y <= 6 && y >= -6);

            //assert!(x != 0 || y != 0);
        }
        pub fn add(&mut self, a: GridCoord) {
            Self::validate_rel(a);
            let ind = conv(a);
            self.inner = self.inner | (1 << ind);
        }
        pub fn remove(&mut self, a: GridCoord) {
            Self::validate_rel(a);
            let ind = conv(a);
            self.inner = self.inner & (!(1 << ind));
        }
        pub fn is_empty(&self) -> bool {
            self.inner == 0
        }
        pub fn is_set(&self, a: GridCoord) -> bool {
            Self::validate_rel(a);

            let ind = conv(a);

            self.inner & (1 << ind) != 0
        }
        pub fn iter_mesh(&self, point: GridCoord) -> impl Iterator<Item = GridCoord> {
            let inner = self.inner;

            //let skip_moves = self.swing_moves(point);

            // TABLE
            //     .iter()
            //     .enumerate()
            //     .filter(move |(x, _)| inner & (1 << x) != 0)
            //     .map(move |(_, x)| point.add(GridCoord(*x)))
            let mesh_moves = (0..128)
                .filter(move |x| inner & (1 << x) != 0)
                .map(move |a| {
                    let x = a / 13;
                    let y = a % 13;
                    point.add(GridCoord([x - 6, y - 6]))
                });

            mesh_moves //.chain(skip_moves)
        }
    }

    use super::GridCoord;

    // #[derive(PartialEq, Eq, Debug, Clone)]
    // pub struct RelativeMesh {
    //     inner: Mesh,
    // }

    // impl RelativeMesh {
    //     pub fn new() -> Self {
    //         RelativeMesh { inner: Mesh::new() }
    //     }

    //     pub fn validate_rel(a: GridCoord) {
    //         let x = a.0[0];
    //         let y = a.0[1];

    //         assert!(x <= 6 && x >= -6);
    //         assert!(y <= 6 && y >= -6);

    //         //assert!(x != 0 || y != 0);
    //     }

    //     pub fn add_normal_cell(&mut self, a: GridCoord) {
    //         self.inner.add(a);
    //     }
    //     pub fn remove_normal_cell(&mut self, a: GridCoord) {
    //         self.inner.remove(a);
    //     }
    //     fn is_set(&self, a: GridCoord) -> bool {
    //         self.inner.is_set(a)
    //     }

    //     pub fn iter_mesh(&self, point: GridCoord) -> impl Iterator<Item = GridCoord> {
    //         self.inner.iter_mesh(point)
    //     }
    //     pub fn is_empty(&self) -> bool {
    //         self.inner.inner == 0
    //     }
    // }
    fn conv(a: GridCoord) -> usize {
        let [x, y] = a.0;
        //     let ind=x/7+y%7;
        //     // -3 -2 -1 0 1 2 3
        //     // -6 -5 -4 -3 -2 -1 0 1 2 3 4 5 6
        // ind as usize
        ((x + 6) * 13 + (y + 6)) as usize

        // TABLE
        //     .iter()
        //     .enumerate()
        //     .find(|(_, x)| **x == a.0)
        //     .expect("Could not find the coord in table")
        //     .0
    }
}

pub fn path(
    mesh: &movement_mesh::SmallMesh,
    a: GridCoord,
    walls: &movement_mesh::SmallMesh,
) -> impl Iterator<Item = HDir> {
    let mesh_iter = {
        movement_mesh::SmallMesh::validate_rel(a);
        let x = a.0[0];
        let y = a.0[1];
        let first = if GridCoord([0, 0]).to_cube().dist(&a.to_cube()) == 1 {
            Some([GridCoord([0, 0]).dir_to(&a)])
        } else {
            None
        };

        //diagonal
        let second = if first.is_none() && (x.abs() == 1 || y.abs() == 1) {
            //TODO inefficient
            let mut k = GridCoord([0, 0])
                .to_cube()
                .neighbours()
                .filter(|x| x.dist(&a.to_cube()) == 1);
            let first = k.next().unwrap().to_axial();
            let second = k.next().unwrap().to_axial();

            if
            /*self.is_set(first)||*/
            !walls.is_set(first) {
                Some([GridCoord([0, 0]).dir_to(&first), first.dir_to(&a)])
            } else {
                Some([GridCoord([0, 0]).dir_to(&second), second.dir_to(&a)])
            }
        } else {
            None
        };

        let third = if first.is_none() && second.is_none() && (x.abs() == 2 || y.abs() == 2) {
            let h = GridCoord([0, 0]).dir_to(&a);
            Some([h, h])
        } else {
            None
        };

        // size 3 spokes
        let fourth = if first.is_none() && second.is_none() && (x.abs() == 3 || y.abs() == 3) {
            let h = GridCoord([0, 0]).dir_to(&a);
            Some([h, h, h])
        } else {
            None
        };

        let a = first.into_iter().flatten();
        let b = second.into_iter().flatten();
        let c = third.into_iter().flatten();
        let d = fourth.into_iter().flatten();
        a.chain(b).chain(c).chain(d)
    };

    mesh_iter.into_iter()
}

pub mod bitfield {
    use std::ops::{Deref, DerefMut};

    use super::GridCoord;

    #[test]
    fn bitfield() {
        let mut m = BitField::new();

        for k in -16..16 {
            dbg!("handling=k", k);
            m.set_coord(GridCoord([k, k]), true);

            assert!(m.is_coord_set(GridCoord([k, k])), "boo={}", k);
        }
    }

    use fixedbitset::FixedBitSet;

    #[derive(Clone, Debug, Hash, Eq, PartialEq)]
    pub struct BitField {
        pub inner: FixedBitSet,
    }

    impl Deref for BitField {
        type Target = FixedBitSet;
        fn deref(&self) -> &FixedBitSet {
            &self.inner
        }
    }

    impl DerefMut for BitField {
        fn deref_mut(&mut self) -> &mut FixedBitSet {
            &mut self.inner
        }
    }
    impl BitField {
        pub fn new() -> Self {
            BitField {
                inner: FixedBitSet::with_capacity(1024),
            }
        }
        pub fn from_iter(a: impl IntoIterator<Item = GridCoord>) -> Self {
            let mut k = BitField::new();
            for a in a {
                k.set_coord(a, true);
            }
            k
        }

        pub fn set_coord(&mut self, a: GridCoord, val: bool) {
            let x = a.0[0];
            let y = a.0[1];
            assert!(x <= 16 && x >= -16 && y <= 16 && y >= -16, "val={:?}", a);

            let ind = conv(a);
            self.inner.set(ind, val);
        }

        pub fn is_coord_set(&self, a: GridCoord) -> bool {
            let ind = conv(a);

            self.inner[ind]
        }
        pub fn iter_mesh(&self, point: GridCoord) -> impl Iterator<Item = GridCoord> + '_ {
            self.inner.ones().map(move |a| {
                let x = a / 32;
                let y = a % 32;
                point.add(GridCoord([x as i16 - 16, y as i16 - 16]))
            })
        }
    }
    fn conv(a: GridCoord) -> usize {
        let [x, y] = a.0;
        ((x + 16) * 32 + (y + 16)) as usize
    }
}
