use crate::hex::HDir;

use super::*;

pub mod small_mesh {

    pub fn explore_outward_two() -> impl Iterator<Item = (HDir, [HDir; 3])> {
        HDir::all().map(move |dir| {
            let straight = dir;
            let left = dir.rotate60_left();
            let right = dir.rotate60_right();
            (dir, [straight, left, right])
        })
    }

    use crate::hex;
    use crate::hex::HDir;
    #[test]
    fn test_mesh() {
        let k1 = Axial::from_arr([2, 0]);
        let k2 = Axial::from_arr([2, -2]);
        let k3 = Axial::from_arr([-2, 1]);

        let mut mesh = SmallMesh::new();
        mesh.add(k1);
        mesh.add(k2);
        mesh.add(k3);

        assert!(mesh.is_set(k1));
        assert!(mesh.is_set(k2));
        assert!(mesh.is_set(k3));
        assert!(!mesh.is_set(Axial::from_arr([-2, 2])));

        let res: Vec<_> = mesh.iter_mesh(Axial::from_arr([0; 2])).collect();

        assert_eq!(
            res,
            vec!(
                Axial::from_arr([-2, 1]),
                Axial::from_arr([2, -2]),
                Axial::from_arr([2, 0])
            )
        )
    }

    #[derive(Default, PartialOrd, Ord, PartialEq, Eq, Debug,Clone)]
    pub struct SmallMesh {
        pub inner: [u64; 4],
    }

    impl SmallMesh {
        pub fn new() -> SmallMesh {
            SmallMesh { inner: [0; 4] }
        }
        pub fn from_iter(it: impl Iterator<Item = Axial>) -> SmallMesh {
            let mut m = SmallMesh::new();
            for a in it {
                m.add(a);
            }
            m
        }

        #[must_use]
        pub fn validate_rel(a: Axial) -> bool {
            let ind = conv(a);

            ind < 256

            // let x = a.q;
            // let y = a.r;

            // assert!((-6..=6).contains(&x));
            // assert!((-6..=6).contains(&y));

            //assert!(x != 0 || y != 0);
        }
        pub fn add(&mut self, a: Axial) {
            assert!(Self::validate_rel(a));

            let ind = conv(a);
            let (a, b) = ind_to_foo(ind);
            self.inner[a] |= 1 << b;
            //self.inner |= 1 << ind;
        }
        pub fn remove(&mut self, a: Axial) {
            assert!(Self::validate_rel(a));
            let ind = conv(a);
            let (a, b) = ind_to_foo(ind);
            self.inner[a] &= !(1 << b);
        }
        pub fn is_empty(&self) -> bool {
            self.inner[0] == 0 && self.inner[1] == 0 && self.inner[2] == 0 && self.inner[3] == 0
        }
        pub fn is_set(&self, a: Axial) -> bool {
            if !Self::validate_rel(a) {
                return false;
            }

            let ind = conv(a);
            let (a, b) = ind_to_foo(ind);

            self.inner[a] & (1 << b) != 0
        }
        pub fn iter_mesh(&self, point: Axial) -> impl Iterator<Item = Axial> {
            let inner = self.inner;

            //let skip_moves = self.swing_moves(point);

            // TABLE
            //     .iter()
            //     .enumerate()
            //     .filter(move |(x, _)| inner & (1 << x) != 0)
            //     .map(move |(_, x)| point.add(GridCoord(*x)))

            (0usize..256)
                .filter(move |&x| {
                    let (a, b) = ind_to_foo(x);

                    inner[a] & (1 << b) != 0
                })
                .map(move |a| {
                    let x = a / 16;
                    let y = a % 16;
                    point.add(Axial::from_arr([
                        (x - 8) as hex::CoordNum,
                        (y - 8) as hex::CoordNum,
                    ]))
                }) //.chain(skip_moves)
        }
    }

    use super::Axial;

    fn ind_to_foo(a: usize) -> (usize, usize) {
        assert!(a >= 0 && a < 256);

        // 0
        // 64
        // 128
        // 192
        // 256

        let block = a / 64;
        let block_ind = a % 64;
        (block, block_ind)
    }

    fn conv(a: Axial) -> usize {
        let Axial { q, r } = a;
        //     let ind=x/7+y%7;
        //     // -3 -2 -1 0 1 2 3
        //     // -6 -5 -4 -3 -2 -1 0 1 2 3 4 5 6
        // ind as usize
        ((q as isize + 8) * 16 + (r as isize + 8)) as usize

        // TABLE
        //     .iter()
        //     .enumerate()
        //     .find(|(_, x)| **x == a.0)
        //     .expect("Could not find the coord in table")
        //     .0
    }
}

#[derive(Debug, Clone)]
pub struct MyPath(pub [Option<HDir>; 3]);

