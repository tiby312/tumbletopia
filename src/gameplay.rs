use super::*;

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
    fn create() -> Self;
}

impl<L, Z: Zoo, F: FnMut(&mut Z::G<'_>) -> Stage<L>> GameStepper<Z> for WaitForCustom<Z, F> {
    type Result = L;
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<Self::Result> {
        (self.func)(game)
    }
}

pub fn next() -> Next {
    Next
}

#[derive(Copy, Clone)]
pub struct Next;
impl<Z: Zoo> GameStepper<Z> for Next {
    type Result = ();
    fn step(&mut self, _: &mut Z::G<'_>) -> Stage<Self::Result> {
        Stage::NextStage(())
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
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<Self::Result> {
        if let Some(a) = self.a.as_mut() {
            match a.step(game) {
                Stage::Stay => Stage::Stay,
                Stage::NextStage(e) => Stage::NextStage(Some(e)),
            }
        } else {
            Stage::NextStage(None)
        }
    }
    fn get_animation(&mut self, game: &Z::G<'_>) -> Option<&crate::animation::Animation<Warrior>> {
        todo!()
        //self.a.as_ref().map(|a|a.get_animation())
    }
}

pub trait GameStepper<Z: Zoo> {
    type Result;
    //Return if you are done with this stage.
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<Self::Result>;

    fn get_animation(&mut self, game: &Z::G<'_>) -> Option<&crate::animation::Animation<Warrior>> {
        None
    }

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

pub struct Looper2<Z, A, F> {
    zoo: Z,
    a: Option<A>,
    func: F,
    finished: bool,
}

pub struct Once<Z, A, K> {
    zoo: Z,
    func: Option<A>,
    floop: Option<K>,
}
pub fn once<Z: Zoo, A: FnOnce(&mut Z::G<'_>) -> L, L: GameStepper<Z>>(
    zoo: Z,
    func: A,
) -> Once<Z, A, L> {
    Once {
        zoo,
        func: Some(func),
        floop: None,
    }
}
impl<Z: Zoo, A: FnOnce(&mut Z::G<'_>) -> L, L: GameStepper<Z>> GameStepper<Z> for Once<Z, A, L> {
    type Result = L::Result;
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<Self::Result> {
        if let Some(func) = self.func.take() {
            let a = func(game);
            self.floop = Some(a);
            Stage::Stay
        } else {
            if let Some(aa) = self.floop.as_mut() {
                aa.step(game)
            } else {
                unreachable!()
            }
        }
    }
}

// pub struct Fuse<A>{
//     a:A,
//     voo:bool
// }
// impl<Z:Zoo,A:GameStepper<Z>> GameStepper<Z> for Fuse<A>{
//     type Result=A::Result;
//     fn step(&mut self,game:&mut Z::G<'_>)->Stage<Self::Result>{
//         if voo{

//         }
//         match self.a.step(game){
//             Stage::NextStage(a)=>{
//                 Stage::NextStage(a)
//             },
//             Stage::Stay=>{
//                 Stage::Stay
//             }
//         }

//     }
// }

pub enum LooperRes<A, B> {
    Loop(A),
    Finish(B),
}

impl<Z: Zoo, A: GameStepper<Z>, K, F: FnMut(A::Result, &mut Z::G<'_>) -> LooperRes<A, K>>
    GameStepper<Z> for Looper2<Z, A, F>
{
    type Result = K;
    fn step(&mut self, game: &mut Z::G<'_>) -> Stage<Self::Result> {
        if self.finished {
            return Stage::Stay;
        }

        let a = if let Some(a) = &mut self.a {
            match a.step(game) {
                Stage::Stay => {
                    return Stage::Stay;
                }
                Stage::NextStage(a) => a,
            }
        } else {
            log!("hayaaa");
            unreachable!();
        };

        match (self.func)(a, game) {
            LooperRes::Loop(a) => {
                log!("staying");
                self.a = Some(a);
                log!("staying2");
                Stage::Stay
            }
            LooperRes::Finish(b) => {
                self.finished = true;
                log!("Finished!!!!!");
                Stage::NextStage(b)
            }
        }
    }
}

pub fn looper2<
    Z: Zoo,
    A: GameStepper<Z>,
    K,
    F: FnMut(A::Result, &mut Z::G<'_>) -> LooperRes<A, K>,
>(
    start: A,
    func: F,
) -> Looper2<Z, A, F> {
    Looper2 {
        zoo: Z::create(),
        a: Some(start),
        func,
        finished: false,
    }
}

// pub fn looper<Z: Zoo, A: GameStepper<Z>, F: FnMut(&mut Z::G<'_>) -> Option<A>>(
//     zoo: Z,
//     func: F,
// ) -> Looper<Z, A, F> {
//     Looper { a: None, func, zoo }
// }

// impl<Z: Zoo, A: GameStepper<Z>, F: FnMut(&mut Z::G<'_>) -> Option<A>> GameStepper<Z>
//     for Looper<Z, A, F>
// {
//     type Result = Next;
//     fn step(&mut self, game: &mut Z::G<'_>) -> Stage<Self::Result> {
//         if let Some(mut a) = self.a.take() {
//             match a.step(game) {
//                 Stage::Stay => {
//                     self.a = Some(a);
//                     Stage::Stay
//                 }
//                 Stage::NextStage(_) => {
//                     if let Some(jj) = (self.func)(game) {
//                         self.a = Some(jj);
//                         Stage::Stay
//                     } else {
//                         Stage::NextStage(next())
//                     }
//                 }
//             }
//         } else {
//             if let Some(jj) = (self.func)(game) {
//                 self.a = Some(jj);
//                 Stage::Stay
//             } else {
//                 Stage::NextStage(next())
//             }
//         }
//     }
// }
