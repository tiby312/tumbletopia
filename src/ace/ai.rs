use crate::mesh::bitfield::BitField;

use super::*;

pub type Eval = i64;
const MATE: i64 = 1_000_000;

pub struct Evaluator {
    workspace: BitField,
    workspace2: BitField,
    workspace3: BitField,
}
impl Default for Evaluator {
    fn default() -> Self {
        Self {
            workspace: Default::default(),
            workspace2: Default::default(),
            workspace3: Default::default(),
        }
    }
}
impl Evaluator {
    pub fn cant_move(&mut self, team: ActiveTeam) -> Eval {
        match team {
            ActiveTeam::Cats => -MATE,
            ActiveTeam::Dogs => MATE,
        }
    }
    //cats maximizing
    //dogs minimizing
    pub fn absolute_evaluate(
        &mut self,
        view: &GameState,
        world: &board::MyWorld,
        _debug: bool,
    ) -> Eval {
        // Doesnt seem that important that the AI knows exactly when it is winning.
        // (There doesnt seem to be many tactical combinations near the end of the game).
        // match view.game_is_over() {
        //     Some(GameOver::CatWon) => {
        //         return MATE;
        //     }
        //     Some(GameOver::DogWon) => {
        //         return -MATE;
        //     }
        //     Some(GameOver::Tie) => {}
        //     None => {}
        // }

        let ship_allowed = {
            let temp = &mut self.workspace;
            temp.clear();
            temp.union_with(&view.env.terrain.land);
            temp.toggle_range(..);
            temp.intersect_with(world.get_game_cells());
            temp
        };

        let num_cats = view.factions.cats.units.count_ones(..) as i64;
        let num_dogs = view.factions.dogs.units.count_ones(..) as i64;

        let mut cat_influence = view.factions.cats.units.clone();

        let mut dog_influence = view.factions.dogs.units.clone();

        doop(
            7,
            &mut dog_influence,
            &mut cat_influence,
            &ship_allowed,
            &mut self.workspace2,
            &mut self.workspace3,
        );

        let num_cat_influence = cat_influence.count_ones(..) as i64;
        let num_dog_influence = dog_influence.count_ones(..) as i64;

        let dog_distance = view
            .factions
            .dogs
            .units
            .iter_mesh()
            .map(|a| a.to_cube().dist(&Axial::zero().to_cube()) as i64)
            .sum::<i64>();
        let cat_distance = view
            .factions
            .cats
            .units
            .iter_mesh()
            .map(|a| a.to_cube().dist(&Axial::zero().to_cube()) as i64)
            .sum::<i64>();

        //The AI will try to avoid the center.
        //The more influlence is at stake, the more precious each piece is
        (num_cat_influence - num_dog_influence) * 100
            + (-cat_distance + dog_distance) * 1
            + (num_cats - num_dogs) * 2000
    }
}

fn around(point: Axial) -> impl Iterator<Item = Axial> {
    point.to_cube().ring(1).map(|b| b.to_axial())
}

pub fn expand_mesh(mesh: &mut BitField, workspace: &mut BitField) {
    workspace.clear();
    workspace.union_with(mesh);

    for a in workspace.iter_mesh() {
        for b in around(a) {
            if mesh.valid_coord(b) {
                mesh.set_coord(b, true);
            }
        }
    }
}

fn doop(
    iteration: usize,
    dogs: &mut BitField,
    cats: &mut BitField,
    allowed_cells: &BitField,
    cache: &mut BitField,
    mut cache2: &mut BitField,
) {
    dogs.intersect_with(allowed_cells);
    cats.intersect_with(allowed_cells);
    if dogs.count_ones(..) == 0 && cats.count_ones(..) == 0 {
        return;
    }

    // let mut cache2 = BitField::new();
    // let mut cache = BitField::new();

    for _i in 0..iteration {
        cache.clear();
        cache.union_with(dogs);
        expand_mesh(dogs, cache2);
        let dogs_changed = cache != dogs;
        cache.clear();
        cache.union_with(cats);
        expand_mesh(cats, cache2);
        let cats_changed = cache != cats;
        if !dogs_changed && !cats_changed {
            break;
        }
        dogs.intersect_with(allowed_cells);
        cats.intersect_with(allowed_cells);

        cache2.clear();
        let contested = &mut cache2;
        contested.union_with(dogs);
        contested.intersect_with(cats);

        contested.toggle_range(..);

        dogs.intersect_with(&contested);
        cats.intersect_with(&contested);
    }
}

