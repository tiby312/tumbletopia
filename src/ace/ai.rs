use crate::movement::MovementMesh;

use super::{
    selection::{MoveLog, RegularSelection},
    *,
};

pub type Eval = i64; //(f64);

const MATE: i64 = 1_000_000;
//cats maximizing
//dogs minimizing
fn absolute_evaluate(view: &GameState) -> Eval {
    //TODO check for checks!!!
    //let view = view.absolute();
    let num_cats = view.cats.units.len();
    let num_dogs = view.dogs.units.len();
    let diff = num_cats as i64 - num_dogs as i64;

    let Some(cat_king)=view
        .cats
        .units
        .iter()
        .find(|a| a.typ == Type::Para)
    else {
        return -MATE;
    };

    let Some(dog_king)=view
        .dogs
        .units
        .iter()
        .find(|a| a.typ == Type::Para)
    else
    {
        return MATE;
    };

    //TODO check if warriors are restricted

    //how close cats are to dog king.
    let cat_distance_to_dog_king = view
        .cats
        .units
        .iter()
        .map(|x| {
            let free = selection::has_restricted_movement(x, &view.view(ActiveTeam::Cats));
            let free = if free { 2 } else { 1 };
            let x = x.position.to_cube().dist(&dog_king.position.to_cube());
            x * x * free
        })
        .fold(0, |acc, f| acc + f) as i64;

    let cat_distance_to_cat_king = view
        .cats
        .units
        .iter()
        .map(|x| {
            let free = selection::has_restricted_movement(x, &view.view(ActiveTeam::Cats));
            let free = if free { 2 } else { 1 };
            let x = x.position.to_cube().dist(&cat_king.position.to_cube());
            x * x * free
        })
        .fold(0, |acc, f| acc + f) as i64;

    //how close dogs are to cat king.
    let dog_distance_to_cat_king = view
        .dogs
        .units
        .iter()
        .map(|x| {
            let free = selection::has_restricted_movement(x, &view.view(ActiveTeam::Dogs));
            let free = if free { 2 } else { 1 };
            let x = x.position.to_cube().dist(&cat_king.position.to_cube());
            x * x * free
        })
        .fold(0, |acc, f| acc + f) as i64;

    let dog_distance_to_dog_king = view
        .dogs
        .units
        .iter()
        .map(|x| {
            let free = selection::has_restricted_movement(x, &view.view(ActiveTeam::Dogs));
            let free = if free { 2 } else { 1 };
            let x = x.position.to_cube().dist(&dog_king.position.to_cube());
            x * x * free
        })
        .fold(0, |acc, f| acc + f) as i64;

    let val = diff * 1000 - cat_distance_to_dog_king
        + cat_distance_to_cat_king
        + dog_distance_to_cat_king
        - dog_distance_to_dog_king;
    //assert!(!val.is_nan());
    val
}

// pub fn captures_possible(node: GameViewMut<'_, '_>) -> bool {
//     let num_enemy = node.that_team.units.len();
//     for a in for_all_moves(&node) {
//         if a.game_after_move.that_team.units.len() < num_enemy {
//             return true;
//         }
//     }

//     let num_friendly = node.this_team.units.len();
//     for a in for_all_moves(&node) {
//         if a.game_after_move.this_team.units.len() < num_friendly {
//             return true;
//         }
//     }

//     false
// }

pub fn we_in_check(view: GameView<'_>) -> bool {
    let Some(king_pos)=view.this_team.units.iter().find(|a|a.typ==Type::Para) else{
        return false
    };

    for a in view.this_team.units.iter().filter(|a| a.typ == Type::Para) {}

    true
}
pub fn game_is_over(view: GameView<'_>) -> bool {
    if view
        .this_team
        .units
        .iter()
        .find(|a| a.typ == Type::Para)
        .is_none()
    {
        return true;
    };

    if view
        .that_team
        .units
        .iter()
        .find(|a| a.typ == Type::Para)
        .is_none()
    {
        return true;
    };

    false
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
            let eval = absolute_evaluate(&node);
            self.consider_leaf(node.clone(), eval);
            eval
        }
    }
}

