use super::*;

pub struct GameHandle;
impl gameplay::Zoo for GameHandle {
    type G<'a> = Stuff<'a>;
}

pub struct Stuff<'a> {
    pub team: &'a mut usize,
    pub grid_matrix: &'a grids::GridMatrix,
    pub this_team: &'a mut Tribe,
    pub that_team: &'a mut Tribe,
    pub mouse: Option<[f32; 2]>,
    pub reset: bool,
}

fn select_unit() -> impl GameStepper<GameHandle, Result = WarriorPointer<GridCoord>> {
    gameplay::looper(
        (),
        |()| WaitMouseInput,
        |mouse_world, stuff| {
            let cell: GridCoord = GridCoord(stuff.grid_matrix.to_grid((mouse_world).into()).into());

            let Some(unit)=stuff.this_team.find_slow(&cell) else {
                return gameplay::LooperRes::Loop(());
            };

            if !unit.is_selectable() {
                return gameplay::LooperRes::Loop(());
            }

            let pos = unit.slim();

            gameplay::LooperRes::Finish(pos)
        },
    )
}

fn attack_init(
    ss: &movement::PossibleMoves,
    g1: &mut Stuff,
    current: &WarriorPointer<GridCoord>,
    target: &WarriorPointer<GridCoord>,
) -> impl GameStepper<GameHandle, Result = ()> {
    let damage = 5;
    let counter_damage = 5;
    let cc = *current;

    let kill_self = g1.this_team.lookup_mut(current).health <= counter_damage;

    if g1.that_team.lookup_mut(target).health <= damage {
        let c = g1.this_team.lookup_take(*current);

        gameplay::Either::A(kill_animator(ss, c, target, g1).map(move |this_unit, g1| {
            let target = this_unit.slim();
            g1.that_team.lookup_take(target);
            g1.this_team.add(this_unit);

            let mut current_cat = g1.this_team.lookup_mut(&target);
            current_cat.moved = true;
        }))
    } else {
        let c = g1.this_team.lookup_take(*current);
        let tt = *target;
        gameplay::Either::B(
            attack_animator(ss, c, target, g1).map(move |this_unit, g1| {
                let target = tt;
                g1.this_team.add(this_unit);
                let mut target_cat = g1.that_team.lookup_mut(&target);
                target_cat.health -= damage;

                let mut current_cat = g1.this_team.lookup_mut(&cc);
                current_cat.moved = true;

                //if !target_cat.moved{
                if kill_self {
                    g1.this_team.lookup_take(cc);
                } else {
                    current_cat.health -= counter_damage;
                }
                //}
            }),
        )
    }
    .map(|_, _| ())
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
    start.move_deficit = *aa;

    let tt = *target;
    let aa = animation::Animation::new(start.position, dd, &g1.grid_matrix, start);
    let aaa = AnimationTicker::new(aa).map(move |res, _| {
        let mut warrior = res.into_data();
        warrior.position = tt;
        warrior
    });
    aaa
}

