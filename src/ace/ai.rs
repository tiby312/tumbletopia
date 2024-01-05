use crate::{
    ace::selection::generate_unit_possible_moves_inner, movement::MovementMesh, moves::ActualMove,
};

use super::{
    selection::{MoveLog, RegularSelection},
    *,
};

pub type Eval = i64; //(f64);

const MATE: i64 = 1_000_000;

const MAX_LIBERTY_DEPTH: usize = 5;

fn find_a_dog(
    a: GridCoord,
    game: &GameState,
    visited: &mut Vec<GridCoord>,
    depth: usize,
) -> Option<GridCoord> {
    if depth > 10 {
        return None;
    }

    if !game.world.filter().filter(&a).to_bool() {
        return None;
    }

    if visited.contains(&a) {
        return None;
    }

    if game.land.contains(&a) {
        return None;
    }

    if let Some(g) = game.dogs.units.iter().find(|c| c.position == a) {
        return Some(g.position);
    }

    visited.push(a);

    for (_, b) in a.to_cube().ring(1) {
        let b = b.to_axial();

        if let Some(g) = find_a_dog(b, game, visited, depth + 1) {
            return Some(g);
        }
    }

    return None;
}

fn find_isolated(game: &GameState) -> (Vec<GridCoord>, Vec<GridCoord>) {
    let mut dogs: Vec<_> = game.dogs.units.iter().map(|a| a.position).collect();

    let cats: Vec<_> = game
        .cats
        .units
        .iter()
        .map(|a| a.position)
        .filter(|cat| {
            let mut v = vec![];
            if let Some(dog) = find_a_dog(*cat, game, &mut v, 0) {
                dogs.retain(|a| *a != dog);

                false
            } else {
                true
            }
        })
        .collect();

    (cats, dogs)
}

//number of nearby liberties
fn num_liberties(
    a: GridCoord,
    visited: &mut Vec<GridCoord>,
    liberties: &mut Vec<GridCoord>,
    game: &GameState,
    depth: usize,
    depth_counter: &mut usize,
) {
    //console_dbg!(depth,visited.len());
    if depth > MAX_LIBERTY_DEPTH {
        return;
    }

    if !game.world.filter().filter(&a).to_bool() {
        return;
    }

    if visited.contains(&a) {
        return;
    }

    *depth_counter = (*depth_counter).max(depth);

    visited.push(a);

    for (_, b) in a.to_cube().ring(1) {
        let b = b.to_axial();

        if !game.land.contains(&b) {
            if !liberties.contains(&b) {
                liberties.push(b);
            }
            num_liberties(b, visited, liberties, game, depth + 1, depth_counter);
        }
    }
}

enum ClosestRet {
    Cat,
    Dog,
    None,
}

//TODO
fn is_accesible_by_land_and_sea() {}

fn dog_or_cat_closest(point: GridCoord, game: &GameState, is_land: bool) -> ClosestRet {
    let check_terrain = |point| {
        if !game.world.filter().filter(&point).to_bool() {
            return false;
        }

        if is_land {
            if !game.land.contains(&point) {
                //assert!(!game.forest.contains(&point));
                return false;
            }

            if game.forest.contains(&point) {
                return false;
            }
        } else {
            if game.land.contains(&point) {
                return false;
            }
        }
        return true;
    };

    let closest_something = bfs_find(point, |point| {
        if !check_terrain(point) {
            return false;
        }

        let a = game
            .cats
            .units
            .iter()
            .filter(|a| {
                if is_land {
                    a.typ == Type::Foot
                } else {
                    a.typ == Type::Ship
                }
            })
            .find(|a| a.position == point)
            .is_some();

        let b = game
            .dogs
            .units
            .iter()
            .filter(|a| {
                if is_land {
                    a.typ == Type::Foot
                } else {
                    a.typ == Type::Ship
                }
            })
            .find(|a| a.position == point)
            .is_some();

        a || b
    });

    let mut cat_found = false;
    let mut dog_found = false;
    if let Some((_, closest_something)) = closest_something {
        for point in closest_something {
            if cat_found && dog_found {
                break;
            }
            if !cat_found {
                let a = game
                    .cats
                    .units
                    .iter()
                    .filter(|a| {
                        if is_land {
                            a.typ == Type::Foot
                        } else {
                            a.typ == Type::Ship
                        }
                    })
                    .find(|a| a.position == point)
                    .is_some();

                if a {
                    cat_found = true;
                }
            }

            if !dog_found {
                let b = game
                    .dogs
                    .units
                    .iter()
                    .filter(|a| {
                        if is_land {
                            a.typ == Type::Foot
                        } else {
                            a.typ == Type::Ship
                        }
                    })
                    .find(|a| a.position == point)
                    .is_some();

                if b {
                    dog_found = true;
                }
            }
        }
    }

    //console_dbg!(closest_dog,closest_cat);

    match (cat_found, dog_found) {
        (false, false) => ClosestRet::None,
        (false, true) => ClosestRet::Dog,
        (true, false) => ClosestRet::Cat,
        (true, true) => ClosestRet::None,
    }
}

