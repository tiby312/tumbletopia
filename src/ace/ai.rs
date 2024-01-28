use crate::movement::bitfield::BitField;

use super::*;

pub type Eval = i64;
const MATE: i64 = 1_000_000;

//cats maximizing
//dogs minimizing
pub fn absolute_evaluate(view: &GameState, _debug: bool) -> Eval {
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
        let water = {
            let mut t = view.env.land.grass.clone();
            t.union_with(&view.env.land.snow);
            t.toggle_range(..);
            t
        };
        let mut t = view.world.get_game_cells().clone();
        t.intersect_with(&water);
        t
    };

    let mut cat_ships = BitField::from_iter(
        view.factions
            .cats
            .iter()
            .map(|a| a.position)
            .filter(|&a| !view.env.land.is_coord_set(a)),
    );
    let mut dog_ships = BitField::from_iter(
        view.factions
            .dogs
            .iter()
            .map(|a| a.position)
            .filter(|&a| !view.env.land.is_coord_set(a)),
    );

    doop(7, &mut dog_ships, &mut cat_ships, &ship_allowed);

    let foot_snow = {
        let mut land = view.env.land.snow.clone();
        let mut t = view.env.forest.clone();
        t.toggle_range(..);
        land.intersect_with(&t);
        land
    };
    let mut cat_foot_snow = BitField::from_iter(
        view.factions
            .cats
            .iter()
            .filter(|a| a.typ == Type::Snow)
            .map(|a| a.position)
            .filter(|&a| view.env.land.snow.is_coord_set(a)),
    );
    let mut dog_foot_snow = BitField::from_iter(
        view.factions
            .dogs
            .iter()
            .filter(|a| a.typ == Type::Snow)
            .map(|a| a.position)
            .filter(|&a| view.env.land.snow.is_coord_set(a)),
    );

    doop(7, &mut dog_foot_snow, &mut cat_foot_snow, &foot_snow);

    let foot_grass = {
        let mut land = view.env.land.grass.clone();
        let mut t = view.env.forest.clone();
        t.toggle_range(..);
        land.intersect_with(&t);
        land
    };
    let mut cat_foot_grass = BitField::from_iter(
        view.factions
            .cats
            .iter()
            .filter(|a| a.typ == Type::Grass)
            .map(|a| a.position)
            .filter(|&a| view.env.land.grass.is_coord_set(a)),
    );
    let mut dog_foot_grass = BitField::from_iter(
        view.factions
            .dogs
            .iter()
            .filter(|a| a.typ == Type::Grass)
            .map(|a| a.position)
            .filter(|&a| view.env.land.grass.is_coord_set(a)),
    );

    doop(7, &mut dog_foot_grass, &mut cat_foot_grass, &foot_grass);

    let s = cat_ships.count_ones(..) as i64 - dog_ships.count_ones(..) as i64;
    let r = cat_foot_snow.count_ones(..) as i64 - dog_foot_snow.count_ones(..) as i64;
    let t = cat_foot_grass.count_ones(..) as i64 - dog_foot_grass.count_ones(..) as i64;

    //let x=0;
    //let y = cat_ship_grass.count_ones(..) as i64 - dog_ship_grass.count_ones(..) as i64;
    if _debug {
        //console_dbg!("SNOW VAL=",x);
        //console_dbg!("GRASS VAL=",x);
    }
    s + 2 * (r + t) //+ x + y
}

