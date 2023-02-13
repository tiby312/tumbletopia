use super::*;

pub struct AndThen<A, B, N> {
    first: Option<(A, B)>,
    second: Option<N>,
}
impl<
        'a,
        Z: Zoo,
        A: GameStepper<Z>,
        K: GameStepper<Z>,
        B: FnOnce(A::Result, &mut Z::G<'_>) -> K,
    > GameStepper<Z> for AndThen<A, B, K>
{
    type Result = K::Result;
    //Return if you are done with this stage.
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<()> {
        if let Some((a, _)) = &mut self.first {
            match a.step(game) {
                Stage::Stay => Stage::Stay,
                Stage::NextStage(()) => {
                    let (a, b) = self.first.take().unwrap();
                    let j = a.consume(game);

                    //TODO would be more consistent with Once if the function was called
                    //in the same iteration as the first step call to second.
                    let nn = b(j, game);
                    self.second = Some(nn);
                    Stage::Stay
                }
            }
        } else {
            self.second.as_mut().unwrap().step(game)
        }
    }
    fn consume(self, game: &mut Z::G<'_>) -> Self::Result {
        self.second.unwrap().consume(game)
    }

    fn get_selection(&self) -> Option<&crate::CellSelection> {
        if let Some((a, _)) = self.first.as_ref() {
            a.get_selection()
        } else {
            self.second.as_ref().unwrap().get_selection()
        }
    }
    fn get_animation(&self) -> Option<&crate::animation::Animation<Warrior>> {
        if let Some((a, _)) = self.first.as_ref() {
            a.get_animation()
        } else {
            self.second.as_ref().unwrap().get_animation()
        }
    }
}

pub enum Stage<T> {
    NextStage(T),
    Stay,
}

#[derive(Copy, Clone)]
pub struct WaitForCustom<Z, F, R> {
    zoo: Z,
    func: F,
    res: Option<R>,
}
pub fn wait_custom<L, Z: Zoo, F: FnMut(&mut Z::G<'_>) -> Stage<L>>(
    zoo: Z,
    func: F,
) -> WaitForCustom<Z, F, L> {
    WaitForCustom {
        zoo,
        func,
        res: None,
    }
}

impl<L, Z: Zoo, F: FnMut(&mut Z::G<'_>) -> Stage<L>> GameStepper<Z> for WaitForCustom<Z, F, L> {
    type Result = L;
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<()> {
        match (self.func)(game) {
            Stage::Stay => Stage::Stay,
            Stage::NextStage(o) => {
                self.res = Some(o);
                Stage::NextStage(())
            }
        }
    }
    fn consume(self, game: &mut Z::G<'_>) -> Self::Result {
        self.res.unwrap()
    }
}

pub trait Zoo {
    type G<'b>
    where
        Self: 'b;
    fn create() -> Self;
}

pub fn next() -> Next {
    Next
}

#[derive(Copy, Clone)]
pub struct Next;
impl<Z: Zoo> GameStepper<Z> for Next {
    type Result = ();
    fn step(&mut self, _: &mut Z::G<'_>) -> Stage<()> {
        Stage::NextStage(())
    }
    fn consume(self, game: &mut Z::G<'_>) -> Self::Result {
        ()
    }
}

pub struct Optional<A> {
    a: Option<A>,
}

pub fn optional<Z: Zoo, A: GameStepper<Z>>(a: Option<A>) -> Optional<A> {
    Optional { a }
}

impl<Z: Zoo, A: GameStepper<Z>> GameStepper<Z> for Optional<A> {
    type Result = Option<A::Result>;
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<()> {
        if let Some(a) = self.a.as_mut() {
            match a.step(game) {
                Stage::Stay => Stage::Stay,
                Stage::NextStage(()) => Stage::NextStage(()),
            }
        } else {
            Stage::NextStage(())
        }
    }
    fn consume(self, game: &mut Z::G<'_>) -> Self::Result {
        if let Some(a) = self.a {
            Some(a.consume(game))
        } else {
            None
        }
    }
    fn get_selection(&self) -> Option<&crate::CellSelection> {
        if let Some(a) = self.a.as_ref() {
            if let Some(b) = a.get_selection() {
                Some(b)
            } else {
                None
            }
        } else {
            None
        }
    }
    fn get_animation(&self) -> Option<&crate::animation::Animation<Warrior>> {
        if let Some(a) = self.a.as_ref() {
            if let Some(b) = a.get_animation() {
                Some(b)
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub trait GameStepper<Z: Zoo> {
    type Result;
    //Return if you are done with this stage.
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<()>;

    fn consume(self, game: &mut Z::G<'_>) -> Self::Result
    where
        Self: Sized,
    {
        todo!()
    }

    fn get_selection(&self) -> Option<&crate::CellSelection> {
        None
    }
    fn get_animation(&self) -> Option<&crate::animation::Animation<Warrior>> {
        None
    }

    fn and_then<K: GameStepper<Z>, B: FnOnce(Self::Result, &mut Z::G<'_>) -> K>(
        self,
        other: B,
    ) -> AndThen<Self, B, K>
    where
        Self: Sized,
    {
        AndThen {
            first: Some((self, other)),
            second: None,
        }
    }
}

pub struct Looper<Z, A, F> {
    zoo: Z,
    a: Option<A>,
    func: F,
}

pub enum LooperRes<A, B> {
    Loop(A),
    Finish(B),
}
impl<A> LooperRes<A, ()> {
    pub fn infinite(self) -> LooperRes<A, ()> {
        self
    }
}
pub struct Looper2<Z, A, F, K, P, H> {
    _start_val: std::marker::PhantomData<P>,
    start_func: H,
    _zoo: Z,
    a: Option<A>,
    func: F,
    finished: Option<K>,
}

impl<
        Z: Zoo,
        A: GameStepper<Z>,
        K,
        F: FnMut(A::Result, &mut Z::G<'_>) -> LooperRes<P, K>,
        P,
        H: FnMut(P) -> A,
    > GameStepper<Z> for Looper2<Z, A, F, K, P, H>
{
    type Result = K;
    fn get_selection(&self) -> Option<&crate::CellSelection> {
        if let Some(a) = self.a.as_ref() {
            a.get_selection()
        } else {
            None
        }
    }
    fn get_animation(&self) -> Option<&crate::animation::Animation<Warrior>> {
        if let Some(a) = self.a.as_ref() {
            a.get_animation()
        } else {
            None
        }
    }
    fn consume(self, game: &mut Z::G<'_>) -> Self::Result {
        self.finished.unwrap()
    }
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<()> {
        if self.finished.is_some() {
            return Stage::Stay;
        }

        let a = if let Some(a) = &mut self.a {
            match a.step(game) {
                Stage::Stay => {
                    return Stage::Stay;
                }
                Stage::NextStage(()) => self.a.take().unwrap().consume(game),
            }
        } else {
            unreachable!();
        };

        match (self.func)(a, game) {
            LooperRes::Loop(p) => {
                self.a = Some((self.start_func)(p));
                Stage::Stay
            }
            LooperRes::Finish(b) => {
                self.finished = Some(b);
                Stage::NextStage(())
            }
        }
    }
}

pub fn looper<
    Z: Zoo,
    P,
    A: GameStepper<Z>,
    H: FnMut(P) -> A,
    K,
    F: FnMut(A::Result, &mut Z::G<'_>) -> LooperRes<P, K>,
>(
    mut start: H,
    start_val: P,
    func: F,
) -> Looper2<Z, A, F, K, P, H> {
    let elem = start(start_val);

    Looper2 {
        _zoo: Z::create(),
        a: Some(elem),
        func,
        finished: None,
        _start_val: std::marker::PhantomData,
        start_func: start,
    }
}

// pub fn looper2<
//     Z: Zoo,
//     A: GameStepper<Z>,
//     K,
//     F: FnMut(A::Result, &mut Z::G<'_>) -> LooperRes<A, K>,
// >(
//     start: A,
//     func: F,
// ) -> Looper2<Z, A, F, K> {
//     Looper2 {
//         zoo: Z::create(),
//         a: Some(start),
//         func,
//         finished: None,
//     }
// }

pub fn team_view(
    a: [&mut UnitCollection<Warrior>; 2],
    ind: usize,
) -> [&mut UnitCollection<Warrior>; 2] {
    let [a, b] = a;
    match ind {
        0 => [a, b],
        1 => [b, a],
        _ => {
            unreachable!()
        }
    }
}
