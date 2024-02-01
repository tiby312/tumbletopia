use crate::state::{GameHandle, Stuff};

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
    fn get_animation(&self) -> Option<&crate::animation::Animation<WarriorPointer<Warrior>>> {
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

pub fn next<Z: Zoo>() -> Next<Z> {
    Next(Z::create())
}


#[derive(Copy, Clone)]
pub struct Next<Z>(Z);
impl<Z: Zoo> GameStepper<Z> for Next<Z> {
    type Result = ();
    type Int = ();
    fn step(&mut self, _: &mut Z::G<'_>) -> Stage<()> {
        Stage::NextStage(())
    }
    fn consume(self, _: &mut Z::G<'_>, _: ()) -> Self::Result {
        ()
    }
}


pub struct Or<A, B> {
    a: A,
    b: B,
}
impl<Z: Zoo, A: GameStepper<Z>, B: GameStepper<Z, Result = A::Result>> GameStepper<Z> for Or<A, B> {
    type Result = A::Result;
    type Int = Either<A::Int, B::Int>;

    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<Self::Int> {
        let a = self.a.step(game);

        match a {
            Stage::NextStage(a) => return Stage::NextStage(Either::A(a)),
            Stage::Stay => {}
        }

        let b = self.b.step(game);

        match b {
            Stage::NextStage(a) => return Stage::NextStage(Either::B(a)),
            Stage::Stay => {}
        }

        Stage::Stay
    }

    fn consume(self, game: &mut Z::G<'_>, i: Self::Int) -> Self::Result {
        match i {
            Either::A(a) => self.a.consume(game, a),
            Either::B(a) => self.b.consume(game, a),
        }
    }
    fn get_selection(&self) -> Option<&crate::CellSelection> {
        //TODO correct behavior?
        self.a.get_selection()
    }

    fn get_animation(&self) -> Option<&crate::animation::Animation<WarriorPointer<Warrior>>> {
        self.a.get_animation()
    }
}

pub enum Either<A, B> {
    A(A),
    B(B),
}
impl<Z: Zoo, A: GameStepper<Z>, B: GameStepper<Z>> GameStepper<Z> for Either<A, B> {
    type Result = Either<A::Result, B::Result>;
    type Int = Either<A::Int, B::Int>;

    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<Self::Int> {
        match self {
            Either::A(a) => match a.step(game) {
                Stage::NextStage(a) => Stage::NextStage(Either::A(a)),
                Stage::Stay => Stage::Stay,
            },
            Either::B(a) => match a.step(game) {
                Stage::NextStage(a) => Stage::NextStage(Either::B(a)),
                Stage::Stay => Stage::Stay,
            },
        }
    }

    fn consume(self, game: &mut Z::G<'_>, i: Self::Int) -> Self::Result {
        match (self, i) {
            (Either::A(a), Either::A(aa)) => Either::A(a.consume(game, aa)),
            (Either::B(b), Either::B(bb)) => Either::B(b.consume(game, bb)),
            _ => unreachable!(),
        }
    }

    fn get_selection(&self) -> Option<&crate::CellSelection> {
        match self {
            Either::A(a) => a.get_selection(),
            Either::B(a) => a.get_selection(),
        }
    }

    fn get_animation(&self) -> Option<&crate::animation::Animation<WarriorPointer<Warrior>>> {
        match self {
            Either::A(a) => a.get_animation(),
            Either::B(a) => a.get_animation(),
        }
    }
}

// pub struct Optional<A> {
//     a: Option<A>,
// }

// pub fn optional<Z: Zoo, A: GameStepper<Z>>(a: Option<A>) -> Option<A> {
//     a
// }

impl<Z: Zoo, A: GameStepper<Z>> GameStepper<Z> for Option<A> {
    type Result = Option<A::Result>;
    type Int = Option<A::Int>;
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<Self::Int> {
        if let Some(a) = self.as_mut() {
            match a.step(game) {
                Stage::Stay => Stage::Stay,
                Stage::NextStage(a) => Stage::NextStage(Some(a)),
            }
        } else {
            Stage::NextStage(None)
        }
    }
    fn consume(self, game: &mut Z::G<'_>, i: Self::Int) -> Self::Result {
        if let Some(a) = self {
            Some(a.consume(game, i.unwrap()))
        } else {
            None
        }
    }
    fn get_selection(&self) -> Option<&crate::CellSelection> {
        if let Some(a) = self.as_ref() {
            if let Some(b) = a.get_selection() {
                Some(b)
            } else {
                None
            }
        } else {
            None
        }
    }
    fn get_animation(&self) -> Option<&crate::animation::Animation<WarriorPointer<Warrior>>> {
        if let Some(a) = self.as_ref() {
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

enum EitherOr<A, B> {
    A(A),
    B(B),
    None,
}
impl<A, B> EitherOr<A, B> {
    fn take(&mut self) -> Self {
        let mut k = EitherOr::None;
        std::mem::swap(&mut k, self);
        k
    }
    fn unwrap_a(self) -> A {
        match self {
            EitherOr::A(a) => a,
            _ => unreachable!(),
        }
    }

    fn unwrap_b(self) -> B {
        match self {
            EitherOr::B(a) => a,
            _ => unreachable!(),
        }
    }
}

pub struct Chain<A, B> {
    inner: EitherOr<A, B>,
}
impl<Z: Zoo, A: GameStepper<Z, Result = B>, B: GameStepper<Z>> GameStepper<Z> for Chain<A, B> {
    type Result = B::Result;
    type Int = B::Int;
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<Self::Int> {
        match &mut self.inner {
            EitherOr::A(a) => match a.step(game) {
                Stage::Stay => Stage::Stay,
                Stage::NextStage(i) => {
                    let b = self.inner.take().unwrap_a().consume(game, i);
                    self.inner = EitherOr::B(b);
                    Stage::Stay
                }
            },
            EitherOr::B(b) => b.step(game),
            EitherOr::None => unreachable!(),
        }
    }
    fn consume(self, game: &mut Z::G<'_>, a: Self::Int) -> B::Result {
        self.inner.unwrap_b().consume(game, a)
    }

    fn get_selection(&self) -> Option<&crate::CellSelection> {
        match &self.inner {
            EitherOr::A(a) => a.get_selection(),
            EitherOr::B(a) => a.get_selection(),
            EitherOr::None => unreachable!(),
        }
    }
    fn get_animation(&self) -> Option<&crate::animation::Animation<WarriorPointer<Warrior>>> {
        match &self.inner {
            EitherOr::A(a) => a.get_animation(),
            EitherOr::B(a) => a.get_animation(),
            EitherOr::None => unreachable!(),
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
    fn get_animation(&self) -> Option<&crate::animation::Animation<WarriorPointer<Warrior>>> {
        None
    }

    fn or<O: GameStepper<Z, Result = Self::Result>>(self, other: O) -> Or<Self, O>
    where
        Self: Sized,
    {
        Or { a: self, b: other }
    }

    fn optional_some(self) -> Option<Self>
    where
        Self: Sized,
    {
        Some(self)
    }

    fn flatten(self) -> Chain<Self, Self::Result>
    where
        Self::Result: GameStepper<Z> + Sized,
        Self: Sized,
    {
        Chain {
            inner: EitherOr::A(self),
        }
    }

    fn map<K, B: FnOnce(Self::Result, &mut Z::G<'_>) -> K>(self, func: B) -> Map<Self, B>
    where
        Self: Sized,
    {
        Map { elem: self, func }
    }

    fn either_a<X>(self) -> Either<Self, X>
    where
        Self: Sized,
    {
        Either::A(self)
    }

    fn either_b<X>(self) -> Either<X, Self>
    where
        Self: Sized,
    {
        Either::B(self)
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

#[derive(Copy, Clone)]
pub struct Looper<A, H, Lo> {
    val: Option<Lo>,
    start_func: H,
    a: Option<A>,
}

impl<
        Z: Zoo,
        A: GameStepper<Z, Result = LooperRes<Lo, Res>>,
        Lo,
        Res,
        H: FnMut(Lo, &mut Z::G<'_>) -> A,
    > GameStepper<Z> for Looper<A, H, Lo>
{
    type Result = Res;
    type Int = Res;
    fn get_selection(&self) -> Option<&crate::CellSelection> {
        self.a.as_ref().map(|a| a.get_selection()).flatten()
    }
    fn get_animation(&self) -> Option<&crate::animation::Animation<WarriorPointer<Warrior>>> {
        self.a.as_ref().map(|a| a.get_animation()).flatten()
    }
    fn consume(self, g: &mut Z::G<'_>, a: Self::Int) -> Self::Result {
        a
    }
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<Self::Int> {
        if self.a.is_none() {
            //First iteration
            self.a = Some((self.start_func)(self.val.take().unwrap(), game));
        }
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

        match a {
            LooperRes::Loop(lll) => {
                self.a = Some((self.start_func)(lll, game));
                Stage::Stay
            }
            LooperRes::Finish(res) => Stage::NextStage(res),
        }
    }
}

pub fn looper<
    Res,
    Lo,
    A: GameStepper<GameHandle, Result = LooperRes<Lo, Res>>,
    H: FnMut(Lo, &mut Stuff) -> A,
>(
    val: Lo,
    mut start: H,
) -> Looper<A, H, Lo> {
    Looper {
        val: Some(val),
        a: None,
        start_func: start,
    }
}
