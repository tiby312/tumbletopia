use super::*;

pub struct Doopo;
impl gameplay::Zoo for Doopo {
    type G<'a> = Stuff<'a>;
    fn create() -> Self {
        Doopo
    }
}

pub struct Stuff<'a> {
    pub a: &'a mut Game,
    pub mouse: Option<[f32; 2]>,
}

pub fn create_state_machine() -> impl GameStepper<Doopo> {

    
    let wait_mouse_input = || {
        gameplay::wait_custom(Doopo, |e| {
            if let Some(m) = e.mouse {
                gameplay::Stage::NextStage(m)
            } else {
                gameplay::Stage::Stay
            }
        })
    };

    
    let select_unit = move |team| {
        gameplay::looper(
            move |()| wait_mouse_input(),
            (),
            move |mouse_world, stuff| {
                let game = &mut stuff.a;
                let [this_team, that_team] =
                    gameplay::team_view([&mut game.cats, &mut game.dogs], team);

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
                );

                gameplay::LooperRes::Finish(pos)
            },
        )
    };

    let handle_move = move |team| {
        let k = move |team| {
            select_unit(team)
                .and_then(move |c, _game| PlayerCellAsk::new(c, team))
                .and_then(move |(c, cell), g1| {
                    let game = &mut g1.a;
                    if let Some(cell) = cell {
                        let [this_team, _that_team] =
                            gameplay::team_view([&mut game.cats, &mut game.dogs], team);

                        match c {
                            CellSelection::MoveSelection(ss, _attack) => {
                                let mut c = this_team.remove(ss.start());
                                let (dd, aa) = ss.get_path_data(cell).unwrap();
                                c.position = cell;
                                c.move_deficit = *aa;
                                c.moved = true;
                                let aa =
                                    animation::Animation::new(ss.start(), dd, &game.grid_matrix, c);
                                let aaa = AnimationTicker::new(aa).and_then(move |res, game| {
                                    let warrior = res.into_data();
                                    let [this_team, _that_team] = gameplay::team_view(
                                        [&mut game.a.cats, &mut game.a.dogs],
                                        team,
                                    );

                                    this_team.elem.push(warrior);
                                    gameplay::next()
                                });
                                gameplay::optional(Some(aaa))
                            }
                            CellSelection::BuildSelection(_) => todo!(),
                        }
                    } else {
                        gameplay::optional(None)
                    }
                })
        };

        gameplay::looper(
            move |()| k(team),
            (),
            move |res, _stuff| match res {
                Some(_animation) => gameplay::LooperRes::Finish(()),
                None => gameplay::LooperRes::Loop(()),
            },
        )
    };

    let mut counter = 0;
    let testo = gameplay::looper(
        move |c| handle_move(c),
        0,
        move |_res, _stuff| {
            counter += 1;
            if counter > 1 {
                counter = 0;
            }
            gameplay::LooperRes::Loop(counter).infinite()
        },
    );

    testo
}


struct AnimationTicker {
    a: animation::Animation<Warrior>,
}
impl AnimationTicker {
    pub fn new(a: animation::Animation<Warrior>) -> Self {
        Self { a }
    }
}
impl GameStepper<Doopo> for AnimationTicker {
    type Result = animation::Animation<Warrior>;
    fn consume(self, _: &mut Stuff<'_>) -> Self::Result {
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
    team: usize,
    found_cell: Option<GridCoord>,
}

impl PlayerCellAsk {
    pub fn new(a: CellSelection, team: usize) -> Self {
        Self {
            a,
            team,
            found_cell: None,
        }
    }
}
impl GameStepper<Doopo> for PlayerCellAsk {
    type Result = (CellSelection, Option<GridCoord>);
    fn get_selection(&self) -> Option<&CellSelection> {
        Some(&self.a)
    }
    fn consume(self, _: &mut Stuff<'_>) -> Self::Result {
        (self.a, self.found_cell)
    }
    fn step(&mut self, g1: &mut Stuff<'_>) -> gameplay::Stage<()> {
        let game = &mut g1.a;
        if let Some(mouse_world) = g1.mouse {
            let cell: GridCoord =
                GridCoord(game.grid_matrix.to_grid((mouse_world).into()).into());

            match &self.a {
                CellSelection::MoveSelection(ss, _) => {
                    if movement::contains_coord(ss.iter_coords(), &cell) {
                        self.found_cell = Some(cell);
                    }
                    gameplay::Stage::NextStage(())
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
