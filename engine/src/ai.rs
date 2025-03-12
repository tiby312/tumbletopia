use crate::{
    board::MyWorld,
    moves::{get_num_attack, SpokeInfo},
};

use super::*;

pub type Eval = i64;
//const MATE: i64 = 1_000_000;
use tinyvec::ArrayVec;
pub const MAX_NODE_VISIT: usize = 3000;

pub fn should_pass(
    a: &ai::Res,
    _team: Team,
    _game_orig: &mut GameState,
    _world: &MyWorld,
    //TODO pass in all history instead
    _move_history: &MoveHistory,
) -> bool {
    //try with -sr-se--se--se----r

    if a.line.is_empty() {
        return true;
    }

    // let mut game = game_orig.clone();
    // let score_before = game.score(world);

    // // let k = ActualMove {
    // //     moveto: moves::PASS_MOVE_INDEX,
    // // };
    // //let _e = k.apply(team, &mut game, &SmallMesh::new(), world);

    // let fogs = std::array::from_fn(|_| mesh::small_mesh::SmallMesh::new());
    // let foo = iterative_deepening2(&game, &fogs, world, team.not(), 3);

    // let mut tt = team.not();
    // if let Some(foo) = foo {
    //     let principal_variation: Vec<_> = foo
    //         .line
    //         .iter()
    //         .map(|x| {
    //             let res = move_build::to_letter_coord(&mesh::small_mesh::inverse(x.moveto), world);
    //             format!("{}{}", res.0, res.1)
    //         })
    //         .collect();
    //     console_dbg!("should pass", principal_variation);

    //     for a in foo.line {
    //         let _ = a.apply(tt, &mut game, &SmallMesh::new(), world);
    //         tt = tt.not();

    //         let score_after = game.score(world);

    //         console_dbg!(score_before, score_after);

    //         if score_after != score_before {
    //             return false;
    //         }
    //     }
    // }

    // //If we do pass, what are the opponents best moves. And does it change the score?

    // let mut moves_to_use=a.line.clone();
    // let mut team_counter=team;
    // let mut game = game_orig.clone();

    // let opponent_just_passed = if let Some((k, e)) = move_history.inner.last() {
    //     console_dbg!("last move",move_build::to_letter_coord(&small_mesh::inverse(k.moveto),world));
    //     k.undo(team.not(), e, &mut game);
    //     team_counter=team_counter.not();
    //     moves_to_use.insert(0,k.clone());

    //     console_dbg!(moves_to_use,team_counter);
    //     k.moveto == moves::PASS_MOVE_INDEX
    // } else {
    //     false
    // };

    // //TODO remove this clone

    // let score_before = game.score(world);

    // let fog = SmallMesh::new();

    // for (i,aa) in moves_to_use.into_iter().enumerate(){
    //     let _effect = aa.apply(team_counter, &mut game, &fog, world);
    //     let s = game.score(world);

    //     if i==0{
    //         assert_eq!(&game,game_orig);
    //     }
    //     //dont pass if we forsee any fluctuation in the score
    //     if s != score_before {
    //         return false;
    //     }
    //     // if let Some((_, fa)) = effect.destroyed_unit {
    //     //     if fa != team {
    //     //         console_dbg!("Not passing because there are captures in principal variation");
    //     //         return false;
    //     //     }
    //     // }
    //     team_counter = team_counter.not();
    // }
    // let score_after = game.score(world);

    // console_dbg!(score_before, score_after);

    // if opponent_just_passed {
    //     match team {
    //         ActiveTeam::White => {
    //             if score_before.white > score_before.black {
    //                 return true;
    //             }
    //         }
    //         ActiveTeam::Black => {
    //             if score_before.black > score_before.white {
    //                 return true;
    //             }
    //         }
    //         ActiveTeam::Neutral => {}
    //     }
    // }

    // //let a = &a.line[0];
    // //let effect = a.apply(team, game, &fog, world);

    // let res = if score_after == score_before {
    //     console_dbg!("I WANT TO PASS");
    //     true
    // } else {
    //     false
    // };
    // //a.undo(team, &effect, game);
    // res
    // //false

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
    // pub fn process_game_over(&mut self, a: unit::GameOver) -> Eval {
    //     match a {
    //         unit::GameOver::WhiteWon => MATE,
    //         unit::GameOver::BlackWon => -MATE,
    //         unit::GameOver::Tie => 0,
    //     }
    // }

    //white maximizing
    //black minimizing
    pub fn absolute_evaluate(
        &mut self,
        game: &GameState,
        world: &board::MyWorld,
        spoke_info: &moves::SpokeInfo,
        _debug: bool,
    ) -> Eval {
        let mut total_foo = 0;
        let strength_parity = 0;
        for index in world.get_game_cells().inner.iter_ones() {
            let num_attack = get_num_attack(spoke_info, index);

            // let mut num_attack: [i64; 2] = [0, 0];

            // for (_, rest) in game.factions.iter_end_points(world, index) {
            //     if let Some((_, team)) = rest {
            //         if team == Team::Neutral {
            //             continue;
            //         }
            //         num_attack[team] += 1;
            //     }
            // }

            let temp_score = if let Some((height, tt)) = game.factions.get_cell_inner(index) {
                let height = height as i64;
                if tt != Team::Neutral {
                    // strength_parity +=
                    //     6i64 - ((height + 1).max(num_attack[tt]) - num_attack[tt.not()]).abs();
                    // if num_attack[-tt] > height && num_attack[-tt] >= num_attack[tt] {
                    //     strength_parity += -tt.value();
                    // } else {
                    //     strength_parity += tt.value();
                    // }
                    //}
                    if num_attack[-tt] > height && num_attack[-tt] >= num_attack[tt] {
                        -tt.value()
                    } else {
                        tt.value()
                    }
                } else {
                    0
                }

                //tt.value()
            } else {
                if num_attack[Team::White] > num_attack[Team::Black] {
                    1
                } else if num_attack[Team::Black] > num_attack[Team::White] {
                    -1
                } else {
                    0
                }
            };
            total_foo += temp_score;
        }
        total_foo * 100000 + strength_parity
    }
}

