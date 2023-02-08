
pub enum AndThen<A, B, N> {
    First(A, B),
    Second(N),
}
impl<A: GameStepper, K: GameStepper, B: FnMut(A::Result, &mut Game) -> K> GameStepper
    for AndThen<A, B, K>
{
    type Result = K::Result;
    //Return if you are done with this stage.
    fn step(&mut self, game: &mut Game, mouse: Option<[f32; 2]>) -> Option<Self::Result> {
        match self {
            AndThen::First(a, b) => {
                if let Some(j) = a.step(game, mouse) {
                    let nn = b(j, game);
                    *self = AndThen::Second(nn);
                }
                None
            }
            AndThen::Second(n) => n.step(game, mouse),
        }
    }
}

pub struct WaitForInput;
impl GameStepper for WaitForInput {
    type Result = [f32; 2];
    fn step(&mut self, game: &mut Game, mouse: Option<[f32; 2]>) -> Option<Self::Result> {
        mouse
    }
}
pub struct Empty;
impl GameStepper for Empty {
    type Result = ();
    fn step(&mut self, game: &mut Game, mouse: Option<[f32; 2]>) -> Option<Self::Result> {
        Some(())
    }
}




pub struct Game;
pub trait GameStepper {
    type Result;
    //Return if you are done with this stage.
    fn step(&mut self, game: &mut Game, mouse: Option<[f32; 2]>) -> Option<Self::Result>;

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
    fn step(&mut self, game: &mut Game, mouse: Option<[f32; 2]>) -> Option<Self::Result> {
        if let Some(mut a) = self.a.take() {
            if let Some(_) = a.step(game, mouse) {
                self.a = Some((self.func)(game))
            } else {
                self.a = Some(a);
            }
        } else {
            self.a = Some((self.func)(game))
        }
        None
    }
}