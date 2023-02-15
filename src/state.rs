use super::*;

pub struct GameHandle;
impl gameplay::Zoo for GameHandle {
    type G<'a> = Stuff<'a>;
}

pub struct Stuff<'a> {
    pub team: &'a mut usize,
    pub grid_matrix: &'a grids::GridMatrix,
    pub this_team: &'a mut UnitCollection<Warrior>,
    pub that_team: &'a mut UnitCollection<Warrior>,
    pub mouse: Option<[f32; 2]>,
    pub reset: bool,
}

fn select_unit() -> impl GameStepper<GameHandle, Result = CellSelection> {
    gameplay::looper(
        (),
        |()| WaitMouseInput,
        |mouse_world, stuff| {
            let cell: GridCoord = GridCoord(stuff.grid_matrix.to_grid((mouse_world).into()).into());

            let Some(unit)=stuff.this_team.find(&cell) else {
                return gameplay::LooperRes::Loop(());
            };

            if !unit.is_selectable() {
                return gameplay::LooperRes::Loop(());
            }

            let pos = get_cat_move_attack_matrix(
                unit,
                stuff.this_team.filter().chain(stuff.that_team.filter()),
                terrain::Grass,
                &stuff.grid_matrix,
                false,
            );

            gameplay::LooperRes::Finish(pos)
        },
    )
}

fn attack_init(ss:&movement::PossibleMoves,g1: &mut Stuff, current: &GridCoord, target: &GridCoord)->impl GameStepper<GameHandle,Result=()> {
    let cc=*current;
    attack_animator(ss,current,target,g1).map(move |target,g1|{
        let target_cat_pos = target;

        let target_cat = g1.that_team.find_mut(&target_cat_pos).unwrap();
        target_cat.health -= 1;
    
        let current_cat = g1.this_team.find_mut(&cc).unwrap();
        current_cat.moved = true;
    })
    
}


fn attack_animator(
    ss: &movement::PossibleMoves,
    start: &GridCoord,
    target: &GridCoord,
    g1: &mut Stuff,
) -> impl GameStepper<GameHandle, Result = GridCoord> {
    let mut c = g1.this_team.remove(start);
    let (dd, aa) = ss.get_path_data(target).unwrap();
    //c.position = *target;
    c.move_deficit = *aa;

    let tt = *target;
    let aa = animation::Animation::new(ss.start(), dd, &g1.grid_matrix, c);
    let aaa = AnimationTicker::new(aa).map(move |res, game| {
        let warrior = res.into_data();

        game.this_team.elem.push(warrior);

        tt
    });
    aaa
}

fn animator(
    ss: &movement::PossibleMoves,
    start: &GridCoord,
    target: &GridCoord,
    g1: &mut Stuff,
) -> impl GameStepper<GameHandle, Result = GridCoord> {
    let mut c = g1.this_team.remove(start);
    let (dd, aa) = ss.get_path_data(target).unwrap();
    c.position = *target;
    c.move_deficit = *aa;

    let tt = *target;
    let aa = animation::Animation::new(ss.start(), dd, &g1.grid_matrix, c);
    let aaa = AnimationTicker::new(aa).map(move |res, game| {
        let warrior = res.into_data();

        game.this_team.elem.push(warrior);

        tt
    });
    aaa
}

