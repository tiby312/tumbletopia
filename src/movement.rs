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

// //TODO a direction is only 6 values. Left over values when
// //put into 3 bits.
// #[derive(Copy, Clone)]
// pub struct Path {
//     //TODO optimize this to be just one 64bit integer?
//     //20 moves is just max possible moves
//     moves: [HexDir; 20],
//     num_moves: u8,
// }

// impl std::fmt::Debug for Path {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "Path:")?;
//         for a in self.moves.iter().take(self.num_moves as usize) {
//             write!(f, "{:?},", a)?;
//         }
//         writeln!(f)
//     }
// }
// impl Path {
//     pub fn new() -> Self {
//         Path {
//             moves: [HexDir { dir: 0 }; 20],
//             num_moves: 0,
//         }
//     }
//     pub fn into_moves(self) -> impl Iterator<Item = HexDir> {
//         self.moves.into_iter().take(self.num_moves as usize)
//     }

//     pub fn get_moves(&self) -> &[HexDir] {
//         &self.moves[0..self.num_moves as usize]
//     }
//     pub fn add(&mut self, a: HexDir) -> bool {
//         if self.num_moves >= 20 {
//             return false;
//         }

//         self.moves[self.num_moves as usize] = a;
//         self.num_moves += 1;
//         true
//     }

//     pub fn get_end_coord(&self, mut start: GridCoord) -> GridCoord {
//         for m in self.moves.iter().take(self.num_moves as usize) {
//             start = start.add(m.to_relative());
//         }
//         start
//     }

//     pub fn total_cost(&self) -> MoveUnit {
//         let mut total = 0;
//         for a in self.get_moves() {
//             total += self.move_cost(*a).0;
//         }
//         MoveUnit(total)
//     }
//     fn move_cost(&self, _: HexDir) -> MoveUnit {
//         MoveUnit(1)
//     }
// }

#[derive(Hash, Default, Debug, Copy, Clone, PartialEq, Eq)]
#[must_use]
pub struct GridCoord(pub [i16; 2]);
impl GridCoord {
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
    pub fn to_cube(self) -> hex::Cube {
        let a = self.0;
        hex::Cube([a[0], a[1], -a[0] - a[1]])
    }
    pub fn advance_by(self, m: HexDir, val: usize) -> GridCoord {
        (0..val).fold(self, |acc, o| acc.advance(m))
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
    // fn extend(self) -> ExtendFilter<Self>
    // where
    //     Self: Sized,
    // {
    //     ExtendFilter { filter: self }
    // }

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

// pub struct ExtendFilter<F> {
//     filter: F,
// }
// impl<A: Filter> Filter for ExtendFilter<A> {
//     fn filter(&self, a: &GridCoord) -> FilterRes {
//         match self.filter.filter(a) {
//             FilterRes::Accept => FilterRes::Accept,
//             FilterRes::Stop => FilterRes::DontAccept,
//         }
//     }
// }

pub fn contains_coord<I: Iterator<Item = GridCoord>>(mut it: I, b: GridCoord) -> bool {
    it.find(|a| *a == b).is_some()
}

///
///
///
///

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

        let mut mesh = MovementMesh::new(vec![]);
        mesh.add_normal_cell(k1);
        mesh.add_normal_cell(k2);

        let res: Vec<_> = mesh.path(GridCoord([1, -2])).collect();
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
        pub fn iter_cells(&self, point: GridCoord) -> impl Iterator<Item = (HexDir, GridCoord)> {
            let radius = 2;
            let num_cell = 8;
            let i = self.relative_anchor_point.to_cube();

            let i1 = if self.clockwise {
                Some(i.ring(radius))
            } else {
                None
            };
            let i2 = if !self.clockwise {
                Some(i.cc_ring(radius))
            } else {
                None
            };

            let i = i1.into_iter().flatten().chain(i2.into_iter().flatten());

            let i = i.map(|(d, a)| (d, a.to_axial()));
            let ii = i.clone();
            let i = i.chain(ii);

            let iiii = i.skip_while(|(_, z)| *z != GridCoord([0; 2]));

            iiii.take(num_cell).map(move |(d, z)| (d, point.add(z)))
        }
    }