use std::collections::VecDeque;

struct OnePoint {
    team: ActiveTeam,
    point: GridCoord,
    depth: usize,
}

struct Territory {
    num_dog_controlled: i64,
    num_cat_controlled: i64,
}

fn dog_or_cat_closest2(game: &GameState, is_land: bool) -> Territory {
    let check_terrain = |point| {
        if !game.world.filter().filter(&point).to_bool() {
            return false;
        }

        if is_land {
            if !game.land.contains(&point) {
                //assert!(!game.forest.contains(&point));
                return false;
            }

            if game.forest.contains(&point) {
                return false;
            }
        } else {
            if game.land.contains(&point) {
                return false;
            }
        }
        return true;
    };

    let mut visited = vec![];

    let mut dogs_to_consider = Vec::new();
    let mut cats_to_consider = Vec::new();

    for point in game
        .dogs
        .units
        .iter()
        .filter(|a| {
            if is_land {
                a.typ == Type::Foot
            } else {
                a.typ == Type::Ship
            }
        })
        .map(|a| a.position)
    {
        dogs_to_consider.push(point);
    }

    for point in game
        .cats
        .units
        .iter()
        .filter(|a| {
            if is_land {
                a.typ == Type::Foot
            } else {
                a.typ == Type::Ship
            }
        })
        .map(|a| a.position)
    {
        cats_to_consider.push(point)
    }

    let mut num_dog_controlled = 0;
    let mut num_cat_controlled = 0;
    for _ in 0..10 {
        let mut next_dog_points = vec![];
        let mut next_cat_points = vec![];

        for d in dogs_to_consider.drain(..) {
            for p in d.to_cube().ring(1).map(|(_, b)| b.to_axial()) {
                if check_terrain(p) {
                    if !visited.contains(&p) {
                        //TODO no need to store depth in these???
                        next_dog_points.push(p);
                    }
                }
            }
        }

        for d in cats_to_consider.drain(..) {
            for p in d.to_cube().ring(1).map(|(_, b)| b.to_axial()) {
                if check_terrain(p) {
                    if !visited.contains(&p) {
                        //TODO no need to store depth in these???
                        next_cat_points.push(p);
                    }
                }
            }
        }

        //remove duplicate territory contested by teamates.
        next_cat_points.dedup();
        next_dog_points.dedup();

        //if a territory is contested by both sides, just remove it from both sides.
        next_cat_points.retain(|a| {
            if let Some((k, _)) = next_dog_points.iter().enumerate().find(|(_, b)| *b == a) {
                let _ = next_dog_points.remove(k);
                false
            } else {
                true
            }
        });

        for a in next_cat_points.iter().chain(next_dog_points.iter()) {
            visited.push(*a);
        }

        num_dog_controlled += next_dog_points.len() as i64;
        num_cat_controlled += next_cat_points.len() as i64;
        //console_dbg!(num_cat_controlled,num_dog_controlled);
        dogs_to_consider.append(&mut next_dog_points);
        cats_to_consider.append(&mut next_cat_points);
    }

    Territory {
        num_dog_controlled,
        num_cat_controlled,
    }
}

