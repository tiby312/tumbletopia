use crate::gameplay::Zoo;

use super::*;

pub struct GameHandle;
impl gameplay::Zoo for GameHandle {
    type G<'a> = Stuff<'a>;
    fn create() -> Self {
        GameHandle
    }
}

pub struct Stuff<'a> {
    pub team: &'a mut usize,
    pub grid_matrix: &'a grids::GridMatrix,
    pub this_team: &'a mut Tribe,
    pub that_team: &'a mut Tribe,
    pub mouse: Option<[f32; 2]>,
    pub end_turn: bool,
}

fn select_unit() -> impl GameStepper<GameHandle, Result = WarriorPointer<GridCoord>> {
    gameplay::looper((), |_, _| {
        WaitMouseInput.map(|mouse_world, stuff| {
            let cell: GridCoord = GridCoord(stuff.grid_matrix.to_grid((mouse_world).into()).into());

            let Some(unit)=stuff.this_team.find_slow(&cell) else {
                return gameplay::LooperRes::Loop(());
            };

            if !unit.selectable() {
                return gameplay::LooperRes::Loop(());
            }

            let pos = unit.slim();

            gameplay::LooperRes::Finish(pos)
        })
    })
}

//Returns whether or not the unit moved to a new location or not.
fn attack_init(
    ss: &movement::PossibleMoves,
    g1: &mut Stuff,
    current: &WarriorPointer<GridCoord>,
    target: &WarriorPointer<GridCoord>,
) -> impl GameStepper<GameHandle, Result = Option<WarriorPointer<GridCoord>>> {
    //Only counter if non neg
    // let counter_damage = if g1.this_team.lookup_mut(current).move_bank.0>=0{
    //     5
    // }else{
    //     0
    // };
    // let damage = 5;
    // let counter_damage = 5;
    let damage = 5;
    let counter_damage = 5;

    let cc = *current;

    let kill_self = g1.this_team.lookup_mut(current).health <= counter_damage;

    let (path, _) = ss.get_path_data(target).unwrap();

    //let attack_stamina_cost=2;
    let total_cost = path.total_cost();
    log!(format!("total_cost:{:?}", total_cost));
    if g1.that_team.lookup_mut(target).health <= damage {
        let c = g1.this_team.lookup_take(*current);

        //TODO pass path instead!!!
        kill_animator(ss, c, target, g1)
            .map(move |this_unit, g1| {
                let target = this_unit.slim();
                g1.that_team.lookup_take(target);
                g1.this_team.add(this_unit);

                let mut current_cat = g1.this_team.lookup_mut(&target);

                current_cat.attacked = true;
                //dont need to double sub because we moved there
                //current_cat.stamina.0-=attack_stamina_cost;
                Some(target)
            })
            .either_a()
    } else {
        let c = g1.this_team.lookup_take(*current);
        let tt = *target;

        attack_animator(ss, c, target, g1)
            .map(move |this_unit, g1| {
                let target = tt;
                g1.this_team.add(this_unit);
                let mut target_cat = g1.that_team.lookup_mut(&target);
                target_cat.health -= damage;

                let mut current_cat = g1.this_team.lookup_mut(&cc);

                if kill_self {
                    g1.this_team.lookup_take(cc);
                } else {
                    current_cat.attacked = true;
                    current_cat.health -= counter_damage;
                    current_cat.stamina.0 -= total_cost.0;
                    //current_cat.stamina.0 -= attack_stamina_cost;
                }
                None
            })
            .either_b()
    }
    .map(|a, _| match a {
        gameplay::Either::A(a) => a,
        gameplay::Either::B(a) => a,
    })
}

fn attack_animator(
    ss: &movement::PossibleMoves,
    start: WarriorPointer<Warrior>,
    target: &GridCoord,
    g1: &mut Stuff,
) -> impl GameStepper<GameHandle, Result = WarriorPointer<Warrior>> {
    let (dd, _) = ss.get_path_data(target).unwrap();
    //start.move_deficit = *aa;

    let aa = animation::Animation::new(start.position, dd, &g1.grid_matrix, start);
    let aaa = AnimationTicker::new(aa).map(move |res, _| {
        let warrior = res.into_data();
        //warrior.position=tt;
        warrior
    });
    aaa
}

fn kill_animator(
    ss: &movement::PossibleMoves,
    start: WarriorPointer<Warrior>,
    target: &GridCoord,
    g1: &mut Stuff,
) -> impl GameStepper<GameHandle, Result = WarriorPointer<Warrior>> {
    move_animator(ss, start, target, g1)
}

//TODO make generic!!!!???
fn move_animator(
    ss: &movement::PossibleMoves,
    mut start: WarriorPointer<Warrior>,
    target: &GridCoord,
    g1: &mut Stuff,
) -> impl GameStepper<GameHandle, Result = WarriorPointer<Warrior>> {
    let (dd, aa) = ss.get_path_data(target).unwrap();
    start.stamina.0 -= dd.total_cost().0;

    //let extra=dd.diag_move_cost();
    //start.move_bank.0-=extra.0;

    let tt = *target;
    let aa = animation::Animation::new(start.position, dd, &g1.grid_matrix, start);
    let aaa = AnimationTicker::new(aa).map(move |res, _| {
        let mut warrior = res.into_data();
        warrior.position = tt;
        warrior
    });
    aaa
}

