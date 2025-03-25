use crate::{
    board::MyWorld,
    moves::{get_num_attack, SpokeInfo},
};

use super::*;

pub type Eval = i64;

use hex::PASS_MOVE_INDEX;
use tinyvec::ArrayVec;
pub const MAX_NODE_VISIT: usize = 1_000_000;

pub fn calculate_secure_points(game: &GameState, world: &MyWorld) -> [i64; 2] {
    let reinforce = |team, game: &mut GameState| {
        let mut spoke = SpokeInfo::new(game);
        moves::update_spoke_info(&mut spoke, world, game);
        let fog = &mesh::small_mesh::SmallMesh::new();

        for &index in world.land_as_vec.iter() {
            let n = get_num_attack(&spoke, index);

            if let Some((h, m)) = game.factions.get_cell_inner(index) {
                if m == team && n[team] > h as i64 {
                    ActualMove(index).apply(team, game, fog, world, Some(&spoke));
                    let _s = spoke.process_move_better(ActualMove(index), team, world, game);
                }
            }
        }
    };

    let expand = |team, game: &mut GameState| {
        let fog = &mesh::small_mesh::SmallMesh::new();
        let mut progress = true;

        let mut spoke = SpokeInfo::new(game);
        moves::update_spoke_info(&mut spoke, world, game);

        while progress {
            progress = false;
            for &index in world.land_as_vec.iter() {
                if let Some(f) = game.playable(index, team, world, &spoke) {
                    if !f.is_suicidal() {
                        let _e = ActualMove(index).apply(team, game, fog, world, Some(&spoke));
                        let _s = spoke.process_move_better(ActualMove(index), team, world, game);
                        progress = true;
                    }
                }
            }
        }
    };

    let mut score = [0i64; 2];

    for team in [Team::White, Team::Black] {
        let mut game = game.clone();
        expand(!team, &mut game);
        reinforce(!team, &mut game);
        expand(team, &mut game);
        for &index in world.land_as_vec.iter() {
            if let Some((_, f)) = game.factions.get_cell_inner(index) {
                if f == team {
                    score[f] += 1;
                }
            }
        }
    }

    score
}

pub fn should_pass(
    a: &ai::Res,
    _team: Team,
    game_orig: &mut GameState,
    world: &MyWorld,
    //TODO pass in all history instead
    move_history: &MoveHistory,
) -> bool {
    //try with -sr-se--se--se----r

    if a.line.is_empty() {
        return true;
    }

    //If the user wants the game to end, just end the game.
    if let Some(&(f, _)) = move_history.inner.last() {
        if f.0 == hex::PASS_MOVE_INDEX {
            log!("AI:Passing since player wants the game to end");
            return true;
        }
    }

    // let points = calculate_secure_points(game_orig, world);

    // let mut winner = None;
    // for team in [Team::White, Team::Black] {
    //     if 2 * points[team] as usize > world.land_as_vec.len() {
    //         winner = Some(team);
    //         break;
    //     }
    // }

    // if let Some(_win) = winner {
    //     log!("Found a winner. {:?}. choosing to pass.", _win);

    //     return true;
    // }

    false
}

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
    //white maximizing
    //black minimizing
    pub fn absolute_evaluate(
        &mut self,
        game: &GameState,
        world: &board::MyWorld,
        spoke_info: &moves::SpokeInfo,
        _debug: bool,
    ) -> Eval {
        let mut overall_score = 0;
        let mut territory = 0;
        let mut overall_strength = 0;
        let mut contested = 0;
        for &index in world.land_as_vec.iter() {
            let num_attack = get_num_attack(spoke_info, index);

            let temp_score = if let Some((height, tt)) = game.factions.get_cell_inner(index) {
                let height = height as i64;
                if tt != Team::Neutral {
                    let s = num_attack[tt].saturating_sub(1).max(height) - num_attack[-tt];
                    overall_strength += s * tt.value();
                    territory += 1;
                    if num_attack[-tt] > height && num_attack[-tt] >= num_attack[tt] {
                        -tt.value()
                    } else {
                        tt.value()
                    }
                } else {
                    0
                }
            } else {
                if num_attack[Team::White] > num_attack[Team::Black] {
                    territory += 1;
                    1
                } else if num_attack[Team::Black] > num_attack[Team::White] {
                    territory += 1;
                    -1
                } else {
                    contested += 1;
                    0
                }
            };
            overall_score += temp_score;
        }
        overall_score * territory + 2 * overall_strength * contested
    }
}

pub enum Flag {
    Exact,
    UpperBound,
    LowerBound,
}
pub struct TTEntry {
    pv: ArrayVec<[moves::ActualMove; STACK_SIZE]>,
    flag: Flag,
    depth: usize,
    value: i64,
}

