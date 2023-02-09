pub enum AndThen<A, B, N> {
    First(A, B),
    Second(N),
}
impl<G, E, A: GameStepper<G, E>, K: GameStepper<G, E>, B: FnMut(A::Result, &mut G, &E) -> K>
    GameStepper<G, E> for AndThen<A, B, K>
{
    type Result = K::Result;
    //Return if you are done with this stage.
    fn step(&mut self, game: &mut G, events: &E) -> Stage<Self::Result> {
        match self {
            AndThen::First(a, b) => {
                match a.step(game, events) {
                    Stage::Stay => {}
                    Stage::NextStage(j) => {
                        let nn = b(j, game, events);
                        *self = AndThen::Second(nn);
                    }
                }
                Stage::Stay
            }
            AndThen::Second(n) => n.step(game, events),
        }
    }
}

pub enum Stage<T> {
    NextStage(T),
    Stay,
}

#[derive(Copy, Clone)]
pub struct WaitForCustom<F> {
    func: F,
}
pub fn wait_custom<G, E, F: FnMut(&mut G, &E) -> Stage<L>, L>(func: F) -> WaitForCustom<F> {
    WaitForCustom { func }
}

impl<F: FnMut(&mut G, &E) -> Stage<L>, L, G, E> GameStepper<G, E> for WaitForCustom<F> {
    type Result = L;
    fn step(&mut self, game: &mut G, events: &E) -> Stage<Self::Result> {
        (self.func)(game, events)
    }
}

pub fn empty() -> Empty {
    Empty
}

#[derive(Copy, Clone)]
pub struct Empty;
impl<G, E> GameStepper<G, E> for Empty {
    type Result = ();
    fn step(&mut self, _: &mut G, _: &E) -> Stage<Self::Result> {
        Stage::NextStage(())
    }
}

pub trait GameStepper<G, E> {
    type Result;
    //Return if you are done with this stage.
    fn step(&mut self, game: &mut G, events: &E) -> Stage<Self::Result>;

    fn and_then<K: GameStepper<G, E>, B: FnMut(Self::Result, &mut G, &E) -> K>(
        self,
        other: B,
    ) -> AndThen<Self, B, K>
    where
        Self: Sized,
    {
        AndThen::First(self, other)
    }
}

pub struct Looper<A, F> {
    a: Option<A>,
    func: F,
}
pub fn looper<G, E, A: GameStepper<G, E>, F: FnMut(&mut G, &E) -> A>(func: F) -> Looper<A, F> {
    Looper { a: None, func }
}

impl<G, E, A: GameStepper<G, E>, F: FnMut(&mut G, &E) -> A> GameStepper<G, E> for Looper<A, F> {
    type Result = A::Result;
    fn step(&mut self, game: &mut G, events: &E) -> Stage<Self::Result> {
        if let Some(mut a) = self.a.take() {
            match a.step(game, events) {
                Stage::Stay => {
                    self.a = Some(a);
                }
                Stage::NextStage(_) => self.a = Some((self.func)(game, events)),
            }
        } else {
            self.a = Some((self.func)(game, events))
        }
        Stage::Stay
    }
}