fn handle_player_move_inner() -> impl GameStepper<GameHandle, Result = Option<()>> {
    select_unit()
        .map(|c, _| PlayerCellAsk::new(c))
        .wait()
        .map(|(c, cell), g1| {
            let Some(cell)=cell else{
                return gameplay::optional(None);
            };

            let (ss, att) = match c {
                CellSelection::MoveSelection(ss, a) => (ss, a),
                _ => unreachable!(),
            };

            let target = match cell {
                PlayerCellAskRes::Attack(cell) => {
                    let n=attack_init(&att,g1, att.start(), &cell);

                    return gameplay::optional(Some(gameplay::Either::A(n)));
                }
                PlayerCellAskRes::MoveTo(target) => target,
            };

            let aaa = animator(&ss, ss.start(), &target, g1)
                .map(|target, game| {
                    let unit = game.this_team.find(&target).unwrap();
                    let pos = get_cat_move_attack_matrix(
                        unit,
                        game.this_team.filter().chain(game.that_team.filter()),
                        terrain::Grass,
                        &game.grid_matrix,
                        true,
                    );
                    PlayerCellAsk::new(pos)
                })
                .wait()
                .map(|(ss, b), game| {
                    let (ss,att) = match ss {
                        CellSelection::MoveSelection(ss, att) => (ss,att),
                        _ => unreachable!(),
                    };

                    if let Some(b) = b {
                        match b {
                            PlayerCellAskRes::Attack(cell) => {
                                gameplay::Either::A(attack_init(&att,game, ss.start(), &cell))
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        let current_cat = game.this_team.find_mut(ss.start()).unwrap();
                        current_cat.moved = true;
                        gameplay::Either::B(gameplay::next())
                    }
                }).wait();

            gameplay::optional(Some(gameplay::Either::B(aaa)))
        })
        .wait()
        .map(|a, _| a.map(|_| ()))
}

fn handle_player_move() -> impl GameStepper<GameHandle, Result = ()> {
    gameplay::looper(
        (),
        move |()| handle_player_move_inner(),
        move |res, _stuff| match res {
            Some(_) => gameplay::LooperRes::Finish(()),
            None => gameplay::LooperRes::Loop(()),
        },
    )
}

pub fn create_state_machine() -> impl GameStepper<GameHandle> {
    let wait_reset_button = || {
        WaitResetButton.map(|_, g1| {
            for a in g1.this_team.elem.iter_mut() {
                a.moved = false;
            }
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
    a: animation::Animation<Warrior>,
}
impl AnimationTicker {
    pub fn new(a: animation::Animation<Warrior>) -> Self {
        Self { a }
    }
}
impl GameStepper<GameHandle> for AnimationTicker {
    type Result = animation::Animation<Warrior>;
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

    fn get_animation(&self) -> Option<&crate::animation::Animation<Warrior>> {
        Some(&self.a)
    }
}

struct PlayerCellAsk {
    a: CellSelection,
}

impl PlayerCellAsk {
    pub fn new(a: CellSelection) -> Self {
        Self { a }
    }
}
enum PlayerCellAskRes {
    Attack(GridCoord),
    MoveTo(GridCoord),
}
impl GameStepper<GameHandle> for PlayerCellAsk {
    type Result = (CellSelection, Option<PlayerCellAskRes>);
    type Int = Option<PlayerCellAskRes>;
    fn get_selection(&self) -> Option<&CellSelection> {
        Some(&self.a)
    }
    fn consume(self, _: &mut Stuff<'_>, grid_coord: Self::Int) -> Self::Result {
        (self.a, grid_coord)
    }
    fn step(&mut self, g1: &mut Stuff<'_>) -> gameplay::Stage<Self::Int> {
        if let Some(mouse_world) = g1.mouse {
            let cell: GridCoord = GridCoord(g1.grid_matrix.to_grid((mouse_world).into()).into());

            match &self.a {
                CellSelection::MoveSelection(ss, attack) => {
                    let target_cat_pos = &cell;

                    let current_attack = g1.this_team.find_mut(ss.start()).unwrap().moved;

                    let aa = if !current_attack
                        && movement::contains_coord(attack.iter_coords(), target_cat_pos)
                        && g1.that_team.find(target_cat_pos).is_some()
                    {
                        Some(PlayerCellAskRes::Attack(cell))
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

pub fn team_view(
    a: [&mut UnitCollection<Warrior>; 2],
    ind: usize,
) -> [&mut UnitCollection<Warrior>; 2] {
    let [a, b] = a;
    match ind {
        0 => [a, b],
        1 => [b, a],
        _ => {
            unreachable!()
        }
    }
}
