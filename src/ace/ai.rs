use crate::mesh::bitfield::BitField;

use super::*;

pub type Eval = i64;
//const MATE: i64 = 1_000;

pub struct Evaluator {
    workspace: BitField,
    workspace2: BitField,
    workspace3: BitField,
}
impl Default for Evaluator {
    fn default() -> Self {
        Self {
            workspace: Default::default(),
            workspace2: Default::default(),
            workspace3: Default::default(),
        }
    }
}
impl Evaluator {
    // pub fn check_mate(&mut self, view:&GameState,team: ActiveTeam) -> Option<Eval> {
    //     match team{
    //         ActiveTeam::White=>{
    //             if view.factions.white.get(UnitType::King).count_ones(..)==0{
    //                 return Some(-MATE)
    //             }
    //         },
    //         ActiveTeam::Black=>{
    //             if view.factions.black.get(UnitType::King).count_ones(..)==0{
    //                 return Some(-MATE)
    //             }
    //         }
    //     }
    //     None
    // }
    //white maximizing
    //black minimizing
    pub fn absolute_evaluate(
        &mut self,
        view: &GameState,
        world: &board::MyWorld,
        _debug: bool,
    ) -> Eval {
        if let Some(k) = view.game_is_over(world) {
            match k {
                GameOver::WhiteWon => {
                    return {
                        //console_dbg!("Found white one");
                        i64::MAX
                    }
                }
                GameOver::BlackWon => return { i64::MIN },
                GameOver::Tie => {}
            }
        }

        // let ship_allowed = {
        //     let temp = &mut self.workspace;
        //     temp.clear();
        //     temp.union_with(&view.env.terrain.land);
        //     temp.toggle_range(..);
        //     let k = BitField::from_iter(world.get_game_cells().iter_mesh());

        //     temp.intersect_with(&k);
        //     temp
        // };

        let mut num_white = 0;
        for dir in OParity::all() {
            num_white += view
                .factions
                .get_board(dir)
                .get_all_team(ActiveTeam::White)
                .count_ones() as i64;
        }

        let mut num_black = 0;
        for dir in OParity::all() {
            num_black += view
                .factions
                .get_board(dir)
                .get_all_team(ActiveTeam::Black)
                .count_ones() as i64;
        }

        //TODO remove this allocation
        // let mut white_influence = view.factions.white.all_alloc();

        // let mut black_influence = view.factions.black.all_alloc();

        // let mut white_influence = BitField::from_iter(white_influence.iter_mesh());
        // let mut black_influence = BitField::from_iter(black_influence.iter_mesh());

        // doop(
        //     7,
        //     &mut black_influence,
        //     &mut white_influence,
        //     &ship_allowed,
        //     &mut self.workspace2,
        //     &mut self.workspace3,
        // );

        // let num_white_influence = white_influence.count_ones(..) as i64;
        // let num_black_influence = black_influence.count_ones(..) as i64;

        // let black_distance = view
        //     .factions
        //     .black
        //     .iter_mesh()
        //     .map(|a| a.to_cube().dist(&Axial::zero().to_cube()) as i64)
        //     .sum::<i64>();
        // let white_distance = view
        //     .factions
        //     .white
        //     .iter_mesh()
        //     .map(|a| a.to_cube().dist(&Axial::zero().to_cube()) as i64)
        //     .sum::<i64>();

        // //The AI will try to avoid the center.
        // //The more influlence is at stake, the more precious each piece is
        // (num_white_influence - num_black_influence) * 100
        //     + (-white_distance + black_distance) * 1
        (num_white - num_black)
    }
}

fn around(point: Axial) -> impl Iterator<Item = Axial> {
    point.to_cube().ring(1).map(|b| b.to_axial())
}

pub fn expand_mesh(mesh: &mut BitField, workspace: &mut BitField) {
    workspace.clear();
    workspace.union_with(mesh);

    for a in workspace.iter_mesh() {
        for b in around(a) {
            if mesh.valid_coord(b) {
                mesh.set_coord(b, true);
            }
        }
    }
}

