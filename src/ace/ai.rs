use crate::movement::MovementMesh;

use super::{
    selection::{MoveLog, RegularSelection},
    *,
};

pub type Eval = f64; //(f64);

//cats maximizing
//dogs minimizing
fn absolute_evaluate(view: &GameState) -> Eval {
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
        return -1_000_000.0;
    };

    let Some(dog_king)=view
        .dogs
        .units
        .iter()
        .find(|a| a.typ == Type::Para)
    else
    {
        return 1_000_000.0;
    };

    //how close cats are to dog king.
    let cat_distance_to_dog_king = view
        .cats
        .units
        .iter()
        .map(|x| x.position.to_cube().dist(&dog_king.position.to_cube()))
        .fold(0, |acc, f| acc + f);

    //how close dogs are to cat king.
    let dog_distance_to_cat_king = view
        .dogs
        .units
        .iter()
        .map(|x| x.position.to_cube().dist(&cat_king.position.to_cube()))
        .fold(0, |acc, f| acc + f);

    let val =
        diff as f64 * 100.0 - cat_distance_to_dog_king as f64 + dog_distance_to_cat_king as f64;
    assert!(!val.is_nan());
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
    a: std::collections::HashMap<Vec<moves::ActualMove>, PossibleMove>,
}
impl MoveOrdering {
    pub fn get_best_prev_move(&self, path: &[moves::ActualMove]) -> Option<&PossibleMove> {
        self.a.get(path)
    }
    pub fn get_best_prev_move_mut(
        &mut self,
        path: &[moves::ActualMove],
    ) -> Option<&mut PossibleMove> {
        self.a.get_mut(path)
    }
    pub fn insert(&mut self, path: &[moves::ActualMove], m: PossibleMove) {
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
    pub fn lookup_leaf(&mut self, a: &GameState) -> Option<&Eval> {
        if let Some(a) = self.a.get(a) {
            self.saves += 1;
            Some(a)
        } else {
            None
        }
    }
    pub fn consider_leaf(&mut self, game: GameState, eval: Eval) {
        if let Some(v) = self.a.get_mut(&game) {
            *v = eval;
        } else {
            let _ = self.a.insert(game, eval);
        }
    }
}

pub fn iterative_deepening<'a>(game: &GameState, team: ActiveTeam) -> (Option<PossibleMove>, Eval) {
    let mut count = Counter { count: 0 };
    let mut results = Vec::new();
    let mut table = LeafTranspositionTable::new();

    let mut foo1 = MoveOrdering {
        a: std::collections::HashMap::new(),
    };

    //TODO stop searching if we found a game ending move.
    for depth in 0..5 {
        let res = ai::AlphaBeta {
            table: &mut table,
            check_first: &mut foo1,
            calls: &mut count,
            path: &mut vec![],
            debug: false,
        }
        .alpha_beta(game, team, depth, f64::NEG_INFINITY, f64::INFINITY);

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

        results.push(res);
    }

    console_dbg!(table.saves);
    console_dbg!(table.a.len());
    console_dbg!(count);
    results.dedup_by_key(|x| x.1);

    results.pop().unwrap()
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

pub fn quiescence_search(
    node: &GameState,
    team: ActiveTeam,
    table: &mut LeafTranspositionTable,
    calls: &mut Counter,
    depth: usize,
    alpha: f64,
    beta: f64,
) -> Eval {
    if game_is_over(node.view(team)) {
        calls.add_eval();
        if let Some(n) = table.lookup_leaf(&node) {
            return *n;
        } else {
            let val = absolute_evaluate(&node);
            table.consider_leaf(node.clone(), val);
            return val;
        }
    }
    let it = for_all_moves_ext(node.clone(), team, true).map(|x| {
        quiescence_search(
            &x.game_after_move,
            team.not(),
            table,
            calls,
            depth,
            alpha,
            beta,
        )
    });

    if team == ActiveTeam::Cats {
        let max = it.max_by(|a, b| a.partial_cmp(b).unwrap());
        //alpha=alpha
    } else {
    }

    return 0.0;
}

pub struct AlphaBeta<'a> {
    table: &'a mut LeafTranspositionTable,
    check_first: &'a mut MoveOrdering,
    calls: &'a mut Counter,
    path: &'a mut Vec<moves::ActualMove>,
    debug: bool,
}

