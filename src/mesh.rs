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

    #[derive(Default, PartialOrd, Ord, PartialEq, Eq, Debug, Clone)]
    pub struct SmallMesh {
        pub inner: u128,
    }

    impl SmallMesh {
        pub fn new() -> SmallMesh {
            SmallMesh { inner: 0 }
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

            ind < 128

            // let x = a.q;
            // let y = a.r;

            // assert!((-6..=6).contains(&x));
            // assert!((-6..=6).contains(&y));

            //assert!(x != 0 || y != 0);
        }
        pub fn add(&mut self, a: Axial) {
            assert!(Self::validate_rel(a));

            let ind = conv(a);
            self.inner |= 1 << ind;
        }
        pub fn remove(&mut self, a: Axial) {
            assert!(Self::validate_rel(a));
            let ind = conv(a);
            self.inner &= !(1 << ind);
        }
        pub fn is_empty(&self) -> bool {
            self.inner == 0
        }
        pub fn is_set(&self, a: Axial) -> bool {
            if !Self::validate_rel(a) {
                return false;
            }

            let ind = conv(a);

            self.inner & (1 << ind) != 0
        }
        pub fn iter_mesh(&self, point: Axial) -> impl Iterator<Item = Axial> {
            let inner = self.inner;

            //let skip_moves = self.swing_moves(point);

            // TABLE
            //     .iter()
            //     .enumerate()
            //     .filter(move |(x, _)| inner & (1 << x) != 0)
            //     .map(move |(_, x)| point.add(GridCoord(*x)))

            (0..128)
                .filter(move |x| inner & (1 << x) != 0)
                .map(move |a| {
                    let x = a / 13;
                    let y = a % 13;
                    point.add(Axial::from_arr([x - 6, y - 6]))
                }) //.chain(skip_moves)
        }
    }

    use super::Axial;

    fn conv(a: Axial) -> usize {
        let Axial { q, r } = a;
        //     let ind=x/7+y%7;
        //     // -3 -2 -1 0 1 2 3
        //     // -6 -5 -4 -3 -2 -1 0 1 2 3 4 5 6
        // ind as usize
        ((q + 6) * 13 + (r + 6)) as usize

        // TABLE
        //     .iter()
        //     .enumerate()
        //     .find(|(_, x)| **x == a.0)
        //     .expect("Could not find the coord in table")
        //     .0
    }
}

pub struct MyPath([Option<HDir>; 3]);

impl IntoIterator for MyPath {
    type Item = HDir;

    type IntoIter = std::iter::Flatten<std::array::IntoIter<Option<HDir>, 3>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter().flatten()
    }
}

pub fn path(
    _mesh: &small_mesh::SmallMesh,
    unit: Axial,
    target: Axial,
    walls: &small_mesh::SmallMesh,
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

    let k = 'foo: {
        if walls.is_set(target.sub(&unit)) {
            assert_eq!(unit.to_cube().dist(&target.to_cube()), 1);
            break 'foo [Some(unit.dir_to(&target)), None, None];
        }

        for (a, adir) in neighbours(&unit) {
            if walls.is_set(a.sub(&unit)) {
                continue;
            }

            if a == target {
                break 'foo [Some(adir), None, None];
            }

            for (b, bdir) in neighbours(&a) {
                if walls.is_set(b.sub(&unit)) {
                    continue;
                }

                if b == target {
                    if capturing && !game.is_trap(team, world, b.advance(bdir)) {
                        continue;
                    }
                    break 'foo [Some(adir), Some(bdir), None];
                }

                for (c, cdir) in neighbours(&b) {
                    if walls.is_set(c.sub(&unit)) {
                        continue;
                    }

                    if c == target {
                        if capturing && !game.is_trap(team, world, c.advance(cdir)) {
                            continue;
                        }
                        break 'foo [Some(adir), Some(bdir), Some(cdir)];
                    }
                }
            }
        }

        unreachable!(
            "could not find path {:?}:{:?}:{:?}",
            target,
            Axial::zero().to_cube().dist(&target.to_cube()),
            walls.is_set(target)
        );
    };

    return MyPath(k);
}

pub fn path_old(
    _mesh: &small_mesh::SmallMesh,
    a: Axial,
    walls: &small_mesh::SmallMesh,
) -> impl Iterator<Item = HDir> {
    let mesh_iter = {
        assert!(small_mesh::SmallMesh::validate_rel(a));

        let x = a.q;
        let y = a.r;
        let first = if Axial::from_arr([0, 0]).to_cube().dist(&a.to_cube()) == 1 {
            Some([Axial::from_arr([0, 0]).dir_to(&a)])
        } else {
            None
        };

        let second = if Axial::from_arr([0, 0]).to_cube().dist(&a.to_cube()) == 2 {
            //diagonal
            let diag = if first.is_none() && (x.abs() == 1 || y.abs() == 1) {
                //TODO inefficient
                let mut k = Axial::from_arr([0, 0])
                    .to_cube()
                    .neighbours()
                    .filter(|x| x.dist(&a.to_cube()) == 1);
                let first = k.next().unwrap().to_axial();
                let second = k.next().unwrap().to_axial();

                if
                /*self.is_set(first)||*/
                !walls.is_set(first) {
                    Some([Axial::from_arr([0, 0]).dir_to(&first), first.dir_to(&a)])
                } else {
                    Some([Axial::from_arr([0, 0]).dir_to(&second), second.dir_to(&a)])
                }
            } else {
                None
            };

            let orth = if first.is_none() && diag.is_none() && (x.abs() == 2 || y.abs() == 2) {
                let h = Axial::from_arr([0, 0]).dir_to(&a);
                Some([h, h])
            } else {
                None
            };

            Some(diag.into_iter().flatten().chain(orth.into_iter().flatten()))
        } else {
            None
        };

        let a = first.into_iter().flatten();
        a.chain(second.into_iter().flatten())
    };

    mesh_iter.into_iter()
}

pub mod bitfield {
    use std::ops::{Deref, DerefMut};

    use super::Axial;

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
        pub fn from_iter<K: Into<Axial>>(a: impl IntoIterator<Item = K>) -> Self {
            let mut k = BitField::new();
            for a in a {
                k.set_coord(a.into(), true);
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
                Axial::from_arr([x as i16 - 16, y as i16 - 16])
            })
        }
    }
    fn conv(a: Axial) -> usize {
        let Axial { q, r } = a;
        ((q + 16) * 32 + (r + 16)) as usize
    }
}
