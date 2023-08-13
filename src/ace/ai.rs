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

fn absolute_evaluate(view: &GameViewMut<'_, '_>) -> Eval {
    let num_this = view.this_team.units.len();
    let num_that = view.that_team.units.len();
    let diff = num_this as i64 - num_that as i64;

    let this_king_dead: i64 = if view
        .this_team
        .units
        .iter()
        .find(|a| a.typ == Type::Para)
        .is_none()
    {
        -100000
    } else {
        0
    };

    let that_king_dead: i64 = if view
        .that_team
        .units
        .iter()
        .find(|a| a.typ == Type::Para)
        .is_none()
    {
        10000
    } else {
        0
    };

    let mut g = Eval(diff as i64 + this_king_dead + that_king_dead);

    if view.team == ActiveTeam::Dogs {
        g.0 = -g.0;
    }

    g
}

pub fn min_max<'a>(
    mut node: GameThing<'a>,
    depth: usize,
    debug: bool,
) -> (Option<moves::ActualMove>, Eval) {
    //console_dbg!(depth);
    if depth == 0 {
        (None, absolute_evaluate(&node.view()))
    } else {
        let v = node.view();

        let foo = for_all_moves(&v).map(|(mut x, m)| {
            let (_, p) = min_max(x.not(), depth - 1, debug);
            if depth == 2 {
                console_dbg!(m, p);
            }
            (m, p)
        });

        let (m, ev) = if v.team == ActiveTeam::Dogs {
            if depth == 2 {
                console_dbg!("mining!");
            }
            foo.min_by_key(|a| a.1).unwrap()
        } else {
            if depth == 2 {
                console_dbg!("maxing!");
            }
            foo.max_by_key(|a| a.1).unwrap()
        };

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