fn doop(
    iteration: usize,
    black: &mut BitField,
    white: &mut BitField,
    allowed_cells: &BitField,
    cache: &mut BitField,
    mut cache2: &mut BitField,
) {
    black.intersect_with(allowed_cells);
    white.intersect_with(allowed_cells);
    if black.count_ones(..) == 0 && white.count_ones(..) == 0 {
        return;
    }

    // let mut cache2 = BitField::new();
    // let mut cache = BitField::new();

    for _i in 0..iteration {
        cache.clear();
        cache.union_with(black);
        expand_mesh(black, cache2);
        let black_changed = cache != black;
        cache.clear();
        cache.union_with(white);
        expand_mesh(white, cache2);
        let white_changed = cache != white;
        if !black_changed && !white_changed {
            break;
        }
        black.intersect_with(allowed_cells);
        white.intersect_with(allowed_cells);

        cache2.clear();
        let contested = &mut cache2;
        contested.union_with(black);
        contested.intersect_with(white);

        contested.toggle_range(..);

        black.intersect_with(&contested);
        white.intersect_with(&contested);
    }
}

struct TranspositionTable {
    a: std::collections::BTreeMap<u64, (moves::ActualMove, Eval)>,
}
impl TranspositionTable {
    pub fn update(&mut self, a: &GameState, m: moves::ActualMove, eval: Eval) {
        let k = a.hash_me();
        if let Some(foo) = self.a.get_mut(&k) {
            *foo = (m, eval);
        } else {
            self.a.insert(k, (m, eval));
        }
    }
    pub fn get(&self, a: &GameState) -> Option<&(moves::ActualMove, Eval)> {
        self.a.get(&a.hash_me())
    }
}

// //TODO use bump allocator!!!!!
// struct PrincipalVariation {
//     a: std::collections::BTreeMap<Vec<moves::ActualMove>, (moves::ActualMove, Eval)>,
// }
// impl PrincipalVariation {
//     pub fn get_best_prev_move(
//         &self,
//         path: &[moves::ActualMove],
//     ) -> Option<&(moves::ActualMove, Eval)> {
//         self.a.get(path)
//     }
//     pub fn get_best_prev_move_mut(
//         &mut self,
//         path: &[moves::ActualMove],
//     ) -> Option<&mut (moves::ActualMove, Eval)> {
//         self.a.get_mut(path)
//     }

//     pub fn update(&mut self, path: &[moves::ActualMove], aaa: &moves::ActualMove, eval: Eval) {
//         //if let Some(aaa) = &ret {
//         if let Some(foo) = self.get_best_prev_move_mut(path) {
//             *foo = (aaa.clone(), eval);
//         } else {
//             self.insert(path, aaa.clone(), eval);
//         }
//         //}
//     }
//     pub fn insert(&mut self, path: &[moves::ActualMove], m: moves::ActualMove, eval: Eval) {
//         self.a.insert(path.to_vec(), (m, eval));
//     }
// }

pub fn iterative_deepening(
    game: &GameState,
    world: &board::MyWorld,
    team: ActiveTeam,
) -> moves::ActualMove {
    let mut count = Counter { count: 0 };
    let mut results = Vec::new();

    let max_iterative_depth = 4; //6;
                                 //let max_depth = 2;

    let mut foo1 = TranspositionTable {
        a: std::collections::BTreeMap::new(),
    };
    let mut evaluator = Evaluator::default();

    //TODO stop searching if we found a game ending move.
    for d in 0..max_iterative_depth {
        let depth = d + 1;

        //TODO should this be outside the loop?
        let mut k = KillerMoves::new(max_iterative_depth + 4 + 4);

        let mut aaaa = ai::AlphaBeta {
            //table: &mut table,
            prev_cache: &mut foo1,
            calls: &mut count,
            path: &mut vec![],
            killer_moves: &mut k,
            max_ext: 0,
        };

        let mut kk = game.clone();
        let res = aaaa.alpha_beta(
            &mut kk,
            world,
            ABAB::new(),
            team,
            0,
            depth,
            0,
            &mut evaluator,
        );
        assert_eq!(&kk, game);

        let mov = foo1.get(game).cloned().unwrap();
        let res = EvalRet { mov, eval: res };

        let eval = res.eval;

        console_dbg!("searching", depth, eval);

        results.push(res);

        let short_target = if team == ActiveTeam::White {
            i64::MAX
        } else {
            i64::MIN
        };

        if eval == short_target {
            console_dbg!("found a mate");
            break;
        }

        //doop.poke(team, game.clone()).await;
    }

    console_dbg!(&results);

    //console_dbg!(count);
    //console_dbg!(&results);

    let true_eval = results.last().unwrap().clone();

    console_dbg!("Eval", true_eval.eval);

    // let mov = if let Some(a) = results
    //     .iter()
    //     .rev()
    //     .find(|a| a.eval == target_eval && a.mov != ActualMove::SkipTurn)
    // {
    //     a.clone()
    // } else {
    //     results.pop().unwrap()
    // };

    if team == ActiveTeam::White {
        results.retain(|e| e.eval > i64::MIN);
    } else {
        results.retain(|e| e.eval < i64::MAX);
    };

    let best_move = if let Some(fo) = results.last() {
        fo.clone()
    } else {
        true_eval
    };

    console_dbg!("move", best_move);
    best_move.mov.0
}

