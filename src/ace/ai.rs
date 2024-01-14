use crate::movement::bitfield::BitField;

use super::*;

pub type Eval = i64; //(f64);

const MATE: i64 = 1_000_000;

//cats maximizing
//dogs minimizing
pub fn absolute_evaluate(view: &GameState, _debug: bool) -> Eval {
    // Doesnt seem that important that the AI knows exactly when it is winning.
    // (There doesnt seem to be many tactical combinations near the end of the game).
    // match view.game_is_over() {
    //     Some(GameOver::CatWon) => {
    //         return MATE;
    //     }
    //     Some(GameOver::DogWon) => {
    //         return -MATE;
    //     }
    //     Some(GameOver::Tie) => {}
    //     None => {}
    // }

    let ship_allowed = {
        let water = {
            let mut t = view.env.land.clone();
            t.toggle_range(..);
            t
        };
        let mut t = view.world.get_game_cells().clone();
        t.intersect_with(&water);
        t
    };

    let mut cat_ships = BitField::from_iter(
        view.factions
            .cats
            .iter()
            .filter(|a| a.typ == Type::Ship)
            .map(|a| a.position),
    );
    let mut dog_ships = BitField::from_iter(
        view.factions
            .dogs
            .iter()
            .filter(|a| a.typ == Type::Ship)
            .map(|a| a.position),
    );

    doop(&mut dog_ships, &mut cat_ships, &ship_allowed);

    let foot_grass = {
        let mut land = view.env.land.clone();
        let mut t = view.env.forest.clone();
        t.toggle_range(..);
        land.intersect_with(&t);
        land
    };
    let mut cat_foot = BitField::from_iter(
        view.factions
            .cats
            .iter()
            .filter(|a| a.typ == Type::Foot)
            .map(|a| a.position),
    );
    let mut dog_foot = BitField::from_iter(
        view.factions
            .dogs
            .iter()
            .filter(|a| a.typ == Type::Foot)
            .map(|a| a.position),
    );

    doop(&mut dog_foot, &mut cat_foot, &foot_grass);

    let s = cat_ships.count_ones(..) as i64 - dog_ships.count_ones(..) as i64;
    let r = cat_foot.count_ones(..) as i64 - dog_foot.count_ones(..) as i64;
    s + r
}

fn doop(mut dogs: &mut BitField, mut cats: &mut BitField, allowed_cells: &BitField) {
    fn around(point: GridCoord) -> impl Iterator<Item = GridCoord> {
        point.to_cube().ring(1).map(|(_, b)| b.to_axial())
    }

    fn expand_mesh(mesh: &mut BitField, workspace: &mut BitField) {
        workspace.clear();
        workspace.union_with(mesh);

        for a in workspace.iter_mesh(GridCoord([0; 2])) {
            for b in around(a) {
                mesh.set_coord(b, true);
            }
        }
    }

    let mut nomans = BitField::new();
    let mut w = BitField::new();
    let mut contested = BitField::new();
    for _ in 0..5 {
        expand_mesh(&mut dogs, &mut w);
        expand_mesh(&mut cats, &mut w);

        dogs.intersect_with(&allowed_cells);
        cats.intersect_with(&allowed_cells);

        contested.clear();
        contested.union_with(dogs);
        contested.intersect_with(cats);
        nomans.union_with(&contested);

        contested.toggle_range(..);

        dogs.intersect_with(&contested);
        cats.intersect_with(&contested);
    }
}

//TODO use bump allocator!!!!!
//TODO just store best move? not gamestate?
pub struct MoveOrdering {
    a: std::collections::HashMap<Vec<moves::ActualMove>, moves::ActualMove>,
}
impl MoveOrdering {
    pub fn get_best_prev_move(&self, path: &[moves::ActualMove]) -> Option<&moves::ActualMove> {
        self.a.get(path)
    }
    pub fn get_best_prev_move_mut(
        &mut self,
        path: &[moves::ActualMove],
    ) -> Option<&mut moves::ActualMove> {
        self.a.get_mut(path)
    }

