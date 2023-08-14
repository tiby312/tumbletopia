use super::{
    selection::{MoveLog, RegularSelection},
    *,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Eval(i64);
impl Eval {
    // pub fn neg(mut self) -> Eval {
    //     self.0 = -self.0;
    //     self
    // }

    // // pub fn horrible()->Eval{
    // //     let mut g=Eval(-666);
    // //     g
    // // }
    // pub fn mul(mut self,color:i8)->Eval{
    //     self.0=self.0*color as i64;
    //     self
    // }
}

//cats maximizing
//dogs minimizing
fn absolute_evaluate(view: &GameViewMut<'_, '_>) -> Eval {
    let view = view.absolute();
    let num_cats = view.cats.units.len();
    let num_dogs = view.dogs.units.len();
    let diff = num_cats as i64 - num_dogs as i64;

    if view
        .cats
        .units
        .iter()
        .find(|a| a.typ == Type::Para)
        .is_none()
    {
        return Eval(-100_000);
    };

    if view
        .dogs
        .units
        .iter()
        .find(|a| a.typ == Type::Para)
        .is_none()
    {
        return Eval(100_000);
    };

    Eval(diff as i64)
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
        let foo = for_all_moves(&v).map(|( x, m)| {
            let (_, p) = min_max(x.not(), depth - 1, debug);
            writeln!(&mut s, "\t\t{:?}", (&m, p)).unwrap();
            (m, p)
        });

        let (m, ev) = if v.team == ActiveTeam::Dogs {
            foo.min_by_key(|a| a.1).unwrap()
        } else {
            foo.max_by_key(|a| a.1).unwrap()
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
        .flat_map(|a| a.generate(view).into_iter().map(move |b| (a.clone(), b)))
        .flat_map(|(s, m)| {
            let mut v = view.duplicate();
            let mut mm = MoveLog::new();

            let first = if let Some(l) = s
                .execute_no_animation(m.target, &mut v.view(), &mut mm)
                .unwrap()
            {
                let cll = l.select();
                let mut kk = v.view().duplicate();
                Some(cll.generate(&mut kk.view()).into_iter().map(move |m| {
                    let mut klkl = kk.view().duplicate();
                    let mut mm2 = MoveLog::new();

                    let mut vfv = klkl.view();
                    cll.execute_no_animation(m.target, &mut vfv, &mut mm2)
                        .unwrap();

                    (klkl, mm2.inner[0].clone())
                }))
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