//TODO use bump allocator!!!!!
struct PrincipalVariation {
    a: std::collections::BTreeMap<Vec<moves::ActualMove>, (moves::ActualMove,Eval)>,
}
impl PrincipalVariation {
    pub fn get_best_prev_move(&self, path: &[moves::ActualMove]) -> Option<&(moves::ActualMove,Eval)> {
        self.a.get(path)
    }
    pub fn get_best_prev_move_mut(
        &mut self,
        path: &[moves::ActualMove],
    ) -> Option<&mut (moves::ActualMove,Eval)> {
        self.a.get_mut(path)
    }

    pub fn update(&mut self, path: &[moves::ActualMove], aaa: &moves::ActualMove,eval:Eval) {
        //if let Some(aaa) = &ret {
        if let Some(foo) = self.get_best_prev_move_mut(path) {
            *foo = (aaa.clone(),eval);
        } else {
            self.insert(path, aaa.clone(),eval);
        }
        //}
    }
    pub fn insert(&mut self, path: &[moves::ActualMove], m: moves::ActualMove,eval:Eval) {
        self.a.insert(path.to_vec(), (m,eval));
    }
}

pub fn iterative_deepening(
    game: &GameState,
    world: &board::MyWorld,
    team: ActiveTeam,
) -> moves::ActualMove {
    let mut count = Counter { count: 0 };
    let mut results = Vec::new();

    let max_iterative_depth = 4;
    //let max_depth = 2;

    let mut foo1 = PrincipalVariation {
        a: std::collections::BTreeMap::new(),
    };
    let mut evaluator = Evaluator::default();

    //TODO stop searching if we found a game ending move.
    for depth in 1..max_iterative_depth {
        console_dbg!("searching", depth);

        //TODO should this be outside the loop?
        let mut k = KillerMoves::new(max_iterative_depth + 4 + 4);

        let mut aaaa = ai::AlphaBeta {
            //table: &mut table,
            prev_cache: &mut foo1,
            calls: &mut count,
            path: &mut vec![],
            killer_moves: &mut k,
            max_ext: 0,
        };

        let mut kk = game.clone();
        let res = aaaa.alpha_beta(
            &mut kk,
            world,
            ABAB::new(),
            team,
            0,
            depth,
            0,
            &mut evaluator,
        );
        assert_eq!(&kk, game);

        let mov = foo1.a.get(&[] as &[_]).cloned().unwrap();
        let res = EvalRet { mov, eval: res };

        let eval = res.eval;
        console_dbg!(eval);

        results.push(res);

        if eval.abs() == MATE {
            console_dbg!("found a mate");
            break;
        }

        //doop.poke(team, game.clone()).await;
    }

    console_dbg!(count);
    console_dbg!(&results);

    let _target_eval = results.last().unwrap().eval;
    // let mov = if let Some(a) = results
    //     .iter()
    //     .rev()
    //     .find(|a| a.eval == target_eval && a.mov != ActualMove::SkipTurn)
    // {
    //     a.clone()
    // } else {
    //     results.pop().unwrap()
    // };
    let mov = results.pop().unwrap();

    let m = mov;

    console_dbg!("AI MOVE::", m.mov, m.eval);

    m.mov.0
}

#[derive(Debug)]
struct Counter {
    count: u128,
}
impl Counter {
    pub fn add_eval(&mut self) {
        self.count += 1;
    }
}

