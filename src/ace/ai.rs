use super::{
    selection::{MoveLog, RegularSelection},
    *,
};

pub type Eval = f64; //(f64);

//cats maximizing
//dogs minimizing
fn absolute_evaluate(view: &GameViewMut<'_, '_>) -> Eval {
    let view = view.absolute();
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
        if a.0.that_team.units.len() < num_enemy {
            return true;
        }
    }

    let num_friendly = node.this_team.units.len();
    for a in for_all_moves(&node) {
        if a.0.this_team.units.len() < num_friendly {
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

pub fn alpha_beta<'a>(
    mut node: GameThing<'a>,
    depth: usize,
    debug: bool,
    mut alpha: f64,
    mut beta: f64,
) -> (Option<moves::ActualMove>, Eval) {
    //console_dbg!(depth);
    if depth == 0 || game_is_over(node.view()) {
        (None, absolute_evaluate(&node.view()))
    } else {
        let v = node.view();

        //use std::fmt::Write;
        //let mut s = String::new();

        if v.team == ActiveTeam::Cats {
            let mut mm: Option<moves::ActualMove> = None;
            let mut value = f64::NEG_INFINITY;
            for (i, (x, m)) in for_all_moves(&v).enumerate() {
                let t = alpha_beta(x.not(), depth - 1, debug, alpha, beta);
                value = value.max(t.1);
                if value == t.1 {
                    mm = Some(m);
                }
                if t.1 > beta {
                    break;
                }
                alpha = alpha.max(value)
            }
            (mm, value)
        } else {
            let mut mm: Option<moves::ActualMove> = None;
            let mut value = f64::INFINITY;
            for (i, (x, m)) in for_all_moves(&v).enumerate() {
                let t = alpha_beta(x.not(), depth - 1, debug, alpha, beta);
                value = value.min(t.1);
                if value == t.1 {
                    mm = Some(m);
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
) -> (Option<moves::ActualMove>, Eval) {
    //console_dbg!(depth);
    if depth == 0 || game_is_over(node.view()) {
        (None, absolute_evaluate(&node.view()))
    } else {
        let v = node.view();

        use std::fmt::Write;
        let mut s = String::new();
        let foo = for_all_moves(&v).map(|(x, m)| {
            let (_, p) = min_max(x.not(), depth - 1, debug);
            writeln!(&mut s, "\t\t{:?}", (&m, p)).unwrap();
            (m, p)
        });

        let (m, ev) = if v.team == ActiveTeam::Dogs {
            foo.min_by(|a, b| a.1.partial_cmp(&b.1).expect("float cmp fail"))
                .unwrap()
        } else {
            foo.max_by(|a, b| a.1.partial_cmp(&b.1).expect("float cmp fail"))
                .unwrap()
        };

        writeln!(&mut s, "{:?}", (v.team, depth, &m, ev)).unwrap();
        //gloo::console::log!(s);

        (Some(m), ev)
    }
}

fn for_all_moves<'b, 'c>(
    view: &'b GameViewMut<'_, 'c>,
) -> impl Iterator<Item = (GameThing<'c>, moves::ActualMove)> + 'b {
    let foo = (view.duplicate(), moves::ActualMove::SkipTurn);

    view.this_team
        .units
        .iter()
        .map(|a| RegularSelection { unit: a.clone() })
        .flat_map(|a| {
            a.generate(view)
                .iter_mesh(a.unit.position)
                .map(move |f| (a.clone(), f))
        })
        .flat_map(|(s, m)| {
            let mut v = view.duplicate();
            let mut mm = MoveLog::new();

            let first = if let Some(l) = s.execute_no_animation(m, &mut v.view(), &mut mm).unwrap()
            {
                let cll = l.select();

                let mut kk = v.view().duplicate();
                Some(
                    cll.generate(&mut kk.view())
                        .iter_mesh(l.coord())
                        .map(move |m| {
                            let mut klkl = kk.view().duplicate();
                            let mut mm2 = MoveLog::new();

                            let mut vfv = klkl.view();
                            cll.execute_no_animation(m, &mut vfv, &mut mm2).unwrap();

                            (klkl, mm2.inner[0].clone())
                        }),
                )
            } else {
                None
            };

            let second = if first.is_none() {
                Some([(v, mm.inner[0].clone())])
            } else {
                None
            };

            let f1 = first.into_iter().flatten();
            let f2 = second.into_iter().flatten();
            f1.chain(f2)
        })
        .chain([foo].into_iter())
}