#[derive(Debug)]
struct Counter {
    count: u128,
}
impl Counter {
    pub fn add_eval(&mut self) {
        self.count += 1;
    }
}

struct AlphaBeta<'a> {
    //table: &'a mut LeafTranspositionTable,
    prev_cache: &'a mut TranspositionTable,
    calls: &'a mut Counter,
    path: &'a mut Vec<moves::ActualMove>,
    killer_moves: &'a mut KillerMoves,
    max_ext: usize,
}

struct KillerMoves {
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
struct EvalRet<T> {
    pub mov: T,
    pub eval: Eval,
}

impl<'a> AlphaBeta<'a> {
    fn alpha_beta(
        &mut self,
        game_after_move: &mut GameState,
        world: &board::MyWorld,
        mut ab: ABAB,
        team: ActiveTeam,
        depth: usize,
        max_depth: usize,
        ext: usize,
        evaluator: &mut Evaluator,
    ) -> Eval {
        self.max_ext = self.max_ext.max(ext);

        //TODO optimize this. we know who moved last turn. Only need to check one team.
        if let Some(f) = game_after_move.game_is_over(world) {
            return match f {
                GameOver::WhiteWon => i64::MAX,
                GameOver::BlackWon => i64::MIN,
                GameOver::Tie => 0,
            };
        }

        if depth >= max_depth
        /*+ 2*/
        {
            self.calls.add_eval();
            return evaluator.absolute_evaluate(game_after_move, world, false);
        }

        let mut quiet_position = true;
        let mut moves = vec![];

        game_after_move.for_all_moves_fast(team, world, |e, m, stat| {
            if e.destroyed_unit.is_some() {
                quiet_position = false;
            }

            if depth < max_depth {
                moves.push((m, stat.hash_me()));
            } else {
                if e.destroyed_unit.is_some() {
                    moves.push((m, stat.hash_me()));
                }
            }
        });

        // if depth >= max_depth && quiet_position {
        //     self.calls.add_eval();
        //     return evaluator.absolute_evaluate(game_after_move, world, false);
        // }

        if moves.is_empty() {
            return evaluator.absolute_evaluate(game_after_move, world, false);
        }

        let mut num_sorted = 0;

        for _ in 0..2 {
            let ind = if team == ActiveTeam::White {
                moves[num_sorted..]
                    .iter()
                    .enumerate()
                    .filter_map(|(i, x)| {
                        if let Some((_, k)) = self.prev_cache.a.get(&x.1) {
                            Some((i, k))
                        } else {
                            None
                        }
                    })
                    .max_by_key(|&(_, x)| x)
            } else {
                moves[num_sorted..]
                    .iter()
                    .enumerate()
                    .filter_map(|(i, x)| {
                        if let Some((_, k)) = self.prev_cache.a.get(&x.1) {
                            Some((i, k))
                        } else {
                            None
                        }
                    })
                    .min_by_key(|&(_, x)| x)
            };

            if let Some((ind, _)) = ind {
                moves.swap(num_sorted + ind, num_sorted);
                num_sorted += 1;
            }
        }

        for a in self.killer_moves.get(usize::try_from(depth).unwrap()) {
            if let Some((x, _)) = moves[num_sorted..]
                .iter()
                .enumerate()
                .find(|(_, x)| x.0 == *a)
            {
                moves.swap(num_sorted + x, num_sorted);
                num_sorted += 1;
            }
        }

        // if team==ActiveTeam::Dogs && num_sorted == 3{
        //     console_dbg!(team,moves.iter().map(|&(_,x)|{
        //         let mut num = None;
        //         //self.path.push(x.clone());
        //         if let Some((_, k)) = self.prev_cache.a.get(&x) {
        //             num = Some(*k);
        //         }
        //         num
        //     }).collect::<Vec<_>>());
        // }

        let moves: Vec<_> = moves.drain(..).map(|x| x.0).collect();

        let (eval, m) = if team == ActiveTeam::White {
            self.floopy(
                depth,
                max_depth,
                ext,
                evaluator,
                team,
                game_after_move,
                world,
                ab,
                abab::Maximizer,
                moves,
            )
        } else {
            self.floopy(
                depth,
                max_depth,
                ext,
                evaluator,
                team,
                game_after_move,
                world,
                ab,
                abab::Minimizer,
                moves,
            )
        };

        //assert_ne!(eval,i64::MAX,"invalid eval");
        //assert_ne!(eval,i64::MIN,"invalid eval");

        if let Some(kk) = m {
            self.prev_cache.update(game_after_move, kk, eval);
        }
        eval
    }