fn bfs_find(
    point: GridCoord,
    mut func: impl FnMut(GridCoord) -> bool,
) -> Option<(usize, Vec<GridCoord>)> {
    let mut visited = vec![];
    use std::collections::VecDeque;
    let mut queue = VecDeque::new();

    queue.push_back((point, 0));

    while let Some((p, depth)) = queue.pop_front() {
        if depth > 10 {
            break;
        }

        if visited.contains(&p) {
            continue;
        }

        visited.push(p);

        if !func(p) {
            for p in p.to_cube().ring(1).map(|(_, b)| b.to_axial()) {
                queue.push_back((p, depth + 1));
            }
        } else {
            let mut k = vec![p];
            while let Some((z, d)) = queue.pop_front() {
                if d == depth {
                    k.push(z);
                } else {
                    break;
                }
            }

            return Some((depth, k));
        }
    }

    None
}

fn is_closest_cat_or_dog(
    point: GridCoord,
    visited: &mut Vec<GridCoord>,
    game: &GameState,
    depth: usize,
    is_land: bool,
) -> ClosestRet {
    if depth > 10 {
        return ClosestRet::None;
    }

    if !game.world.filter().filter(&point).to_bool() {
        return ClosestRet::None;
    }

    if visited.contains(&point) {
        return ClosestRet::None;
    }

    if is_land {
        if !game.land.contains(&point) {
            //assert!(!game.forest.contains(&point));
            return ClosestRet::None;
        }

        if game.forest.contains(&point) {
            return ClosestRet::None;
        }
    } else {
        if game.land.contains(&point) {
            return ClosestRet::None;
        }
    }

    let k = game
        .dogs
        .units
        .iter()
        .filter(|a| {
            if is_land {
                a.typ == Type::Foot
            } else {
                a.typ == Type::Ship
            }
        })
        .find(|a| a.position == point);

    let j = game
        .cats
        .units
        .iter()
        .filter(|a| {
            if is_land {
                a.typ == Type::Foot
            } else {
                a.typ == Type::Ship
            }
        })
        .find(|a| a.position == point);

    visited.push(point);

    match (k, j) {
        (Some(_), Some(_)) => return ClosestRet::None,
        (Some(_), None) => return ClosestRet::Dog,
        (None, Some(_)) => return ClosestRet::Cat,
        (None, None) => {}
    }

    //find closest unit.

    let target = game
        .dogs
        .units
        .iter()
        .chain(game.cats.units.iter())
        .filter(|a| {
            if is_land {
                a.typ == Type::Foot
            } else {
                a.typ == Type::Ship
            }
        })
        .min_by_key(|x| x.position.to_cube().dist(&point.to_cube()))
        .unwrap();

    // //TODO look at these in a better order.
    let mut children: Vec<_> = point.to_cube().ring(1).map(|(_, b)| b.to_axial()).collect();
    children.sort_unstable_by_key(|a| a.to_cube().dist(&target.position.to_cube()));

    //TODO use bfs instead???
    for b in children {
        match is_closest_cat_or_dog(b, visited, game, depth + 1, is_land) {
            ClosestRet::Dog => return ClosestRet::Dog,
            ClosestRet::Cat => return ClosestRet::Cat,
            ClosestRet::None => {}
        }
    }
    ClosestRet::None
}

//cats maximizing
//dogs minimizing
fn absolute_evaluate(view: &GameState) -> Eval {
    //     let mut points_land = 0;

    //     for a in view
    //         .world
    //         .iter_cells()
    //         .map(|x| x.to_axial())
    //         .filter(|x| !view.land.contains(x))
    //     {
    //         match dog_or_cat_closest(a, view, false) {
    //             ClosestRet::Cat => {
    //                 points_land += 1;
    //             }
    //             ClosestRet::Dog => {
    //                 points_land -= 1;
    //             }
    //             _ => {}
    //         }
    //     }

    //     let mut points = 0;

    //     for a in view.land.iter().filter(|x| !view.forest.contains(x)) {
    //         match dog_or_cat_closest(*a, view, true) {
    //             ClosestRet::Cat => {
    //                 points += 1;
    //             }
    //             ClosestRet::Dog => {
    //                 points -= 1;
    //             }
    //             _ => {}
    //         }
    //     }

    //     points_land / 4 + points

    let water = dog_or_cat_closest2(view, false);
    let land = dog_or_cat_closest2(view, true);
    water.num_cat_controlled - water.num_dog_controlled
        + (land.num_cat_controlled - land.num_dog_controlled) / 2
}

