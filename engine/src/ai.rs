use super::*;

use gloo_console::console_dbg;

pub type Eval = i64;
const MATE: i64 = 1_000_000;
use tinyvec::ArrayVec;
pub struct Evaluator {
    // workspace: BitField,
    // workspace2: BitField,
    // workspace3: BitField,
}
impl Default for Evaluator {
    fn default() -> Self {
        Self {
            // workspace: Default::default(),
            // workspace2: Default::default(),
            // workspace3: Default::default(),
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
        for index in world.get_game_cells().inner.iter_ones() {
            let mut num_white = 0;
            let mut num_black = 0;
            for (_, rest) in game.factions.iter_end_points(world, index) {
                if let Some((_, team)) = rest {
                    match team {
                        ActiveTeam::White => num_white += 1,
                        ActiveTeam::Black => num_black += 1,
                        ActiveTeam::Neutral => {}
                    }
                }
            }

            if let Some((height, tt)) = game.factions.get_cell_inner(index) {
                let height = height as i64;

                let curr_strength = match tt {
                    ActiveTeam::White => height.max(num_white - 1),
                    ActiveTeam::Black => -height.max(num_black - 1),
                    ActiveTeam::Neutral => 0,
                };

                strength += curr_strength;

                stack_count += 1;

                match tt {
                    ActiveTeam::White => {
                        if num_black > height {
                            score -= 1
                        } else {
                            score += 1
                        }
                    }
                    ActiveTeam::Black => {
                        if num_white > height {
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

// fn around(point: Axial) -> impl Iterator<Item = Axial> {
//     point.to_cube().ring(1).map(|b| b.to_axial())
// }

// pub fn expand_mesh(mesh: &mut BitField, workspace: &mut BitField) {
//     workspace.clear();
//     workspace.union_with(mesh);

//     for a in workspace.iter_mesh() {
//         for b in around(a) {
//             if mesh.valid_coord(b) {
//                 mesh.set_coord(b, true);
//             }
//         }
//     }
// }

struct TranspositionTable {
    a: std::collections::BTreeMap<u64, moves::ActualMove>,
}
impl TranspositionTable {
    pub fn update_inner(&mut self, k: u64, m: moves::ActualMove) {
        if let Some(foo) = self.a.get_mut(&k) {
            *foo = m;
        } else {
            self.a.insert(k, m);
        }
    }
    pub fn update(&mut self, a: &GameState, m: moves::ActualMove) {
        self.update_inner(a.hash_me(), m)
    }
    pub fn get(&self, a: &GameState) -> Option<&moves::ActualMove> {
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

const STACK_SIZE: usize = 5 + 4;

pub fn iterative_deepening(
    game: &GameState,
    world: &board::MyWorld,
    team: ActiveTeam,
    move_history: &MoveHistory,
) -> moves::ActualMove {
    let mut results = Vec::new();

    let num_iter = 4;
    //let max_depth = 2;

    let mut foo1 = TranspositionTable {
        a: std::collections::BTreeMap::new(),
    };
    let mut evaluator = Evaluator::default();

    let mut moves = vec![];
    let mut history = MoveHistory::new();

    //So that we can detect consecutive passes
    if let Some(f) = move_history.inner.last() {
        history.push(f.clone());
    }

    //TODO stop searching if we found a game ending move.
    for depth in [1, 2, 3] {
        gloo_console::info!(format!("searching depth={}", depth));

        let mut k = KillerMoves::new(num_iter + 4 + 4);
        assert!(moves.is_empty());
        //assert!(history.inner.is_empty());
        let mut history = history.clone();

        let mut aaaa = ai::AlphaBeta {
            prev_cache: &mut foo1,
            killer_moves: &mut k,
            evaluator: &mut evaluator,
            world,
            moves: &mut moves,
            history: &mut history,
        };

        let mut kk = game.clone();
        let (res, mov) = aaaa.alpha_beta(&mut kk, ABAB::new(), team, depth);
        assert_eq!(&kk, game);

        {
            let mut gg = kk.clone();
            let mut tt = team;
            let mut vals = vec![];
            for m in mov.iter().rev() {
                vals.push((gg.hash_me(), m.clone()));
                m.apply(tt, &mut gg, world);
                tt = tt.not();
            }
            for (v, k) in vals.into_iter().rev() {
                foo1.update_inner(v, k);
            }

            //Store the PV into the transposition table
            // let mut tt = team;
            // let mut gg = kk.clone();
            // let mut effects = vec![];
            // for m in mov.iter() {
            //     let effect1 = m.apply(tt, &mut gg, world);
            //     effects.push(effect1);
            //     tt = tt.not();
            // }

            // for (m, effect) in mov.iter().rev().zip(effects.iter().rev()) {
            //     tt = tt.not();
            //     m.undo(tt, effect, &mut gg);
            //     foo1.update(&gg, m.clone());

            // }
            // assert_eq!(gg, kk);
            gloo_console::info!(format!("transpotion table size={}", foo1.a.len()));
        }

        let mov = mov.last().unwrap().clone();

        // let Some(mov) = foo1.get(game).cloned() else {
        //     console_dbg!("Couldnt find a move???");
        //     panic!("OVER");
        // };
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

    console_dbg!("transpotiion table len=", foo1.a.len());

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

    m.mov
}

struct AlphaBeta<'a> {
    prev_cache: &'a mut TranspositionTable,
    killer_moves: &'a mut KillerMoves,
    evaluator: &'a mut Evaluator,
    world: &'a board::MyWorld,
    moves: &'a mut Vec<u8>,
    history: &'a mut MoveHistory,
}

struct KillerMoves {
    a: Vec<tinyvec::ArrayVec<[moves::ActualMove; 2]>>,
}

impl KillerMoves {
    pub fn new(a: usize) -> Self {
        let v = (0..a).map(|_| tinyvec::ArrayVec::new()).collect();
        Self { a: v }
    }
    pub fn get(&self, depth: usize) -> &[moves::ActualMove] {
        &self.a[depth]
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

pub fn evaluate_a_continuation(
    game: &GameState,
    world: &board::MyWorld,
    team_to_play: ActiveTeam,
    m: impl IntoIterator<Item = ActualMove>,
) -> Eval {
    let mut game = game.clone();
    let mut team = team_to_play;
    for cand in m {
        {
            let j = cand;
            j.apply(team, &mut game, world)
        };
        team = team.not();
    }

    Evaluator::default().absolute_evaluate(&game, world, false)
}

#[derive(Debug, Clone)]
struct EvalRet<T> {
    pub mov: T,
    pub eval: Eval,
}

impl<'a> AlphaBeta<'a> {
    fn quiesance(
        &mut self,
        game: &mut GameState,
        mut ab: ABAB,
        team: ActiveTeam,
        depth: usize,
    ) -> (Eval, ArrayVec<[ActualMove; STACK_SIZE]>) {
        if let Some(g) = game.game_is_over(self.world, team, self.history) {
            return (self.evaluator.cant_move(team), tinyvec::array_vec!());
        }

        if depth == 0 {
            return (
                self.evaluator.absolute_evaluate(game, self.world, false),
                tinyvec::array_vec![],
            );
        }

        let (_, captures, _) = game.generate_possible_moves_movement(self.world, None, team, false);

        let start_move_index = self.moves.len();

        self.moves.extend(captures.inner.iter_ones().map(|x| {
            let x: u8 = x.try_into().unwrap();
            x
        }));

        let end_move_index = self.moves.len();

        let moves = &mut self.moves[start_move_index..end_move_index];

        if moves.is_empty() {
            return (
                self.evaluator.absolute_evaluate(game, self.world, false),
                tinyvec::array_vec![],
            );
        }

        let mut ab_iter = ab.ab_iter(team.is_white());
        for _ in start_move_index..end_move_index {
            let cand = ActualMove {
                moveto: self.moves.pop().unwrap() as usize,
            };
            let effect = cand.apply(team, game, self.world);

            let (eval, m) = self.quiesance(game, ab_iter.clone_ab_values(), team.not(), depth - 1);

            cand.undo(team, &effect, game);

            if !ab_iter.consider((cand, m), eval) {
                self.moves.drain(start_move_index..);
                break;
            }
        }

        assert_eq!(self.moves.len(), start_move_index);
        //self.moves.drain(start_move_index..end_move_index);

        let (eval, j) = ab_iter.finish();
        if let Some((cand, mut m)) = j {
            m.push(cand);
            (eval, m)
        } else {
            (eval, tinyvec::array_vec![])
        }
    }
    fn alpha_beta(
        &mut self,
        game: &mut GameState,
        mut ab: ABAB,
        team: ActiveTeam,
        depth: usize,
    ) -> (Eval, ArrayVec<[ActualMove; STACK_SIZE]>) {
        if let Some(g) = game.game_is_over(self.world, team, self.history) {
            console_dbg!("found game over!!!!");
            return (self.evaluator.cant_move(team), tinyvec::array_vec!());
        }

        if depth == 0 {
            return self.quiesance(game, ab, team, 4);
        }

        let (all_moves, captures, reinfocements) =
            game.generate_possible_moves_movement(self.world, None, team, false);

        let start_move_index = self.moves.len();

        self.moves.extend(all_moves.inner.iter_ones().map(|x| {
            let x: u8 = x.try_into().unwrap();
            x
        }));

        let end_move_index = self.moves.len();

        let moves = &mut self.moves[start_move_index..end_move_index];

        //This is impossible since you can always pass
        assert!(!moves.is_empty());

        let move_value = |index: usize| {
            if captures.inner[index] {
                return 4;
            }

            if reinfocements.inner[index] {
                return 0;
            }

            if let Some(a) = self.prev_cache.get(&game) {
                if a.moveto == index {
                    return 1000;
                }
            }

            for (i, a) in self
                .killer_moves
                .get(usize::try_from(depth).unwrap())
                .iter()
                .enumerate()
            {
                if a.moveto == index {
                    return 800 - i as isize;
                }
            }

            // let spokes=game.factions.iter_end_points(self.world, index);
            // let sum=spokes.into_iter().fold(0,|acc,f|acc+f.0);

            1 //+sum as isize
        };

        moves.sort_by_cached_key(|&f| move_value(f as usize));

        // let dbg: Vec<_> = moves.iter().skip(10).map(|x| move_value(x)).rev().collect();
        // gloo::console::info!(format!("depth {} {:?}",depth,dbg));

        let mut ab_iter = ab.ab_iter(team.is_white());
        for _ in start_move_index..end_move_index {
            //moves.into_iter()
            let cand = ActualMove {
                moveto: self.moves.pop().unwrap() as usize,
            };
            let effect: move_build::MoveEffect = cand.apply(team, game, self.world);
            self.history.push((cand, effect));

            let (eval, m) = self.alpha_beta(game, ab_iter.clone_ab_values(), team.not(), depth - 1);

            let (cand, effect) = self.history.inner.pop().unwrap();

            cand.undo(team, &effect, game);

            if !ab_iter.consider((cand.clone(), m), eval) {
                if effect.destroyed_unit.is_none() {
                    self.killer_moves.consider(depth, cand.clone());
                }

                self.prev_cache.update(game, cand);

                self.moves.drain(start_move_index..);
                break;
            }
        }

        assert_eq!(self.moves.len(), start_move_index);

        let (eval, m) = ab_iter.finish();

        if let Some((cand, mut m)) = m {
            m.push(cand);
            (eval, m)
        } else {
            (eval, tinyvec::array_vec![])
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

    pub struct ABIter<'a, T> {
        value: i64,
        a: &'a mut ABAB,
        mm: Option<T>,
        keep_going: bool,
        maximizing: bool,
    }

    impl<'a, T: Clone> ABIter<'a, T> {
        pub fn finish(self) -> (Eval, Option<T>) {
            (self.value, self.mm)
        }
        pub fn clone_ab_values(&self) -> ABAB {
            self.a.clone()
        }
        pub fn consider(&mut self, t: T, eval: Eval) -> bool {
            //TODO monomorphize internally for maximizing and minimizing.

            //TODO should be less than or equal instead maybe?
            let mmm = if self.maximizing {
                eval > self.value
            } else {
                eval < self.value
            };
            if mmm {
                self.mm = Some(t);
                self.value = eval;
            }

            let cond = if self.maximizing {
                eval > self.a.beta
            } else {
                eval < self.a.alpha
            };

            if cond {
                assert!(mmm);
                self.keep_going = false;
            }

            if self.maximizing {
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

        pub fn ab_iter<T: Clone>(&mut self, maximizing: bool) -> ABIter<T> {
            let value = if maximizing { i64::MIN } else { i64::MAX };
            ABIter {
                value,
                a: self,
                mm: None,
                keep_going: true,
                maximizing,
            }
        }
    }
}
