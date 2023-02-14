use super::*;

pub struct Map<A, F> {
    elem: A,
    func: F,
}
impl<Z: Zoo, A: GameStepper<Z>, F: FnOnce(A::Result, &mut Z::G<'_>) -> X, X> GameStepper<Z>
    for Map<A, F>
{
    type Result = X;
    type Int = A::Int;
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<Self::Int> {
        self.elem.step(game)
    }
    fn consume(self, game: &mut Z::G<'_>, a: Self::Int) -> Self::Result {
        let s = self.elem.consume(game, a);
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
    type Int = ();
    fn step(&mut self, _: &mut Z::G<'_>) -> Stage<()> {
        Stage::NextStage(())
    }
    fn consume(self, _: &mut Z::G<'_>, _: ()) -> Self::Result {
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
    type Int = Option<A::Int>;
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<Self::Int> {
        if let Some(a) = self.a.as_mut() {
            match a.step(game) {
                Stage::Stay => Stage::Stay,
                Stage::NextStage(a) => Stage::NextStage(Some(a)),
            }
        } else {
            Stage::NextStage(None)
        }
    }
    fn consume(self, game: &mut Z::G<'_>, i: Self::Int) -> Self::Result {
        if let Some(a) = self.a {
            Some(a.consume(game, i.unwrap()))
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
    type Int = B::Int;
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<Self::Int> {
        if let Some(a) = &mut self.first {
            match a.step(game) {
                Stage::Stay => Stage::Stay,
                Stage::NextStage(i) => {
                    let b = self.first.take().unwrap().consume(game, i);
                    self.second = Some(b);
                    Stage::Stay
                }
            }
        } else {
            self.second.as_mut().unwrap().step(game)
        }
    }
    fn consume(self, game: &mut Z::G<'_>, a: Self::Int) -> B::Result {
        self.second.unwrap().consume(game, a)
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
    type Int;
    //Return if you are done with this stage.
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<Self::Int>;

    fn consume(self, _: &mut Z::G<'_>, _: Self::Int) -> Self::Result
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

pub struct Looper<A, F, H> {
    start_func: H,
    a: Option<A>,
    func: F,
}

impl<
        Z: Zoo,
        A: GameStepper<Z>,
        K,
        F: FnMut(A::Result, &mut Z::G<'_>) -> LooperRes<P, K>,
        P,
        H: FnMut(P) -> A,
    > GameStepper<Z> for Looper<A, F, H>
{
    type Result = K;
    type Int = K;
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
    fn consume(self, _: &mut Z::G<'_>, a: K) -> Self::Result {
        a
    }
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<K> {
        

        let a = if let Some(a) = &mut self.a {
            match a.step(game) {
                Stage::Stay => {
                    return Stage::Stay;
                }
                Stage::NextStage(o) => self.a.take().unwrap().consume(game, o),
            }
        } else {
            unreachable!();
        };

        match (self.func)(a, game) {
            LooperRes::Loop(p) => {
                self.a = Some((self.start_func)(p));
                Stage::Stay
            }
            LooperRes::Finish(b) => Stage::NextStage(b),
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
) -> Looper<A, F, H> {
    let elem = start(start_val);

    Looper {
        a: Some(elem),
        func,
        start_func: start,
    }
}
