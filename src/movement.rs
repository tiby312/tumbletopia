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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct HexDir {
    dir: u8,
}

impl HexDir {
    pub fn to_relative(&self) -> GridCoord {
        hex::Cube(hex::OFFSETS[self.dir as usize]).to_axial()
    }
}

//TODO a direction is only 6 values. Left over values when
//put into 3 bits.
#[derive(Copy, Clone)]
pub struct Path {
    //TODO optimize this to be just one 64bit integer?
    //20 moves is just max possible moves
    moves: [HexDir; 20],
    num_moves: u8,
}

impl std::fmt::Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Path:")?;
        for a in self.moves.iter().take(self.num_moves as usize) {
            write!(f, "{:?},", a)?;
        }
        writeln!(f)
    }
}
impl Path {
    pub fn new() -> Self {
        Path {
            moves: [HexDir { dir: 0 }; 20],
            num_moves: 0,
        }
    }
    pub fn into_moves(self) -> impl Iterator<Item = HexDir> {
        self.moves.into_iter().take(self.num_moves as usize)
    }

    pub fn get_moves(&self) -> &[HexDir] {
        &self.moves[0..self.num_moves as usize]
    }
    pub fn add(mut self, a: HexDir) -> Option<Self> {
        if self.num_moves >= 20 {
            return None;
        }

        self.moves[self.num_moves as usize] = a;
        self.num_moves += 1;
        Some(self)
    }

    pub fn get_end_coord(&self, mut start: GridCoord) -> GridCoord {
        for m in self.moves.iter().take(self.num_moves as usize) {
            start = start.add(m.to_relative());
        }
        start
    }