    pub fn update(&mut self, path: &[moves::ActualMove], aaa: &moves::ActualMove) {
        //if let Some(aaa) = &ret {
        if let Some(foo) = self.get_best_prev_move_mut(path) {
            *foo = aaa.clone();
        } else {
            self.insert(path, aaa.clone());
        }
        //}
    }
    pub fn insert(&mut self, path: &[moves::ActualMove], m: moves::ActualMove) {
        self.a.insert(path.iter().cloned().collect(), m);
    }
}

pub fn iterative_deepening<'a>(game: &GameState, team: ActiveTeam) -> moves::ActualMove {
    let mut count = Counter { count: 0 };
    let mut results = Vec::new();

    let max_depth = 4;
    let mut foo1 = MoveOrdering {
        a: std::collections::HashMap::new(),
    };
    //TODO stop searching if we found a game ending move.
    for depth in 1..max_depth {
        console_dbg!("searching", depth);

        //TODO should this be outside the loop?
        let mut k = KillerMoves::new(max_depth);

        let mut aaaa = ai::AlphaBeta {
            //table: &mut table,
            prev_cache: &mut foo1,
            calls: &mut count,
            path: &mut vec![],
            killer_moves: &mut k,
            max_ext: 0,
        };

        let mut kk = game.clone();
        let res = aaaa.alpha_beta(&mut kk, ABAB::new(), team, depth, 0);
        assert_eq!(&kk, game);

        let mov = foo1.a.get(&[] as &[_]).cloned().unwrap();
        let res = EvalRet { mov, eval: res };

        let eval = res.eval;
        console_dbg!(eval);

        results.push(res);

        if eval.abs() == MATE {
            console_dbg!("found a mate");
            break;
        }
    }

    console_dbg!(count);
    console_dbg!(&results);

    //TODO THIS CAUSES ISSUES
    //results.dedup_by_key(|x| x.eval);

    //console_dbg!("deduped",&results);

    let target_eval = results.last().unwrap().eval;
    // let mov = if let Some(a) = results
    //     .iter()
    //     .rev()
    //     .find(|a| a.eval == target_eval && a.mov != ActualMove::SkipTurn)
    // {
    //     a.clone()
    // } else {
    //     results.pop().unwrap()
    // };
    let mov = results.pop().unwrap();
    //let mov =
    let m = mov;

    console_dbg!("AI MOVE::", m.mov, m.eval);

    m.mov
}

#[derive(Debug)]
pub struct Counter {
    count: u128,
}
impl Counter {
    pub fn add_eval(&mut self) {
        self.count += 1;
    }
}

pub struct AlphaBeta<'a> {
    //table: &'a mut LeafTranspositionTable,
    prev_cache: &'a mut MoveOrdering,
    calls: &'a mut Counter,
    path: &'a mut Vec<moves::ActualMove>,
    killer_moves: &'a mut KillerMoves,
    max_ext: usize,
}

pub struct KillerMoves {
    a: Vec<smallvec::SmallVec<[moves::ActualMove; 2]>>,
}