pub fn iterative_deepening<'a>(game: &GameState, team: ActiveTeam) -> moves::ActualMove {
    let mut count = Counter { count: 0 };
    let mut results = Vec::new();
    let mut table = LeafTranspositionTable::new();

    let mut foo1 = MoveOrdering {
        a: std::collections::HashMap::new(),
    };

    let max_depth = 6;

    //TODO stop searching if we found a game ending move.
    for depth in 1..max_depth {
        console_dbg!("searching", depth);

        //TODO should this be outside the loop?
        let mut k = KillerMoves::new(max_depth);

        let pp = PossibleMove {
            the_move: moves::ActualMove::SkipTurn,
            //mesh: MovementMesh::new(),
            game_after_move: game.clone(),
        };
        let mut aaaa = ai::AlphaBeta {
            table: &mut table,
            prev_cache: &mut foo1,
            calls: &mut count,
            path: &mut vec![],
            killer_moves: &mut k,
            max_ext: 0,
        };
        let res = aaaa.alpha_beta(pp, ABAB::new(), team, depth, 0);

        console_dbg!(aaaa.max_ext);
        // assert_eq!(
        //     res.mov.as_ref(),

        // );

        let mov = foo1
            .a
            .get(&[moves::ActualMove::SkipTurn] as &[_])
            .cloned()
            .unwrap();
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
        results.push(res);

        if eval.abs() == MATE {
            console_dbg!("found a mate");
            break;
        }
    }

    // console_dbg!(table.saves);
    // console_dbg!(table.a.len());
    console_dbg!(count);
    results.dedup_by_key(|x| x.eval);

    let mov = results.pop().unwrap();

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

pub struct EvalRet<T> {
    pub mov: T,
    pub eval: Eval,
}

impl<'a> AlphaBeta<'a> {
    // pub fn quiensense_search(
    //     &mut self,
    //     cand: PossibleMove,
    //     ab: ABAB,
    //     team: ActiveTeam,
    //     depth: usize,
    // ) -> EvalRet {
    //     let the_move = cand.the_move;
    //     let node = cand.game_after_move;
    //     self.path.push(the_move.clone());
    //     let all_moves: Vec<_> = for_all_capture_and_jump_moves(node.clone(), team).collect();
    //     //console_dbg!(all_moves.len());
    //     let ret = if depth == 0 || game_is_over(node.view(team)) || all_moves.is_empty() {
    //         //(None,quiescence_search(node, team,table,calls, 5, alpha, beta))
    //         //TODO do Quiescence Search
    //         self.calls.add_eval();
    //         self.table.lookup_leaf_all(&node)
    //     } else {
    //         //let pvariation = self.prev_cache.get_best_prev_move(self.path).cloned();

    //         let it = all_moves.into_iter();
    //         let foo = |cand, ab| self.quiensense_search(cand, ab, team.not(), depth - 1);
    //         let ret = if team == ActiveTeam::Cats {
    //             ab.maxxer(it, foo)
    //         } else {
    //             ab.minner(it, foo)
    //         };

    //         //self.prev_cache.update(&self.path, &ret);

    //         ret
    //     };
    //     let k = self.path.pop().unwrap();
    //     assert_eq!(k, the_move);
    //     ret
    // }

