pub enum AndThen<A, B, N> {
    First(A, B),
    Second(N),
}
impl<G, A: GameStepper<G>, K: GameStepper<G>, B: FnMut(A::Result, &mut G) -> K> GameStepper<G>
    for AndThen<A, B, K>
{
    type Result = K::Result;
    //Return if you are done with this stage.
    fn step(&mut self, game: &mut G) -> Stage<Self::Result> {
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
pub struct WaitForCustom<F> {
    func: F,
}
pub fn wait_custom<G, F: FnMut(&mut G) -> Stage<L>, L>(func: F) -> WaitForCustom<F> {
    WaitForCustom { func }
}

impl<F: FnMut(&mut G) -> Stage<L>, L, G> GameStepper<G> for WaitForCustom<F> {
    type Result = L;
    fn step(&mut self, game: &mut G) -> Stage<Self::Result> {
        (self.func)(game)
    }
}

pub struct Game;
pub trait GameStepper<G> {
    type Result;
    //Return if you are done with this stage.
    fn step(&mut self, game: &mut G) -> Stage<Self::Result>;

    fn and_then<K: GameStepper<G>, B: FnMut(Self::Result, &mut G) -> K>(
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
pub fn looper<G, A: GameStepper<G>, F: FnMut(&mut G) -> A>(func: F) -> Looper<A, F> {
    Looper { a: None, func }
}

impl<G, A: GameStepper<G>, F: FnMut(&mut G) -> A> GameStepper<G> for Looper<A, F> {
    type Result = A::Result;
    fn step(&mut self, game: &mut G) -> Stage<Self::Result> {
        if let Some(mut a) = self.a.take() {
            match a.step(game) {
                Stage::Stay => {
                    self.a = Some(a);
                }
                Stage::NextStage(o) => self.a = Some((self.func)(game)),
            }
        } else {
            self.a = Some((self.func)(game))
        }
        Stage::Stay
    }
}