    pub fn total_cost(&self) -> MoveUnit {
        let mut total = 0;
        for a in self.get_moves() {
            total += self.move_cost(*a).0;
        }
        MoveUnit(total)
    }
    fn move_cost(&self, _: HexDir) -> MoveUnit {
        MoveUnit(1)
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
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
    fn advance(self, m: HexDir) -> GridCoord {
        self.add(m.to_relative())
    }
    fn sub(mut self, o: &GridCoord) -> Self {
        self.0[0] -= o.0[0];
        self.0[1] -= o.0[1];
        self
    }
    fn add(mut self, o: GridCoord) -> Self {
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

pub fn contains_coord<'a, I: Iterator<Item = &'a GridCoord>>(mut it: I, b: &GridCoord) -> bool {
    it.find(|a| *a == b).is_some()
}

//Represents all the legal moves for a specific piece.
#[derive(Debug, Clone)]
struct PossibleMoves {
    //Has the end coord,path from current, and the remainder cost to get there.
    //cells that are the furthest away will have a move unit of zero.
    //TODO start with the remainder when determining attack squares
    moves: Vec<(GridCoord, Path, MoveUnit)>,
    start: GridCoord,
}

///
///
///
///

pub mod movement_mesh {

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

        let mut mesh = MovementMesh::new();
        mesh.add(k1);
        mesh.add(k2);
        mesh.add(k3);

        assert!(mesh.is_set(k1));
        assert!(mesh.is_set(k2));
        assert!(mesh.is_set(k3));
        assert!(!mesh.is_set(GridCoord([-2, 2])));

        let res: Vec<_> = mesh.iter_mesh().collect();

        assert_eq!(
            res,
            vec!(GridCoord([-2, 1]), GridCoord([2, -2]), GridCoord([2, 0]))
        )
    }

    fn generate_range(n: i16) -> impl Iterator<Item = GridCoord> {
        (-n..n + 1).flat_map(move |q| {
            ((-n).max(-q - n)..n.min(-q + n) + 1).map(move |r| GridCoord([q, r]))
        })
    }

    const TABLE: [[i16; 2]; 19] = [
        [-2, 0],
        [-2, 1],
        [-2, 2],
        [-1, -1],
        [-1, 0],
        [-1, 1],
        [-1, 2],
        [0, -2],
        [0, -1],
        [0, 0],
        [0, 1],
        [0, 2],
        [1, -2],
        [1, -1],
        [1, 0],
        [1, 1],
        [2, -2],
        [2, -1],
        [2, 0],
    ];

    use std::ops::Index;

    use super::GridCoord;

    #[derive(Debug, Copy, Clone)]
    pub struct MovementMesh {
        //A ring of size two not including the center cell has 1+6+12=19 cells.

        //We need an additional bit to describe the path that needs to be taken to each that spot.
        //Either left or right. (only applies for diagonal outer cells)
        inner: i32,
    }

    fn validate_rel(a: GridCoord) {
        let x = a.0[0];
        let y = a.0[1];

        assert!(x <= 2 && x >= -2);
        assert!(y <= 2 && y >= -2);
        assert!(x != 0 || y != 0);
    }
    impl MovementMesh {
        pub fn new() -> Self {
            MovementMesh { inner: 0 }
        }
        //TODO
        // pub fn path(&self,a:GridCoord)->impl Iterator<Item=HexDir>{
        //     validate_rel(a);
        //     let x=a.0[0];
        //     let y=a.0[1];
        //     let first=if x>=-1 && x<=1  && y>-1 && y<=1{
        //         Some(GridCoord([0,0]).dir_to(&a))
        //     }else{
        //         None
        //     };

        //     if first.is_none(){
        //         //diagonal
        //         if x.abs()==1 || y.abs()==1{

        //         }

        //         else{
        //             let h=GridCoord([0,0]).dir_to(other);
        //             [h,h]
        //         }

        //     }
        // }
        pub fn add(&mut self, a: GridCoord) {
            validate_rel(a);
            let ind = conv(a);
            dbg!(ind);
            self.inner = self.inner | (1 << ind);
        }

        pub fn is_set(&self, a: GridCoord) -> bool {
            validate_rel(a);

            let ind = conv(a);
            self.inner & (1 << ind) != 0
        }

        pub fn iter_mesh(&self) -> impl Iterator<Item = GridCoord> {
            let inner = self.inner;
            TABLE
                .iter()
                .enumerate()
                .filter(move |(x, _)| inner & (1 << x) != 0)
                .map(|(_, x)| GridCoord(*x))
        }
    }
    fn conv(a: GridCoord) -> usize {
        dbg!(a);
        TABLE
            .iter()
            .enumerate()
            .find(|(_, x)| **x == a.0)
            .expect("Could not find the coord in table")
            .0
    }
    fn conv_inv(ind: usize) -> GridCoord {
        GridCoord(TABLE[ind])
    }
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

pub trait PathHave {
    type Foo;
    fn path(&self, a: Path) -> Self::Foo;
}
pub struct WithPath;
pub struct NoPath;

impl PathHave for NoPath {
    type Foo = ();
    fn path(&self, _: Path) -> () {
        ()
    }
}
impl PathHave for WithPath {
    type Foo = Path;
    fn path(&self, a: Path) -> Path {
        a
    }
}

pub fn compute_moves<K: MoveStrategy, F: Filter, F2: Filter, M: MoveCost, PH: PathHave>(
    movement: &K,
    filter: &F,
    skip_filter: &F2,
    mo: &M,
    coord: GridCoord,
    remaining_moves: MoveUnit,
    slide_rule: bool,
    ph: PH,
) -> Vec<MoveCand<PH::Foo>> {
    let m = PossibleMoves::new(
        movement,
        filter,
        skip_filter,
        mo,
        coord,
        remaining_moves,
        slide_rule,
    );

    let moves = m
        .moves
        .into_iter()
        .map(|(target, path, _)| MoveCand {
            target,
            path: ph.path(path),
        })
        .collect();
    moves
}

impl PossibleMoves {
    fn new<K: MoveStrategy, F: Filter, F2: Filter, M: MoveCost>(
        movement: &K,
        filter: &F,
        skip_filter: &F2,
        mo: &M,
        coord: GridCoord,
        remaining_moves: MoveUnit,
        slide_rule: bool,
    ) -> Self {
        let remaining_moves = MoveUnit(remaining_moves.0);
        let mut p = PossibleMoves {
            moves: vec![],
            start: coord,
        };
        p.explore_path(
            movement,
            filter,
            skip_filter,
            mo,
            Path::new(),
            remaining_moves,
            slide_rule,
        );
        p
    }

    // pub fn get_path_data(&self, g: &GridCoord) -> Option<(&Path, &MoveUnit)> {
    //     self.moves.iter().find(|a| &a.0 == g).map(|a| (&a.1, &a.2))
    // }

    // pub fn start(&self) -> &GridCoord {
    //     &self.start
    // }

    // pub fn iter_coords(&self) -> impl Iterator<Item = &GridCoord> {
    //     self.moves.iter().map(|a| &a.0)
    // }

    fn explore_path<K: MoveStrategy, F: Filter, F2: Filter, M: MoveCost>(
        &mut self,
        movement: &K,
        continue_filter: &F,
        skip_filter: &F2,
        mo: &M,
        current_path: Path,
        remaining_moves: MoveUnit,
        slide_rule: bool,
    ) {
        // if remaining_moves.0 == 0 {
        //      return;
        // }

        // 2-OG
        // warrior has 2 move points
        // warrior moves to grass and expends its 2 move points
        // warrior cant move anymore

        // 2-ORG
        // warrior has 2 move points
        // warrior moves to road on grass and expends 1 move point (2-1)
        // warrior has 1 move point.
        // warrior moves to grass and expends 2 move points.
        // warrior has -1 move points. can't move anymore.

        // 2-ORRG
        // warrior has 2 move points
        // warrior moves to road on grass and expends 1 move point (2-1)
        // warrior has 1 move point
        // warrior moves to road on grass and expends 1 move point?????
        // warrior has 0 move points. cant move anymore.

        let curr_pos = current_path.get_end_coord(self.start);

        //log!(format!("rem:{:?}",remaining_moves.0));
        for a in K::adjacent() {
            let target_pos = curr_pos.advance(a);

            if slide_rule {
                let aaa = a.to_relative().to_cube().rotate_60_left();
                let bbb = a.to_relative().to_cube().rotate_60_right();

                let ttt1 = match continue_filter.filter(&target_pos.add(aaa.to_axial())) {
                    FilterRes::Stop => false,
                    FilterRes::Accept => true,
                };

                let ttt2 = match continue_filter.filter(&target_pos.add(bbb.to_axial())) {
                    FilterRes::Stop => false,
                    FilterRes::Accept => true,
                };

                if !ttt1 && !ttt2 {
                    continue;
                }
            }

            match continue_filter.filter(&target_pos) {
                FilterRes::Stop => continue,
                FilterRes::Accept => {}
            }

            let skip = match skip_filter.filter(&target_pos) {
                FilterRes::Stop => true,
                FilterRes::Accept => false,
            };

            //We must have remaining moves to satisfy ALL move cost.
            // if remaining_moves.0<current_path.move_cost(a).0{
            //     continue;
            // }

            let move_cost = current_path.move_cost(a);
            // if move_cost.0>remaining_moves.0{
            //     move_cost.0=remaining_moves.0;
            // }
            //TODO road should HALF the cost?
            let cost = mo.foop(target_pos, move_cost);

            //todo!("Need to allow cardinal movement at 1 point. Not working???");

            //as long as we have SOME remainv moves, we can go to this square even
            //if it is really expensive.
            // if !(remaining_moves.0 > 0) {
            //     continue;
            // }
            //Allow 1 point remainder!!!!
            // if remaining_moves.0 +2 <= 2 {
            //     continue;
            // }

            if !(remaining_moves.0 >= cost.0) {
                //-1
                continue;
            }

            //subtract move cost
            let rr = remaining_moves.sub(cost);

            if !skip {
                if !self.consider(&current_path, a, rr) {
                    continue;
                }
            }

            //if !stop {
            self.explore_path(
                movement,
                continue_filter,
                skip_filter,
                mo,
                current_path.add(a).unwrap(),
                rr,
                slide_rule,
            )
            //}
        }
    }

    fn consider(&mut self, path: &Path, m: HexDir, cost: MoveUnit) -> bool {
        //if this move unit is greater than what we already have, replace it.
        //we found a quicker way to get to the same square.

        //if it is not quicker, imediately stop everything.
        let new_path = path.add(m).unwrap();
        let coord = new_path.get_end_coord(self.start);

        //we found a match now lets compare
        let index =
            if let Some((index, _)) = self.moves.iter().enumerate().find(|(_, a)| a.0 == coord) {
                index
            } else {
                self.moves.push((coord, new_path, cost));
                return true;
            };

        if cost.0 > self.moves[index].2 .0 {
            let og = &mut self.moves[index];
            let new = &mut (coord, new_path, cost);
            core::mem::swap(og, new);
            // self.moves.push();
            // self.moves.swap_remove(index);
            return true;
        }

        return false;
    }
}

// //normal terrain is 2.
// //road is 1.
// fn terrain_cost(a: GridCoord) -> MoveUnit {
//     MoveUnit(2)
// }
