use std::collections::{BTreeMap, BTreeSet};

use crate::{
    board::{world_bitfield, world_mesh_iter},
    movement::{bitfield::BitField, movement_mesh::Mesh},
};

use super::{selection::MoveLog, *};

pub type Eval = i64; //(f64);

const MATE: i64 = 1_000_000;

//cats maximizing
//dogs minimizing
pub fn absolute_evaluate(view: &GameState, debug: bool) -> Eval {
    let water = {
        let mut t = BitField::from_iter(view.env.land.iter().copied());
        t.toggle_range(..);
        t
    };

    let grass = {
        let mut land = BitField::from_iter(view.env.land.iter().copied());
        let mut t = BitField::from_iter(view.env.forest.iter().copied());
        t.toggle_range(..);
        land.intersect_with(&t);
        land
    };

    let allowed = {
        let mut t = world_bitfield();
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

    doop(&mut dog_ships, &mut cat_ships, &allowed);

    doop(&mut dog_foot, &mut cat_foot, &grass);

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
                mesh.add(b);
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

pub struct LeafTranspositionTable {
    a: std::collections::HashMap<GameState, Eval>,
    saves: usize,
}

impl LeafTranspositionTable {
    pub fn new() -> Self {
        LeafTranspositionTable {
            a: std::collections::HashMap::new(),
            saves: 0,
        }
    }
    fn lookup_leaf(&mut self, a: &GameState) -> Option<&Eval> {
        if let Some(a) = self.a.get(a) {
            self.saves += 1;
            Some(a)
        } else {
            None
        }
    }
    fn consider_leaf(&mut self, game: GameState, eval: Eval) {
        if let Some(v) = self.a.get_mut(&game) {
            *v = eval;
        } else {
            let _ = self.a.insert(game, eval);
        }
    }
    pub fn lookup_leaf_all(&mut self, node: &GameState) -> Eval {
        if let Some(&eval) = self.lookup_leaf(&node) {
            eval
        } else {
            let eval = absolute_evaluate(&node, false);
            self.consider_leaf(node.clone(), eval);
            eval
        }
    }
}

pub fn iterative_deepening<'a>(game: &GameState, team: ActiveTeam) -> moves::ActualMove {
    let mut count = Counter { count: 0 };
    let mut results = Vec::new();
    let mut table = LeafTranspositionTable::new();

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
            table: &mut table,
            prev_cache: &mut foo1,
            calls: &mut count,
            path: &mut vec![],
            killer_moves: &mut k,
            max_ext: 0,
        };
        let res = aaaa.alpha_beta(game.clone(), ABAB::new(), team, depth, 0);

        let mov = foo1.a.get(&[] as &[_]).cloned().unwrap();
        let res = EvalRet { mov, eval: res };

        // let res = ai::alpha_beta(
        //     game,
        //     team,
        //     depth,
        //     false,
        //     f64::NEG_INFINITY,
        //     f64::INFINITY,
        //     &mut table,
        //     &mut foo1,
        //     &mut count,
        //     &mut vec![],
        // );

        let eval = res.eval;
        console_dbg!(eval);

        results.push(res);

        if eval.abs() == MATE {
            console_dbg!("found a mate");
            break;
        }
    }

    // console_dbg!(table.saves);
    // console_dbg!(table.a.len());
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
    table: &'a mut LeafTranspositionTable,
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
        mut game_after_move: GameState,
        ab: ABAB,
        team: ActiveTeam,
        depth: usize,
        ext: usize,
    ) -> Eval {
        self.max_ext = self.max_ext.max(ext);
        //let the_move = cand.the_move.clone();
        let mut gg = game_after_move.clone();

        let ret = if depth == 0
            || moves::partial_move::game_is_over(&mut game_after_move, team).is_some()
        {
            //console_dbg!(game_is_over(cand.game_after_move.view(team)));

            self.calls.add_eval();
            let e = self.table.lookup_leaf_all(&game_after_move);

            // if self.prev_cache.a.get(self.path).is_none(){
            //     self.prev_cache.update(&self.path, &cand.the_move);
            // }
            //console_dbg!("FOOO",e);
            e

            // let (m, eval) = self.quiensense_search(cand, ab, team, 3);

            // eval
        } else {
            let gg2 = game_after_move.clone();

            let node = game_after_move;

            let pvariation = self.prev_cache.get_best_prev_move(self.path).cloned();

            let pvariation = pvariation.map(|x| {
                moves::partial_move::execute_move_no_ani(&mut gg, team, x.clone());
                PossibleMove {
                    the_move: x,
                    game_after_move: gg,
                }
            });

            // let it = reorder_front(
            //     pvariation,
            //     ,
            // );

            // let enemy_king_pos=if let Some(enemy_king)=cand.game_after_move.view(team).that_team.units.iter().find(|x|x.typ==Type::Para){
            //     let pos=enemy_king.position;
            //     Some(pos)
            // }else{
            //     None
            // };

            let mut moves = moves::partial_move::for_all_moves_fast(node.clone(), team);

            //console_dbg!("FOOOO",moves.len());
            //console_dbg!(moves.iter().map(|x|&x.1.the_move).collect::<Vec<_>>());

            let num_checky = moves.iter().count();
            //console_dbg!(num_checky);
            // if is_check(&moves[0].1.game_after_move) {
            //     moves[0].0 = true;
            // }

            //console_dbg!(moves.len());

            let mut num_sorted = 0;
            if let Some(p) = pvariation {
                let f = moves
                    .iter()
                    .enumerate()
                    .find(|(_, x)| **x == p.the_move)
                    .unwrap();
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
                let new_depth = depth - 1; //.saturating_sub(inhibit);
                                           //assert!(new_depth<6);
                                           //console_dbg!(ext,depth);

                let mut gg = gg2.clone();
                moves::partial_move::execute_move_no_ani(&mut gg, team, cand);

                ssself.path.push(cand.clone());
                let eval = ssself.alpha_beta(gg, ab, team.not(), new_depth, ext);
                let k = ssself.path.pop().unwrap();

                //console_dbg!("inner eval=",eval);
                EvalRet { eval, mov: cc }
            };

            if team == ActiveTeam::Cats {
                //console_dbg!("maxing");
                if let Some(ret) = ab.maxxer(moves, self, foo, |ss, m, _| {
                    ss.killer_moves.consider(depth, m);
                }) {
                    //console_dbg!("FOUND",ret.eval);

                    self.prev_cache.update(&self.path, &ret.mov);
                    ret.eval
                } else {
                    Eval::MIN
                }
            } else {
                //console_dbg!("mining");
                if let Some(ret) = ab.minner(moves, self, foo, |ss, m, _| {
                    ss.killer_moves.consider(depth, m);
                }) {
                    self.prev_cache.update(&self.path, &ret.mov);
                    //console_dbg!("FOUND",ret.eval);
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

pub struct MyMoveFinder {}
impl abab_simple::MoveFinder for MyMoveFinder {
    type EE = Eval;
    type T = GameState;
    type Mo = moves::ActualMove;
    type Finder = std::vec::IntoIter<PossibleMove>;

    fn eval(&mut self, game: &Self::T) -> Self::EE {
        absolute_evaluate(game, false)
    }

    fn min_eval(&self) -> Self::EE {
        Eval::MIN
    }

    fn max_eval(&self) -> Self::EE {
        Eval::MAX
    }

    fn apply_move(&mut self, game: &mut Self::T, a: Self::Mo) {
        let mut mm = MoveLog::new();
        todo!();
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
        finder.next().map(|x| x.the_move)
    }
}
mod abab_simple {

    pub trait MoveFinder {
        type EE: PartialOrd + Ord + Copy;
        type T: Clone;
        type Mo: Copy;
        type Finder;
        fn eval(&mut self, game: &Self::T) -> Self::EE;

        fn min_eval(&self) -> Self::EE;
        fn max_eval(&self) -> Self::EE;

        fn apply_move(&mut self, game: &mut Self::T, a: Self::Mo);

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
        game_state: X::T,
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
            game_state: X::T,
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
                        let mut ga = game_state.clone();
                        self.data.apply_move(&mut ga, mo);
                        self.path.push(mo);
                        let (eval, move_list) = self.alpha_beta(depth - 1, ga, !maximizer);
                        self.path.pop();

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
                        let mut ga = game_state.clone();
                        self.data.apply_move(&mut ga, mo);
                        self.path.push(mo);
                        let (eval, move_list) = self.alpha_beta(depth - 1, ga, !maximizer);
                        self.path.pop();

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
            mut payload: &mut P,
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PossibleMove {
    pub the_move: moves::ActualMove,
    //pub mesh: MovementMesh,
    pub game_after_move: GameState,
}

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