pub fn we_in_check(view: GameView<'_>) -> bool {
    let Some(king_pos) = view.this_team.units.iter().find(|a| a.typ == Type::Foot) else {
        return false;
    };

    for a in view.this_team.units.iter().filter(|a| a.typ == Type::Foot) {}

    true
}

//TODO use bump allocator!!!!!
//TODO just store best move? not gamestate?
pub struct MoveOrdering {
    a: std::collections::HashMap<Vec<moves::ActualMove>, moves::ActualMove>,
}
impl MoveOrdering {
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

pub struct LeafTranspositionTable {
    a: std::collections::HashMap<GameState, Eval>,
    saves: usize,
}

impl LeafTranspositionTable {
    pub fn new() -> Self {
        LeafTranspositionTable {
            a: std::collections::HashMap::new(),
            saves: 0,
        }
    }
    fn lookup_leaf(&mut self, a: &GameState) -> Option<&Eval> {
        if let Some(a) = self.a.get(a) {
            self.saves += 1;
            Some(a)
        } else {
            None
        }
    }
    fn consider_leaf(&mut self, game: GameState, eval: Eval) {
        if let Some(v) = self.a.get_mut(&game) {
            *v = eval;
        } else {
            let _ = self.a.insert(game, eval);
        }
    }
    pub fn lookup_leaf_all(&mut self, node: &GameState) -> Eval {
        if let Some(&eval) = self.lookup_leaf(&node) {
            eval
        } else {
            let eval = absolute_evaluate(&node);
            self.consider_leaf(node.clone(), eval);
            eval
        }
    }
}

pub fn iterative_deepening<'a>(game: &GameState, team: ActiveTeam) -> moves::ActualMove {
    let mut count = Counter { count: 0 };
    let mut results = Vec::new();
    let mut table = LeafTranspositionTable::new();

    let max_depth = 4;
    let mut foo1 = MoveOrdering {
        a: std::collections::HashMap::new(),
    };
    //TODO stop searching if we found a game ending move.
    for depth in 1..max_depth {
        console_dbg!("searching", depth);

        //TODO should this be outside the loop?
        let mut k = KillerMoves::new(max_depth);

        let mut aaaa = ai::AlphaBeta {
            table: &mut table,
            prev_cache: &mut foo1,
            calls: &mut count,
            path: &mut vec![],
            killer_moves: &mut k,
            max_ext: 0,
        };
        let res = aaaa.alpha_beta(game.clone(), ABAB::new(), team, depth, 0);

        let mov = foo1.a.get(&[] as &[_]).cloned().unwrap();
        let res = EvalRet { mov, eval: res };

        // let res = ai::alpha_beta(
        //     game,
        //     team,
        //     depth,
        //     false,
        //     f64::NEG_INFINITY,
        //     f64::INFINITY,
        //     &mut table,
        //     &mut foo1,
        //     &mut count,
        //     &mut vec![],
        // );

        let eval = res.eval;
        console_dbg!(eval);

        results.push(res);

        if eval.abs() == MATE {
            console_dbg!("found a mate");
            break;
        }
    }

