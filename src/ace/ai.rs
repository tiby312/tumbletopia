use crate::movement::MovementMesh;

use super::{
    selection::{MoveLog, RegularSelection},
    *,
};

pub type Eval = i64; //(f64);

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
        return -1_000_000;
    };

    let Some(dog_king)=view
        .dogs
        .units
        .iter()
        .find(|a| a.typ == Type::Para)
    else
    {
        return 1_000_000;
    };

    //how close cats are to dog king.
    let cat_distance_to_dog_king = view
        .cats
        .units
        .iter()
        .map(|x| x.position.to_cube().dist(&dog_king.position.to_cube()))
        .fold(0, |acc, f| acc + f) as i64;

    //how close dogs are to cat king.
    let dog_distance_to_cat_king = view
        .dogs
        .units
        .iter()
        .map(|x| x.position.to_cube().dist(&cat_king.position.to_cube()))
        .fold(0, |acc, f| acc + f) as i64;

    let val = diff * 100 - cat_distance_to_dog_king + dog_distance_to_cat_king;
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

    pub fn update(&mut self, path: &[moves::ActualMove], ret: &EvalRet) {
        if let Some(aaa) = &ret.mov {
            if let Some(foo) = self.get_best_prev_move_mut(path) {
                *foo = aaa.clone();
            } else {
                self.insert(path, aaa.clone());
            }
        }
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
    pub fn lookup_leaf_all(&mut self, node: &GameState) -> EvalRet {
        if let Some(&eval) = self.lookup_leaf(&node) {
            EvalRet { mov: None, eval }
        } else {
            let eval = absolute_evaluate(&node);
            self.consider_leaf(node.clone(), eval);
            EvalRet { mov: None, eval }
        }
    }
}

pub fn iterative_deepening<'a>(game: &GameState, team: ActiveTeam) -> EvalRet {
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
            prev_cache: &mut foo1,
            calls: &mut count,
            path: &mut vec![],
            debug: false,
        }
        .alpha_beta(moves::ActualMove::SkipTurn, game, team, depth, ABAB::new());

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
    results.dedup_by_key(|x| x.eval);

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

    return 0;
}

pub struct AlphaBeta<'a> {
    table: &'a mut LeafTranspositionTable,
    prev_cache: &'a mut MoveOrdering,
    calls: &'a mut Counter,
    path: &'a mut Vec<moves::ActualMove>,
    debug: bool,
}

pub struct EvalRetGeneric<T> {
    pub mov: Option<T>,
    pub eval: Eval,
}
type EvalRet = EvalRetGeneric<PossibleMove>;

impl<'a> AlphaBeta<'a> {
    pub fn alpha_beta(
        &mut self,
        the_move: moves::ActualMove,
        node: &GameState,
        team: ActiveTeam,
        depth: usize,
        ab: ABAB,
    ) -> EvalRet {
        self.path.push(the_move.clone());
        let ret = if depth == 0 || game_is_over(node.view(team)) {
            //(None,quiescence_search(node, team,table,calls, 5, alpha, beta))
            //TODO do Quiescence Search
            self.calls.add_eval();
            self.table.lookup_leaf_all(&node)
        } else {
            let pvariation = self.prev_cache.get_best_prev_move(self.path).cloned();

            let it = reorder_front(pvariation, for_all_moves(node.clone(), team));
            let foo = |cand: PossibleMove, ab| {
                self.alpha_beta(
                    cand.the_move,
                    &cand.game_after_move,
                    team.not(),
                    depth - 1,
                    ab,
                )
            };
            let ret = if team == ActiveTeam::Cats {
                ab.maxxer(it, foo)
            } else {
                ab.minner(it, foo)
            };

            self.prev_cache.update(&self.path, &ret);

            ret
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

        pub fn minner<T: Clone>(
            mut self,
            it: impl Iterator<Item = T>,
            mut func: impl FnMut(T, Self) -> EvalRet,
        ) -> EvalRetGeneric<T> {
            let mut mm: Option<T> = None;

            let mut value = i64::MAX;
            for cand in it {
                let t = func(cand.clone(), self.clone());

                value = value.min(t.eval);
                if value == t.eval {
                    mm = Some(cand);
                }
                if t.eval < self.alpha {
                    break;
                }
                self.beta = self.beta.min(value)
            }

            EvalRetGeneric {
                mov: mm,
                eval: value,
            }
        }
        pub fn maxxer<T: Clone>(
            mut self,
            it: impl Iterator<Item = T>,
            mut func: impl FnMut(T, Self) -> EvalRet,
        ) -> EvalRetGeneric<T> {
            let mut mm: Option<T> = None;

            let mut value = i64::MIN;
            for cand in it {
                let t = func(cand.clone(), self.clone());

                value = value.max(t.eval);
                if value == t.eval {
                    mm = Some(cand);
                }
                if t.eval > self.beta {
                    break;
                }
                self.alpha = self.alpha.max(value)
            }
            EvalRetGeneric {
                mov: mm,
                eval: value,
            }
        }
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
