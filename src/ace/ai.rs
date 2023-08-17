use crate::movement::MovementMesh;

use super::{
    selection::{MoveLog, RegularSelection},
    *,
};

pub type Eval = f64; //(f64);

//cats maximizing
//dogs minimizing
fn absolute_evaluate(view: &AbsoluteGameView<'_, '_>) -> Eval {
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

pub fn captures_possible(node: GameViewMut<'_, '_>) -> bool {
    let num_enemy = node.that_team.units.len();
    for a in for_all_moves(&node) {
        if a.game_after_move.that_team.units.len() < num_enemy {
            return true;
        }
    }

    let num_friendly = node.this_team.units.len();
    for a in for_all_moves(&node) {
        if a.game_after_move.this_team.units.len() < num_friendly {
            return true;
        }
    }

    false
}

pub fn game_is_over(view: GameViewMut<'_, '_>) -> bool {
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
    let mut s = std::collections::hash_map::DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub struct TranspositionTable<'a> {
    a: std::collections::HashMap<
       u64,
        (usize, (PossibleMove<'a>, Eval)),
    >,
}
impl<'a> TranspositionTable<'a> {
    pub fn new() -> Self {
        TranspositionTable {
            a: std::collections::HashMap::new(),
        }
    }
    pub fn lookup(
        &self,
        a: GameThing<'a>,
        depth:usize
    ) -> Option<&(usize, (PossibleMove<'a>, Eval))> {
        let k=calculate_hash(&a);
    
        if let Some(a)=self.a.get(&k){
            if depth<=a.0{
                Some(a)
            }else{
                None
            }
        }else{
            None
        }
    }
    pub fn consider(
        &mut self,
        val: (PossibleMove<'a>,Eval),
        depth: usize,
    ) {
        let k=calculate_hash(&val.0.game_after_move);

        if let Some((old_depth, v)) = self.a.get_mut(&k) {
            if depth > *old_depth {
                *old_depth = depth;
                *v = val;
            }
        } else {
            let _ = self.a.insert(k, (depth, val));
        }
    }
}

pub fn iterative_deepening<'a>(game: &GameViewMut<'_, 'a>) -> (Option<PossibleMove<'a>>, Eval) {
    //TODO add transpotion table!!!!
    
    let mut count = Counter { count: 0 };

    let mut results = Vec::new();
    let mut principal_variation = None;
    for depth in 0..5 {
        let res = ai::alpha_beta(
            game.duplicate(),
            depth,
            false,
            f64::NEG_INFINITY,
            f64::INFINITY,
            principal_variation,
            &mut count,
        );
        principal_variation = res.0.clone();
        results.push(res);
    }
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
pub fn alpha_beta<'a>(
    mut node: GameThing<'a>,
    depth: usize,
    debug: bool,
    mut alpha: f64,
    mut beta: f64,
    principal_variation: Option<PossibleMove<'a>>,
    calls: &mut Counter,
) -> (Option<PossibleMove<'a>>, Eval) {
    if let Some(k) = &principal_variation {
        assert_eq!(k.game_after_move.team, node.team);
    }

    //console_dbg!(depth);
    if depth == 0 || game_is_over(node.view()) {
        calls.add_eval();
        (None, absolute_evaluate(&node.view().absolute()))
    } else {
        let v = node.view();

        //use std::fmt::Write;
        //let mut s = String::new();

        if v.team == ActiveTeam::Cats {
            let mut mm: Option<PossibleMove> = None;
            let mut value = f64::NEG_INFINITY;
            for (i, cand) in principal_variation
                .clone()
                .into_iter()
                .chain(for_all_moves(&v).filter(|cand| {
                    if let Some(p) = &principal_variation {
                        if p == cand {
                            false
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                }))
                .enumerate()
            {
                let t = alpha_beta(
                    cand.game_after_move.clone().not(),
                    depth - 1,
                    debug,
                    alpha,
                    beta,
                    None,
                    calls,
                );
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
            for (i, cand) in principal_variation
                .clone()
                .into_iter()
                .chain(for_all_moves(&v).filter(|cand| {
                    if let Some(p) = &principal_variation {
                        if p == cand {
                            false
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                }))
                .enumerate()
            {
                let t = alpha_beta(
                    cand.game_after_move.clone().not(),
                    depth - 1,
                    debug,
                    alpha,
                    beta,
                    None,
                    calls,
                );
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

pub fn min_max<'a>(
    mut node: GameThing<'a>,
    depth: usize,
    debug: bool,
) -> (Option<moves::ActualMove>, MovementMesh, Eval) {
    //console_dbg!(depth);
    if depth == 0 || game_is_over(node.view()) {
        (None, MovementMesh::new(), absolute_evaluate(&node.view().absolute()))
    } else {
        let v = node.view();

        use std::fmt::Write;
        let mut s = String::new();
        let foo = for_all_moves(&v).map(|cand| {
            let (_, _, p) = min_max(cand.game_after_move.not(), depth - 1, debug);
            (cand.the_move, p, cand.mesh)
        });

        let (m, ev, mesh) = if v.team == ActiveTeam::Dogs {
            foo.min_by(|a, b| a.1.partial_cmp(&b.1).expect("float cmp fail"))
                .unwrap()
        } else {
            foo.max_by(|a, b| a.1.partial_cmp(&b.1).expect("float cmp fail"))
                .unwrap()
        };

        writeln!(&mut s, "{:?}", (v.team, depth, &m, ev)).unwrap();
        //gloo::console::log!(s);

        (Some(m), mesh, ev)
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct PossibleMove<'a> {
    pub the_move: moves::ActualMove,
    pub mesh: MovementMesh,
    pub game_after_move: GameThing<'a>,
}
impl<'a> PossibleMove<'a> {
    pub fn skip_turn(a: &GameThing<'a>) -> Self {
        Self {
            the_move: moves::ActualMove::SkipTurn,
            mesh: MovementMesh::new(),
            game_after_move: a.clone(),
        }
    }
}

fn for_all_moves<'b, 'c>(
    view: &'b GameViewMut<'_, 'c>,
) -> impl Iterator<Item = PossibleMove<'c>> + 'b {
    let foo = PossibleMove {
        the_move: moves::ActualMove::SkipTurn,
        game_after_move: view.duplicate(),
        mesh: MovementMesh::new(),
    };

    view.this_team
        .units
        .iter()
        .map(|a| RegularSelection { unit: a.clone() })
        .flat_map(|a| {
            let mesh = a.generate(view);
            mesh.iter_mesh(a.unit.position)
                .map(move |f| (a.clone(), mesh, f))
        })
        .flat_map(|(s, mesh, m)| {
            let mut v = view.duplicate();
            let mut mm = MoveLog::new();

            let first = if let Some(l) = s
                .execute_no_animation(m, mesh, &mut v.view(), &mut mm)
                .unwrap()
            {
                let cll = l.select();

                let mut kk = v.view().duplicate();
                let mesh2 = cll.generate(&mut kk.view());
                Some(mesh2.iter_mesh(l.coord()).map(move |m| {
                    let mut klkl = kk.view().duplicate();
                    let mut mm2 = MoveLog::new();

                    let mut vfv = klkl.view();
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
