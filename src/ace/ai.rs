use crate::mesh::bitfield::BitField;

use super::*;

pub type Eval = i64;
const MATE: i64 = 1_000_000;

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
    pub fn cant_move(&mut self, team: ActiveTeam) -> Eval {
        match team {
            ActiveTeam::White => -MATE,
            ActiveTeam::Black => MATE,
            ActiveTeam::Neutral => unreachable!(),
        }
    }
    //white maximizing
    //black minimizing
    pub fn absolute_evaluate(
        &mut self,
        game: &GameState,
        world: &board::MyWorld,
        _debug: bool,
    ) -> Eval {
        let mut score = 0;
        let mut stack_count = 0;
        let mut territory_count = 0;
        let mut strength = 0;
        let mut contested = 0;
        let mut unseen = 0;
        for unit in world.get_game_cells().iter_mesh() {
            let mut num_white = 0;
            let mut num_black = 0;
            for (_, rest) in game.iter_end_points(world, unit) {
                if let Some((_, team)) = rest {
                    match team {
                        ActiveTeam::White => num_white += 1,
                        ActiveTeam::Black => num_black += 1,
                        ActiveTeam::Neutral => {}
                    }
                }
            }

            if let Some((val, tt)) = game.factions.get_cell(unit) {
                let val = val as i64;

                let curr_strength = match tt {
                    ActiveTeam::White => val.max(num_white - 1) + 2,
                    ActiveTeam::Black => -(val.max(num_black - 1)) + 2,
                    ActiveTeam::Neutral => 0,
                };

                strength += curr_strength;

                stack_count += 1;

                match tt {
                    ActiveTeam::White => {
                        if num_black > val {
                            score -= 1
                        } else {
                            score += 1
                        }
                    }
                    ActiveTeam::Black => {
                        if num_white > val {
                            score += 1
                        } else {
                            score -= 1
                        }
                    }
                    ActiveTeam::Neutral => {}
                }
            } else {
                let ownership = num_white - num_black;

                if ownership > 0 {
                    score += ownership;
                    territory_count += 1;
                } else if ownership < 0 {
                    score += ownership;
                    territory_count += 1;
                } else {
                    //The diff is zero, so if num_white is positive, so too must be black indicating they are contesting.
                    if num_white > 0 {
                        contested += 1
                    } else {
                        unseen += 1;
                    }
                }
            };
        }

        (stack_count + territory_count) * score + (unseen + contested) * strength
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

    let num_iter = 2;
    //let max_depth = 2;

    let mut foo1 = TranspositionTable {
        a: std::collections::BTreeMap::new(),
    };
    let mut evaluator = Evaluator::default();

    //TODO stop searching if we found a game ending move.
    for depth in [1, 2, 3, 4,5,6] {
        //let depth=2;

        //let depth = d + 1;
        console_dbg!("searching", depth);

        //TODO should this be outside the loop?
        let mut k = KillerMoves::new(num_iter + 4 + 4);

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

        let Some(mov) = foo1.get(game).cloned() else {
            console_dbg!("Couldnt find a move???");
            panic!("OVER");
        };
        let res = EvalRet { mov, eval: res };

        let eval = res.eval;
        //console_dbg!(eval);

        results.push(res);

        if eval.abs() == MATE {
            console_dbg!("found a mate");
            //break;
        }

        //doop.poke(team, game.clone()).await;
    }

    //console_dbg!(count);
    //console_dbg!(&results);

    let _target_eval = results.last().unwrap().eval;
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

    let m = mov;

    console_dbg!("AI evaluation::", m.mov, m.eval);

    m.mov.0
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

impl GameState {
    pub fn evaluate_a_continuation(
        &self,
        world: &board::MyWorld,
        team_to_play: ActiveTeam,
        m: impl IntoIterator<Item = ActualMove>,
    ) -> Eval {
        let mut game = self.clone();
        let mut team = team_to_play;
        for cand in m {
            {
                let j = cand.as_move();
                j.apply(team, &mut game, world)
            };
            team = team.not();

            // {
            //     move_build::MovePhase { moveto: cand.moveto }.undo(team, &effect, game);
            // }
        }

        Evaluator::default().absolute_evaluate(&game, world, false)
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

        if depth >= max_depth {
            if game_after_move.hash_me() == 12916878750629778790 {
                console_dbg!(
                    "FOO",
                    evaluator.absolute_evaluate(game_after_move, world, false)
                );
            }
            self.calls.add_eval();
            return evaluator.absolute_evaluate(game_after_move, world, false);
        }

        //let mut quiet_position = true;
        let mut moves = vec![];

        game_after_move.for_all_moves_fast(team, world, |m, stat| {
            // if e.destroyed_unit.is_some() {
            //     quiet_position = false;
            // }

            moves.push((m, stat.hash_me()));

            // if depth < max_depth {
            //     moves.push((m, stat.hash_me()));
            // } else {
            //     if e.destroyed_unit.is_some() {
            //         moves.push((m, stat.hash_me()));
            //     }
            // }
        });

        // if depth >= max_depth && quiet_position {
        //     self.calls.add_eval();
        //     return evaluator.absolute_evaluate(game_after_move, world, false);
        // }

        if moves.is_empty() {
            return evaluator.cant_move(team);
        }

        let mut num_sorted = 0;

        for _ in 0..2 {
            let ind = match team {
                ActiveTeam::White => moves[num_sorted..]
                    .iter()
                    .enumerate()
                    .filter_map(|(i, x)| {
                        if let Some((_, k)) = self.prev_cache.a.get(&x.1) {
                            Some((i, k))
                        } else {
                            None
                        }
                    })
                    .max_by_key(|&(_, x)| x),
                ActiveTeam::Black => moves[num_sorted..]
                    .iter()
                    .enumerate()
                    .filter_map(|(i, x)| {
                        if let Some((_, k)) = self.prev_cache.a.get(&x.1) {
                            Some((i, k))
                        } else {
                            None
                        }
                    })
                    .min_by_key(|&(_, x)| x),
                ActiveTeam::Neutral => {
                    unreachable!()
                }
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

        let moves: Vec<_> = moves.drain(..).map(|x| x.0).collect();

        let (eval, m) = match team {
            ActiveTeam::White => self.floopy(
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
            ),
            ActiveTeam::Black => self.floopy(
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
            ),
            ActiveTeam::Neutral => {
                unreachable!()
            }
        };

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
        for cand in moves {
            let effect = {
                let j = cand.as_move();
                j.apply(team, game_after_move, world)
            };

            self.path.push(cand);

            if game_after_move.hash_me() == 12916878750629778790 {
                console_dbg!("!!!!!!!");
            }
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

            if game_after_move.hash_me() == 12916878750629778790 {
                console_dbg!("CONSIDERING THIS STATE!!!!", depth, eval);
            }

            let mov = self.path.pop().unwrap();
            {
                //let k = mov.as_extra();
                move_build::MovePhase { moveto: mov.moveto }.undo(team, &effect, game_after_move);
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
use mesh::small_mesh::SmallMesh;
use moves::EndPoints;
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