pub fn path(
    _mesh: &small_mesh::SmallMesh,
    unit: Axial,
    target: Axial,
    pathable: &small_mesh::SmallMesh,
    game: &GameState,
    team: ActiveTeam,
    world: &board::MyWorld,
    capturing: bool,
) -> MyPath {
    let neighbours = |a: &Axial| {
        let mut k = a.to_cube().neighbours2();
        k.sort_unstable_by_key(|a| a.dist(&target.to_cube()));
        k.map(|x| (x.to_axial(), a.dir_to(&x)))
    };

    // if walls.is_set(target.sub(&unit)) {
    //     assert_eq!(unit.to_cube().dist(&target.to_cube()), 1);
    //     return MyPath([Some(unit.dir_to(&target)), None, None]);
    // }

    //let typ = game.factions.relative(team).this_team.get_type(unit);

    let find = |depth: usize| {
        for (a, adir) in neighbours(&unit) {
            if !pathable.is_set(a.sub(&unit)) {
                continue;
            }

            if a == target {
                return Some(MyPath([Some(adir), None, None]));
            }

            if depth == 1 {
                continue;
            }

            for (b, bdir) in neighbours(&a) {
                if !pathable.is_set(b.sub(&unit)) {
                    continue;
                }

                if b == target {
                    return Some(MyPath([Some(adir), Some(bdir), None]));
                }

                if depth == 2 {
                    continue;
                }

                for (c, cdir) in neighbours(&b) {
                    if !pathable.is_set(c.sub(&unit)) {
                        continue;
                    }

                    if c == target {
                        // if capturing && !game.is_trap(team, world, c.advance(cdir),typ) {
                        //     continue;
                        // }
                        return Some(MyPath([Some(adir), Some(bdir), Some(cdir)]));
                    }
                }
            }
        }

        None
    };

    //similar to iterative deepening. We have to make sure we check for paths
    //at smaller depths before trying larger depths because dfs has no order.
    for a in 1..4 {
        if let Some(a) = find(a) {
            return a;
        }
    }

    unreachable!(
        "could not find path {:?}:{:?}:{:?}",
        target,
        Axial::zero().to_cube().dist(&target.to_cube()),
        pathable.is_set(target)
    );
}

// pub fn path_old(
//     _mesh: &small_mesh::SmallMesh,
//     a: Axial,
//     walls: &small_mesh::SmallMesh,
// ) -> impl Iterator<Item = HDir> {
//     let mesh_iter = {
//         assert!(small_mesh::SmallMesh::validate_rel(a));

//         let x = a.q;
//         let y = a.r;
//         let first = if Axial::from_arr([0, 0]).to_cube().dist(&a.to_cube()) == 1 {
//             Some([Axial::from_arr([0, 0]).dir_to(&a)])
//         } else {
//             None
//         };

//         let second = if Axial::from_arr([0, 0]).to_cube().dist(&a.to_cube()) == 2 {
//             //diagonal
//             let diag = if first.is_none() && (x.abs() == 1 || y.abs() == 1) {
//                 //TODO inefficient
//                 let mut k = Axial::from_arr([0, 0])
//                     .to_cube()
//                     .neighbours()
//                     .filter(|x| x.dist(&a.to_cube()) == 1);
//                 let first = k.next().unwrap().to_axial();
//                 let second = k.next().unwrap().to_axial();

//                 if
//                 /*self.is_set(first)||*/
//                 !walls.is_set(first) {
//                     Some([Axial::from_arr([0, 0]).dir_to(&first), first.dir_to(&a)])
//                 } else {
//                     Some([Axial::from_arr([0, 0]).dir_to(&second), second.dir_to(&a)])
//                 }
//             } else {
//                 None
//             };

//             let orth = if first.is_none() && diag.is_none() && (x.abs() == 2 || y.abs() == 2) {
//                 let h = Axial::from_arr([0, 0]).dir_to(&a);
//                 Some([h, h])
//             } else {
//                 None
//             };

//             Some(diag.into_iter().flatten().chain(orth.into_iter().flatten()))
//         } else {
//             None
//         };

//         let a = first.into_iter().flatten();
//         a.chain(second.into_iter().flatten())
//     };

//     mesh_iter.into_iter()
// }

pub mod bitfield {
    use super::Axial;
    use crate::hex;
    use serde::*;
    use std::ops::{Deref, DerefMut};

    #[test]
    fn bitfield() {
        let mut m = BitField::new();

        for k in -16..16 {
            dbg!("handling=k", k);
            m.set_coord(Axial::from_arr([k, k]), true);

            assert!(m.is_set(Axial::from_arr([k, k])), "boo={}", k);
        }
    }

    use fixedbitset::FixedBitSet;

    #[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
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
    impl Default for BitField {
        fn default() -> Self {
            Self::new()
        }
    }

    impl BitField {
        pub fn new() -> Self {
            BitField {
                inner: FixedBitSet::with_capacity(1024),
            }
        }
        pub fn from_iter<K: std::borrow::Borrow<Axial>>(a: impl IntoIterator<Item = K>) -> Self {
            let mut k = BitField::new();
            for a in a {
                k.set_coord(*a.borrow(), true);
            }
            k
        }

        pub fn valid_coord(&self, a: Axial) -> bool {
            (-16..16).contains(&a.q) && (-16..16).contains(&a.r)
        }
        pub fn set_coord(&mut self, a: Axial, val: bool) {
            let _x = a.q;
            let _y = a.r;
            //assert!(self.contains_coord(a), "val={:?}", a);

            let ind = conv(a);
            self.inner.set(ind, val);
        }

        pub fn is_set(&self, a: Axial) -> bool {
            let ind = conv(a);

            self.inner[ind]
        }
        pub fn iter_mesh(&self) -> impl Iterator<Item = Axial> + '_ {
            self.inner.ones().map(move |a| {
                let x = a / 32;
                let y = a % 32;
                Axial::from_arr([(x - 16) as hex::CoordNum, (y - 16) as hex::CoordNum])
            })
        }
    }
    fn conv(a: Axial) -> usize {
        let Axial { q, r } = a;
        ((q as isize + 16) * 32 + (r as isize + 16)) as usize
    }
}
