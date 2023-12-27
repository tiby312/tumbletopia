use crate::{movement::MovementMesh, moves::ActualMove};

use super::{
    selection::{MoveLog, RegularSelection},
    *,
};

pub type Eval = i64; //(f64);

const MATE: i64 = 1_000_000;
//cats maximizing
//dogs minimizing
fn absolute_evaluate(view: &GameState) -> Eval {


    let mut points=0;
    for a in view.world.iter_cells().map(|x|x.to_axial()).filter(|x|!view.land.contains(x)){
        let closest_cat=view.cats.units.iter().map(|x|x.position.to_cube().dist(&a.to_cube())).min().unwrap();
        let closest_dog=view.dogs.units.iter().map(|x|x.position.to_cube().dist(&a.to_cube())).min().unwrap();
        if closest_cat<closest_dog{
            points+=1;
        }else if closest_cat>closest_dog{
            points-=1;
        }

    }
    console_dbg!(points);
    return points;

    //TODO check for checks!!!
    //let view = view.absolute();
    let num_cats = view.cats.units.len();
    let num_dogs = view.dogs.units.len();
    let diff = num_cats as i64 - num_dogs as i64;
    //console_dbg!("HAYYY",diff);

    let Some(cat_king)=view
        .cats
        .units
        .iter()
        .find(|a| a.typ == Type::King)
    else {
        return -MATE;
    };

    let Some(dog_king)=view
        .dogs
        .units
        .iter()
        .find(|a| a.typ == Type::King)
    else
    {
        return MATE;
    };

    //TODO add dead rekoning look ahead

    //TODO check if warriors are restricted

    fn doop(me: &UnitData, king: &UnitData) -> i64 {
        //TODO handle case where it runs off the board?
        let king_pos = king.position;
        let king_dir = king.direction;

        let distance = me.position.to_cube().dist(&king_pos.to_cube());

        let projected_king_pos = king_pos.add(
            king_dir
                .to_relative()
                .advance_by(king_dir, usize::try_from(distance).unwrap().max(2)),
        );

        let distance_to_projected = me.position.to_cube().dist(&projected_king_pos.to_cube());

        let x = distance_to_projected as i64;
        x * x
    }

    //We multiply by the entire number of units so that
    //the team is more aggressive if it has more pieces.
    let cat_distance_to_dog2 = view
        .cats
        .units
        .iter()
        .map(|x| doop(x, dog_king))
        .fold(0, |acc, f| acc + f)
        * num_cats as i64;

    let dog_distance_to_cat2 = view
        .dogs
        .units
        .iter()
        .map(|x| doop(x, cat_king))
        .fold(0, |acc, f| acc + f)
        * num_dogs as i64;

    fn king_safety(view: &GameState, this_team: ActiveTeam) -> i64 {
        let game = view.view(this_team);

        let king = game
            .this_team
            .units
            .iter()
            .find(|a| a.typ == Type::King)
            .unwrap();

        //TODO dynamically change radius

        let mut enemies: Vec<_> = game
            .that_team
            .units
            .iter()
            //.filter(|x| x.typ == Type::Warrior)
            .map(|x| x.position.to_cube().dist(&king.position.to_cube()))
            .collect();

        let mut friendlies: Vec<_> = game
            .this_team
            .units
            .iter()
            //.filter(|x| x.typ == Type::Warrior)
            .filter(|x| x.position != king.position)
            .map(|x| x.position.to_cube().dist(&king.position.to_cube()))
            .collect();

        enemies.sort();
        friendlies.sort();

        // console_dbg!(enemies);
        // console_dbg!(friendlies);

        let difference: Vec<_> = enemies
            .iter()
            .zip(friendlies.iter())
            .map(|(&a, &b)| a - b)
            .collect();

        //console_dbg!(difference);
        let cost = [-400, -200, -100];

        for (&a, b) in difference.iter().zip(cost) {
            if a < 1 {
                return b;
            }
        }

        0
    }

    //let cat_safety = king_safety(view, ActiveTeam::Cats);
    //let dog_safety = -king_safety(view, ActiveTeam::Dogs);

    let val = diff * 10_000 - cat_distance_to_dog2 + dog_distance_to_cat2;
    // + cat_safety / 20
    // + dog_safety / 20;
    //console_dbg!(val);
    //let val = diff;

    //assert!(!val.is_nan());
    val
}

// pub fn captures_possible(node: GameViewMut<'_, '_>) -> bool {
//     let num_enemy = node.that_team.units.len();
//     for a in for_all_moves(&node) {
//         if a.game_after_move.that_team.units.len() < num_enemy {
//             return true;
//         }
//     }