fn handle_player_move_inner() -> impl GameStepper<GameHandle, Result = Option<()>> {
    //TODO why is type annotation required here?
    let aa = |(sss, c, cell): (WarriorPointer<GridCoord>, _, _), g1: &mut Stuff| {
        let Some(cell)=cell else{
            return gameplay::optional(None);
        };

        let (ss, att) = match c {
            CellSelection::MoveSelection(ss, a) => (ss, a),
            _ => unreachable!(),
        };

        let target = match cell {
            PlayerCellAskRes::Attack(cell) => {
                let n = attack_init(&att, g1, &sss, &cell);

                return gameplay::optional(Some(gameplay::Either::A(n)));
            }
            PlayerCellAskRes::MoveTo(target) => target,
        };

        let doop = g1.this_team.lookup_take(sss);

        let aaa = move_animator(&ss, doop, &target, g1)
            .map(|target, game| {
                let ooo = target.slim();
                game.this_team.add(target);
                //let unit = game.this_team.find(&target).unwrap();
                let unit = game.this_team.lookup(ooo);

                let data = game.this_team.get_movement_data(&unit);

                let pos = get_cat_move_attack_matrix(
                    data,
                    &unit,
                    game.this_team.filter().chain(game.that_team.filter()),
                    terrain::Grass,
                    &game.grid_matrix,
                    true,
                );
                PlayerCellAsk::new(pos, ooo)
            })
            .wait()
            .map(|(lll, ss, b), game| {
                let (_, att) = match ss {
                    CellSelection::MoveSelection(ss, att) => (ss, att),
                    _ => unreachable!(),
                };

                if let Some(b) = b {
                    match b {
                        PlayerCellAskRes::Attack(cell) => {
                            gameplay::Either::A(attack_init(&att, game, &lll, &cell))
                        }
                        _ => unreachable!(),
                    }
                } else {
                    let mut current_cat = game.this_team.lookup_mut(&lll);
                    current_cat.moved = true;
                    gameplay::Either::B(gameplay::next())
                }
            })
            .wait();

        gameplay::optional(Some(gameplay::Either::B(aaa)))
    };

    select_unit()
        .map(|c, stuff| {
            let unit = stuff.this_team.lookup(c);

            let data = stuff.this_team.get_movement_data(&unit);

            let cc = get_cat_move_attack_matrix(
                data,
                &unit,
                stuff.this_team.filter().chain(stuff.that_team.filter()),
                terrain::Grass,
                &stuff.grid_matrix,
                false,
            );

            PlayerCellAsk::new(cc, c)
        })
        .wait()
        .map(aa)
        .wait()
        .map(|a, _| a.map(|_| ()))
}

fn handle_player_move() -> impl GameStepper<GameHandle, Result = ()> {
    gameplay::looper(
        (),
        |()| handle_player_move_inner(),
        |res, _stuff| match res {
            Some(_) => gameplay::LooperRes::Finish(()),
            None => gameplay::LooperRes::Loop(()),
        },
    )
}

pub fn create_state_machine() -> impl GameStepper<GameHandle> {
    let wait_reset_button = || {
        WaitResetButton.map(|_, g1| {
            g1.this_team.reset();
        })
    };

    gameplay::looper(
        (),
        move |()| handle_player_move().or(wait_reset_button()),
        |_, stuff| {
            *stuff.team += 1;
            if *stuff.team > 1 {
                *stuff.team = 0;
            }
            gameplay::LooperRes::Loop(()).infinite()
        },
    )
}

struct WaitResetButton;
impl GameStepper<GameHandle> for WaitResetButton {
    type Result = ();
    type Int = ();
    fn step(&mut self, game: &mut Stuff<'_>) -> gameplay::Stage<()> {
        if game.reset {
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

                    let current_attack = g1.this_team.lookup_mut(&xx).moved;

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
                        None
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

fn get_cat_move_attack_matrix(
    movement: (i8, i8),
    cat: &Warrior,
    cat_filter: impl Filter,
    roads: impl MoveCost,
    gg: &grids::GridMatrix,
    moved: bool,
) -> CellSelection {
    let (movement, attack) = movement;
    let mm = if moved {
        MoveUnit(0)
    } else {
        MoveUnit(movement - 1)
    };

    let mm = movement::PossibleMoves::new(
        &movement::WarriorMovement,
        &gg.filter().chain(cat_filter),
        &terrain::Grass.chain(roads),
        cat.position,
        mm,
    );

    let attack_range = attack - 1;
    let attack = movement::PossibleMoves::new(
        &movement::WarriorMovement,
        &gg.filter().chain(SingleFilter { a: cat.get_pos() }),
        &terrain::Grass,
        cat.position,
        MoveUnit(attack_range),
    );

    CellSelection::MoveSelection(mm, attack)
}
