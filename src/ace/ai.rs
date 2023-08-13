use super::{
    selection::{MoveLog, RegularSelection},
    *,
};

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Eval(i64);
impl Eval {
    pub fn neg(mut self) -> Eval {
        self.0 = -self.0;
        self
    }
}

fn evaluate(view: &GameViewMut<'_, '_>) -> Eval {
    let num_this = view.this_team.units.len();
    let num_that = view.this_team.units.len();
    let diff = num_that - num_this;

    let this_king_dead = view
        .this_team
        .units
        .iter()
        .find(|a| a.typ == Type::Para)
        .is_none();
    let that_king_dead = view
        .this_team
        .units
        .iter()
        .find(|a| a.typ == Type::Para)
        .is_none();

    if this_king_dead {
        return Eval(-10000);
    }
    if that_king_dead {
        return Eval(10000);
    }

    Eval(diff as i64)
}

pub fn min_max<'a>(mut node: GameThing<'a>, depth: usize) -> (Option<moves::ActualMove>, Eval) {
    if depth == 0 {
        (None, evaluate(&node.view()))
    } else {
        let v = node.view();
        let (m, ev) = for_all_moves(&v)
            .map(|(x, m)| {
                let (_, p) = min_max(x.not(), depth - 1);
                (m, p.neg())
            })
            .max_by_key(|a| a.1)
            .unwrap();

        (Some(m), ev)
    }
}

fn for_all_moves<'b, 'c>(
    view: &'b GameViewMut<'_, 'c>,
) -> impl Iterator<Item = (GameThing<'c>, moves::ActualMove)> + 'b {
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
}