impl<'a> AlphaBeta<'a> {
    pub fn alpha_beta(
        &mut self,
        node: &GameState,
        team: ActiveTeam,
        depth: usize,
        mut alpha: f64,
        mut beta: f64,
    ) -> (Option<PossibleMove>, Eval) {
        if depth == 0 || game_is_over(node.view(team)) {
            //(None,quiescence_search(node, team,table,calls, 5, alpha, beta))
            //TODO do Quiescence Search
            self.calls.add_eval();
            if let Some(n) = self.table.lookup_leaf(&node) {
                (None, *n)
            } else {
                let val = absolute_evaluate(&node);
                self.table.consider_leaf(node.clone(), val);
                (None, val)
            }
        } else {
            if team == ActiveTeam::Cats {
                let mut mm: Option<PossibleMove> = None;
                let mut value = f64::NEG_INFINITY;

                let principal_variation = self.check_first.get_best_prev_move(self.path).cloned();

                for cand in reorder_front(principal_variation, for_all_moves(node.clone(), team)) {
                    self.path.push(cand.the_move.clone());
                    let t =
                        self.alpha_beta(&cand.game_after_move, team.not(), depth - 1, alpha, beta);
                    let k = self.path.pop().unwrap();
                    assert_eq!(k, cand.the_move.clone());

                    value = value.max(t.1);
                    if value == t.1 {
                        mm = Some(cand);
                    }
                    if t.1 > beta {
                        break;
                    }
                    alpha = alpha.max(value)
                }

                if let Some(aaa) = &mm {
                    if let Some(foo) = self.check_first.get_best_prev_move_mut(&self.path) {
                        *foo = aaa.clone();
                    } else {
                        self.check_first.insert(self.path, aaa.clone());
                    }
                }

                (mm, value)
                //(mm, mesh_final, value)
            } else {
                let mut mm: Option<PossibleMove> = None;

                let mut value = f64::INFINITY;

                let principal_variation = self.check_first.get_best_prev_move(self.path).cloned();

                for cand in reorder_front(principal_variation, for_all_moves(node.clone(), team)) {
                    self.path.push(cand.the_move.clone());

                    let t =
                        self.alpha_beta(&cand.game_after_move, team.not(), depth - 1, alpha, beta);
                    let k = self.path.pop().unwrap();
                    assert_eq!(k, cand.the_move.clone());

                    value = value.min(t.1);
                    if value == t.1 {
                        mm = Some(cand);
                    }
                    if t.1 < alpha {
                        break;
                    }
                    beta = beta.min(value)
                }

                if let Some(aaa) = &mm {
                    if let Some(foo) = self.check_first.get_best_prev_move_mut(&self.path) {
                        *foo = aaa.clone();
                    } else {
                        self.check_first.insert(self.path, aaa.clone());
                    }
                }
                (mm, value)
            }

            //writeln!(&mut s, "{:?}", (v.team, depth, &m, ev)).unwrap();
            //gloo::console::log!(s);
        }
    }
}

pub fn alpha_beta(
    node: &GameState,
    team: ActiveTeam,
    depth: usize,
    debug: bool,
    mut alpha: f64,
    mut beta: f64,
    table: &mut LeafTranspositionTable,
    check_first: &mut MoveOrdering,
    calls: &mut Counter,
    path: &mut Vec<moves::ActualMove>,
) -> (Option<PossibleMove>, Eval) {
    if depth == 0 || game_is_over(node.view(team)) {
        //(None,quiescence_search(node, team,table,calls, 5, alpha, beta))
        //TODO do Quiescence Search
        calls.add_eval();
        if let Some(n) = table.lookup_leaf(&node) {
            (None, *n)
        } else {
            let val = absolute_evaluate(&node);
            table.consider_leaf(node.clone(), val);
            (None, val)
        }
    } else {
        if team == ActiveTeam::Cats {
            let mut mm: Option<PossibleMove> = None;
            let mut value = f64::NEG_INFINITY;

            let principal_variation = check_first.get_best_prev_move(path).cloned();

            for cand in reorder_front(principal_variation, for_all_moves(node.clone(), team)) {
                path.push(cand.the_move.clone());
                let t = alpha_beta(
                    &cand.game_after_move,
                    team.not(),
                    depth - 1,
                    debug,
                    alpha,
                    beta,
                    table,
                    check_first,
                    calls,
                    path,
                );
                let k = path.pop().unwrap();
                assert_eq!(k, cand.the_move.clone());

                value = value.max(t.1);
                if value == t.1 {
                    mm = Some(cand);
                }
                if t.1 > beta {
                    break;
                }
                alpha = alpha.max(value)
            }

            if let Some(aaa) = &mm {
                if let Some(foo) = check_first.get_best_prev_move_mut(&path) {
                    *foo = aaa.clone();
                } else {
                    check_first.insert(path, aaa.clone());
                }
            }

            (mm, value)
            //(mm, mesh_final, value)
        } else {
            let mut mm: Option<PossibleMove> = None;

            let mut value = f64::INFINITY;

            let principal_variation = check_first.get_best_prev_move(path).cloned();

            for cand in reorder_front(principal_variation, for_all_moves(node.clone(), team)) {
                path.push(cand.the_move.clone());

                let t = alpha_beta(
                    &cand.game_after_move,
                    team.not(),
                    depth - 1,
                    debug,
                    alpha,
                    beta,
                    table,
                    check_first,
                    calls,
                    path,
                );
                let k = path.pop().unwrap();
                assert_eq!(k, cand.the_move.clone());

                value = value.min(t.1);
                if value == t.1 {
                    mm = Some(cand);
                }
                if t.1 < alpha {
                    break;
                }
                beta = beta.min(value)
            }

            if let Some(aaa) = &mm {
                if let Some(foo) = check_first.get_best_prev_move_mut(&path) {
                    *foo = aaa.clone();
                } else {
                    check_first.insert(path, aaa.clone());
                }
            }
            (mm, value)
        }

        //writeln!(&mut s, "{:?}", (v.team, depth, &m, ev)).unwrap();
        //gloo::console::log!(s);
    }
}

