use super::*;

pub struct Map<A, F> {
    elem: A,
    func: F,
}
impl<Z: Zoo, A: GameStepper<Z>, F: FnOnce(A::Result, &mut Z::G<'_>) -> X, X> GameStepper<Z>
    for Map<A, F>
{
    type Result = X;
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<()> {
        self.elem.step(game)
    }
    fn consume(self, game: &mut Z::G<'_>) -> Self::Result {
        let s = self.elem.consume(game);
        (self.func)(s, game)
    }
    fn get_selection(&self) -> Option<&crate::CellSelection> {
        self.elem.get_selection()
    }
    fn get_animation(&self) -> Option<&crate::animation::Animation<Warrior>> {
        self.elem.get_animation()
    }
}

pub enum Stage<T> {
    NextStage(T),
    Stay,
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
    fn consume(self, _: &mut Z::G<'_>) -> Self::Result {
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

pub struct Chain<A, B> {
    first: Option<A>,
    second: Option<B>,
}
impl<Z: Zoo, A: GameStepper<Z, Result = B>, B: GameStepper<Z>> GameStepper<Z> for Chain<A, B> {
    type Result = B::Result;
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<()> {
        if let Some(a) = &mut self.first {
            match a.step(game) {
                Stage::Stay => Stage::Stay,
                Stage::NextStage(()) => {
                    let b = self.first.take().unwrap().consume(game);
                    self.second = Some(b);
                    Stage::Stay
                }
            }
        } else {
            self.second.as_mut().unwrap().step(game)
        }
    }
    fn consume(self, game: &mut Z::G<'_>) -> B::Result {
        self.second.unwrap().consume(game)
    }

    fn get_selection(&self) -> Option<&crate::CellSelection> {
        if let Some(a) = self.first.as_ref() {
            a.get_selection()
        } else {
            self.second.as_ref().unwrap().get_selection()
        }
    }
    fn get_animation(&self) -> Option<&crate::animation::Animation<Warrior>> {
        if let Some(a) = self.first.as_ref() {
            a.get_animation()
        } else {
            self.second.as_ref().unwrap().get_animation()
        }
    }
}

pub trait GameStepper<Z: Zoo> {
    type Result;
    //Return if you are done with this stage.
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<()>;

    fn consume(self, _: &mut Z::G<'_>) -> Self::Result
    where
        Self: Sized;

    fn get_selection(&self) -> Option<&crate::CellSelection> {
        None
    }
    fn get_animation(&self) -> Option<&crate::animation::Animation<Warrior>> {
        None
    }

    fn chain(self) -> Chain<Self, Self::Result>
    where
        Self::Result: GameStepper<Z> + Sized,
        Self: Sized,
    {
        Chain {
            first: Some(self),
            second: None,
        }
    }

    fn map<K, B: FnOnce(Self::Result, &mut Z::G<'_>) -> K>(self, func: B) -> Map<Self, B>
    where
        Self: Sized,
    {
        Map { elem: self, func }
    }

    // fn and_then<K: GameStepper<Z>, B: FnOnce(Self::Result, &mut Z::G<'_>) -> K>(
    //     self,
    //     func: B,
    // ) -> Chain<Map<Self, B>, K>
    // where
    //     Self: Sized,
    // {
    //     self.map(func).chain()
    // }
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

pub struct Looper<A, F, K, H> {
    start_func: H,
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
    > GameStepper<Z> for Looper<A, F, K, H>
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
    fn consume(self, _: &mut Z::G<'_>) -> Self::Result {
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
    start_val: P,
    mut start: H,
    func: F,
) -> Looper<A, F, K, H> {
    let elem = start(start_val);

    Looper {
        a: Some(elem),
        func,
        finished: None,
        start_func: start,
    }
}

//TODO move this
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
