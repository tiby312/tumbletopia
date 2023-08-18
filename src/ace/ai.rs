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

fn calculate_hash<T: std::hash::Hash>(t: &T) -> u64 {
    use std::hash::Hasher;
    //let mut s = std::collections::hash_map::DefaultHasher::new();
    let mut s = seahash::SeaHasher::new();

    t.hash(&mut s);
    s.finish()
}

pub struct CheckFirst {
    a: std::collections::HashMap<GameState, PossibleMove>,
}

pub struct TranspositionTable {
    a: std::collections::HashMap<u64, (usize, Eval)>,
    saves: usize,
}

impl TranspositionTable {
    pub fn new() -> Self {
        TranspositionTable {
            a: std::collections::HashMap::new(),
            saves: 0,
        }
    }
    pub fn lookup(&mut self, a: &GameState, depth: usize) -> Option<Eval> {
        let k = calculate_hash(a);

        if let Some(a) = self.a.get(&k) {
            if depth == a.0 {
                self.saves += 1;
                Some(a.1)
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn consider(&mut self, depth: usize, game: GameState, eval: Eval) {
        let k = calculate_hash(&game);

        if let Some((old_depth, v)) = self.a.get_mut(&k) {
            //if depth == *old_depth {
            *old_depth = depth;
            *v = eval;
            //}
        } else {
            let _ = self.a.insert(k, (depth, eval));
        }
    }
}

pub fn iterative_deepening<'a>(game: &GameState, team: ActiveTeam) -> (Option<PossibleMove>, Eval) {
    //TODO add transpotion table!!!!

    let mut count = Counter { count: 0 };
    let mut results = Vec::new();
    let mut principal_variation = None;
    let mut table = TranspositionTable::new();

    let mut foo1 = CheckFirst {
        a: std::collections::HashMap::new(),
    };

    for depth in 0..5 {
        let mut foo2 = CheckFirst {
            a: std::collections::HashMap::new(),
        };

        let res = ai::alpha_beta(
            game,
            team,
            depth,
            false,
            f64::NEG_INFINITY,
            f64::INFINITY,
            principal_variation,
            &mut table,
            &foo1,
            &mut foo2,
            &mut count,
        );
        foo1 = foo2;
        principal_variation = res.0.clone();
        results.push(res);
    }

    console_dbg!(table.saves);
    console_dbg!(table.a.len());
    //console_dbg!(res);
    console_dbg!(count);
    results.dedup_by_key(|x| x.1);
    //console_dbg!(res);

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
pub fn alpha_beta(
    node: &GameState,
    team: ActiveTeam,
    depth: usize,
    debug: bool,
    mut alpha: f64,
    mut beta: f64,
    mut principal_variation: Option<PossibleMove>,
    table: &mut TranspositionTable,
    check_first: &CheckFirst,
    next_tree: &mut CheckFirst,
    calls: &mut Counter,
) -> (Option<PossibleMove>, Eval) {
    //console_dbg!(depth);
    if depth == 0 || game_is_over(node.view(team)) {
        calls.add_eval();
        if let Some(n) = table.lookup(&node, depth) {
            (None, n)
        } else {
            let val = absolute_evaluate(&node);
            table.consider(depth, node.clone(), val);
            (None, val)
        }
    } else {
        //let v = node.view(team);

        //use std::fmt::Write;
        //let mut s = String::new();

        if team == ActiveTeam::Cats {
            let mut mm: Option<PossibleMove> = None;
            let mut value = f64::NEG_INFINITY;

            // principal_variation=check_first.a.get(node).cloned();

            for cand in reorder_front(principal_variation, for_all_moves(node.clone(), team)) {
                let t = alpha_beta(
                    &cand.game_after_move,
                    team.not(),
                    depth - 1,
                    debug,
                    alpha,
                    beta,
                    None,
                    table,
                    check_first,
                    next_tree,
                    calls,
                );

                if let Some(aaa) = t.0 {
                    next_tree.a.insert(cand.game_after_move.clone(), aaa);
                }

                value = value.max(t.1);
                if value == t.1 {
                    mm = Some(cand);
                }
                if t.1 > beta {
                    break;
                }
                alpha = alpha.max(value)
            }
            (mm, value)
            //(mm, mesh_final, value)
        } else {
            let mut mm: Option<PossibleMove> = None;

            let mut value = f64::INFINITY;

            //let principal_variation=check_first.a.get(node).cloned();

            for cand in reorder_front(principal_variation, for_all_moves(node.clone(), team)) {
                let t = alpha_beta(
                    &cand.game_after_move,
                    team.not(),
                    depth - 1,
                    debug,
                    alpha,
                    beta,
                    None,
                    table,
                    check_first,
                    next_tree,
                    calls,
                );

                if let Some(aaa) = t.0 {
                    next_tree.a.insert(cand.game_after_move.clone(), aaa);
                }

                value = value.min(t.1);
                if value == t.1 {
                    mm = Some(cand);
                }
                if t.1 < alpha {
                    break;
                }
                beta = beta.min(value)
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