struct AlphaBeta<'a> {
    //table: &'a mut LeafTranspositionTable,
    prev_cache: &'a mut PrincipalVariation,
    calls: &'a mut Counter,
    path: &'a mut Vec<moves::ActualMove>,
    killer_moves: &'a mut KillerMoves,
    max_ext: usize,
}

struct KillerMoves {
    a: Vec<smallvec::SmallVec<[moves::ActualMove; 2]>>,
}

impl KillerMoves {
    pub fn new(a: usize) -> Self {
        let v = (0..a).map(|_| smallvec::SmallVec::new()).collect();
        Self { a: v }
    }
    pub fn get(&mut self, depth: usize) -> &mut [moves::ActualMove] {
        &mut self.a[depth]
    }
    pub fn consider(&mut self, depth: usize, m: moves::ActualMove) {
        let a = &mut self.a[depth];

        if a.contains(&m) {
            return;
        }
        if a.len() < 2 {
            a.push(m);
        } else {
            a.swap(0, 1);
            a[0] = m;
        }
    }
}

#[derive(Debug, Clone)]
struct EvalRet<T> {
    pub mov: T,
    pub eval: Eval,
}

impl<'a> AlphaBeta<'a> {
    fn alpha_beta(
        &mut self,
        game_after_move: &mut GameState,
        world: &board::MyWorld,
        mut ab: ABAB,
        team: ActiveTeam,
        depth: usize,
        max_depth: usize,
        ext: usize,
        evaluator: &mut Evaluator,
    ) -> Eval {
        self.max_ext = self.max_ext.max(ext);

        if depth >= max_depth + 2 {
            self.calls.add_eval();
            return evaluator.absolute_evaluate(game_after_move, world, false);
        }

        let mut quiet_position = true;
        let mut moves = vec![];

        game_after_move.for_all_moves_fast(team, world, |e, m| {
            if e.move_effect.destroyed_unit.is_some() {
                quiet_position = false;
            }

            if depth < max_depth {
                moves.push(m);
            } else {
                if e.move_effect.destroyed_unit.is_some() {
                    moves.push(m)
                }
            }
        });

        if depth >= max_depth && quiet_position {
            self.calls.add_eval();
            return evaluator.absolute_evaluate(game_after_move, world, false);
        }

        if moves.is_empty() && depth < max_depth {
            return evaluator.cant_move(team);
        }

        let mut num_sorted = 0;
        //TODO principal variation does not seem to be helping much
        // if let Some(p) = self.prev_cache.get_best_prev_move(self.path) {
        //     let f = moves.iter().enumerate().find(|(_, x)| **x == *p).unwrap();
        //     let swap_ind = f.0;
        //     moves.swap(0, swap_ind);
        //     num_sorted += 1;
        // }
        

        {
            let ind=if team==ActiveTeam::Cats{
                moves[num_sorted..].iter().enumerate().max_by_key(|&(_,x)|{
                    let mut num=0;
                    self.path.push(x.clone());
                    if let Some((_,k))=self.prev_cache.get_best_prev_move(&self.path){
                        num=*k;
                    }
                    self.path.pop();
                    num
                })
            }else{
                moves[num_sorted..].iter().enumerate().min_by_key(|&(_,x)|{
                    let mut num=0;
                    self.path.push(x.clone());
                    if let Some((_,k))=self.prev_cache.get_best_prev_move(&self.path){
                        num=*k;
                    }
                    self.path.pop();
                    num
                })
            };

            if let Some((ind,_))=ind{
            
                moves.swap(ind,num_sorted);
                num_sorted+=1;
            }
        }

        
        for a in self.killer_moves.get(usize::try_from(depth).unwrap()) {
            if let Some((x, _)) = moves[num_sorted..]
                .iter()
                .enumerate()
                .find(|(_, x)| *x == a)
            {
                moves.swap(x, num_sorted);
                num_sorted += 1;
            }
        }





        let (eval, m) = if team == ActiveTeam::Cats {
            self.floopy(
                depth,
                max_depth,
                ext,
                evaluator,
                team,
                game_after_move,
                world,
                ab,
                abab::Maximizer,
                moves,
            )
        } else {
            self.floopy(
                depth,
                max_depth,
                ext,
                evaluator,
                team,
                game_after_move,
                world,
                ab,
                abab::Minimizer,
                moves,
            )
        };

        if let Some(kk) = m {
            self.prev_cache.update(self.path, &kk,eval);
        }
        eval
    }