//     let num_friendly = node.this_team.units.len();
//     for a in for_all_moves(&node) {
//         if a.game_after_move.this_team.units.len() < num_friendly {
//             return true;
//         }
//     }

//     false
// }

pub fn we_in_check(view: GameView<'_>) -> bool {
    let Some(king_pos)=view.this_team.units.iter().find(|a|a.typ==Type::King) else{
        return false
    };

    for a in view.this_team.units.iter().filter(|a| a.typ == Type::King) {}

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

    let mut foo1 = MoveOrdering {
        a: std::collections::HashMap::new(),
    };

    let max_depth = 4;

    //TODO stop searching if we found a game ending move.
    for depth in 1..max_depth {
        console_dbg!("searching", depth);

        //TODO should this be outside the loop?
        let mut k = KillerMoves::new(max_depth);

        let pp = PossibleMove {
            the_move: moves::ActualMove::SkipTurn,
            //mesh: MovementMesh::new(),
            game_after_move: game.clone(),
        };
        let mut aaaa = ai::AlphaBeta {
            table: &mut table,
            prev_cache: &mut foo1,
            calls: &mut count,
            path: &mut vec![],
            killer_moves: &mut k,
            max_ext: 0,
        };
        let res = aaaa.alpha_beta(pp, ABAB::new(), team, depth, 0);

        let mov = foo1
            .a
            .get(&[moves::ActualMove::SkipTurn] as &[_])
            .cloned()
            .unwrap();
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
        mut cand: PossibleMove,
        ab: ABAB,
        team: ActiveTeam,
        depth: usize,
        ext: usize,
    ) -> Eval {
        self.max_ext = self.max_ext.max(ext);
        let the_move = cand.the_move.clone();
        let mut gg = cand.game_after_move.clone();

        self.path.push(the_move.clone());
        let ret = if depth == 0 || game_is_over(&mut cand.game_after_move,team).is_some() {
            //console_dbg!(game_is_over(cand.game_after_move.view(team)));

            self.calls.add_eval();
            let e = self.table.lookup_leaf_all(&cand.game_after_move);

            // if self.prev_cache.a.get(self.path).is_none(){
            //     self.prev_cache.update(&self.path, &cand.the_move);
            // }
            //console_dbg!("FOOO",e);
            e

            // let (m, eval) = self.quiensense_search(cand, ab, team, 3);

            // eval
        } else {
            let node = cand.game_after_move;

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

            let mut moves: Vec<_> = for_all_moves(node.clone(), team)
                .map(|mut x| {
                    //let c = is_check(&x.game_after_move);
                    //let c1 = this_team_in_check(&mut x.game_after_move, team);
                    //let c2 = this_team_in_check(&mut x.game_after_move, team.not());

                    //let c = false;
                    (false, x)
                })
                .collect();
            console_dbg!("FOOOO",moves.len());
            //console_dbg!(moves.iter().map(|x|&x.1.the_move).collect::<Vec<_>>());

            let num_checky = moves.iter().filter(|x| x.0).count();
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
                    .find(|(_, (_, x))| x.the_move == p.the_move)
                    .unwrap();
                let swap_ind = f.0;
                moves.swap(0, swap_ind);
                num_sorted += 1;
            }

            for a in self.killer_moves.get(depth) {
                if let Some((x, _)) = moves[num_sorted..]
                    .iter()
                    .enumerate()
                    .find(|(_, (_, x))| &x.the_move == a)
                {
                    moves.swap(x, num_sorted);
                    num_sorted += 1;
                }
            }

            let foo = |ssself: &mut AlphaBeta, (is_checky, cand): (bool, PossibleMove), ab| {
                let cc = cand.clone();
                let new_depth = depth - 1; //.saturating_sub(inhibit);
                                           //assert!(new_depth<6);
                                           //console_dbg!(ext,depth);
                let eval = ssself.alpha_beta(cand, ab, team.not(), new_depth, ext);
                //console_dbg!("inner eval=",eval);
                EvalRet {
                    eval,
                    mov: (is_checky, cc),
                }
            };

            if team == ActiveTeam::Cats {
                //console_dbg!("maxing");
                if let Some(ret) = ab.maxxer(moves, self, foo, |ss, m, _| {
                    ss.killer_moves.consider(depth, m.1.the_move);
                }) {
                    self.prev_cache.update(&self.path, &ret.mov.1.the_move);
                    ret.eval
                } else {
                    Eval::MIN
                }
            } else {
                //console_dbg!("mining");
                if let Some(ret) = ab.minner(moves, self, foo, |ss, m, _| {
                    ss.killer_moves.consider(depth, m.1.the_move);
                }) {
                    self.prev_cache.update(&self.path, &ret.mov.1.the_move);
                    //console_dbg!("FOUND",ret.eval);
                    ret.eval
                } else {
                    Eval::MAX
                }
            }
        };
        let k = self.path.pop().unwrap();
        assert_eq!(k, the_move);
        //console_dbg!("alpha beta ret=",ret,depth);
        ret
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

            let mesh = selection::generate_unit_possible_moves_inner(unit, &game, None);

            let r = selection::RegularSelection::new(unit);
            let r = r
                .execute_no_animation(o.moveto, mesh, &mut game, &mut game_history)
                .unwrap();
            assert!(r.is_none());
        }
        moves::ActualMove::ExtraMove(o, e) => {
            let unit = game.this_team.find_slow(&o.unit).unwrap().clone();

            let mesh = selection::generate_unit_possible_moves_inner(&unit, &game, None);

            let r = selection::RegularSelection::new(&unit);
            let r = r
                .execute_no_animation(o.moveto, mesh, &mut game, &mut game_history)
                .unwrap();
            //console_dbg!("WOOO");

            //let unit = game.this_team.find_slow(&o.unit).unwrap().clone();

            // let mesh =
            //     selection::generate_unit_possible_moves_inner(&unit, &game, Some(e.unit));

            let rr = r.unwrap();

            let rr = rr.select();
            let mesh = rr.generate(&game);

            rr.execute_no_animation(e.moveto, mesh, &mut game, &mut game_history)
                .unwrap();
        }
        moves::ActualMove::SkipTurn => {}
        moves::ActualMove::GameEnd(_) => todo!(),
    }
}