const STACK_SIZE: usize = 15;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Res {
    pub line: Vec<ActualMove>,
    pub eval: i64,
}

//TODO make the search depth be dependant on how many vacant cells there are!!!!
pub fn calculate_move(
    game: &mut GameState,
    fogs: &[mesh::small_mesh::SmallMesh; 2],
    world: &board::MyWorld,
    team: Team,
    move_history: &MoveHistory,
    zobrist: &Zobrist,
) -> ActualMove {
    let m = if let Some(mo) = iterative_deepening2(game, fogs, world, team, 9, zobrist) {
        if should_pass(&mo, team, game, world, move_history) {
            log!("Choosing to pass!");
            ActualMove(hex::PASS_MOVE_INDEX)
        } else {
            mo.line[0].clone()
        }
    } else {
        ActualMove(hex::PASS_MOVE_INDEX)
    };

    log!("Ai {:?} has selected move = {:?}", team, world.format(&m));
    m
}

pub fn iterative_deepening2(
    game: &GameState,
    fogs: &[mesh::small_mesh::SmallMesh; 2],
    world: &board::MyWorld,
    team: Team,
    len: usize,
    zobrist: &Zobrist,
) -> Option<Res> {
    let mut results = None;

    let mut table = std::collections::HashMap::new();
    let mut evaluator = Evaluator::default();

    let mut moves = vec![];

    let mut spoke_info = SpokeInfo::new(game);
    moves::update_spoke_info(&mut spoke_info, world, game);

    let mut nodes_visited_total = 0;
    let mut qui_nodes_visited_total = 0;
    let mut key = Key::from_scratch(&zobrist, game, world, team);
    let mut killer = KillerMoves::new(STACK_SIZE + 4 + 4);

    let mut game_orig = game.clone();
    let spoke_orig = spoke_info.clone();
    let key_orig = key.clone();

    let mut history_heur: Vec<_> = (0..board::TABLE_SIZE).map(|_| 0).collect();

    for depth in 0..len {
        let depth = depth + 1;
        log!("searching depth={}", depth);

        assert!(moves.is_empty());

        let mut aaaa = ai::AlphaBeta {
            ttable: &mut table,
            killer_moves: &mut killer,
            evaluator: &mut evaluator,
            world,
            moves: &mut moves,
            nodes_visited: &mut nodes_visited_total,
            qui_nodes_visited: &mut qui_nodes_visited_total,
            fogs,
            zobrist,
            history_heur: &mut history_heur,
        };

        let (res, mut mov) = aaaa.negamax(
            &mut game_orig,
            &mut key,
            &mut spoke_info,
            ABAB::new(),
            team,
            depth,
            true,
        );
        assert_eq!(spoke_info, spoke_orig);
        assert_eq!(key_orig, key);
        assert_eq!(&game_orig, game);

        if *aaaa.nodes_visited >= MAX_NODE_VISIT {
            log!("discarding depth {}", depth);
            break;
        }

        //alpha beta returns the main line with the first move at the end
        //reverse it so that the order is in the order of how they are played out.
        mov.reverse();

        log!(
            "regular nodes visited {} quiescence search nodes visited {} eval {} PV for depth {} :{:?}",
            *aaaa.nodes_visited,
            *aaaa.qui_nodes_visited,
            res * team.value(),
            depth,
            world.format(&mov.clone().to_vec())
        );

        if !mov.is_empty() {
            results = Some(Res {
                line: mov.to_vec(),
                eval: res,
            });
        } else {
            //if we can't find a solution now, not going to find it at higher depth i guess?
            break;
        }
    }

    log!(
        "total regular nodes visited={} total quiet visited={}",
        nodes_visited_total,
        qui_nodes_visited_total
    );

    results
}

struct AlphaBeta<'a> {
    ttable: &'a mut std::collections::HashMap<Key, TTEntry>,
    killer_moves: &'a mut KillerMoves,
    evaluator: &'a mut Evaluator,
    world: &'a board::MyWorld,
    moves: &'a mut Vec<ActualMove>,
    nodes_visited: &'a mut usize,
    qui_nodes_visited: &'a mut usize,
    fogs: &'a [mesh::small_mesh::SmallMesh; 2],
    zobrist: &'a Zobrist,
    history_heur: &'a mut [usize],
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