    pub fn alpha_beta(
        &mut self,
        cand: PossibleMove,
        ab: ABAB,
        team: ActiveTeam,
        depth: usize,
        ext: usize,
    ) -> Eval {
        self.max_ext = self.max_ext.max(ext);
        let the_move = cand.the_move.clone();
        let mut gg = cand.game_after_move.clone();

        self.path.push(the_move.clone());
        let ret = if depth == 0 || game_is_over(cand.game_after_move.view(team)) {
            self.calls.add_eval();
            self.table.lookup_leaf_all(&cand.game_after_move)

            //self.quiensense_search(cand, ab, team, 5)
        } else {
            let node = cand.game_after_move;

            let pvariation = self.prev_cache.get_best_prev_move(self.path).cloned();

            // let pvariation = pvariation.map(|x| {
            //     execute_move_no_ani(&mut gg, team, x.clone());
            //     PossibleMove {
            //         the_move: x,
            //         game_after_move: gg,
            //     }
            // });

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

            let mut moves: Vec<_> = for_all_moves(node.clone(), team)
                .map(|mut x| {
                    //let c = is_check(&x.game_after_move);
                    //let c1 = this_team_in_check(&mut x.game_after_move, team);
                    //let c2 = this_team_in_check(&mut x.game_after_move, team.not());

                    //let c = false;
                    (false, x)
                })
                .collect();

            let num_checky = moves.iter().filter(|x| x.0).count();
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
                    .find(|(_, (_, x))| x.the_move == p)
                    .unwrap();
                let swap_ind = f.0;
                moves.swap(0, swap_ind);
                num_sorted += 1;
            }

            for a in num_sorted..moves.len() {
                //moves[a].1.game_after_move

                let interesting_move = {
                    let a = &moves[a].1;

                    let jump_move = if let moves::ActualMove::ExtraMove(_, _) = a.the_move {
                        true
                    } else {
                        false
                    };
                    let b = &a.game_after_move;

                    jump_move
                        || b.dogs.units.len() < gg.dogs.units.len()
                        || b.cats.units.len() < gg.cats.units.len()
                };

                if interesting_move {
                    moves.swap(a, num_sorted);
                    num_sorted += 1;
                }
            }

            for a in self.killer_moves.get(depth) {
                if let Some((x, _)) = moves[num_sorted..]
                    .iter()
                    .enumerate()
                    .find(|(_, (_, x))| &x.the_move == a)
                {
                    moves.swap(x, num_sorted);
                    num_sorted += 1;
                }
            }

            let foo = |ssself: &mut AlphaBeta, (is_checky, cand): (bool, PossibleMove), ab| {
                let new_ext = if ext < 2 && is_checky {
                    //1
                    //1
                    0
                } else {
                    0
                };

                // let inhibit = if num_checky > 0 {
                //     if is_checky {
                //         0
                //     } else {
                //         3
                //     }
                // } else {
                //     0
                // };

                let cc = cand.clone();
                let new_depth = new_ext + depth - 1; //.saturating_sub(inhibit);
                                                     //assert!(new_depth<6);
                                                     //console_dbg!(ext,depth);
                let eval = ssself.alpha_beta(cand, ab, team.not(), new_depth, ext + new_ext);

                EvalRet {
                    eval,
                    mov: (is_checky, cc),
                }
            };

            // let num=if depth>3{
            //     moves.len()
            // }else{
            //     moves.len()
            //     //num_sorted*2
            // };
            // let num=20.min(num_sorted*2);
            // let moves=moves.into_iter().take(num);

            if team == ActiveTeam::Cats {
                if let Some(ret) = ab.maxxer(moves, self, foo, |ss, m, _| {
                    ss.killer_moves.consider(depth, m.1.the_move);
                }) {
                    self.prev_cache.update(&self.path, &ret.mov.1.the_move);
                    ret.eval
                } else {
                    Eval::MIN
                }
            } else {
                if let Some(ret) = ab.minner(moves, self, foo, |_, _, _| {}) {
                    self.prev_cache.update(&self.path, &ret.mov.1.the_move);
                    ret.eval
                } else {
                    Eval::MAX
                }
            }
        };
        let k = self.path.pop().unwrap();
        assert_eq!(k, the_move);
        ret
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
fn this_team_in_check(state: &GameState, team: ActiveTeam) -> bool {
    let mut gg = state.clone();

    //TODO additionally check for jump checks.
    let game = state.view(team);
    let king = if let Some(king) = game.this_team.units.iter().find(|a| a.typ == Type::Para) {
        king.clone()
    } else {
        return false;
    };

    //let mut ee = king.clone();
    //ee.typ = Type::Warrior;
    //let _ = gg.view_mut(team).this_team.find_take(&ee.position);
    //let mesh = selection::generate_unit_possible_moves_inner(&ee, &gg.view_mut(team), None);
    let mesh = movement::compute_moves2(
        king.position,
        &game
            .world
            .filter()
            // .and(
            //     game.that_team
            //         .filter_type(Type::Warrior)
            //         .and(game.that_team.filter())
            //         .not(),
            // )
            .and(game.this_team.filter().not()),
        &game.this_team.filter().or(movement::AcceptCoords::new(
            board::water_border().map(|x| x.to_axial()),
        )),
        false,
        true,
    );

    for a in mesh.iter_mesh(king.position) {
        //for a in king.position.to_cube().range(2){
        //let a=a.to_axial();
        if let Some(unit) = game.that_team.find_slow(&a) {
            let dis = a.to_cube().dist(&king.position.to_cube());
            //console_dbg!(dis);
            if dis == 1 {
                return true;
            }
            let restricted_movement = if let Some(_) = unit
                .position
                .to_cube()
                .ring(1)
                .map(|s| game.that_team.find_slow(&s.to_axial()).is_some())
                .find(|a| *a)
            {
                true
            } else {
                match unit.typ {
                    Type::Warrior => false,
                    Type::Para => true,
                    _ => todo!(),
                }
            };

            if !restricted_movement {
                return true;
            }
        }
    }

    false
}

// fn for_all_capture_and_jump_moves(
//     state: GameState,
//     team: ActiveTeam,
// ) -> impl Iterator<Item = PossibleMove> {
//     let n = state.clone();
//     //let in_check = { in_check(n.clone(), team) || in_check(n.clone(), team.not()) };
//     let enemy_king_pos = if let Some(enemy_king_pos) = state
//         .view(team.not())
//         .this_team
//         .units
//         .iter()
//         .find(|a| a.typ == Type::Para)
//     {
//         Some(enemy_king_pos.position)
//     } else {
//         None
//     };

//     for_all_moves(state, team).filter(move |a| {
//         // let check = if let Some(enemy_king_pos) = enemy_king_pos {
//         //     match &a.the_move {
//         //         moves::ActualMove::NormalMove(o) => o.moveto == enemy_king_pos,
//         //         moves::ActualMove::ExtraMove(_, o) => o.moveto == enemy_king_pos,
//         //         _ => false,
//         //     }
//         // } else {
//         //     false
//         // };

//         let jump_move = if let moves::ActualMove::ExtraMove(_, _) = a.the_move {
//             true
//         } else {
//             false
//         };
//         let b = &a.game_after_move;

//         jump_move
//             || b.dogs.units.len() < n.dogs.units.len()
//             || b.cats.units.len() < n.cats.units.len()
//     })
// }

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

pub fn execute_move_no_ani(
    state: &mut GameState,
    team_index: ActiveTeam,
    the_move: moves::ActualMove,
) {
    let mut game = state.view_mut(team_index);
    let mut game_history = MoveLog::new();

    match the_move {
        moves::ActualMove::NormalMove(o) => {
            let unit = game.this_team.find_slow(&o.unit).unwrap();

            let mesh = selection::generate_unit_possible_moves_inner(unit, &game, None);

            let r = selection::RegularSelection::new(unit);
            let r = r
                .execute_no_animation(o.moveto, mesh, &mut game, &mut game_history)
                .unwrap();
            assert!(r.is_none());
        }
        moves::ActualMove::ExtraMove(o, e) => {
            let unit = game.this_team.find_slow(&o.unit).unwrap().clone();

            let mesh = selection::generate_unit_possible_moves_inner(&unit, &game, None);

            let r = selection::RegularSelection::new(&unit);
            let r = r
                .execute_no_animation(o.moveto, mesh, &mut game, &mut game_history)
                .unwrap();
            console_dbg!("WOOO");

            //let unit = game.this_team.find_slow(&o.unit).unwrap().clone();

            // let mesh =
            //     selection::generate_unit_possible_moves_inner(&unit, &game, Some(e.unit));

            let rr = r.unwrap();

            let rr = rr.select();
            let mesh = rr.generate(&game);

            rr.execute_no_animation(e.moveto, mesh, &mut game, &mut game_history)
                .unwrap();
        }
        moves::ActualMove::SkipTurn => {}
        moves::ActualMove::GameEnd(_) => todo!(),
    }
}

pub struct PartialMove {
    pos: GridCoord,
    moveto: GridCoord,
}
pub fn for_all_moves_v2(state: GameState, team: ActiveTeam) -> impl Iterator<Item = PartialMove> {
    let mut sss = state.clone();
    state
        .clone()
        .into_view(team)
        .this_team
        .units
        .into_iter()
        .map(|a| RegularSelection { unit: a.clone() })
        .flat_map(move |a| {
            let mesh = a.generate(&sss.view_mut(team));
            mesh.iter_mesh(a.unit.position).map(move |f| PartialMove {
                pos: a.unit.position,
                moveto: f,
            })
        })
}

pub fn for_all_moves(state: GameState, team: ActiveTeam) -> impl Iterator<Item = PossibleMove> {
    let foo = PossibleMove {
        the_move: moves::ActualMove::SkipTurn,
        game_after_move: state.clone(),
        //mesh: MovementMesh::new(),
    };

    let mut sss = state.clone();
    let ss = state.clone();
    ss.into_view(team)
        .this_team
        .units
        .into_iter()
        .map(|a| RegularSelection { unit: a.clone() })
        .flat_map(move |a| {
            let mesh = a.generate(&sss.view_mut(team));
            mesh.iter_mesh(a.unit.position)
                .map(move |f| (a.clone(), mesh, f))
        })
        .flat_map(move |(s, mesh, m)| {
            let mut v = state.clone();
            let mut mm = MoveLog::new();

            let first = if let Some(l) = s
                .execute_no_animation(m, mesh, &mut v.view_mut(team), &mut mm)
                .unwrap()
            {
                let cll = l.select();

                //let mut kk = v.view().duplicate();
                let mut kk = v.clone();
                let mesh2 = cll.generate(&mut kk.view_mut(team));
                Some(mesh2.iter_mesh(l.coord()).map(move |m| {
                    let mut klkl = kk.clone();
                    let mut mm2 = MoveLog::new();

                    let mut vfv = klkl.view_mut(team);
                    cll.execute_no_animation(m, mesh2, &mut vfv, &mut mm2)
                        .unwrap();

                    PossibleMove {
                        game_after_move: klkl,
                        //mesh: mesh2,
                        the_move: mm2.inner[0].clone(),
                    }
                }))
            } else {
                None
            };

            let second = if first.is_none() {
                Some([PossibleMove {
                    game_after_move: v,
                    //mesh,
                    the_move: mm.inner[0].clone(),
                }])
            } else {
                None
            };

            let f1 = first.into_iter().flatten();
            let f2 = second.into_iter().flatten();
            f1.chain(f2)
        })
        .chain([foo].into_iter())
}