//Execute a player move. Return whether or not the unit moved as a result.
fn handle_one_execution(
    sss: WarriorPointer<GridCoord>,
    c: CellSelection,
    cell: PlayerCellAskRes,
    g1: &mut Stuff,
) -> impl GameStepper<GameHandle, Result = Option<WarriorPointer<GridCoord>>> {
    let (ss, att) = match c {
        CellSelection::MoveSelection(ss, a) => (ss, a),
        _ => unreachable!(),
    };

    match cell {
        PlayerCellAskRes::Attack(cell) => {
            //If attack handle attack.
            let n = attack_init(&att, g1, &sss, &cell);
            n.either_a()
        }
        PlayerCellAskRes::MoveTo(target) => {
            let doop = g1.this_team.lookup_take(sss);

            let aaa = move_animator(&ss, doop, &target, g1).map(|target, game| {
                let ooo = target.slim();
                game.this_team.add(target);
                Some(ooo)
            });
            aaa.either_b()
        }
    }
    .map(|a, _| match a {
        gameplay::Either::A(a) => a,
        gameplay::Either::B(a) => a,
    })
}

fn handle_player_move_inner() -> impl GameStepper<GameHandle, Result = Option<()>> {

    select_unit()
        .map(move |c, stuff| {
            gameplay::looper(c, |c, stuff| {
                let unit = stuff.this_team.lookup(c);
                let cc = generate_unit_possible_moves(&unit, stuff);
                //Ask the user to pick a possible move and execute it.
                let v = PlayerCellAsk::new(cc, c)
                    .map(|c, stuff| {
                        if let Some(cc) = c.2 {
                            handle_one_execution(c.0, c.1, cc, stuff).optional_some()
                        } else {
                            //TODO break out of the loop here!!!!
                            None
                        }
                    })
                    .flatten();

                //Now check and see if there are any additional moves possible, if so
                //keep the unit selected and loop.
                v.map(|a, game| {
                    match a {
                        Some(Some(a)) => {
                            let unit = game.this_team.lookup(a);

                            if Warrior::has_possible_moves(&unit,game)
                            {
                                gameplay::LooperRes::Loop(a)
                            } else {
                                gameplay::LooperRes::Finish(())
                            }
                        }
                        _ => gameplay::LooperRes::Finish(()),
                    }
                })
            })
        })
        .flatten()
        .map(|a, _| Some(()))
}

fn handle_player_move() -> impl GameStepper<GameHandle, Result = ()> {
    let wait_end_turn_button = || WaitResetButton.map(|_, _| true);

    let loops = move || {
        handle_player_move_inner()
            .map(|_, _| false)
            .or(wait_end_turn_button().map(|_, _| true))
    };

    gameplay::next::<GameHandle>()
        .map(move |_, stuff: &mut Stuff| {
            stuff.this_team.replenish_stamina();

            gameplay::looper((), move |_, _| {
                loops().map(|res, _| {
                    if res {
                        gameplay::LooperRes::Finish(())
                    } else {
                        gameplay::LooperRes::Loop(())
                    }
                })
            })
        })
        .flatten()
        .map(|_, stuff| {
            stuff.this_team.reset_attacked();
        })
}

pub fn create_state_machine() -> impl GameStepper<GameHandle> {
    gameplay::looper((), move |_, _| {
        handle_player_move().map(|_, stuff| {
            *stuff.team += 1;
            if *stuff.team > 1 {
                *stuff.team = 0;
            }
            gameplay::LooperRes::Loop(()).infinite()
        })
    })
}

struct WaitResetButton;
impl GameStepper<GameHandle> for WaitResetButton {
    type Result = ();
    type Int = ();
    fn step(&mut self, game: &mut Stuff<'_>) -> gameplay::Stage<()> {
        if game.end_turn {
            gameplay::Stage::NextStage(())
        } else {
            gameplay::Stage::Stay
        }
    }
    fn consume(self, _: &mut Stuff<'_>, _: ()) -> Self::Result {
        ()
    }
}

struct WaitMouseInput;
impl GameStepper<GameHandle> for WaitMouseInput {
    type Result = [f32; 2];
    type Int = ();
    fn step(&mut self, game: &mut Stuff<'_>) -> gameplay::Stage<()> {
        if let Some(_) = game.mouse {
            gameplay::Stage::NextStage(())
        } else {
            gameplay::Stage::Stay
        }
    }
    fn consume(self, game: &mut Stuff<'_>, _: ()) -> Self::Result {
        game.mouse.unwrap()
    }
}

