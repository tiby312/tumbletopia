pub use self::movement_mesh::MovementMesh;

use super::*;

pub trait MoveStrategy {
    type It: IntoIterator<Item = HexDir>;
    fn adjacent() -> Self::It;
}
pub struct WarriorMovement;
impl MoveStrategy for WarriorMovement {
    type It = std::array::IntoIter<HexDir, 6>;
    fn adjacent() -> Self::It {
        [0, 1, 2, 3, 4, 5].map(|dir| HexDir { dir }).into_iter()
    }
}

#[derive(Copy, Hash, Clone, Debug, Eq, PartialEq, Default)]
pub struct HexDir {
    pub dir: u8,
}

impl HexDir {
    pub fn all() -> impl Iterator<Item = HexDir> {
        (0..6).map(|dir| HexDir { dir })
    }
    pub const fn rotate60_right(&self) -> HexDir {
        // 0->4
        // 1->5
        // 2->0
        // 3->1
        // 4->2
        // 5->3

        HexDir {
            dir: (self.dir + 1) % 6,
        }
    }

    pub const fn rotate60_left(&self) -> HexDir {
        // 0->2
        // 1->3
        // 2->4
        // 3->5
        // 4->0
        // 5->1

        HexDir {
            dir: (self.dir + 5) % 6,
        }
    }

    pub const fn to_relative(&self) -> GridCoord {
        hex::Cube(hex::OFFSETS[self.dir as usize]).to_axial()
    }
}