fn reorder_front(
    a: Option<PossibleMove>,
    b: impl Iterator<Item = PossibleMove>,
) -> impl Iterator<Item = PossibleMove> {
    let mut found_duplicate = false;
    let it = a.clone().into_iter().chain(b.filter(|z| {
        if let Some(p) = &a {
            if p == z {
                found_duplicate = true;
                false
            } else {
                true
            }
        } else {
            true
        }
    }));

    let v: Vec<_> = it.collect();
    if let Some(_) = a {
        assert!(found_duplicate);
    }
    v.into_iter()
}

// pub fn min_max<'a>(
//     mut node: GameThing<'a>,
//     depth: usize,
//     debug: bool,
// ) -> (Option<moves::ActualMove>, MovementMesh, Eval) {
//     //console_dbg!(depth);
//     if depth == 0 || game_is_over(node.view()) {
//         (None, MovementMesh::new(), absolute_evaluate(&node.view().absolute()))
//     } else {
//         let v = node.view();

//         use std::fmt::Write;
//         let mut s = String::new();
//         let foo = for_all_moves(&v).map(|cand| {
//             let (_, _, p) = min_max(cand.game_after_move.not(), depth - 1, debug);
//             (cand.the_move, p, cand.mesh)
//         });

//         let (m, ev, mesh) = if v.team == ActiveTeam::Dogs {
//             foo.min_by(|a, b| a.1.partial_cmp(&b.1).expect("float cmp fail"))
//                 .unwrap()
//         } else {
//             foo.max_by(|a, b| a.1.partial_cmp(&b.1).expect("float cmp fail"))
//                 .unwrap()
//         };

//         writeln!(&mut s, "{:?}", (v.team, depth, &m, ev)).unwrap();
//         //gloo::console::log!(s);

//         (Some(m), mesh, ev)
//     }
// }

#[derive(PartialEq, Eq, Clone)]
pub struct PossibleMove {
    pub the_move: moves::ActualMove,
    pub mesh: MovementMesh,
    pub game_after_move: GameState,
}

fn for_all_moves_ext(
    state: GameState,
    team: ActiveTeam,
    quiet: bool,
) -> impl Iterator<Item = PossibleMove> {
    let n = state.clone();
    for_all_moves(state, team).filter(move |a| {
        let b = &a.game_after_move;
        b.dogs.units.len() < n.dogs.units.len() || b.cats.units.len() < n.cats.units.len()
    })
}

fn for_all_moves(state: GameState, team: ActiveTeam) -> impl Iterator<Item = PossibleMove> {
    let foo = PossibleMove {
        the_move: moves::ActualMove::SkipTurn,
        game_after_move: state.clone(),
        mesh: MovementMesh::new(),
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
                        mesh: mesh2,
                        the_move: mm2.inner[0].clone(),
                    }
                }))
            } else {
                None
            };

            let second = if first.is_none() {
                Some([PossibleMove {
                    game_after_move: v,
                    mesh,
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