struct AnimationTicker {
    a: animation::Animation<WarriorPointer<Warrior>>,
}
impl AnimationTicker {
    pub fn new(a: animation::Animation<WarriorPointer<Warrior>>) -> Self {
        Self { a }
    }
}
impl GameStepper<GameHandle> for AnimationTicker {
    type Result = animation::Animation<WarriorPointer<Warrior>>;
    type Int = ();
    fn consume(self, _: &mut Stuff<'_>, _: ()) -> Self::Result {
        self.a
    }
    fn step(&mut self, _game: &mut Stuff<'_>) -> gameplay::Stage<()> {
        if let Some(_) = self.a.animate_step() {
            gameplay::Stage::Stay
        } else {
            gameplay::Stage::NextStage(())
        }
    }

    fn get_animation(&self) -> Option<&crate::animation::Animation<WarriorPointer<Warrior>>> {
        Some(&self.a)
    }
}

struct PlayerCellAsk {
    a: CellSelection,
    //We know what type of warrior is selected at this point.
    stuff: WarriorPointer<GridCoord>,
}

impl PlayerCellAsk {
    pub fn new(a: CellSelection, stuff: WarriorPointer<GridCoord>) -> Self {
        Self { a, stuff }
    }
}
enum PlayerCellAskRes {
    Attack(WarriorPointer<GridCoord>),
    MoveTo(GridCoord),
}
impl GameStepper<GameHandle> for PlayerCellAsk {
    type Result = (
        WarriorPointer<GridCoord>,
        CellSelection,
        Option<PlayerCellAskRes>,
    );
    type Int = Option<PlayerCellAskRes>;
    fn get_selection(&self) -> Option<&CellSelection> {
        Some(&self.a)
    }
    fn consume(self, _: &mut Stuff<'_>, grid_coord: Self::Int) -> Self::Result {
        (self.stuff, self.a, grid_coord)
    }
    fn step(&mut self, g1: &mut Stuff<'_>) -> gameplay::Stage<Self::Int> {
        if let Some(mouse_world) = g1.mouse {
            let cell: GridCoord = GridCoord(g1.grid_matrix.to_grid((mouse_world).into()).into());

            match &self.a {
                CellSelection::MoveSelection(ss, attack) => {
                    let target_cat_pos = &cell;

                    let xx = g1.this_team.lookup(self.stuff).slim();

                    let current_attack = g1.this_team.lookup_mut(&xx).attacked;

                    let aa = if let Some(aaa) = g1.that_team.find_slow(target_cat_pos) {
                        let aaa = aaa.slim();

                        if !current_attack
                            && movement::contains_coord(attack.iter_coords(), target_cat_pos)
                        {
                            Some(PlayerCellAskRes::Attack(aaa))
                        } else {
                            None
                        }
                    } else if movement::contains_coord(ss.iter_coords(), &cell) {
                        Some(PlayerCellAskRes::MoveTo(cell))
                    } else {
                        let va = g1.this_team.find_slow(&cell).and_then(|a| {
                            if a.selectable() && a.slim() != self.stuff {
                                Some(a)
                            } else {
                                None
                            }
                        });
                        if let Some(va) = va {
                            self.a = generate_unit_possible_moves(&va, g1);
                            self.stuff = va.slim();
                            return gameplay::Stage::Stay;
                        } else {
                            None
                        }
                    };

                    gameplay::Stage::NextStage(aa)
                }
                _ => {
                    todo!()
                }
            }
        } else {
            gameplay::Stage::Stay
        }
    }
}

pub fn team_view(a: [&mut Tribe; 2], ind: usize) -> [&mut Tribe; 2] {
    let [a, b] = a;
    match ind {
        0 => [a, b],
        1 => [b, a],
        _ => {
            unreachable!()
        }
    }
}

pub fn generate_unit_possible_moves(unit: &WarriorPointer<&Warrior>, game: &Stuff) -> CellSelection {
    fn get_cat_move_attack_matrix(
        movement: (i8, i8),
        cat: &Warrior,
        cat_filter: impl Filter,
        roads: impl MoveCost,
        gg: &grids::GridMatrix,
        moved: bool,
    ) -> CellSelection {
        let (movement, attack) = movement;
        let mm = if !cat.attacked {
            cat.stamina
        } else {
            MoveUnit(0)
        };

        let mm = movement::PossibleMoves::new(
            &movement::WarriorMovement,
            &gg.filter().chain(cat_filter),
            &terrain::Grass.chain(roads),
            cat.position,
            mm,
        );

        let attack_range = if !cat.attacked { attack } else { 0 };

        //let attack_range=attack;

        let attack = movement::PossibleMoves::new(
            &movement::WarriorMovement,
            &gg.filter().chain(SingleFilter { a: cat.get_pos() }),
            &terrain::Grass,
            cat.position,
            MoveUnit(attack_range),
        );

        CellSelection::MoveSelection(mm, attack)
    }

    let data = game.this_team.get_movement_data(&unit);

    get_cat_move_attack_matrix(
        data,
        &unit,
        game.this_team.filter().chain(game.that_team.filter()),
        terrain::Grass,
        &game.grid_matrix,
        true,
    )
}