pub struct PartialMove {
    pos: GridCoord,
    moveto: GridCoord,
}
pub fn for_all_moves_v2(state: GameState, team: ActiveTeam) -> impl Iterator<Item = PartialMove> {
    let mut sss = state.clone();
    state
        .clone()
        .into_view(team)
        .this_team
        .units
        .into_iter()
        .map(|a| RegularSelection { unit: a.clone() })
        .flat_map(move |a| {
            let mesh = a.generate(&sss.view_mut(team));
            mesh.iter_mesh(a.unit.position).map(move |f| PartialMove {
                pos: a.unit.position,
                moveto: f,
            })
        })
}

//TODO use this
// pub fn for_all_moves_v3(game:&GameState,team:ActiveTeam) -> impl Iterator<Item = RegularSelection2>+'_ {
//     game.view(team).this_team.units.iter().map(move |a|RegularSelection2{
//         unit:a,
//         mesh:generate_unit_possible_moves_inner3(a, game)
//     })
// }

pub fn for_all_moves(state: GameState, team: ActiveTeam) -> impl Iterator<Item = PossibleMove> {
    // let foo = PossibleMove {
    //     the_move: moves::ActualMove::SkipTurn,
    //     game_after_move: state.clone(),
    //     //mesh: MovementMesh::new(),
    // };

    let mut sss = state.clone();
    let ss = state.clone();
    ss.into_view(team)
        .this_team
        .units
        .into_iter()
        .map(|a| RegularSelection { unit: a.clone() })
        .flat_map(move |a| {
            let mesh = a.generate(&sss.view_mut(team));
            mesh.iter_mesh(a.unit.position)
                .map(move |f| (a.clone(), mesh.clone(), f))
        })
        .flat_map(move |(s, mesh, m)| {
            let mut v = state.clone();
            let mut mm = MoveLog::new();

            let first = if let Some(l) = s
                .execute_no_animation(m, mesh, &mut v.view_mut(team), &mut mm)
                .unwrap()
            {
                let cll = l.select();

                //let mut kk = v.view().duplicate();
                let mut kk = v.clone();
                let mesh2 = cll.generate(&mut kk.view_mut(team));
                Some(mesh2.iter_mesh(l.coord()).map(move |m| {
                    let mut klkl = kk.clone();
                    let mut mm2 = MoveLog::new();

                    let mut vfv = klkl.view_mut(team);
                    cll.execute_no_animation(m, mesh2.clone(), &mut vfv, &mut mm2)
                        .unwrap();

                    PossibleMove {
                        game_after_move: klkl,
                        //mesh: mesh2,
                        the_move: mm2.inner[0].clone(),
                    }
                }))
            } else {
                None
            };

            let second = if first.is_none() {
                Some([PossibleMove {
                    game_after_move: v,
                    //mesh,
                    the_move: mm.inner[0].clone(),
                }])
            } else {
                None
            };

            let f1 = first.into_iter().flatten();
            let f2 = second.into_iter().flatten();
            f1.chain(f2)
        })
        //.chain([foo].into_iter())
}