fn doop(
    iteration: usize,
    mut dogs: &mut BitField,
    mut cats: &mut BitField,
    allowed_cells: &BitField,
) {
    if dogs.count_ones(..) == 0 && cats.count_ones(..) == 0 {
        return;
    }

    fn around(point: GridCoord) -> impl Iterator<Item = GridCoord> {
        point.to_cube().ring(1).map(|(_, b)| b.to_axial())
    }

    fn expand_mesh(mesh: &mut BitField, workspace: &mut BitField) {
        workspace.clear();
        workspace.union_with(mesh);

        for a in workspace.iter_mesh(GridCoord([0; 2])) {
            for b in around(a) {
                mesh.set_coord(b, true);
            }
        }
    }

    let mut nomans = BitField::new();
    let mut w = BitField::new();
    let mut contested = BitField::new();

    let mut cache = BitField::new();

    for _i in 0..iteration {
        cache.clear();
        cache.union_with(dogs);
        expand_mesh(&mut dogs, &mut w);
        let dogs_changed = &cache != dogs;
        cache.clear();
        cache.union_with(cats);
        expand_mesh(&mut cats, &mut w);
        let cats_changed = &cache != cats;
        if !dogs_changed && !cats_changed {
            break;
        }
        dogs.intersect_with(&allowed_cells);
        cats.intersect_with(&allowed_cells);

        contested.clear();
        contested.union_with(dogs);
        contested.intersect_with(cats);
        nomans.union_with(&contested);

        contested.toggle_range(..);

        dogs.intersect_with(&contested);
        cats.intersect_with(&contested);
    }
}

//TODO use bump allocator!!!!!
pub struct PrincipalVariation {
    a: std::collections::HashMap<Vec<moves::ActualMove>, moves::ActualMove>,
}
impl PrincipalVariation {
    pub fn get_best_prev_move(&self, path: &[moves::ActualMove]) -> Option<&moves::ActualMove> {
        self.a.get(path)
    }
    pub fn get_best_prev_move_mut(
        &mut self,
        path: &[moves::ActualMove],
    ) -> Option<&mut moves::ActualMove> {
        self.a.get_mut(path)
    }

    pub fn update(&mut self, path: &[moves::ActualMove], aaa: &moves::ActualMove) {
        //if let Some(aaa) = &ret {
        if let Some(foo) = self.get_best_prev_move_mut(path) {
            *foo = aaa.clone();
        } else {
            self.insert(path, aaa.clone());
        }
        //}
    }
    pub fn insert(&mut self, path: &[moves::ActualMove], m: moves::ActualMove) {
        self.a.insert(path.iter().cloned().collect(), m);
    }
}

pub async fn iterative_deepening<'a>(
    game: &GameState,
    team: ActiveTeam,
    doop: &mut WorkerManager<'a>,
) -> moves::ActualMove {
    let mut count = Counter { count: 0 };
    let mut results = Vec::new();

    let max_depth = 4;
    let mut foo1 = PrincipalVariation {
        a: std::collections::HashMap::new(),
    };
    //TODO stop searching if we found a game ending move.
    for depth in 1..max_depth {
        console_dbg!("searching", depth);

        //TODO should this be outside the loop?
        let mut k = KillerMoves::new(max_depth);

        let mut aaaa = ai::AlphaBeta {
            //table: &mut table,
            prev_cache: &mut foo1,
            calls: &mut count,
            path: &mut vec![],
            killer_moves: &mut k,
            max_ext: 0,
        };

        let mut kk = game.clone();
        let res = aaaa.alpha_beta(&mut kk, ABAB::new(), team, depth, 0);
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

        doop.poke(team).await;

    }

    console_dbg!(count);
    console_dbg!(&results);

    //TODO THIS CAUSES ISSUES
    //results.dedup_by_key(|x| x.eval);

    //console_dbg!("deduped",&results);

    let target_eval = results.last().unwrap().eval;
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
    //let mov =
    let m = mov;

    console_dbg!("AI MOVE::", m.mov, m.eval);

    m.mov
}

#[derive(Debug)]
pub struct Counter {
    count: u128,
}
impl Counter {
    pub fn add_eval(&mut self) {
        self.count += 1;
    }
}

pub struct AlphaBeta<'a> {
    //table: &'a mut LeafTranspositionTable,
    prev_cache: &'a mut PrincipalVariation,
    calls: &'a mut Counter,
    path: &'a mut Vec<moves::ActualMove>,
    killer_moves: &'a mut KillerMoves,
    max_ext: usize,
}