    // console_dbg!(table.saves);
    // console_dbg!(table.a.len());
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
    table: &'a mut LeafTranspositionTable,
    prev_cache: &'a mut MoveOrdering,
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
        mut game_after_move: GameState,
        ab: ABAB,
        team: ActiveTeam,
        depth: usize,
        ext: usize,
    ) -> Eval {
        self.max_ext = self.max_ext.max(ext);
        //let the_move = cand.the_move.clone();
        let mut gg = game_after_move.clone();

        let ret = if depth == 0 || game_is_over(&mut game_after_move, team).is_some() {
            //console_dbg!(game_is_over(cand.game_after_move.view(team)));

            self.calls.add_eval();
            let e = self.table.lookup_leaf_all(&game_after_move);

            // if self.prev_cache.a.get(self.path).is_none(){
            //     self.prev_cache.update(&self.path, &cand.the_move);
            // }
            //console_dbg!("FOOO",e);
            e

            // let (m, eval) = self.quiensense_search(cand, ab, team, 3);

            // eval
        } else {
            let gg2 = game_after_move.clone();

            let node = game_after_move;

            let pvariation = self.prev_cache.get_best_prev_move(self.path).cloned();

            let pvariation = pvariation.map(|x| {
                execute_move_no_ani(&mut gg, team, x.clone());
                PossibleMove {
                    the_move: x,
                    game_after_move: gg,
                }
            });

            // let it = reorder_front(
            //     pvariation,
            //     ,
            // );

            // let enemy_king_pos=if let Some(enemy_king)=cand.game_after_move.view(team).that_team.units.iter().find(|x|x.typ==Type::Para){
            //     let pos=enemy_king.position;
            //     Some(pos)
            // }else{
            //     None
            // };

            let mut moves = for_all_moves_fast(node.clone(), team);

            //console_dbg!("FOOOO",moves.len());
            //console_dbg!(moves.iter().map(|x|&x.1.the_move).collect::<Vec<_>>());

            let num_checky = moves.iter().count();
            //console_dbg!(num_checky);
            // if is_check(&moves[0].1.game_after_move) {
            //     moves[0].0 = true;
            // }

            //console_dbg!(moves.len());

            let mut num_sorted = 0;
            if let Some(p) = pvariation {
                let f = moves
                    .iter()
                    .enumerate()
                    .find(|(_, x)| **x == p.the_move)
                    .unwrap();
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

            let foo = |ssself: &mut AlphaBeta, cand: moves::ActualMove, ab| {
                let cc = cand.clone();
                let new_depth = depth - 1; //.saturating_sub(inhibit);
                                           //assert!(new_depth<6);
                                           //console_dbg!(ext,depth);

                let mut gg = gg2.clone();
                execute_move_no_ani(&mut gg, team, cand);

                ssself.path.push(cand.clone());
                let eval = ssself.alpha_beta(gg, ab, team.not(), new_depth, ext);
                let k = ssself.path.pop().unwrap();

                //console_dbg!("inner eval=",eval);
                EvalRet { eval, mov: cc }
            };

            if team == ActiveTeam::Cats {
                //console_dbg!("maxing");
                if let Some(ret) = ab.maxxer(moves, self, foo, |ss, m, _| {
                    ss.killer_moves.consider(depth, m);
                }) {
                    //console_dbg!("FOUND",ret.eval);

                    self.prev_cache.update(&self.path, &ret.mov);
                    ret.eval
                } else {
                    Eval::MIN
                }
            } else {
                //console_dbg!("mining");
                if let Some(ret) = ab.minner(moves, self, foo, |ss, m, _| {
                    ss.killer_moves.consider(depth, m);
                }) {
                    self.prev_cache.update(&self.path, &ret.mov);
                    //console_dbg!("FOUND",ret.eval);
                    ret.eval
                } else {
                    Eval::MAX
                }
            }
        };
        //console_dbg!("alpha beta ret=",ret,depth);
        ret
    }
}

pub struct MyMoveFinder {}
impl abab_simple::MoveFinder for MyMoveFinder {
    type EE = Eval;
    type T = GameState;
    type Mo = moves::ActualMove;
    type Finder = std::vec::IntoIter<PossibleMove>;

    fn eval(&mut self, game: &Self::T) -> Self::EE {
        absolute_evaluate(game)
    }

    fn min_eval(&self) -> Self::EE {
        Eval::MIN
    }

    fn max_eval(&self) -> Self::EE {
        Eval::MAX
    }

    fn apply_move(&mut self, game: &mut Self::T, a: Self::Mo) {
        let mut mm = MoveLog::new();
        todo!();
    }

