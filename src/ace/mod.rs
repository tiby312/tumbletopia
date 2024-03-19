use super::*;
use crate::movement::movement_mesh::SmallMesh;
mod ai;
pub mod selection;
use crate::{
    movement::{self, GridCoord},
    CellSelection, GameState, UnitData,
};

pub struct GameWrap<'a, T> {
    pub game: &'a GameState,
    pub team: ActiveTeam,
    pub data: T,
}

pub struct GameWrapResponse<'a, T> {
    pub game: &'a GameState,
    pub data: T,
}

pub struct AnimationWrapper<K> {
    pub unwrapper: K,
    pub enu: animation::AnimationCommand,
}

#[derive(Debug, Clone)]
pub enum MousePrompt {
    Selection {
        selection: CellSelection,
        grey: bool,
    },
    None,
}

#[derive(Debug)]
pub enum Command {
    Animate(animation::AnimationCommand),
    GetMouseInput(MousePrompt),
    Nothing,
    Popup(String),
    Poke,
}

#[derive(Debug)]
pub enum Pototo<T> {
    Normal(T),
    EndTurn,
}

#[derive(Debug)]
pub enum Response {
    Mouse(MousePrompt, Pototo<GridCoord>),
    AnimationFinish,
    Ack,
}

use futures::{
    channel::mpsc::{Receiver, Sender},
    SinkExt, StreamExt,
};

pub struct WorkerManager<'a> {
    pub game: *mut GameState,
    pub sender: Sender<GameWrap<'a, Command>>,
    pub receiver: Receiver<GameWrapResponse<'a, Response>>,
}

impl<'a> WorkerManager<'a> {
    pub async fn wait_animation<'c>(
        &mut self,
        animation: animation::AnimationCommand,
        team_index: ActiveTeam,
    ) {
        let game = unsafe { &*self.game };
        self.sender
            .send(GameWrap {
                team: team_index,
                game,
                data: Command::Animate(animation),
            })
            .await
            .unwrap();

        let GameWrapResponse { game: _gg, data } = self.receiver.next().await.unwrap();
        let Response::AnimationFinish = data else {
            unreachable!();
        };
    }

    async fn get_mouse_no_selection<'c>(&mut self, team_index: ActiveTeam) -> Pototo<GridCoord> {
        let (_, c) = self.get_mouse(MousePrompt::None, team_index).await;
        c
    }
    async fn get_mouse_selection<'c>(
        &mut self,
        cell: CellSelection,
        team_index: ActiveTeam,
        grey: bool,
    ) -> (CellSelection, Pototo<GridCoord>) {
        let (b, c) = self
            .get_mouse(
                MousePrompt::Selection {
                    selection: cell,
                    grey,
                },
                team_index,
            )
            .await;

        let MousePrompt::Selection {
            selection,
            grey: grey2,
        } = b
        else {
            unreachable!()
        };
        assert_eq!(grey2, grey);

        (selection, c)
    }

    async fn poke(&mut self, team_index: ActiveTeam) {
        let game = unsafe { &*self.game };

        self.sender
            .send(GameWrap {
                game,
                data: Command::Poke,
                team: team_index,
            })
            .await
            .unwrap();

        let GameWrapResponse { game: _gg, data } = self.receiver.next().await.unwrap();

        let Response::Ack = data else {
            unreachable!();
        };
    }
    async fn send_popup(&mut self, str: &str, team_index: ActiveTeam) {
        let game = unsafe { &*self.game };

        self.sender
            .send(GameWrap {
                game,
                data: Command::Popup(str.into()),
                team: team_index,
            })
            .await
            .unwrap();

        let GameWrapResponse { game: _gg, data } = self.receiver.next().await.unwrap();

        let Response::Ack = data else {
            unreachable!();
        };
    }

    async fn get_mouse<'c>(
        &mut self,
        cell: MousePrompt,
        team_index: ActiveTeam,
    ) -> (MousePrompt, Pototo<GridCoord>) {
        let game = unsafe { &*self.game };

        self.sender
            .send(GameWrap {
                game,
                data: Command::GetMouseInput(cell),
                team: team_index,
            })
            .await
            .unwrap();

        let GameWrapResponse { game: _gg, data } = self.receiver.next().await.unwrap();

        let Response::Mouse(cell, o) = data else {
            unreachable!();
        };

        (cell, o)
    }
}

#[derive(Hash, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ActiveTeam {
    Cats = 0,
    Dogs = 1,
}
impl ActiveTeam {
    pub fn iter(&self) -> impl Iterator<Item = Self> {
        [*self, self.not()].into_iter().cycle()
    }
    pub fn not(&self) -> Self {
        match self {
            ActiveTeam::Cats => ActiveTeam::Dogs,
            ActiveTeam::Dogs => ActiveTeam::Cats,
        }
    }
}

pub struct SelectType {
    warrior: GridCoord,
    team: ActiveTeam,
}
impl SelectType {
    pub fn with(mut self, a: GridCoord) -> Self {
        self.warrior = a;
        self
    }
    pub fn not(mut self) -> Self {
        self.team = self.team.not();
        self
    }
    pub fn with_team(mut self, a: ActiveTeam) -> Self {
        self.team = a;
        self
    }
}