pub enum Flag {
    Exact,
    UpperBound,
    LowerBound,
}
pub struct TTEntry {
    //mov: Option<moves::ActualMove>,
    pv: ArrayVec<[moves::ActualMove; STACK_SIZE]>,
    flag: Flag,
    depth: usize,
    value: i64,
}

struct TranspositionTable {
    a: std::collections::HashMap<Key, TTEntry>,
}
impl TranspositionTable {
    // pub fn update_inner(&mut self, k: Key, m: TTEntry) {
    //     if let Some(foo) = self.a.get_mut(&k) {
    //         *foo = m;
    //     } else {
    //         self.a.insert(k, m);
    //     }
    // }
    // pub fn update(&mut self, a: &Key, m: TTEntry) {
    //     self.update_inner(a, m)
    // }
    // pub fn get(&self, a: &GameState) -> Option<&TTEntry> {
    //     self.a.get(&a.hash_me())
    // }
}

const STACK_SIZE: usize = 8;

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
) -> ActualMove {
    if let Some(mo) = iterative_deepening2(game, fogs, world, team, 8) {
        if should_pass(&mo, team, game, world, move_history) {
            log!("Choosing to pass!");
            ActualMove {
                moveto: hex::PASS_MOVE_INDEX,
            }
        } else {
            mo.line[0].clone()
        }
    } else {
        ActualMove {
            moveto: hex::PASS_MOVE_INDEX,
        }
    }
}

