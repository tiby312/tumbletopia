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

    use serde::{Deserialize, Serialize};

    use hex::HDir;
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

    //use bitvec::boxed::BitBox;

    #[derive(
        Hash, Serialize, Deserialize, Default, PartialOrd, Ord, PartialEq, Eq, Debug, Clone,
    )]
    pub struct SmallMesh {
        //pub inner: [u64; 4],
        //pub inner: BitBox,
        pub inner: BitArr!(for crate::board::TABLE_SIZE),
    }

    use bitvec::prelude::*;
    impl SmallMesh {
        pub fn new() -> SmallMesh {
            SmallMesh {
                inner: bitarr![0;crate::board::TABLE_SIZE],
            }
        }
        pub fn from_iter(it: impl IntoIterator<Item = Axial>) -> SmallMesh {
            let mut m = SmallMesh::new();
            for a in it {
                m.add(a);
            }
            m
        }
        // pub fn count_ones(&self) -> u32 {
        //     self.inner.c
        //     self.inner[0].count_ones()
        //         + self.inner[1].count_ones()
        //         + self.inner[2].count_ones()
        //         + self.inner[3].count_ones()
        // }
        pub fn union_with(&mut self, other: &SmallMesh) {
            *self.inner.as_mut_bitslice() |= &other.inner
        }

        #[must_use]
        pub fn validate_rel(a: Axial) -> bool {
            let ind = a.to_index();
            ind < crate::board::TABLE_SIZE

            // let x = a.q;
            // let y = a.r;

            // assert!((-6..=6).contains(&x));
            // assert!((-6..=6).contains(&y));

            //assert!(x != 0 || y != 0);
        }
        pub fn add(&mut self, a: Axial) {
            //use std::ops::Shl;
            assert!(Self::validate_rel(a));

            let ind = a.to_index();

            //*self.inner.as_mut_bitslice() |= U256::one() << ind;

            self.inner.as_mut_bitslice().set(ind, true);
        }
        pub fn set_coord(&mut self, a: Axial, val: bool) {
            if val {
                self.add(a)
            } else {
                self.remove(a)
            }
        }
        pub fn remove(&mut self, a: Axial) {
            assert!(Self::validate_rel(a));
            let ind = a.to_index();
            //let (a, b) = ind_to_foo(ind);
            //self.inner &= !(U256::one() << ind);
            self.inner.set(ind, false);
        }
        pub fn is_empty(&self) -> bool {
            self.inner.is_empty()
        }
        pub fn is_set(&self, a: Axial) -> bool {
            if !Self::validate_rel(a) {
                return false;
            }

            let ind = a.to_index();
            //let (a, b) = ind_to_foo(ind);

            //self.inner & (U256::one() << ind) != U256::zero()
            self.inner[ind] == true
        }

        pub fn iter_mesh(&self, point: Axial) -> impl Iterator<Item = Axial> + '_ {
            self.inner
                .iter_ones()
                .map(move |a| point.add(Axial::from_index(&a)))
        }
    }

    use super::Axial;
}

// #[derive(Debug, Clone)]
// pub struct MyPath(pub [Option<HDir>; 3]);

// pub fn path(
//     _mesh: &small_mesh::SmallMesh,
//     unit: Axial,
//     target: Axial,
//     pathable: &small_mesh::SmallMesh,
//     game: &GameState,
//     team: ActiveTeam,
//     world: &board::MyWorld,
//     capturing: bool,
// ) -> MyPath {
//     let neighbours = |a: &Axial| {
//         let mut k = a.to_cube().neighbours2();
//         k.sort_unstable_by_key(|a| a.dist(&target.to_cube()));
//         k.map(|x| (x.to_axial(), a.dir_to(&x)))
//     };

//     // if walls.is_set(target.sub(&unit)) {
//     //     assert_eq!(unit.to_cube().dist(&target.to_cube()), 1);
//     //     return MyPath([Some(unit.dir_to(&target)), None, None]);
//     // }

//     //let typ = game.factions.relative(team).this_team.get_type(unit);

//     let find = |depth: usize| {
//         for (a, adir) in neighbours(&unit) {
//             if !pathable.is_set(a.sub(&unit)) {
//                 continue;
//             }

//             if a == target {
//                 return Some(MyPath([Some(adir), None, None]));
//             }

//             if depth == 1 {
//                 continue;
//             }

//             for (b, bdir) in neighbours(&a) {
//                 if !pathable.is_set(b.sub(&unit)) {
//                     continue;
//                 }

//                 if b == target {
//                     return Some(MyPath([Some(adir), Some(bdir), None]));
//                 }

//                 if depth == 2 {
//                     continue;
//                 }

//                 for (c, cdir) in neighbours(&b) {
//                     if !pathable.is_set(c.sub(&unit)) {
//                         continue;
//                     }

//                     if c == target {
//                         // if capturing && !game.is_trap(team, world, c.advance(cdir),typ) {
//                         //     continue;
//                         // }
//                         return Some(MyPath([Some(adir), Some(bdir), Some(cdir)]));
//                     }
//                 }
//             }
//         }

//         None
//     };

//     //similar to iterative deepening. We have to make sure we check for paths
//     //at smaller depths before trying larger depths because dfs has no order.
//     for a in 1..4 {
//         if let Some(a) = find(a) {
//             return a;
//         }
//     }

//     unreachable!(
//         "could not find path {:?}:{:?}:{:?}",
//         target,
//         Axial::zero().to_cube().dist(&target.to_cube()),
//         pathable.is_set(target)
//     );
// }

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