    fn generate_finder(
        &mut self,
        state: &Self::T,
        path: &[Self::Mo],
        maximizer: bool,
    ) -> Self::Finder {
        let team = if maximizer {
            ActiveTeam::Cats
        } else {
            ActiveTeam::Dogs
        };
        todo!();
        //let k: Vec<_> = for_all_moves(state.clone(), team).collect();
        //k.into_iter()
    }

    fn select_move(&mut self, finder: &mut Self::Finder) -> Option<Self::Mo> {
        finder.next().map(|x| x.the_move)
    }
}
mod abab_simple {

    pub trait MoveFinder {
        type EE: PartialOrd + Ord + Copy;
        type T: Clone;
        type Mo: Copy;
        type Finder;
        fn eval(&mut self, game: &Self::T) -> Self::EE;

        fn min_eval(&self) -> Self::EE;
        fn max_eval(&self) -> Self::EE;

        fn apply_move(&mut self, game: &mut Self::T, a: Self::Mo);

        fn generate_finder(
            &mut self,
            state: &Self::T,
            path: &[Self::Mo],
            maximizer: bool,
        ) -> Self::Finder;
        fn select_move(&mut self, finder: &mut Self::Finder) -> Option<Self::Mo>;
    }

    pub fn alpha_beta<X: MoveFinder>(
        data: X,
        depth: usize,
        game_state: X::T,
        maximizer: bool,
    ) -> (X::EE, Vec<X::Mo>) {
        ABAB::new(data).alpha_beta(depth, game_state, maximizer)
    }
    use super::*;
    #[derive(Clone)]
    struct ABAB<X, Y, Z> {
        alpha: Z,
        beta: Z,
        data: X,
        path: Vec<Y>,
    }
    impl<X: MoveFinder> ABAB<X, X::Mo, X::EE> {
        pub fn new(data: X) -> Self {
            ABAB {
                alpha: data.min_eval(),
                beta: data.max_eval(),
                data,
                path: vec![],
            }
        }

        pub fn alpha_beta(
            &mut self,
            depth: usize,
            game_state: X::T,
            maximizer: bool,
        ) -> (X::EE, Vec<X::Mo>) {
            if depth == 0 {
                (self.data.eval(&game_state), vec![])
            } else {
                if maximizer {
                    let mut value = self.data.min_eval();
                    let mut ll = vec![];
                    let mut best_move = None;

                    let mut gs = self
                        .data
                        .generate_finder(&game_state, &self.path, maximizer);
                    while let Some(mo) = self.data.select_move(&mut gs) {
                        let mut ga = game_state.clone();
                        self.data.apply_move(&mut ga, mo);
                        self.path.push(mo);
                        let (eval, move_list) = self.alpha_beta(depth - 1, ga, !maximizer);
                        self.path.pop();

                        if eval > value {
                            value = eval;
                            ll = move_list;
                            best_move = Some(mo);
                        }

                        if value > self.beta {
                            break;
                        }
                        self.alpha = self.alpha.max(value);
                    }

                    ll.push(best_move.unwrap());
                    (value, ll)
                } else {
                    let mut value = self.data.max_eval();
                    let mut ll = vec![];
                    let mut best_move = None;

                    let mut gs = self
                        .data
                        .generate_finder(&game_state, &self.path, maximizer);
                    while let Some(mo) = self.data.select_move(&mut gs) {
                        let mut ga = game_state.clone();
                        self.data.apply_move(&mut ga, mo);
                        self.path.push(mo);
                        let (eval, move_list) = self.alpha_beta(depth - 1, ga, !maximizer);
                        self.path.pop();

                        if eval > value {
                            value = eval;
                            ll = move_list;
                            best_move = Some(mo);
                        }

                        if value < self.alpha {
                            break;
                        }
                        self.beta = self.beta.min(value);
                    }

                    ll.push(best_move.unwrap());
                    (value, ll)
                }
            }
        }
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
    impl ABAB {
        pub fn new() -> Self {
            ABAB {
                alpha: Eval::MIN,
                beta: Eval::MAX,
            }
        }