    #[derive(PartialEq, Eq, Debug, Clone)]
    pub struct Mesh {
        inner: u128,
    }

    impl Mesh {
        pub fn new() -> Mesh {
            Mesh { inner: 0 }
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
        //A ring of size two not including the center cell has 1+6+12=19 cells.

        //We need an additional bit to describe the path that needs to be taken to each that spot.
        //Either left or right. (only applies for diagonal outer cells)
        inner: Mesh,

        //just_swing_inner: Mesh,
        swing_moves: Vec<SwingMoveRay>,
    }

    fn validate_rel(a: GridCoord) {
        let x = a.0[0];
        let y = a.0[1];

        assert!(x <= 6 && x >= -6);
        assert!(y <= 6 && y >= -6);

        assert!(x != 0 || y != 0);
    }
    impl MovementMesh {
        pub fn new(swing_moves: Vec<SwingMoveRay>) -> Self {
            MovementMesh {
                inner: Mesh::new(),
                //just_swing_inner: Mesh::new(),
                swing_moves,
            }
        }
        pub fn add_swing_move(&mut self, a: SwingMoveRay) {
            self.swing_moves.push(a);
        }
        //TODO
        pub fn path(&self, a: GridCoord) -> impl Iterator<Item = HexDir> {
            //let swings=self.swing_moves(GridCoord([0;2])).take_while(|(a,b)|b!=a).collect();
            let mut swing_iter = None;

            //TODO look at swing mesh instead??
            for b in self.swing_moves.iter() {
                if let Some((i, _)) = b
                    .iter_cells(GridCoord([0; 2]))
                    .enumerate()
                    .find(|(_, (_, b))| *b == a)
                {
                    swing_iter = Some(b.iter_cells(GridCoord([0; 2])).take(i).map(|a| a.0));
                }
            }

            let mesh_iter = if swing_iter.is_none() {
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
                    if self.is_set(first) {
                        Some([GridCoord([0, 0]).dir_to(&first), first.dir_to(&a)])
                    } else {
                        //TODO this is not true teamates jumping over each other.
                        //assert!(self.is_set(second));
                        Some([GridCoord([0, 0]).dir_to(&second), second.dir_to(&a)])
                    }
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
                Some(a.chain(b).chain(c).chain(d))
            } else {
                None
            };

            swing_iter
                .into_iter()
                .flatten()
                .chain(mesh_iter.into_iter().flatten())
        }
        // pub fn add_swing_cell(&mut self, a: GridCoord) {
        //     self.just_swing_inner.add(a);
        // }
        pub fn add_normal_cell(&mut self, a: GridCoord) {
            self.inner.add(a);
        }

        fn is_set(&self, a: GridCoord) -> bool {
            self.inner.is_set(a)
        }

        // fn swing_moves(&self, point: GridCoord) -> impl Iterator<Item = GridCoord> {
        //     let kk = self.swing_moves.clone();
        //     kk.into_iter()
        //         .flat_map(move |a| a.iter_cells(point).map(|a| a.1))
        // }

        pub fn iter_swing_mesh(
            &self,
            point: GridCoord,
        ) -> impl Iterator<Item = (HexDir, GridCoord)> {
            self.swing_moves
                .clone()
                .into_iter()
                .flat_map(move |a| a.iter_cells(point).skip(1))

            //.filter(move |a| a.1 != point)
            //self.just_swing_inner.iter_mesh(point)
        }