pub enum LoopRes<T> {
    EndTurn(moves::ActualMove),
    Deselect,
    Select(T),
}

pub async fn reselect_loop(
    doop: &mut WorkerManager<'_>,
    game: &mut GameState,
    team_index: ActiveTeam,
    extra_attack: &mut Option<selection::PossibleExtra>,
    selected_unit: SelectType,
) -> LoopRes<SelectType> {
    console_dbg!(extra_attack.is_some());
    //At this point we know a friendly unit is currently selected.

    //let mut relative_game_view = game.view_mut(selected_unit.team);

    let unwrapped_selected_unit = selected_unit.warrior;

    let unit = game
        .factions
        .relative(selected_unit.team)
        .this_team
        .find_slow(&unwrapped_selected_unit)
        .unwrap();

    let grey = if selected_unit.team == team_index {
        //If we are in the middle of a extra attack move, make sure
        //no other friendly unit is selectable until we finish moving the
        //the unit that has been partially moved.
        if let Some(e) = extra_attack {
            e.coord() != selected_unit.warrior
        } else {
            false
        }
    } else {
        true
    };

    let cca = game.generate_unit_possible_moves_inner2(
        &unit.position,
        unit.typ,
        selected_unit.team,
        extra_attack.as_ref().map(|a| a.prev_move.unit),
    );

    //let cc = relative_game_view.get_unit_possible_moves(&unit, extra_attack);
    let cc = CellSelection::MoveSelection(unwrapped_selected_unit, cca.clone());

    let (cell, pototo) = doop.get_mouse_selection(cc, selected_unit.team, grey).await;

    let mouse_world = match pototo {
        Pototo::Normal(t) => t,
        Pototo::EndTurn => {
            //End the turn. Ok because we are not int he middle of anything.
            //return LoopRes::EndTurn;
            unreachable!();
        }
    };
    let target_cell = mouse_world;

    //This is the cell the user selected from the pool of available moves for the unit
    let CellSelection::MoveSelection(_, ss) = cell else {
        unreachable!()
    };

    //If we just clicked on ourselves, just deselect.
    if target_cell == unwrapped_selected_unit {
        return LoopRes::Deselect;
    }

    let contains = movement::contains_coord(ss.iter_mesh(unwrapped_selected_unit), target_cell);

    //If we select a friendly unit quick swap
    if let Some(target) = game
        .factions
        .relative(selected_unit.team)
        .this_team
        .find_slow(&target_cell)
    {
        if !contains {
            //it should be impossible for a unit to move onto a friendly
            //assert!(!contains);
            return LoopRes::Select(selected_unit.with(target.position));
        }
    }

    //If we select an enemy unit quick swap
    if let Some(target) = game
        .factions
        .relative(selected_unit.team)
        .that_team
        .find_slow(&target_cell)
    {
        if selected_unit.team != team_index || !contains {
            //If we select an enemy unit thats outside of our units range.
            return LoopRes::Select(selected_unit.with(target.position).not());
        }
    }

    //If we selected an empty space, deselect.
    if !contains {
        return LoopRes::Deselect;
    }

    //If we are trying to move an enemy piece, deselect.
    if selected_unit.team != team_index {
        return LoopRes::Deselect;
    }

    // If we are trying to move a piece while in the middle of another
    // piece move, deselect.
    if let Some(e) = extra_attack {
        if unwrapped_selected_unit != e.coord() {
            return LoopRes::Deselect;
        }
    }

    //At this point all re-selecting of units based off of the input has occured.
    //We definately want to act on the action the user took on the selected unit.

    {
        if let Some(e) = extra_attack {
            let iii = moves::PartialMove {
                this_unit: e.coord(),
                target: target_cell,
                is_extra: Some(e.prev_move),
                state: game,
            };

            iii.execute_with_animation(selected_unit.team, doop, cca.clone())
                .await;

            return LoopRes::EndTurn(moves::ActualMove::Normal {
                unit: e.prev_move.unit,
                moveto: e.prev_move.moveto,
                attackto: target_cell,
                effect: move_build::UndoInfo {
                    pushpull: e.prev_effect.clone(),
                    meta: move_build::MetaInfo {
                        fog: move_build::FogInfo(SmallMesh::new()),
                        bomb: move_build::BombInfo(SmallMesh::new()),
                    },
                },
            });
        } else {
            let p = unit.position;
            let this_unit = game
                .factions
                .relative_mut(selected_unit.team)
                .this_team
                .find_slow_mut(&p)
                .unwrap();
            let c = target_cell;
            let mut kk = this_unit.clone();
            kk.position = target_cell;

            let iii = moves::PartialMove {
                this_unit: p,
                target: target_cell,
                is_extra: None,
                state: game,
            };
            let (iii, effect, k) = iii
                .execute_with_animation(selected_unit.team, doop, cca.clone())
                .await;
            assert!(k.is_none());
            {
                //if cont {
                *extra_attack = Some(selection::PossibleExtra::new(iii, effect.unwrap(), kk));
                return LoopRes::Select(selected_unit.with(c).with_team(team_index));
                // } else {
                //     return LoopRes::EndTurn(moves::ActualMove::Powerup {
                //         unit: this_unit.position,
                //         moveto: target_cell,
                //     });
                // }
            }
        }
    }
}