impl<'a> AlphaBeta<'a> {
    fn quiesance(
        &mut self,
        game: &mut GameState,
        key: &mut Key,
        spoke_info: &mut SpokeInfo,
        mut ab: ABAB,
        team: Team,
        depth: usize,
    ) -> Eval {
        if *self.nodes_visited >= MAX_NODE_VISIT {
            return abab::SMALL_VAL;
        }

        let stand_pat = team.value()
            * self
                .evaluator
                .absolute_evaluate(game, self.world, &spoke_info, false);

        if depth == 0 {
            return stand_pat;
        }

        *self.nodes_visited += 1;

        *self.qui_nodes_visited += 1;

        let mut best_value = stand_pat;

        if stand_pat >= ab.beta {
            return stand_pat;
        }
        if ab.alpha < stand_pat {
            ab.alpha = stand_pat
        }

        let captures = game.generate_loud_moves(self.world, team, &spoke_info);

        let start_move_index = self.moves.len();
        self.moves.push(ActualMove(hex::PASS_MOVE_INDEX));

        self.moves
            .extend(captures.inner.iter_ones().map(|x| ActualMove(x)));

        let end_move_index = self.moves.len();

        for _ in start_move_index..end_move_index {
            let cand = self.moves.pop().unwrap();

            let effect = cand.apply(team, game, &self.fogs[team], self.world, Some(&spoke_info));

            key.move_update(&self.zobrist, cand, team, &effect);

            let temp = spoke_info.process_move_better(cand, team, self.world, game);

            let eval = -self.quiesance(game, key, spoke_info, -ab.clone(), -team, depth - 1);

            spoke_info.undo_move(cand, &effect, team, self.world, game, temp);

            cand.undo(team, &effect, game);

            key.move_undo(&self.zobrist, cand, team, &effect);

            if eval >= ab.beta {
                self.moves.drain(start_move_index..);
                return eval;
            }
            if eval > best_value {
                best_value = eval
            }
            if eval > ab.alpha {
                ab.alpha = eval;
            }
        }
        return best_value;
    }

    fn negamax(
        &mut self,
        game: &mut GameState,
        key: &mut Key,
        spoke_info: &mut SpokeInfo,
        mut ab: ABAB,
        team: Team,
        depth: usize,
        update_tt: bool,
    ) -> (Eval, ArrayVec<[ActualMove; STACK_SIZE]>) {
        if *self.nodes_visited >= MAX_NODE_VISIT {
            return (abab::SMALL_VAL, tinyvec::array_vec!());
        }

        if depth == 0 {
            return (
                self.quiesance(game, key, spoke_info, ab, team, 2),
                tinyvec::array_vec!(),
            );
        }

        //null move pruning
        //https://www.chessprogramming.org/Null_Move_Pruning#Pseudocode
        {
            let r = 2;

            let mut ab2 = ab.clone();
            ab2.alpha = -ab.beta;
            ab2.beta = -(ab.beta - 1);
            let (eval, m) = self.negamax(
                game,
                key,
                spoke_info,
                ab2,
                -team,
                depth.saturating_sub(r),
                false,
            );
            let eval = -eval;

            if eval >= ab.beta {
                return (eval, m);
            }
        }

        let entry = self.ttable.get(&key);

        let alpha_orig = ab.alpha;

        //https://en.wikipedia.org/wiki/Negamax
        if let Some(entry) = entry {
            if entry.depth >= depth {
                match entry.flag {
                    Flag::Exact => {
                        entry.value;
                    }
                    Flag::UpperBound => {
                        ab.alpha = ab.alpha.max(entry.value);
                    }
                    Flag::LowerBound => {
                        ab.beta = ab.beta.min(entry.value);
                    }
                }
            }

            if ab.alpha >= ab.beta {
                return (entry.value, entry.pv.clone());
            }
        }

        *self.nodes_visited += 1;

        let loud_moves = game.generate_loud_moves(self.world, team, &spoke_info);

        let interest_moves = game.generate_interesting_moves(self.world, team, &spoke_info);

        let start_move_index = self.moves.len();

        self.moves.push(ActualMove(hex::PASS_MOVE_INDEX));
        self.moves.extend(game.generate_possible_moves_movement(
            self.world,
            team,
            &spoke_info,
            false,
        ));

        let end_move_index = self.moves.len();

        let moves = &mut self.moves[start_move_index..end_move_index];

        let move_value = |index: &ActualMove| {
            let index = index.0;

            if let Some(a) = entry {
                if let Some(p) = a.pv.last() {
                    if p.0 == index {
                        return 10_001;
                    }
                }
            }

            if loud_moves.inner[index] {
                return 10_000;
            }

            if interest_moves.inner[index] {
                return 8_000;
            }

            for (i, a) in self
                .killer_moves
                .get(usize::try_from(depth).unwrap())
                .iter()
                .enumerate()
            {
                if a.0 == index {
                    return 9_000 - i as isize;
                }
            }

            if index == hex::PASS_MOVE_INDEX {
                return 1;
            }

            3
        };

        //TODO https://www.chessprogramming.org/History_Heuristic
        moves.sort_unstable_by_key(|f| move_value(f));

        debug_assert!(!moves.is_empty());
        // log!(
        //     "Move about to look:{:?}",
        //     self.world.format(
        //         &moves
        //             .iter()
        //             .map(|x| ActualMove {
        //                 moveto: *x as usize
        //             })
        //             .collect::<Vec<_>>()
        //     )
        // );
        let mut beta_cutoff = false;
        //tc-s-d-re-srces-s--
        let mut ab_iter = ab.ab_iter();
        for _ in start_move_index..end_move_index {
            let cand = self.moves.pop().unwrap();

            let effect = cand.apply(team, game, &self.fogs[team], self.world, Some(&spoke_info));

            key.move_update(&self.zobrist, cand, team, &effect);

            let temp = spoke_info.process_move_better(cand, team, self.world, game);

            let (eval, mut m) = self.negamax(
                game,
                key,
                spoke_info,
                -ab_iter.clone_ab_values(),
                -team,
                depth - 1,
                true,
            );
            let eval = -eval;

            spoke_info.undo_move(cand, &effect, team, self.world, game, temp);
            // log!(
            //     "consid depth:{} {:?}:{:?}",
            //     depth,
            //     self.world.format(&cand),
            //     self.world.format(&m.clone().to_vec())
            // );

            cand.undo(team, &effect, game);

            key.move_undo(&self.zobrist, cand, team, &effect);
            m.push(cand);
            if !ab_iter.keep_going(m, eval) {
                beta_cutoff = true;
                //2007 without
                if !loud_moves.inner[cand.0] {
                    self.killer_moves.consider(depth, cand);

                    self.history_heur[cand.0] += depth * depth;
                }

                self.moves.drain(start_move_index..);
                break;
            }
        }

        assert_eq!(self.moves.len(), start_move_index);

        let (eval, m) = ab_iter.finish();

        let eval = if m.is_none() {
            assert!(beta_cutoff);

            //If we have no more moves, then we need to evaluate what happens,
            //if black plays a bunch of moves. at this point.
            // team.value()
            //     * self
            //         .evaluator
            //         .absolute_evaluate(game, self.world, &spoke_info, false)
            //team.value()*eval
            eval
        } else {
            eval
        };

        let m = m.unwrap_or_else(|| tinyvec::array_vec![]);

        if update_tt {
            //tc-s-d-re-srces-s--
            let flag = if eval <= alpha_orig {
                Flag::UpperBound
            } else if eval >= ab.beta {
                Flag::LowerBound
            } else {
                Flag::Exact
            };

            let entry = TTEntry {
                value: eval,
                depth,
                flag,
                pv: m.clone(),
            };

            self.ttable.insert(*key, entry);
        }

        (eval, m)
    }
}