        pub fn iter_mesh(&self, point: GridCoord) -> impl Iterator<Item = GridCoord> {
            self.inner
                .iter_mesh(point)
                .chain(self.iter_swing_mesh(point).map(|a| a.1))
            // let mut j = self.inner.clone();
            // //j.inner |= self.just_swing_inner.inner;
            // j.iter_mesh(point)
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
#[derive(Debug, Clone)]
pub struct MoveCand<P> {
    pub target: GridCoord,
    pub path: P,
}

// #[derive(Debug, Clone)]
// pub struct PossibleMoves2<P> {
//     //pub orig: GridCoord,
//     pub moves: Vec<MoveCand<P>>,
// }

// pub trait PathHave {
//     type Foo;
//     fn path(&self, a: Path) -> Self::Foo;
// }
// pub struct WithPath;
// pub struct NoPath;

// impl PathHave for NoPath {
//     type Foo = ();
//     fn path(&self, _: Path) -> () {
//         ()
//     }
// }
// impl PathHave for WithPath {
//     type Foo = Path;
//     fn path(&self, a: Path) -> Path {
//         a
//     }
// }

pub enum ComputeMovesRes {
    Add,
    AddAndStop,
    Stop,
    NoAddContinue,
}

// pub fn compute_moves22<F: FnMut(&GridCoord) -> ComputeMovesRes>(
//     coord: GridCoord,
//     restricted_movement: bool,
//     mut func: F,
// ) -> MovementMesh {
//     fn handle<F: FnMut(&GridCoord) -> ComputeMovesRes>(
//         m: &mut MovementMesh,
//         base: GridCoord,
//         coord: GridCoord,
//         dir: HexDir,
//         mut func: F,
//     ) -> bool {
//         let first = coord.advance(dir);

//         match func(&first) {
//             ComputeMovesRes::Add => {
//                 m.add(first.sub(&base));
//                 true
//             }
//             ComputeMovesRes::AddAndStop => {
//                 m.add(first.sub(&base));
//                 false
//             }
//             ComputeMovesRes::Stop => false,
//             ComputeMovesRes::NoAddContinue => true,
//         }
//     }

//     let mut m = MovementMesh::new(vec![]);

//     for (a, rest) in self::movement_mesh::explore_outward_two() {
//         if handle(&mut m, coord, coord, a, &mut func) {
//             if !restricted_movement {
//                 let first = coord.advance(a);
//                 for a in rest {
//                     let _ = handle(&mut m, coord, first, a, &mut func);
//                 }
//             }
//         }
//     }

//     m
// }
// pub fn compute_moves2<F: Filter, F2: Filter>(
//     coord: GridCoord,
//     filter: &F,
//     skip_filter: &F2,
//     restricted_movement: bool,
//     slide_rule: bool,
// ) -> MovementMesh {
//     let mut m = MovementMesh::new(vec![]);

//     //TODO make this a closure
//     fn handle<F: Filter, F2: Filter>(
//         m: &mut MovementMesh,
//         base: GridCoord,
//         coord: GridCoord,
//         dir: HexDir,
//         filter: &F,
//         skip_filter: &F2,
//         slide_rule: bool,
//     ) -> bool {
//         let first = coord.advance(dir);
//         //TODO first check if this cell is already set

//         if let FilterRes::Stop = filter.filter(&first) {
//             if let FilterRes::Accept = skip_filter.filter(&first) {
//                 m.add(first.sub(&base));
//             }
//             return false;
//         }

//         m.add(first.sub(&base));

//         // if slide_rule {
//         //     let ttt1_skip = match skip_filter.filter(&coord.advance(dir.rotate60_right())) {
//         //         FilterRes::Stop => false,
//         //         FilterRes::Accept => true,
//         //     };

//         //     let ttt2_skip = match skip_filter.filter(&coord.advance(dir.rotate60_left())) {
//         //         FilterRes::Stop => false,
//         //         FilterRes::Accept => true,
//         //     };

//         //     //let skip_foo=ttt1_skip | ttt2_skip;

//         //     let ttt1 = match filter.filter(&coord.advance(dir.rotate60_right())) {
//         //         FilterRes::Stop => false,
//         //         FilterRes::Accept => true,
//         //     };

//         //     let ttt2 = match filter.filter(&coord.advance(dir.rotate60_left())) {
//         //         FilterRes::Stop => false,
//         //         FilterRes::Accept => true,
//         //     };

//         //     if !ttt1 && !ttt2 && !ttt1_skip && !ttt2_skip {
//         //         return false;
//         //     }
//         // }

//         // if let FilterRes::Stop = filter.filter(&first) {
//         //     return false;
//         // }

//         //if let FilterRes::Accept = skip_filter.filter(&first) {
//         m.add(first.sub(&base));
//         //}

//         return true;
//     }

//     for (a, rest) in self::movement_mesh::explore_outward_two() {
//         if handle(&mut m, coord, coord, a, filter, skip_filter, slide_rule) {
//             if !restricted_movement {
//                 let first = coord.advance(a);
//                 for a in rest {
//                     let _ = handle(&mut m, coord, first, a, filter, skip_filter, slide_rule);
//                 }
//             }
//         }
//     }

//     m
// }

// pub fn compute_moves<K: MoveStrategy, F: Filter, F2: Filter, M: MoveCost, PH: PathHave>(
//     movement: &K,
//     filter: &F,
//     skip_filter: &F2,
//     mo: &M,
//     coord: GridCoord,
//     remaining_moves: MoveUnit,
//     slide_rule: bool,
//     ph: PH,
// ) -> Vec<MoveCand<PH::Foo>> {
//     let m = PossibleMoves::new(
//         movement,
//         filter,
//         skip_filter,
//         mo,
//         coord,
//         remaining_moves,
//         slide_rule,
//     );

//     let moves = m
//         .moves
//         .into_iter()
//         .map(|(target, path, _)| MoveCand {
//             target,
//             path: ph.path(path),
//         })
//         .collect();
//     moves
// }

// impl PossibleMoves {
//     fn new<K: MoveStrategy, F: Filter, F2: Filter, M: MoveCost>(
//         movement: &K,
//         filter: &F,
//         skip_filter: &F2,
//         mo: &M,
//         coord: GridCoord,
//         remaining_moves: MoveUnit,
//         slide_rule: bool,
//     ) -> Self {
//         let remaining_moves = MoveUnit(remaining_moves.0);
//         let mut p = PossibleMoves {
//             moves: vec![],
//             start: coord,
//         };
//         p.explore_path(
//             movement,
//             filter,
//             skip_filter,
//             mo,
//             Path::new(),
//             remaining_moves,
//             slide_rule,
//         );
//         p
//     }

//     // pub fn get_path_data(&self, g: &GridCoord) -> Option<(&Path, &MoveUnit)> {
//     //     self.moves.iter().find(|a| &a.0 == g).map(|a| (&a.1, &a.2))
//     // }

//     // pub fn start(&self) -> &GridCoord {
//     //     &self.start
//     // }

//     // pub fn iter_coords(&self) -> impl Iterator<Item = &GridCoord> {
//     //     self.moves.iter().map(|a| &a.0)
//     // }

//     fn explore_path<K: MoveStrategy, F: Filter, F2: Filter, M: MoveCost>(
//         &mut self,
//         movement: &K,
//         continue_filter: &F,
//         skip_filter: &F2,
//         mo: &M,
//         current_path: Path,
//         remaining_moves: MoveUnit,
//         slide_rule: bool,
//     ) {
//         // if remaining_moves.0 == 0 {
//         //      return;
//         // }

//         // 2-OG
//         // warrior has 2 move points
//         // warrior moves to grass and expends its 2 move points
//         // warrior cant move anymore

//         // 2-ORG
//         // warrior has 2 move points
//         // warrior moves to road on grass and expends 1 move point (2-1)
//         // warrior has 1 move point.
//         // warrior moves to grass and expends 2 move points.
//         // warrior has -1 move points. can't move anymore.

//         // 2-ORRG
//         // warrior has 2 move points
//         // warrior moves to road on grass and expends 1 move point (2-1)
//         // warrior has 1 move point
//         // warrior moves to road on grass and expends 1 move point?????
//         // warrior has 0 move points. cant move anymore.

//         let curr_pos = current_path.get_end_coord(self.start);

//         //log!(format!("rem:{:?}",remaining_moves.0));
//         for a in K::adjacent() {
//             let target_pos = curr_pos.advance(a);

//             if slide_rule {
//                 let aaa = a.to_relative().to_cube().rotate_60_left();
//                 let bbb = a.to_relative().to_cube().rotate_60_right();

//                 let ttt1 = match continue_filter.filter(&target_pos.add(aaa.to_axial())) {
//                     FilterRes::Stop => false,
//                     FilterRes::Accept => true,
//                 };

//                 let ttt2 = match continue_filter.filter(&target_pos.add(bbb.to_axial())) {
//                     FilterRes::Stop => false,
//                     FilterRes::Accept => true,
//                 };

//                 if !ttt1 && !ttt2 {
//                     continue;
//                 }
//             }

//             match continue_filter.filter(&target_pos) {
//                 FilterRes::Stop => continue,
//                 FilterRes::Accept => {}
//             }

//             let skip = match skip_filter.filter(&target_pos) {
//                 FilterRes::Stop => true,
//                 FilterRes::Accept => false,
//             };

//             //We must have remaining moves to satisfy ALL move cost.
//             // if remaining_moves.0<current_path.move_cost(a).0{
//             //     continue;
//             // }

//             let move_cost = current_path.move_cost(a);
//             // if move_cost.0>remaining_moves.0{
//             //     move_cost.0=remaining_moves.0;
//             // }
//             //TODO road should HALF the cost?
//             let cost = mo.foop(target_pos, move_cost);

//             //todo!("Need to allow cardinal movement at 1 point. Not working???");

//             //as long as we have SOME remainv moves, we can go to this square even
//             //if it is really expensive.
//             // if !(remaining_moves.0 > 0) {
//             //     continue;
//             // }
//             //Allow 1 point remainder!!!!
//             // if remaining_moves.0 +2 <= 2 {
//             //     continue;
//             // }

//             if !(remaining_moves.0 >= cost.0) {
//                 //-1
//                 continue;
//             }

//             //subtract move cost
//             let rr = remaining_moves.sub(cost);

//             if !skip {
//                 if !self.consider(&current_path, a, rr) {
//                     continue;
//                 }
//             }

//             //if !stop {
//             self.explore_path(
//                 movement,
//                 continue_filter,
//                 skip_filter,
//                 mo,
//                 current_path.add(a).unwrap(),
//                 rr,
//                 slide_rule,
//             )
//             //}
//         }
//     }

//     fn consider(&mut self, path: &Path, m: HexDir, cost: MoveUnit) -> bool {
//         //if this move unit is greater than what we already have, replace it.
//         //we found a quicker way to get to the same square.

//         //if it is not quicker, imediately stop everything.
//         let new_path = path.add(m).unwrap();
//         let coord = new_path.get_end_coord(self.start);

//         //we found a match now lets compare
//         let index =
//             if let Some((index, _)) = self.moves.iter().enumerate().find(|(_, a)| a.0 == coord) {
//                 index
//             } else {
//                 self.moves.push((coord, new_path, cost));
//                 return true;
//             };

//         if cost.0 > self.moves[index].2 .0 {
//             let og = &mut self.moves[index];
//             let new = &mut (coord, new_path, cost);
//             core::mem::swap(og, new);
//             // self.moves.push();
//             // self.moves.swap_remove(index);
//             return true;
//         }

//         return false;
//     }
// }

// //normal terrain is 2.
// //road is 1.
// fn terrain_cost(a: GridCoord) -> MoveUnit {
//     MoveUnit(2)
// }