#[derive(Hash, Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
pub struct GridCoord(pub [i16; 2]);
impl GridCoord {
    pub fn zero()->GridCoord{
        GridCoord([0;2])
    }
    pub fn dir_to(&self, other: &GridCoord) -> HexDir {
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
            .map(|(i, _)| HexDir { dir: i as u8 })
            .unwrap()
    }
    pub fn dir_to2(&self, other: &GridCoord) -> HexDir {
        let mut offset = other.sub(self);

        offset.0[0] = offset.0[0].clamp(-1, 1);
        offset.0[1] = offset.0[1].clamp(-1, 1);

        // assert!(offset.0[0].abs() <= 1);
        // assert!(offset.0[1].abs() <= 1);
        let offset = offset.to_cube();

        hex::OFFSETS
            .iter()
            .rev()
            .enumerate()
            .find(|(_, x)| **x == offset.0)
            .map(|(i, _)| HexDir { dir: i as u8 })
            .unwrap()
            .rotate60_right()
    }
    pub fn to_cube(self) -> hex::Cube {
        let a = self.0;
        hex::Cube([a[0], a[1], -a[0] - a[1]])
    }
    pub fn advance_by(self, m: HexDir, val: usize) -> GridCoord {
        (0..val).fold(self, |acc, _| acc.advance(m))
    }
    pub const fn advance(self, m: HexDir) -> GridCoord {
        self.add(m.to_relative())
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

    pub fn explore_outward_two() -> impl Iterator<Item = (HexDir, [HexDir; 3])> {
        HexDir::all().map(move |dir| {
            let straight = dir;
            let left = dir.rotate60_left();
            let right = dir.rotate60_right();
            (dir, [straight, left, right])
        })
    }

    use crate::movement::HexDir;
    #[test]
    fn test_path() {
        let k1 = GridCoord([1, -1]);
        let k2 = GridCoord([1, -2]);

        let mut mesh = MovementMesh::new();
        mesh.add_normal_cell(k1);
        mesh.add_normal_cell(k2);

        let res: Vec<_> = mesh.path(GridCoord([1, -2]), &Mesh::new()).collect();
        dbg!(res);
        panic!();
    }

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

        let mut mesh = Mesh::new();
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

    // fn generate_range(n: i16) -> impl Iterator<Item = GridCoord> {
    //     (-n..n + 1).flat_map(move |q| {
    //         ((-n).max(-q - n)..n.min(-q + n) + 1).map(move |r| GridCoord([q, r]))
    //     })
    // }

    // const TABLE: [[i16; 2]; 19] = [
    //     [-2, 0],
    //     [-2, 1],
    //     [-2, 2],
    //     [-1, -1],
    //     [-1, 0],
    //     [-1, 1],
    //     [-1, 2],
    //     [0, -2],
    //     [0, -1],
    //     [0, 0],
    //     [0, 1],
    //     [0, 2],
    //     [1, -2],
    //     [1, -1],
    //     [1, 0],
    //     [1, 1],
    //     [2, -2],
    //     [2, -1],
    //     [2, 0],
    // ];

    #[derive(PartialEq, Eq, Debug, Clone)]
    pub struct SwingMoveRay {
        pub swing: SwingMove,
        pub num_steps: usize,
    }

    impl SwingMoveRay {
        pub fn iter_cells(&self, point: GridCoord) -> impl Iterator<Item = (HexDir, GridCoord)> {
            self.swing.iter_cells(point).take(self.num_steps)
        }
    }

    #[derive(PartialEq, Eq, Debug, Clone)]
    pub struct SwingMove {
        pub relative_anchor_point: GridCoord,
        pub radius: i16,
        pub clockwise: bool,
    }
    impl SwingMove {
        pub fn iter_left(&self, point: GridCoord) -> impl Iterator<Item = (HexDir, GridCoord)> {
            // let f=match self.radius{
            //     0=>0,
            //     1=>3,
            //     2=>6,
            //     3=>9,
            //     4=>12,
            //     5=>12,
            //     _=>12
            // };
            let f = 3 * self.radius as usize;
            //radius 1-> 3 (or 4)
            //radius 2-> 6 (or 7 including spot)
            //radius 3-> 9 (or 10)
            //radius 4-> 11 (or 12)
            //radius 5-> 12 (or 13)

            self.iter_cells_inner(point, f, true)
        }
        pub fn iter_right(&self, point: GridCoord) -> impl Iterator<Item = (HexDir, GridCoord)> {
            // let f=match self.radius{
            //     0=>0,
            //     1=>3,
            //     2=>6,
            //     3=>9,
            //     4=>12,
            //     5=>12,
            //     _=>12
            // };
            let f = 3 * self.radius as usize;
            self.iter_cells_inner(point, f, false)
        }

        pub fn iter_cells(&self, point: GridCoord) -> impl Iterator<Item = (HexDir, GridCoord)> {
            self.iter_cells_inner(point, 13, self.clockwise)
        }
        pub fn iter_cells_inner(
            &self,
            point: GridCoord,
            num_cell: usize,
            clockwise: bool,
        ) -> impl Iterator<Item = (HexDir, GridCoord)> {
            let radius = self.radius;
            //let radius = 2;
            //let num_cell = 8;
            //let num_cell = 13;

            // let radius = 3;
            // let num_cell = 32;

            let i = self.relative_anchor_point.to_cube();

            let i1 = if clockwise {
                Some(i.ring(radius))
            } else {
                None
            };
            let i2 = if !clockwise {
                Some(i.cc_ring(radius))
            } else {
                None
            };

            let i = i1.into_iter().flatten().chain(i2.into_iter().flatten());

            let i = i.map(|(d, a)| (d, a.to_axial()));
            let ii = i.clone();
            let i = i.chain(ii);

            let iiii = i.skip_while(|(_, z)| *z != GridCoord([0; 2]));

            iiii.take(num_cell + 2).map(move |(d, z)| (d, point.add(z)))
        }
    }

    #[derive(PartialEq, Eq, Debug, Clone)]
    pub struct Mesh {
        pub inner: u128,
    }

    impl Mesh {
        pub fn new() -> Mesh {
            Mesh { inner: 0 }
        }
        pub fn from_iter(it: impl Iterator<Item = GridCoord>) -> Mesh {
            let mut m = Mesh::new();
            for a in it {
                m.add(a);
            }
            m
        }
        fn validate_rel(a: GridCoord) {
            let x = a.0[0];
            let y = a.0[1];

            assert!(x <= 6 && x >= -6);
            assert!(y <= 6 && y >= -6);

            assert!(x != 0 || y != 0);
        }
        pub fn add(&mut self, a: GridCoord) {
            validate_rel(a);
            let ind = conv(a);
            self.inner = self.inner | (1 << ind);
        }

        pub fn is_set(&self, a: GridCoord) -> bool {
            validate_rel(a);

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

    #[derive(PartialEq, Eq, Debug, Clone)]
    pub struct MovementMesh {
        inner: Mesh,
    }

    fn validate_rel(a: GridCoord) {
        let x = a.0[0];
        let y = a.0[1];

        assert!(x <= 6 && x >= -6);
        assert!(y <= 6 && y >= -6);

        assert!(x != 0 || y != 0);
    }
    impl MovementMesh {
        pub fn new() -> Self {
            MovementMesh { inner: Mesh::new() }
        }

        pub fn path(&self, a: GridCoord, walls: &Mesh) -> impl Iterator<Item = HexDir> {
            let mesh_iter = {
                validate_rel(a);
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

                    // else if !walls.is_set(first){
                    //     panic!("its hapening");
                    //     Some([GridCoord([0, 0]).dir_to(&first), first.dir_to(&a)])
                    // }else{
                    //     Some([GridCoord([0, 0]).dir_to(&second), second.dir_to(&a)])
                    // }
                    // if  || !walls.is_set(first) {

                    // } else {
                    //     //TODO this is not true teamates jumping over each other.
                    //     //assert!(self.is_set(second));
                    // }
                } else {
                    None
                };

                let third = if first.is_none() && second.is_none() && (x.abs() == 2 || y.abs() == 2)
                {
                    let h = GridCoord([0, 0]).dir_to(&a);
                    Some([h, h])
                } else {
                    None
                };

                // size 3 spokes
                let fourth =
                    if first.is_none() && second.is_none() && (x.abs() == 3 || y.abs() == 3) {
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

        pub fn add_normal_cell(&mut self, a: GridCoord) {
            self.inner.add(a);
        }

        fn is_set(&self, a: GridCoord) -> bool {
            self.inner.is_set(a)
        }

        pub fn iter_mesh(&self, point: GridCoord) -> impl Iterator<Item = GridCoord> {
            self.inner.iter_mesh(point)
        }
        pub fn is_empty(&self) -> bool {
            self.inner.inner == 0
        }
    }
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
    // fn conv_inv(ind: usize) -> GridCoord {
    //     GridCoord(TABLE[ind])
    // }
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
