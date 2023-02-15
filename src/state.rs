use super::*;

pub struct GameHandle;
impl gameplay::Zoo for GameHandle {
    type G<'a> = Stuff<'a>;
    fn create() -> Self {
        GameHandle
    }
}

pub struct Stuff<'a> {
    pub a: &'a mut Game,
    pub mouse: Option<[f32; 2]>,
    pub reset: bool,
}

pub fn create_state_machine() -> impl GameStepper<GameHandle> {
    let select_unit = || {
        gameplay::looper(
            (),
            |()| WaitMouseInput,
            |mouse_world, stuff| {
                let game = &mut stuff.a;
                let [this_team, that_team] = team_view([&mut game.cats, &mut game.dogs], game.team);

                let cell: GridCoord =
                    GridCoord(game.grid_matrix.to_grid((mouse_world).into()).into());

                let Some(unit)=this_team.find(&cell) else {
                    return gameplay::LooperRes::Loop(());
                };

                if !unit.is_selectable() {
                    return gameplay::LooperRes::Loop(());
                }

                let pos = get_cat_move_attack_matrix(
                    unit,
                    this_team.filter().chain(that_team.filter()),
                    terrain::Grass,
                    &game.grid_matrix,
                    false,
                );

                gameplay::LooperRes::Finish(pos)
            },
        )
    };

    let handle_move = move || {
        let k = move || {
            select_unit()
                .map(|c, _| PlayerCellAsk::new(c))
                .wait()
                .map(|(c, cell), g1| {
                    let game = &mut g1.a;
                    if let Some(cell) = cell {
                        let [this_team, that_team] =
                            team_view([&mut game.cats, &mut game.dogs], game.team);
                        match c {
                            CellSelection::MoveSelection(ss, _attack) => match cell {
                                PlayerCellAskRes::Attack(cell) => {
                                    let target_cat_pos = &cell;

                                    let target_cat = that_team.find_mut(target_cat_pos).unwrap();
                                    target_cat.health -= 1;

                                    let current_cat = this_team.find_mut(ss.start()).unwrap();
                                    current_cat.moved = true;
                                    gameplay::optional(Some(gameplay::Either::A(gameplay::Next)))
                                }
                                PlayerCellAskRes::MoveTo(cell) => {
                                    let mut c = this_team.remove(ss.start());
                                    let (dd, aa) = ss.get_path_data(cell).unwrap();
                                    c.position = cell;
                                    c.move_deficit = *aa;
                                    //c.moved = true;
                                    let aa = animation::Animation::new(
                                        ss.start(),
                                        dd,
                                        &game.grid_matrix,
                                        c,
                                    );
                                    let aaa = AnimationTicker::new(aa)
                                        .map(move |res, game| {
                                            let warrior = res.into_data();
                                            let [this_team, that_team] = team_view(
                                                [&mut game.a.cats, &mut game.a.dogs],
                                                game.a.team,
                                            );

                                            this_team.elem.push(warrior);

                                            let unit = this_team.elem.last().unwrap();

                                            let pos = get_cat_move_attack_matrix(
                                                unit,
                                                this_team.filter().chain(that_team.filter()),
                                                terrain::Grass,
                                                &game.a.grid_matrix,
                                                true,
                                            );

                                            PlayerCellAsk::new(pos)
                                        })
                                        .wait()
                                        .map(move |(ss, b), game| {
                                            let ss = match ss {
                                                CellSelection::MoveSelection(ss, _) => ss,
                                                _ => unreachable!(),
                                            };

                                            let game = &mut game.a;
                                            let [this_team, that_team] = team_view(
                                                [&mut game.cats, &mut game.dogs],
                                                game.team,
                                            );
                                            if let Some(b) = b {
                                                match b {
                                                    PlayerCellAskRes::Attack(cell) => {
                                                        log!("ATTACKING");
                                                        let target_cat_pos = &cell;

                                                        let target_cat = that_team
                                                            .find_mut(target_cat_pos)
                                                            .unwrap();
                                                        target_cat.health -= 1;

                                                        let current_cat =
                                                            this_team.find_mut(ss.start()).unwrap();
                                                        current_cat.moved = true;
                                                    }
                                                    _ => unreachable!(),
                                                }
                                            } else {
                                                let current_cat =
                                                    this_team.find_mut(ss.start()).unwrap();
                                                current_cat.moved = true;
                                                log!("EMPTY");
                                            }
                                            ()
                                        });
                                    gameplay::optional(Some(gameplay::Either::B(aaa)))
                                }
                            },
                            CellSelection::BuildSelection(_) => todo!(),
                        }
                    } else {
                        gameplay::optional(None)
                    }
                })
                .wait()
        };

        gameplay::looper(
            (),
            move |()| k(),
            move |res, _stuff| match res {
                Some(_animation) => gameplay::LooperRes::Finish(()),
                None => gameplay::LooperRes::Loop(()),
            },
        )
    };

    let wait_reset_button = || {
        WaitResetButton.map(move |_, g1| {
            let game = &mut g1.a;
            let [this_team, _] = team_view([&mut game.cats, &mut game.dogs], game.team);
            for a in this_team.elem.iter_mut() {
                a.moved = false;
            }
        })
    };

    //let handle_move2=move ||;

    let testo = gameplay::looper(
        (),
        move |()| handle_move().or(wait_reset_button()),
        move |_, stuff| {
            stuff.a.team += 1;
            if stuff.a.team > 1 {
                stuff.a.team = 0;
            }
            gameplay::LooperRes::Loop(()).infinite()
        },
    );

    testo
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
        let game = &mut g1.a;
        if let Some(mouse_world) = g1.mouse {
            let cell: GridCoord = GridCoord(game.grid_matrix.to_grid((mouse_world).into()).into());

            match &self.a {
                CellSelection::MoveSelection(ss, attack) => {
                    let [this_team, other_team] =
                        team_view([&mut game.cats, &mut game.dogs], game.team);

                    let target_cat_pos = &cell;

                    let current_attack = this_team.find_mut(ss.start()).unwrap().moved;

                    let aa = if !current_attack
                        && movement::contains_coord(attack.iter_coords(), target_cat_pos)
                        && other_team.find(target_cat_pos).is_some()
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