    fn floopy<D: ace::ai::abab::Doop>(
        &mut self,
        depth: usize,
        max_depth: usize,
        ext: usize,
        evaluator: &mut Evaluator,
        team: ActiveTeam,
        game_after_move: &mut GameState,
        world: &board::MyWorld,
        mut ab: ABAB,
        doop: D,
        moves: Vec<moves::ActualMove>,
    ) -> (i64, Option<moves::ActualMove>) {
        let mut ab_iter = ab.ab_iter(doop);
        for cand in moves {
            let effect = {
                let j = cand.as_move();
                let k = j.apply(team, game_after_move, world);
                let j = j
                    .into_attack(cand.attackto)
                    .apply(team, game_after_move, world, &k);
                k.combine(j)
            };

            self.path.push(cand);
            let eval = self.alpha_beta(
                game_after_move,
                world,
                ab_iter.clone_ab_values(),
                team.not(),
                depth + 1,
                max_depth,
                ext,
                evaluator,
            );

            let mov = self.path.pop().unwrap();
            {
                let k = mov.as_extra();
                k.undo(&effect.extra_effect, game_after_move).undo(
                    team,
                    &effect.move_effect,
                    game_after_move,
                );
            }

            let keep_going = ab_iter.consider(&mov, eval);
            
            if !keep_going {
                self.killer_moves.consider(depth, mov);
                break;
            }
        }
        ab_iter.finish()
    }
}

use abab::ABAB;
mod abab {
    use super::*;
    #[derive(Clone)]
    pub struct ABAB {
        alpha: Eval,
        beta: Eval,
    }

    pub trait Doop {
        fn maximizing(&self) -> bool;
    }
    pub struct Maximizer;
    impl Doop for Maximizer {
        fn maximizing(&self) -> bool {
            true
        }
    }
    pub struct Minimizer;
    impl Doop for Minimizer {
        fn maximizing(&self) -> bool {
            false
        }
    }

    pub struct ABIter<'a, T, D: Doop> {
        value: i64,
        a: &'a mut ABAB,
        mm: Option<T>,
        keep_going: bool,
        doop: D,
    }

    impl<'a, T: Clone, D: Doop> ABIter<'a, T, D> {
        pub fn finish(self) -> (Eval, Option<T>) {
            (self.value, self.mm)
        }
        pub fn clone_ab_values(&self) -> ABAB {
            self.a.clone()
        }
        pub fn consider(&mut self, t: &T, eval: Eval) -> bool {
            
            //TODO should be less than or equal instead maybe?
            let mmm = if self.doop.maximizing() {
                eval > self.value
            } else {
                eval < self.value
            };
            if mmm {
                self.mm = Some(t.clone());
                self.value = eval;
            }

            let cond = if self.doop.maximizing() {
                eval > self.a.beta
            } else {
                eval < self.a.alpha
            };

            if cond {
                assert!(mmm);
                self.keep_going = false;
            }

            if self.doop.maximizing() {
                self.a.alpha = self.a.alpha.max(self.value);
            } else {
                self.a.beta = self.a.beta.min(self.value);
            }

            self.keep_going
        }
    }

    impl ABAB {
        pub fn new() -> Self {
            ABAB {
                alpha: Eval::MIN,
                beta: Eval::MAX,
            }
        }

        pub fn ab_iter<T: Clone, D: Doop>(&mut self, doop: D) -> ABIter<T, D> {
            let value = if doop.maximizing() {
                i64::MIN
            } else {
                i64::MAX
            };
            ABIter {
                value,
                a: self,
                mm: None,
                keep_going: true,
                doop,
            }
        }
    }
}