pub fn game_init() -> GameState {
    let powerup = true;
    let d = 5;
    let cats = [[-d, d], [0, -d], [d, 0]];
    let cats = cats
        .into_iter()
        .map(|a| UnitData {
            position: GridCoord(a),
            typ: Type::Warrior { powerup },
            has_powerup: false,
        })
        .collect();

    //player
    let dogs = [[d, -d], [-d, 0], [0, d]];
    let dogs = dogs
        .into_iter()
        .map(|a| UnitData {
            position: GridCoord(a),
            typ: Type::Warrior { powerup },
            has_powerup: false,
        })
        .collect();

    let powerups = vec![]; //vec![[1, 1], [1, -2], [-2, 1]];

    let world = Box::leak(Box::new(board::MyWorld::new()));

    let fog = world.get_game_cells().clone();

    let mut k = GameState {
        factions: Factions {
            dogs: Tribe { units: dogs },
            cats: Tribe { units: cats },
        },
        env: Environment {
            land: BitField::from_iter([]),
            forest: BitField::from_iter([]),
            fog,
            powerups: powerups.into_iter().map(GridCoord).collect(),
        },
        world,
    };

    for a in k.factions.cats.iter().chain(k.factions.dogs.iter()) {
        move_build::uncover_fog(a.position, &mut k.env);
    }

    k
}

pub mod share {
    pub const SAMPLE_GAME:&str="TY5RAsAgCEIpT7D7n9UYUm3Wx1MkCfB5EExMLhNM1lmaXM1UP3Sldr+qUFXd2K8Pzw4z26y8FOm++a3VnqmMUJJZmlPh/H92/7L5+V8=";

    use super::*;
    pub fn load(s: &str) -> selection::MoveLog {
        use base64::prelude::*;
        let k = BASE64_STANDARD.decode(s).unwrap();
        let k = miniz_oxide::inflate::decompress_to_vec(&k).unwrap();
        selection::MoveLog::deserialize(k)
    }
    pub fn save(game_history: &selection::MoveLog) -> String {
        use base64::prelude::*;
        let k = game_history.serialize();
        let k = miniz_oxide::deflate::compress_to_vec(&k, 6);
        BASE64_STANDARD.encode(k)
    }
}

pub async fn main_logic<'a>(game: &'a mut GameState, mut doop: WorkerManager<'a>) {
    let mut game_history = selection::MoveLog::new();

    //Loop over each team!
    'game_loop: for team_index in ActiveTeam::Dogs.iter() {
        if let Some(g) = game.game_is_over(team_index) {
            console_dbg!("Game over=", g);
            break 'game_loop;
        }

        //Add AIIIIII.
        if team_index == ActiveTeam::Cats {
            //{
            doop.send_popup("AI Thinking", team_index).await;
            let the_move = ai::iterative_deepening(game, team_index, &mut doop).await;
            doop.send_popup("", team_index).await;
            the_move.execute_move_ani(game, team_index, &mut doop).await;
            game_history.push(the_move);

            continue;
        }

        let m = handle_player(game, &mut doop, team_index).await;

        game_history.push(m);

        ai::absolute_evaluate(game, true);
    }

    //console_dbg!(share::save(&game_history));
}

async fn handle_player(
    game: &mut GameState,
    doop: &mut WorkerManager<'_>,
    team_index: ActiveTeam,
) -> moves::ActualMove {
    let mut extra_attack = None;
    //Keep allowing the user to select units
    loop {
        //Loop until the user clicks on a selectable unit in their team.
        let mut selected_unit = loop {
            let data = doop.get_mouse_no_selection(team_index).await;
            let cell = match data {
                Pototo::Normal(a) => a,
                Pototo::EndTurn => {
                    log!("cant end turn!");
                    //game_history.push(moves::ActualMove::SkipTurn);
                    unreachable!();
                    //break 'select_loop;
                }
            };
            //let game = game.view_mut(team_index);

            if let Some(unit) = game
                .factions
                .relative(team_index)
                .this_team
                .find_slow(&cell)
            {
                break SelectType {
                    warrior: unit.position,
                    team: team_index,
                };
            }
            if let Some(unit) = game
                .factions
                .relative(team_index)
                .that_team
                .find_slow(&cell)
            {
                break SelectType {
                    warrior: unit.position,
                    team: team_index.not(),
                };
            }
        };

        //Keep showing the selected unit's options and keep handling the users selections
        //Until the unit is deselected.
        loop {
            let a = match reselect_loop(doop, game, team_index, &mut extra_attack, selected_unit)
                .await
            {
                LoopRes::EndTurn(m) => {
                    return m;
                    //game_history.push(m);
                    //break 'select_loop;
                }
                LoopRes::Deselect => break,
                LoopRes::Select(a) => a,
            };
            selected_unit = a;
        }
    }
}