pub fn iterative_deepening2(
    game: &GameState,
    fogs: &[mesh::small_mesh::SmallMesh; 2],
    world: &board::MyWorld,
    team: Team,
    len: usize, //move_history: &MoveHistory,
) -> Option<Res> {
    let mut results = None; // = Vec::new();

    let mut table = TranspositionTable {
        a: std::collections::HashMap::new(),
    };
    let mut evaluator = Evaluator::default();

    let mut moves = vec![];
    // let mut history = MoveHistory::new();

    // //So that we can detect consecutive passes
    // if let Some(f) = move_history.inner.last() {
    //     history.push(f.clone());
    // }

    let zobrist = &Zobrist::new();

    let mut ss = SpokeInfo::new(game);
    moves::update_spoke_info(&mut ss, world, game);

    let mut nodes_visited_total = 0;

    let key = Key::from_scratch(&zobrist, game, world);

    //TODO stop searching if we found a game ending move.
    for depth in 0..len {
        let depth = depth + 1;
        log!("searching depth={}", depth);

        //3 = num iter
        let mut killer = KillerMoves::new(3 + 4 + 4);
        assert!(moves.is_empty());

        //let mut history = history.clone();

        let mut aaaa = ai::AlphaBeta {
            prev_cache: &mut table,
            killer_moves: &mut killer,
            evaluator: &mut evaluator,
            world,
            moves: &mut moves,
            nodes_visited: &mut nodes_visited_total,
            fogs,
            zobrist,
        };

        let mut kk = game.clone();

        let mut ss2 = ss.clone();
        let (res, mut mov) = aaaa.negamax(&mut kk, key, &mut ss, ABAB::new(), team, depth, true);
        assert_eq!(ss, ss2);
        assert_eq!(&kk, game);

        // if *aaaa.nodes_visited >= MAX_NODE_VISIT {
        //     log!("discarding depth {}", depth);
        //     break;
        // }

        //alpha beta returns the main line with the first move at the end
        //reverse it so that the order is in the order of how they are played out.
        mov.reverse();

        // {
        //     //Update the transposition table in the right order
        //     let mut gg = kk.clone();
        //     let mut tt = team;

        //     let mut ggg=vec!();
        //     for (i,m) in mov.iter().enumerate(){
        //         m.apply(tt,&mut gg,&fogs[tt.index()],world);

        //         let entry=TTEntry{
        //             flag:Flag::Exact,
        //             pv:tinyvec::ArrayVec::from_iter(mov[i..].iter().cloned()),
        //             //TODO correct???
        //             depth:depth,
        //             value:res
        //         };

        //         ggg.push((gg.hash_me(),entry));

        //         tt=tt.not();
        //     }

        //     for (a,b) in ggg.into_iter().rev(){
        //         table.update_inner(a, b);
        //     }
        // }

        log!(
            "num visited {} eval {} PV for depth {} :{:?}",
            *aaaa.nodes_visited,
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

    log!("nodes visited={}", nodes_visited_total);

    // console_dbg!("transpotiion table len=", table.a.len());

    // let mov = results.unwrap();

    // let m = mov;

    // console_dbg!("AI evaluation::", m.mov, m.eval);

    // (m.mov, m.eval)
    // Res { line: mov, eval: () }
    results
}

struct AlphaBeta<'a> {
    prev_cache: &'a mut TranspositionTable,
    killer_moves: &'a mut KillerMoves,
    evaluator: &'a mut Evaluator,
    world: &'a board::MyWorld,
    moves: &'a mut Vec<ActualMove>,
    nodes_visited: &'a mut usize, //history: &'a mut MoveHistory,
    fogs: &'a [mesh::small_mesh::SmallMesh; 2],
    zobrist: &'a Zobrist,
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

// pub fn evaluate_a_continuation(
//     game: &GameState,
//     world: &board::MyWorld,
//     team_to_play: ActiveTeam,
//     m: impl IntoIterator<Item = ActualMove>,
// ) -> Eval {
//     let mut game = game.clone();
//     let mut team = team_to_play;
//     for cand in m {
//         {
//             let j = cand;
//             j.apply(team, &mut game, world)
//         };
//         team = team.not();
//     }

//     Evaluator::default().absolute_evaluate(&game, world, false)
// }

impl<'a> AlphaBeta<'a> {
    // fn quiesance(
    //     &mut self,
    //     game: &mut GameState,
    //     fogs: &[SmallMesh; 2],
    //     mut ab: ABAB,
    //     team: Team,
    //     depth: usize,
    // ) -> (Eval, ArrayVec<[ActualMove; STACK_SIZE]>) {
    //     *self.nodes_visited += 1;

    //     // if let Some(g) = game.game_is_over(self.world, team, self.history) {
    //     //     return (self.evaluator.process_game_over(g), tinyvec::array_vec!());
    //     // }
    //     let mut spoke_info = moves::SpokeInfo::new(game);
    //     moves::update_spoke_info(&mut spoke_info, self.world, game);

    //     if depth == 0 {
    //         return (
    //             self.evaluator
    //                 .absolute_evaluate(game, self.world, &spoke_info, false),
    //             tinyvec::array_vec![],
    //         );
    //     }

    //     let captures = game.generate_loud_moves(self.world, team, &spoke_info);

    //     let start_move_index = self.moves.len();

    //     self.moves.extend(captures.inner.iter_ones().map(|x| {
    //         let x: u8 = x.try_into().unwrap();
    //         x
    //     }));

    //     let end_move_index = self.moves.len();

    //     let moves = &mut self.moves[start_move_index..end_move_index];

    //     if moves.is_empty() {
    //         return (
    //             self.evaluator
    //                 .absolute_evaluate(game, self.world, &spoke_info, false),
    //             tinyvec::array_vec![],
    //         );
    //     }

    //     let mut ab_iter = ab.ab_iter(team.is_white());
    //     for _ in start_move_index..end_move_index {
    //         let cand = ActualMove {
    //             moveto: self.moves.pop().unwrap() as usize,
    //         };
    //         let effect = cand.apply(team, game, &fogs[team.index()], self.world);

    //         let (eval, m) =
    //             self.quiesance(game, fogs, ab_iter.clone_ab_values(), team.not(), depth - 1);

    //         cand.undo(team, &effect, game);

    //         if !ab_iter.consider((cand, m), eval) {
    //             self.moves.drain(start_move_index..);
    //             break;
    //         }
    //     }

    //     assert_eq!(self.moves.len(), start_move_index);
    //     //self.moves.drain(start_move_index..end_move_index);

    //     let (eval, j) = ab_iter.finish();
    //     if let Some((cand, mut m)) = j {
    //         m.push(cand);
    //         (eval, m)
    //     } else {
    //         (eval, tinyvec::array_vec![])
    //     }
    // }

    fn negamax(
        &mut self,
        game: &mut GameState,
        mut key: Key,
        spoke_info: &mut SpokeInfo,
        mut ab: ABAB,
        team: Team,
        depth: usize,
        update_tt: bool,
    ) -> (Eval, ArrayVec<[ActualMove; STACK_SIZE]>) {
        // 1107 with TT
        // 1117
        // if *self.nodes_visited >= MAX_NODE_VISIT {
        //     return (SMALL_VAL, tinyvec::array_vec!());
        // }

        // let mut spoke_info = moves::SpokeInfo::new(game);
        // moves::update_spoke_info(&mut spoke_info, self.world, game);

        if depth == 0 {
            return (
                team.value()
                    * self
                        .evaluator
                        .absolute_evaluate(game, self.world, &spoke_info, false),
                tinyvec::array_vec![],
            );
            //return self.quiesance(game, fogs, ab, team, /*4*/ 4);
        }

        //null move pruning
        {
            let r = 2;

            //pos.make_null_move();
            //int v = -search(pos, -beta, -(beta - 1), depth - r);
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

            //pos.undo_null_move();
            if eval >= ab.beta {
                //log!("NULL MOVE PRUNINGGG");
                return (eval, m);
            }
        }

        let entry = self
            .prev_cache
            .a
            .get(&key);

        //https://en.wikipedia.org/wiki/Negamax
        let alpha_orig = ab.alpha;
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
                //log!("Found a hit!");

                return (entry.value, entry.pv.clone());
            }
        }

        *self.nodes_visited += 1;

        let (all_moves, captures, _reinfocements) =
            game.generate_possible_moves_movement(self.world, team, &spoke_info);

        let loud_moves = game.generate_loud_moves(self.world, team, &spoke_info);

        let start_move_index = self.moves.len();

        self.moves.extend(
            all_moves
                .inner
                .iter_ones()
                .map(|x| ActualMove { moveto: x }),
        );

        let end_move_index = self.moves.len();

        let moves = &mut self.moves[start_move_index..end_move_index];

        // if moves.is_empty() {
        //     return (
        //         team.value()
        //             * self
        //                 .evaluator
        //                 .absolute_evaluate(game, self.world, &spoke_info, false),
        //         tinyvec::array_vec![],
        //     );
        // }

        //let fa = self.prev_cache.get(&game);
        let move_value = |index: &ActualMove| {
            let index = index.moveto;

            if let Some(a) = entry {
                if let Some(p) = a.pv.last() {
                    if p.moveto == index {
                        return 1000;
                    }
                }
            }

            //TODO actually tes this one
            if captures.inner[index] {
                return 11;
            }

            if loud_moves.inner[index] {
                return 10;
            }

            for (i, a) in self
                .killer_moves
                .get(usize::try_from(depth).unwrap())
                .iter()
                .enumerate()
            {
                if a.moveto == index {
                    return 4 - i as isize;
                }
            }

            1
        };

        moves.sort_unstable_by_key(|f| move_value(f));

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

        // alpha beta not workinggggggg
        //tc-s-d-re-srces-s--
        let mut ab_iter = ab.ab_iter();
        for _ in start_move_index..end_move_index {
            let cand = self.moves.pop().unwrap();

            let mut gg = game.clone();

            let effect = cand.apply(team, game, &self.fogs[team.index()], self.world);

            key.move_update(&self.zobrist, cand.clone(), team, &effect);

            spoke_info.process_move(cand.clone(), team, self.world, game);
            // {
            //     let mut kk = spoke_info.clone();
            //     kk.process_move(cand.clone(), team, self.world, game);
            //     kk.undo_move(cand.clone(), effect.clone(), team, self.world, game);
            //     kk.process_move(cand.clone(), team, self.world, game);

            //     cand.apply(team, &mut gg, &self.fogs[team], self.world);
            //     let mut kk2 = SpokeInfo::new(&gg);
            //     moves::update_spoke_info(&mut kk2, self.world, &mut gg);
            //     //println!("ok");

            //     for index in self.world.get_game_cells().inner.iter_ones() {
            //         //let index=cand.clone().moveto;
            //         for dir in hex::HDir::all() {
            //             let a = kk.get(index, dir);
            //             let b = kk2.get(index, dir);

            //             assert_eq!(a, b);
            //         }

            //         for (i, (_, rest)) in game
            //             .factions
            //             .iter_end_points(self.world, index)
            //             .iter()
            //             .enumerate()
            //         {
            //             //let hexdir=hex::HDir::from(i as u8);

            //             if let Some(rest) = rest {
            //                 // let a=kk.get(rest.index,dir);
            //                 // let b=kk2.get(rest.index,dir);
            //                 for dir in hex::HDir::all() {
            //                     let a = kk.get(rest.index, dir);
            //                     let b = kk2.get(rest.index, dir);

            //                     assert_eq!(
            //                         a,
            //                         b,
            //                         "{:?}",
            //                         self.world.format(&ActualMove { moveto: rest.index })
            //                     );
            //                 }
            //             }
            //         }
            //     }

            //     //let j=kk.data & kk2.data;
            //     //println!("{:?}",j.iter_ones().map())
            //     // for a in kk.data.iter(){

            //     // }
            //     assert_eq!(kk.data[0], kk2.data[0]);

            //     // for (a,b) in kk.data[1].iter_ones().zip(kk2.data[1].iter_ones()){
            //     //     let ff=vec![ActualMove{moveto:a},ActualMove{moveto:b}];

            //     //     assert_eq!(a,b,"issue:{:?}",self.world.format(&ff));
            //     // }
            //     assert_eq!(kk.data[1], kk2.data[1]);
            // }

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

            spoke_info.undo_move(cand.clone(), effect.clone(), team, self.world, game);
            // log!(
            //     "consid depth:{} {:?}:{:?}",
            //     depth,
            //     self.world.format(&cand),
            //     self.world.format(&m.clone().to_vec())
            // );

            let cc = cand.clone();
            cand.undo(team, &effect, game);

            key.move_undo(&self.zobrist, cand.clone(), team, &effect);
            m.push(cand);
            if !ab_iter.keep_going(m, eval) {
                //2007 without
                if !loud_moves.inner[cc.moveto] {
                    self.killer_moves.consider(depth, cc);
                }

                self.moves.drain(start_move_index..);
                break;
            }
        }

        assert_eq!(self.moves.len(), start_move_index);

        let (eval, m) = ab_iter.finish();
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

            self.prev_cache
                .a
                .insert(key, entry);
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
            //TODO should be less than or equal instead maybe?

            if eval > self.value {
                self.mm = Some(t);
                self.value = eval;
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