impl KillerMoves {
    pub fn new(a: usize) -> Self {
        let v = (0..a).map(|_| smallvec::SmallVec::new()).collect();
        Self { a: v }
    }
    pub fn get(&mut self, depth: usize) -> &mut [moves::ActualMove] {
        &mut self.a[depth]
    }
    pub fn consider(&mut self, depth: usize, m: moves::ActualMove) {
        let a = &mut self.a[depth];

        if a.contains(&m) {
            return;
        }
        if a.len() < 2 {
            a.push(m);
        } else {
            a.swap(0, 1);
            a[0] = m;
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvalRet<T> {
    pub mov: T,
    pub eval: Eval,
}

impl<'a> AlphaBeta<'a> {
    pub fn alpha_beta(
        &mut self,
        game_after_move: &mut GameState,
        ab: ABAB,
        team: ActiveTeam,
        depth: usize,
        ext: usize,
    ) -> Eval {
        self.max_ext = self.max_ext.max(ext);

        let ret = if depth == 0 || game_after_move.game_is_over().is_some() {
            self.calls.add_eval();
            absolute_evaluate(&game_after_move, false)
        } else {
            let pvariation = self.prev_cache.get_best_prev_move(self.path).cloned();

            let mut moves = game_after_move.for_all_moves_fast(team);

            let mut num_sorted = 0;
            if let Some(p) = pvariation {
                let f = moves.iter().enumerate().find(|(_, x)| **x == p).unwrap();
                let swap_ind = f.0;
                moves.swap(0, swap_ind);
                num_sorted += 1;
            }

            for a in self.killer_moves.get(depth) {
                if let Some((x, _)) = moves[num_sorted..]
                    .iter()
                    .enumerate()
                    .find(|(_, x)| *x == a)
                {
                    moves.swap(x, num_sorted);
                    num_sorted += 1;
                }
            }

            let foo = |ssself: &mut AlphaBeta, cand: moves::ActualMove, ab| {
                let cc = cand.clone();
                let new_depth = depth - 1;

                cand.execute_move_no_ani(game_after_move, team);
                ssself.path.push(cand.clone());
                let eval = ssself.alpha_beta(game_after_move, ab, team.not(), new_depth, ext);

                cand.execute_undo(game_after_move, team);
                let _ = ssself.path.pop().unwrap();

                EvalRet { eval, mov: cc }
            };

            if team == ActiveTeam::Cats {
                if let Some(ret) = ab.maxxer(moves, self, foo, |ss, m, _| {
                    ss.killer_moves.consider(depth, m);
                }) {
                    self.prev_cache.update(&self.path, &ret.mov);
                    ret.eval
                } else {
                    Eval::MIN
                }
            } else {
                if let Some(ret) = ab.minner(moves, self, foo, |ss, m, _| {
                    ss.killer_moves.consider(depth, m);
                }) {
                    self.prev_cache.update(&self.path, &ret.mov);
                    ret.eval
                } else {
                    Eval::MAX
                }
            }
        };
        //console_dbg!("alpha beta ret=",ret,depth);
        ret
    }
}

// #[derive(Debug, PartialEq, Eq, Clone)]
// pub struct PossibleMove {
//     pub the_move: moves::ActualMove,
//     //pub mesh: MovementMesh,
//     pub game_after_move: GameState,
// }

//TODO pass readonly

// pub struct PossibleMoveWithMesh {
//     pub the_move: moves::ActualMove,
//     //pub mesh: MovementMesh,
//     pub game_after_move: GameState,
//     pub mesh: MovementMesh,
// }
// impl PossibleMoveWithMesh {
//     pub fn into(self) -> PossibleMove {
//         PossibleMove {
//             the_move: self.the_move,
//             game_after_move: self.game_after_move,
//         }
//     }
// }

//TODO this has duplicated logic
// pub fn apply_move(mo: moves::ActualMove, state: &mut GameState, team: ActiveTeam) {
//     let moves::ActualMove::ExtraMove(
//         moves::PartialMoveSigl {
//             unit: pos,
//             moveto: mm,
//         },
//         moves::PartialMoveSigl {
//             unit: _,
//             moveto: sm,
//         },
//     ) = mo
//     else {
//         unreachable!()
//     };

//     let pp = state.view_mut(team).this_team.find_slow_mut(&pos).unwrap();

//     pp.position = mm;

//     if pp.typ == Type::Ship {
//         state.land.push(sm);
//     } else if pp.typ == Type::Foot {
//         state.forest.push(sm);
//     }
// }

//TODO use this!!!

// pub fn for_all_moves(state: GameState, team: ActiveTeam) -> impl Iterator<Item = PossibleMove> {
//     let mut sss = state.clone();
//     let ss = state.clone();
//     ss.into_view(team)
//         .this_team
//         .units
//         .into_iter()
//         .map(|a| RegularSelection { unit: a.clone() })
//         .flat_map(move |a| {
//             let mesh = a.generate(&sss.view_mut(team));
//             mesh.iter_mesh(a.unit.position)
//                 .map(move |f| (a.clone(), mesh.clone(), f))
//         })
//         .flat_map(move |(s, mesh, m)| {
//             let mut v = state.clone();
//             let mut mm = MoveLog::new();

//             let first = if let Some(l) = s
//                 .execute_no_animation(m, mesh, &mut v.view_mut(team), &mut mm)
//                 .unwrap()
//             {
//                 //console_dbg!("YOOOOOOO");
//                 let cll = l.select();

//                 //let mut kk = v.view().duplicate();
//                 let mut kk = v.clone();
//                 let mesh2 = cll.generate(&mut kk.view_mut(team));
//                 Some(mesh2.iter_mesh(l.coord()).map(move |m| {
//                     let mut klkl = kk.clone();
//                     let mut mm2 = MoveLog::new();

//                     let mut vfv = klkl.view_mut(team);
//                     cll.execute_no_animation(m, mesh2.clone(), &mut vfv, &mut mm2)
//                         .unwrap();

//                     PossibleMove {
//                         game_after_move: klkl,
//                         //mesh: mesh2,
//                         the_move: mm2.inner[0].clone(),
//                     }
//                 }))
//             } else {
//                 None
//             };

//             let second = if first.is_none() {
//                 //console_dbg!("NEVER HAPPEN");
//                 Some([PossibleMove {
//                     game_after_move: v,
//                     //mesh,
//                     the_move: mm.inner[0].clone(),
//                 }])
//             } else {
//                 None
//             };

//             let f1 = first.into_iter().flatten();
//             let f2 = second.into_iter().flatten();
//             f1.chain(f2)
//         })
//     //.chain([foo].into_iter())
// }

mod abab_simple {
    pub struct MyMoveFinder {}
    impl abab_simple::MoveFinder for MyMoveFinder {
        type EE = Eval;
        type T = GameState;
        type Mo = moves::ActualMove;
        type Finder = std::vec::IntoIter<moves::ActualMove>;

        fn eval(&mut self, game: &Self::T) -> Self::EE {
            absolute_evaluate(game, false)
        }

        fn min_eval(&self) -> Self::EE {
            Eval::MIN
        }

        fn max_eval(&self) -> Self::EE {
            Eval::MAX
        }

        fn apply_move(&mut self, game: &mut Self::T, maximizer: bool, a: Self::Mo) {
            let team = if maximizer {
                ActiveTeam::Cats
            } else {
                ActiveTeam::Dogs
            };
            a.execute_move_no_ani(game, team);
        }
        fn unapply_move(&mut self, game: &mut Self::T, maximizer: bool, a: Self::Mo) {
            let team = if maximizer {
                ActiveTeam::Cats
            } else {
                ActiveTeam::Dogs
            };
            a.execute_undo(game, team);
        }
        fn generate_finder(
            &mut self,
            state: &Self::T,
            path: &[Self::Mo],
            maximizer: bool,
        ) -> Self::Finder {
            let team = if maximizer {
                ActiveTeam::Cats
            } else {
                ActiveTeam::Dogs
            };
            todo!();
            //let k: Vec<_> = for_all_moves(state.clone(), team).collect();
            //k.into_iter()
        }

        fn select_move(&mut self, finder: &mut Self::Finder) -> Option<Self::Mo> {
            finder.next().map(|x| x)
        }
    }

    pub trait MoveFinder {
        type EE: PartialOrd + Ord + Copy;
        type T;
        type Mo: Copy;
        type Finder;
        fn eval(&mut self, game: &Self::T) -> Self::EE;

        fn min_eval(&self) -> Self::EE;
        fn max_eval(&self) -> Self::EE;

        fn apply_move(&mut self, game: &mut Self::T, maximizer: bool, a: Self::Mo);
        fn unapply_move(&mut self, game: &mut Self::T, maximizer: bool, a: Self::Mo);

        fn generate_finder(
            &mut self,
            state: &Self::T,
            path: &[Self::Mo],
            maximizer: bool,
        ) -> Self::Finder;
        fn select_move(&mut self, finder: &mut Self::Finder) -> Option<Self::Mo>;
    }

    pub fn alpha_beta<X: MoveFinder>(
        data: X,
        depth: usize,
        mut game_state: &mut X::T,
        maximizer: bool,
    ) -> (X::EE, Vec<X::Mo>) {
        ABAB::new(data).alpha_beta(depth, game_state, maximizer)
    }
    use super::*;
    #[derive(Clone)]
    struct ABAB<X, Y, Z> {
        alpha: Z,
        beta: Z,
        data: X,
        path: Vec<Y>,
    }
    impl<X: MoveFinder> ABAB<X, X::Mo, X::EE> {
        pub fn new(data: X) -> Self {
            ABAB {
                alpha: data.min_eval(),
                beta: data.max_eval(),
                data,
                path: vec![],
            }
        }

        pub fn alpha_beta(
            &mut self,
            depth: usize,
            game_state: &mut X::T,
            maximizer: bool,
        ) -> (X::EE, Vec<X::Mo>) {
            if depth == 0 {
                (self.data.eval(&game_state), vec![])
            } else {
                if maximizer {
                    let mut value = self.data.min_eval();
                    let mut ll = vec![];
                    let mut best_move = None;

                    let mut gs = self
                        .data
                        .generate_finder(&game_state, &self.path, maximizer);
                    while let Some(mo) = self.data.select_move(&mut gs) {
                        //let mut ga = game_state.clone();
                        self.data.apply_move(game_state, maximizer, mo);
                        self.path.push(mo);
                        let (eval, move_list) = self.alpha_beta(depth - 1, game_state, !maximizer);
                        self.path.pop();
                        self.data.unapply_move(game_state, maximizer, mo);

                        if eval > value {
                            value = eval;
                            ll = move_list;
                            best_move = Some(mo);
                        }

                        if value > self.beta {
                            break;
                        }
                        self.alpha = self.alpha.max(value);
                    }

                    ll.push(best_move.unwrap());
                    (value, ll)
                } else {
                    let mut value = self.data.max_eval();
                    let mut ll = vec![];
                    let mut best_move = None;

                    let mut gs = self
                        .data
                        .generate_finder(&game_state, &self.path, maximizer);
                    while let Some(mo) = self.data.select_move(&mut gs) {
                        self.data.apply_move(game_state, maximizer, mo);
                        self.path.push(mo);
                        let (eval, move_list) = self.alpha_beta(depth - 1, game_state, !maximizer);
                        self.path.pop();
                        self.data.apply_move(game_state, maximizer, mo);

                        if eval > value {
                            value = eval;
                            ll = move_list;
                            best_move = Some(mo);
                        }

                        if value < self.alpha {
                            break;
                        }
                        self.beta = self.beta.min(value);
                    }

                    ll.push(best_move.unwrap());
                    (value, ll)
                }
            }
        }
    }
}
use abab::ABAB;
mod abab {
    use super::*;
    #[derive(Clone)]
    pub struct ABAB {
        alpha: Eval,
        beta: Eval,
    }
    impl ABAB {
        pub fn new() -> Self {
            ABAB {
                alpha: Eval::MIN,
                beta: Eval::MAX,
            }
        }

        pub fn minner<P, T: Clone>(
            mut self,
            it: impl IntoIterator<Item = T>,
            payload: &mut P,
            mut func: impl FnMut(&mut P, T, Self) -> EvalRet<T>,
            mut func2: impl FnMut(&mut P, T, Self),
        ) -> Option<EvalRet<T>> {
            let mut mm: Option<T> = None;

            let mut value = i64::MAX;
            for cand in it {
                let t = func(payload, cand.clone(), self.clone());

                value = value.min(t.eval);
                if value == t.eval {
                    mm = Some(cand.clone());
                }
                if t.eval < self.alpha {
                    func2(payload, cand, self.clone());
                    break;
                }
                self.beta = self.beta.min(value)
            }

            if let Some(mm) = mm {
                Some(EvalRet {
                    mov: mm,
                    eval: value,
                })
            } else {
                None
            }
        }
        pub fn maxxer<P, T: Clone>(
            mut self,
            it: impl IntoIterator<Item = T>,
            mut payload: &mut P,
            mut func: impl FnMut(&mut P, T, Self) -> EvalRet<T>,
            mut func2: impl FnMut(&mut P, T, Self),
        ) -> Option<EvalRet<T>> {
            let mut mm: Option<T> = None;

            let mut value = i64::MIN;
            for cand in it {
                let t = func(&mut payload, cand.clone(), self.clone());

                value = value.max(t.eval);
                if value == t.eval {
                    mm = Some(cand.clone());
                }
                if t.eval > self.beta {
                    func2(&mut payload, cand, self.clone());
                    break;
                }
                self.alpha = self.alpha.max(value)
            }

            if let Some(mm) = mm {
                Some(EvalRet {
                    mov: mm,
                    eval: value,
                })
            } else {
                None
            }
        }
    }
}