    fn floopy<D: ace::ai::abab::Doop>(
        &mut self,
        depth: usize,
        max_depth: usize,
        ext: usize,
        evaluator: &mut Evaluator,
        team: ActiveTeam,
        game_after_move: &mut GameState,
        world: &board::MyWorld,
        mut ab: ABAB,
        doop: D,
        moves: Vec<moves::ActualMove>,
    ) -> (i64, Option<moves::ActualMove>) {
        let mut ab_iter = ab.ab_iter(doop);

        assert!(!moves.is_empty());
        for cand in moves {
            //console_dbg!("Considering:",cand.original,cand.moveto,team,depth,cand.dir);
            let effect = {
                let j = cand.as_move();
                let k = j.apply(team, game_after_move, world);
                // let j = j
                //     .into_attack(cand.attackto)
                //     .apply(team, game_after_move, world, &k);
                // k.combine(j)
                k
            };

            self.path.push(cand);
            let eval = self.alpha_beta(
                game_after_move,
                world,
                ab_iter.clone_ab_values(),
                team.not(),
                depth + 1,
                max_depth,
                ext,
                evaluator,
            );

            let mov = self.path.pop().unwrap();
            {
                let k = mov.as_move();
                //k.undo(&effect.extra_effect, game_after_move)
                //k.u

                k.undo(team, &effect, game_after_move);
            }

            let keep_going = ab_iter.consider(&mov, eval);

            if !keep_going {
                self.killer_moves.consider(depth, mov);
                break;
            }
        }
        ab_iter.finish()
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

    pub trait Doop {
        fn maximizing(&self) -> bool;
    }
    pub struct Maximizer;
    impl Doop for Maximizer {
        fn maximizing(&self) -> bool {
            true
        }
    }
    pub struct Minimizer;
    impl Doop for Minimizer {
        fn maximizing(&self) -> bool {
            false
        }
    }

    pub struct ABIter<'a, T, D: Doop> {
        value: i64,
        a: &'a mut ABAB,
        mm: Option<T>,
        keep_going: bool,
        doop: D,
    }

    impl<'a, T: Clone, D: Doop> ABIter<'a, T, D> {
        pub fn finish(self) -> (Eval, Option<T>) {
            //assert!(!self.keep_going);
            //assert_ne!(self.value,i64::MAX);
            //assert_ne!(self.value,i64::MIN);
            (self.value, self.mm)
        }
        pub fn clone_ab_values(&self) -> ABAB {
            self.a.clone()
        }
        pub fn consider(&mut self, t: &T, eval: Eval) -> bool {
            //TODO should be less than or equal instead maybe?
            let mmm = if self.doop.maximizing() {
                eval > self.value
            } else {
                eval < self.value
            };
            if mmm {
                self.mm = Some(t.clone());
                self.value = eval;
            }

            let cond = if self.doop.maximizing() {
                eval > self.a.beta
            } else {
                eval < self.a.alpha
            };

            if cond {
                assert!(mmm);
                self.keep_going = false;
            }

            if self.doop.maximizing() {
                self.a.alpha = self.a.alpha.max(self.value);
            } else {
                self.a.beta = self.a.beta.min(self.value);
            }

            self.keep_going
        }
    }

    impl ABAB {
        pub fn new() -> Self {
            ABAB {
                alpha: Eval::MIN,
                beta: Eval::MAX,
            }
        }

        pub fn ab_iter<T: Clone, D: Doop>(&mut self, doop: D) -> ABIter<T, D> {
            let value = if doop.maximizing() {
                i64::MIN
            } else {
                i64::MAX
            };
            ABIter {
                value,
                a: self,
                mm: None,
                keep_going: true,
                doop,
            }
        }
    }
}
