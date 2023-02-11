pub enum AndThen<A, B, N> {
    First(A, B),
    Second(N),
}
impl<'a, Z: Zoo, A: GameStepper<Z>, K: GameStepper<Z>, B: FnMut(A::Result, &mut Z::G<'_>) -> K>
    GameStepper<Z> for AndThen<A, B, K>
{
    type Result = K::Result;
    //Return if you are done with this stage.
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<Self::Result> {
        match self {
            AndThen::First(a, b) => {
                match a.step(game) {
                    Stage::Stay => {}
                    Stage::NextStage(j) => {
                        let nn = b(j, game);
                        *self = AndThen::Second(nn);
                    }
                }
                Stage::Stay
            }
            AndThen::Second(n) => n.step(game),
        }
    }
}

pub enum Stage<T> {
    NextStage(T),
    Stay,
}

#[derive(Copy, Clone)]
pub struct WaitForCustom<Z, F> {
    zoo: Z,
    func: F,
}
pub fn wait_custom<L, Z: Zoo, F: FnMut(&mut Z::G<'_>) -> Stage<L>>(
    zoo: Z,
    func: F,
) -> WaitForCustom<Z, F> {
    WaitForCustom { zoo, func }
}

pub trait Zoo {
    type G<'b>
    where
        Self: 'b;
}

impl<L, Z: Zoo, F: FnMut(&mut Z::G<'_>) -> Stage<L>> GameStepper<Z> for WaitForCustom<Z, F> {
    type Result = L;
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<Self::Result> {
        (self.func)(game)
    }
}

pub fn empty() -> Empty {
    Empty
}

#[derive(Copy, Clone)]
pub struct Empty;
impl<Z: Zoo> GameStepper<Z> for Empty {
    type Result = ();
    fn step(&mut self, _: &mut Z::G<'_>) -> Stage<Self::Result> {
        Stage::NextStage(())
    }
}

pub trait GameStepper<Z: Zoo> {
    type Result;
    //Return if you are done with this stage.
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<Self::Result>;

    //TODO use this!
    //fn draw(&mut self, game: &Z::G<'_>){}

    fn and_then<K: GameStepper<Z>, B: FnMut(Self::Result, &mut Z::G<'_>) -> K>(
        self,
        other: B,
    ) -> AndThen<Self, B, K>
    where
        Self: Sized,
    {
        AndThen::First(self, other)
    }
}

pub struct Looper<Z, A, F> {
    zoo: Z,
    a: Option<A>,
    func: F,
}
pub fn looper<Z: Zoo, A: GameStepper<Z>, F: FnMut(&mut Z::G<'_>) -> Option<A>>(
    zoo: Z,
    func: F,
) -> Looper<Z, A, F> {
    Looper { a: None, func, zoo }
}

impl<Z: Zoo, A: GameStepper<Z>, F: FnMut(&mut Z::G<'_>) -> Option<A>> GameStepper<Z>
    for Looper<Z, A, F>
{
    type Result = Empty;
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<Self::Result> {
        if let Some(mut a) = self.a.take() {
            match a.step(game) {
                Stage::Stay => {
                    self.a = Some(a);
                    Stage::Stay
                }
                Stage::NextStage(_) => {
                    if let Some(jj) = (self.func)(game) {
                        self.a = Some(jj);
                        Stage::Stay
                    } else {
                        Stage::NextStage(empty())
                    }
                }
            }
        } else {
            if let Some(jj) = (self.func)(game) {
                self.a = Some(jj);
                Stage::Stay
            } else {
                Stage::NextStage(empty())
            }
        }
    }
}
