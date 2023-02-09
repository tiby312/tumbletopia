pub enum AndThen<A, B, N> {
    First(A, B),
    Second(N),
}
impl<A: GameStepper, K: GameStepper, B: FnMut(A::Result, &mut Game) -> K> GameStepper
    for AndThen<A, B, K>
{
    type Result = K::Result;
    //Return if you are done with this stage.
    fn step(&mut self, game: &mut Game, mouse: Option<[f32; 2]>) -> Stage<Self::Result> {
        match self {
            AndThen::First(a, b) => {
                match a.step(game, mouse) {
                    Stage::Stay => {}
                    Stage::NextStage(j) => {
                        let nn = b(j, game);
                        *self = AndThen::Second(nn);
                    }
                }
                Stage::Stay
            }
            AndThen::Second(n) => n.step(game, mouse),
        }
    }
}

pub enum Stage<T> {
    NextStage(T),
    Stay,
}


#[derive(Copy,Clone)]
pub struct WaitForCustom<F> {
    func: F,
}
impl<F: FnMut(&mut Game, Option<[f32; 2]>) -> Stage<L>, L> WaitForCustom<F> {
    pub fn new(func: F) -> Self {
        Self { func }
    }
}
impl<F: FnMut(&mut Game, Option<[f32; 2]>) -> Stage<L>, L> GameStepper for WaitForCustom<F> {
    type Result = L;
    fn step(&mut self, game: &mut Game, mouse: Option<[f32; 2]>) -> Stage<Self::Result> {
        (self.func)(game, mouse)
    }
}



pub struct Game;
pub trait GameStepper {
    type Result;
    //Return if you are done with this stage.
    fn step(&mut self, game: &mut Game, mouse: Option<[f32; 2]>) -> Stage<Self::Result>;

    fn and_then<K: GameStepper, B: FnMut(Self::Result, &mut Game) -> K>(
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
impl<A: GameStepper, F: FnMut(&mut Game) -> A> Looper<A, F> {
    pub fn new(func: F) -> Self {
        Self { a: None, func }
    }
}
impl<A: GameStepper, F: FnMut(&mut Game) -> A> GameStepper for Looper<A, F> {
    type Result = A::Result;
    fn step(&mut self, game: &mut Game, mouse: Option<[f32; 2]>) -> Stage<Self::Result> {
        if let Some(mut a) = self.a.take() {
            match a.step(game, mouse) {
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