        pub fn minner<P, T: Clone>(
            mut self,
            it: impl IntoIterator<Item = T>,
            mut payload: &mut P,
            mut func: impl FnMut(&mut P, T, Self) -> EvalRet<T>,
            mut func2: impl FnMut(&mut P, T, Self),
        ) -> Option<EvalRet<T>> {
            let mut mm: Option<T> = None;

            let mut value = i64::MAX;
            for cand in it {
                let t = func(payload, cand.clone(), self.clone());

                value = value.min(t.eval);
                if value == t.eval {
                    mm = Some(cand.clone());
                }
                if t.eval < self.alpha {
                    func2(payload, cand, self.clone());
                    break;
                }
                self.beta = self.beta.min(value)
            }

            if let Some(mm) = mm {
                Some(EvalRet {
                    mov: mm,
                    eval: value,
                })
            } else {
                None
            }
        }
        pub fn maxxer<P, T: Clone>(
            mut self,
            it: impl IntoIterator<Item = T>,
            mut payload: &mut P,
            mut func: impl FnMut(&mut P, T, Self) -> EvalRet<T>,
            mut func2: impl FnMut(&mut P, T, Self),
        ) -> Option<EvalRet<T>> {
            let mut mm: Option<T> = None;

            let mut value = i64::MIN;
            for cand in it {
                let t = func(&mut payload, cand.clone(), self.clone());

                value = value.max(t.eval);
                if value == t.eval {
                    mm = Some(cand.clone());
                }
                if t.eval > self.beta {
                    func2(&mut payload, cand, self.clone());
                    break;
                }
                self.alpha = self.alpha.max(value)
            }

            if let Some(mm) = mm {
                Some(EvalRet {
                    mov: mm,
                    eval: value,
                })
            } else {
                None
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PossibleMove {
    pub the_move: moves::ActualMove,
    //pub mesh: MovementMesh,
    pub game_after_move: GameState,
}

//TODO pass readonly

// pub struct PossibleMoveWithMesh {
//     pub the_move: moves::ActualMove,
//     //pub mesh: MovementMesh,
//     pub game_after_move: GameState,
//     pub mesh: MovementMesh,
// }
// impl PossibleMoveWithMesh {
//     pub fn into(self) -> PossibleMove {
//         PossibleMove {
//             the_move: self.the_move,
//             game_after_move: self.game_after_move,
//         }
//     }
// }

pub fn execute_move_no_ani(
    state: &mut GameState,
    team_index: ActiveTeam,
    the_move: moves::ActualMove,
) {
    let mut game = state.view_mut(team_index);
    let mut game_history = MoveLog::new();

    match the_move {
        moves::ActualMove::NormalMove(o) => {
            let unit = game.this_team.find_slow(&o.unit).unwrap();

            // let mesh = selection::generate_unit_possible_moves_inner(
            //     &unit.position,
            //     unit.typ,
            //     &game,
            //     false,
            // );

            let r = selection::RegularSelection::new(unit);
            let r = r
                .execute_no_animation(o.moveto, &mut game, &mut game_history)
                .unwrap();
            assert!(r.is_none());
        }
        moves::ActualMove::ExtraMove(o, e) => {
            let unit = game.this_team.find_slow(&o.unit).unwrap().clone();

            // let mesh = selection::generate_unit_possible_moves_inner(
            //     &unit.position,
            //     unit.typ,
            //     &game,
            //     true,
            // );

            let r = selection::RegularSelection::new(&unit);
            let r = r
                .execute_no_animation(o.moveto, &mut game, &mut game_history)
                .unwrap();

            let rr = r.unwrap();

            let rr = rr.select();
            //let mesh = rr.generate(&game);

            rr.execute_no_animation(e.moveto, &mut game, &mut game_history)
                .unwrap();
        }
        moves::ActualMove::SkipTurn => {}
        moves::ActualMove::GameEnd(_) => todo!(),
    }
}

//TODO this has duplicated logic
// pub fn apply_move(mo: moves::ActualMove, state: &mut GameState, team: ActiveTeam) {
//     let moves::ActualMove::ExtraMove(
//         moves::PartialMoveSigl {
//             unit: pos,
//             moveto: mm,
//         },
//         moves::PartialMoveSigl {
//             unit: _,
//             moveto: sm,
//         },
//     ) = mo
//     else {
//         unreachable!()
//     };

//     let pp = state.view_mut(team).this_team.find_slow_mut(&pos).unwrap();

//     pp.position = mm;

//     if pp.typ == Type::Ship {
//         state.land.push(sm);
//     } else if pp.typ == Type::Foot {
//         state.forest.push(sm);
//     }
// }

//TODO use this!!!
pub fn for_all_moves_fast(mut state: GameState, team: ActiveTeam) -> Vec<moves::ActualMove> {
    let mut movs = Vec::new();
    for i in 0..state.view_mut(team).this_team.units.len() {
        let pos = state.view_mut(team).this_team.units[i].position;
        let typ = state.view_mut(team).this_team.units[i].typ;

        let mesh = generate_unit_possible_moves_inner(&pos, typ, &state.view_mut(team), false);
        for mm in mesh.iter_mesh(pos) {
            //Temporarily move the player in the game world.
            state.view_mut(team).this_team.units[i].position = mm;

            let second_mesh =
                generate_unit_possible_moves_inner(&mm, typ, &state.view_mut(team), true);

            for sm in second_mesh.iter_mesh(mm) {
                // movs.push(OneMove {
                //     start: pos,
                //     end: mm,
                //     hex: mm.dir_to(&sm),
                // })
                movs.push(moves::ActualMove::ExtraMove(
                    moves::PartialMoveSigl {
                        unit: pos,
                        moveto: mm,
                    },
                    moves::PartialMoveSigl {
                        unit: mm,
                        moveto: sm,
                    },
                ))
            }
        }
        //revert it back.
        state.view_mut(team).this_team.units[i].position = pos;
    }
    movs
}

// pub fn for_all_moves(state: GameState, team: ActiveTeam) -> impl Iterator<Item = PossibleMove> {
//     let mut sss = state.clone();
//     let ss = state.clone();
//     ss.into_view(team)
//         .this_team
//         .units
//         .into_iter()
//         .map(|a| RegularSelection { unit: a.clone() })
//         .flat_map(move |a| {
//             let mesh = a.generate(&sss.view_mut(team));
//             mesh.iter_mesh(a.unit.position)
//                 .map(move |f| (a.clone(), mesh.clone(), f))
//         })
//         .flat_map(move |(s, mesh, m)| {
//             let mut v = state.clone();
//             let mut mm = MoveLog::new();

//             let first = if let Some(l) = s
//                 .execute_no_animation(m, mesh, &mut v.view_mut(team), &mut mm)
//                 .unwrap()
//             {
//                 //console_dbg!("YOOOOOOO");
//                 let cll = l.select();

//                 //let mut kk = v.view().duplicate();
//                 let mut kk = v.clone();
//                 let mesh2 = cll.generate(&mut kk.view_mut(team));
//                 Some(mesh2.iter_mesh(l.coord()).map(move |m| {
//                     let mut klkl = kk.clone();
//                     let mut mm2 = MoveLog::new();

//                     let mut vfv = klkl.view_mut(team);
//                     cll.execute_no_animation(m, mesh2.clone(), &mut vfv, &mut mm2)
//                         .unwrap();

//                     PossibleMove {
//                         game_after_move: klkl,
//                         //mesh: mesh2,
//                         the_move: mm2.inner[0].clone(),
//                     }
//                 }))
//             } else {
//                 None
//             };

//             let second = if first.is_none() {
//                 //console_dbg!("NEVER HAPPEN");
//                 Some([PossibleMove {
//                     game_after_move: v,
//                     //mesh,
//                     the_move: mm.inner[0].clone(),
//                 }])
//             } else {
//                 None
//             };

//             let f1 = first.into_iter().flatten();
//             let f2 = second.into_iter().flatten();
//             f1.chain(f2)
//         })
//     //.chain([foo].into_iter())
// }