use abab::ABAB;
mod abab {
    use std::ops::Neg;

    use super::*;
    #[derive(Clone)]
    pub struct ABAB {
        pub alpha: Eval,
        pub beta: Eval,
    }

    impl Neg for ABAB {
        type Output = ABAB;

        fn neg(self) -> Self::Output {
            ABAB {
                alpha: -self.beta,
                beta: -self.alpha,
            }
        }
    }

    pub struct ABIter<'a, T> {
        value: i64,
        a: &'a mut ABAB,
        mm: Option<T>,
    }

    impl<'a, T: Clone> ABIter<'a, T> {
        pub fn finish(self) -> (Eval, Option<T>) {
            (self.value, self.mm)
        }
        pub fn clone_ab_values(&self) -> ABAB {
            self.a.clone()
        }
        pub fn keep_going(&mut self, t: T, eval: Eval) -> bool {
            if self.mm.is_none() {
                self.value = eval;
                self.mm = Some(t)
            } else {
                if eval > self.value {
                    self.mm = Some(t);
                    self.value = eval;
                }
            }

            self.a.alpha = self.a.alpha.max(self.value);

            if self.a.alpha >= self.a.beta {
                false
            } else {
                true
            }
        }
    }

    impl ABAB {
        pub fn new() -> Self {
            ABAB {
                alpha: SMALL_VAL,
                beta: BIG_VAL,
            }
        }

        //ALWAYS MAXIMIZE
        pub fn ab_iter<T: Clone>(&mut self) -> ABIter<T> {
            let value = SMALL_VAL;
            ABIter {
                value,
                a: self,
                mm: None,
            }
        }
    }

    pub const SMALL_VAL: i64 = Eval::MIN + 10;
    pub const BIG_VAL: i64 = Eval::MAX - 10;
}