pub struct KillerMoves {
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
pub struct EvalRet<T> {
    pub mov: T,
    pub eval: Eval,
}

impl<'a> AlphaBeta<'a> {
    pub fn alpha_beta(
        &mut self,
        game_after_move: &mut GameState,
        mut ab: ABAB,
        team: ActiveTeam,
        depth: usize,
        ext: usize,
    ) -> Eval {
        self.max_ext = self.max_ext.max(ext);

        let foo = if depth == 0 {
            None
        } else {
            let moves = game_after_move.for_all_moves_fast(team);

            if !moves.is_empty() {
                Some(moves)
            } else {
                None
            }
        };

        let Some(mut moves) = foo else {
            self.calls.add_eval();
            return absolute_evaluate(&game_after_move, false);
        };

        let mut num_sorted = 0;
        if let Some(p) = self.prev_cache.get_best_prev_move(self.path) {
            let f = moves.iter().enumerate().find(|(_, x)| **x == *p).unwrap();
            let swap_ind = f.0;
            moves.swap(0, swap_ind);
            num_sorted += 1;
        }

        for a in self.killer_moves.get(depth) {
            if let Some((x, _)) = moves[num_sorted..]
                .iter()
                .enumerate()
                .find(|(_, x)| *x == a)
            {
                moves.swap(x, num_sorted);
                num_sorted += 1;
            }
        }

        let mut kk = ab.ab_iter(team == ActiveTeam::Cats);
        for cand in moves {
            let new_depth = depth - 1;

            cand.execute_move_no_ani(game_after_move, team);
            self.path.push(cand);
            let eval = self.alpha_beta(
                game_after_move,
                kk.clone_ab_values(),
                team.not(),
                new_depth,
                ext,
            );

            let mov = self.path.pop().unwrap();
            mov.execute_undo(game_after_move, team);

            let (keep_going, consider_good_move) = kk.consider(&mov, eval);

            if consider_good_move {
                self.killer_moves.consider(depth, mov);
            }
            if !keep_going {
                break;
            }
            
        }

        let (eval, m) = kk.finish();
        if let Some(kk) = m {
            self.prev_cache.update(&self.path, &kk);
        }
        eval
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

    pub struct ABIter<'a, T> {
        value: i64,
        a: &'a mut ABAB,
        mm: Option<T>,
        keep_going: bool,
        maximizing: bool,
    }

    impl<'a, T: Clone> ABIter<'a, T> {
        pub fn finish(self) -> (Eval, Option<T>) {
            (self.value, self.mm)
        }
        pub fn clone_ab_values(&self) -> ABAB {
            self.a.clone()
        }
        pub fn consider(&mut self, t: &T, eval: Eval) -> (bool, bool) {
            let mut found_something = false;

            //TODO should be less than or equal instead maybe?
            let mmm = if self.maximizing {
                eval > self.value
            } else {
                eval < self.value
            };
            if mmm {
                self.mm = Some(t.clone());
                self.value = eval;
            }

            let cond = if self.maximizing {
                eval > self.a.beta
            } else {
                eval < self.a.alpha
            };

            if cond {
                self.keep_going = false;
                found_something = true;
            }

            if self.maximizing {
                self.a.alpha = self.a.alpha.max(self.value);
            } else {
                self.a.beta = self.a.beta.min(self.value);
            }

            (self.keep_going, found_something)
        }
    }

    impl ABAB {
        pub fn new() -> Self {
            ABAB {
                alpha: Eval::MIN,
                beta: Eval::MAX,
            }
        }
        pub fn ab_iter<T: Clone>(&mut self, maximizing: bool) -> ABIter<T> {
            let value = if maximizing { i64::MIN } else { i64::MAX };
            ABIter {
                value,
                a: self,
                mm: None,
                keep_going: true,
                maximizing,
            }
        }
    }
}
