use super::*;
mod ai;
pub mod selection;
use crate::{CellSelection, GameState, UnitData};

pub struct GameWrap<T> {
    pub game: GameState,
    pub team: ActiveTeam,
    pub data: T,
}

pub struct GameWrapResponse<T> {
    pub game: GameState,
    pub data: T,
}

pub struct AnimationWrapper<K> {
    pub unwrapper: K,
    pub enu: animation::AnimationCommand,
}

#[derive(Debug)]
pub enum Command {
    Animate(animation::AnimationCommand),
    GetMouseInputSelection {
        selection: CellSelection,
        grey: bool,
    },
    GetMouseInputNoSelect,
    Nothing,
    Popup(String),
    Poke,
}

#[derive(Debug)]
pub enum Response {
    MouseWithSelection(CellSelection, Pototo<Axial>),
    Mouse(Pototo<Axial>),
    AnimationFinish,
    Ack,
}

#[derive(Debug)]
pub enum Pototo<T> {
    Normal(T),
    EndTurn,
}

use futures::{
    channel::mpsc::{Receiver, Sender},
    SinkExt, StreamExt,
};

pub struct WorkerManager {
    pub sender: Sender<GameWrap<Command>>,
    pub receiver: Receiver<GameWrapResponse<Response>>,
}

// trait Foop {
//     type Ret;
//     fn command(&self) -> Command;
//     fn unpack(&self, r: Response) -> Self::Ret;
// }

// struct MousePrompt2(CellSelection);
// impl Foop for MousePrompt2{
//     type Ret=CellSelection;
//     fn command(&self)->Command{
//         Command::GetMouseInput(self.0)
//     }
//     fn unpack(&self,r:Response)->Self::Ret{
//         let Response::Mouse(cell, o) = r else {
//             unreachable!();
//         };

//         cell
//     }
// }

impl WorkerManager {
    pub async fn wait_animation(
        &mut self,
        animation: animation::AnimationCommand,
        team: ActiveTeam,
        game: GameState,
    ) -> GameState {
        self.sender
            .send(GameWrap {
                team,
                game,
                data: Command::Animate(animation),
            })
            .await
            .unwrap();

        let GameWrapResponse { game, data } = self.receiver.next().await.unwrap();
        let Response::AnimationFinish = data else {
            unreachable!();
        };
        game
    }

    async fn get_mouse_selection(
        &mut self,
        cell: CellSelection,
        team: ActiveTeam,
        game: GameState,
        grey: bool,
    ) -> (CellSelection, Pototo<Axial>, GameState) {
        let (a, b) = self
            .send_command(
                team,
                game,
                Command::GetMouseInputSelection {
                    selection: cell,
                    grey,
                },
            )
            .await;

        let Response::MouseWithSelection(cell, o) = b else {
            unreachable!();
        };

        (cell, o, a)
    }

    async fn get_mouse_no_selection(
        &mut self,
        team: ActiveTeam,
        game: GameState,
    ) -> (Pototo<Axial>, GameState) {
        let (a, b) = self
            .send_command(team, game, Command::GetMouseInputNoSelect)
            .await;

        let Response::Mouse(o) = b else {
            unreachable!();
        };

        (o, a)
    }

    async fn poke(&mut self, team: ActiveTeam, game: GameState) {
        self.sender
            .send(GameWrap {
                game,
                data: Command::Poke,
                team,
            })
            .await
            .unwrap();

        let GameWrapResponse { game: _gg, data } = self.receiver.next().await.unwrap();

        let Response::Ack = data else {
            unreachable!();
        };
    }
    async fn send_popup(&mut self, str: &str, team: ActiveTeam, game: GameState) -> GameState {
        self.sender
            .send(GameWrap {
                game,
                data: Command::Popup(str.into()),
                team,
            })
            .await
            .unwrap();

        let GameWrapResponse { game, data } = self.receiver.next().await.unwrap();

        let Response::Ack = data else {
            unreachable!();
        };

        game
    }

    // async fn doop<F: Foop>(
    //     &mut self,
    //     team: ActiveTeam,
    //     game: GameState,
    //     f: F,
    // ) -> (GameState, F::Ret) {
    //     let (game, ret) = self.send_command(team, game, f.command()).await;
    //     (game, f.unpack(ret))
    // }

    //TODO use
    async fn send_command(
        &mut self,
        team: ActiveTeam,
        game: GameState,
        co: Command,
    ) -> (GameState, Response) {
        self.sender
            .send(GameWrap {
                game,
                data: co,
                team,
            })
            .await
            .unwrap();

        let GameWrapResponse { game, data } = self.receiver.next().await.unwrap();

        (game, data)
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
    warrior: Axial,
    team: ActiveTeam,
}
impl SelectType {
    pub fn with(mut self, a: Axial) -> Self {
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
    EndTurn((moves::ActualMove, move_build::CombinedEffect)),
    Deselect,
    Select(T),
}

pub async fn reselect_loop(
    doop: &mut WorkerManager,
    mut game: GameState,
    team: ActiveTeam,
    have_moved: &mut Option<selection::HaveMoved>,
    selected_unit: SelectType,
) -> (GameState, LoopRes<SelectType>) {
    console_dbg!(have_moved.is_some());
    //At this point we know a friendly unit is currently selected.

    //let mut relative_game_view = game.view_mut(selected_unit.team);

    let unwrapped_selected_unit = selected_unit.warrior;

    let unit = game
        .factions
        .relative(selected_unit.team)
        .this_team
        .find_slow(&unwrapped_selected_unit)
        .unwrap()
        .clone();

    let grey = if selected_unit.team == team {
        //If we are in the middle of a extra attack move, make sure
        //no other friendly unit is selectable until we finish moving the
        //the unit that has been partially moved.
        if let Some(e) = have_moved {
            e.the_move.moveto != selected_unit.warrior
        } else {
            false
        }
    } else {
        true
    };

    let cca = if let Some(have_moved) = have_moved {
        (selected_unit.warrior == have_moved.the_move.moveto).then(|| {
            game.generate_possible_moves_extra(&have_moved.the_move, unit.typ, selected_unit.team)
        })
    } else {
        None
    };

    let cca = cca.unwrap_or_else(|| {
        game.generate_possible_moves_movement(&unit.position, unit.typ, selected_unit.team)
    });

    //let cc = relative_game_view.get_unit_possible_moves(&unit, extra_attack);
    let cc = CellSelection::MoveSelection(unwrapped_selected_unit, cca, have_moved.clone());

    let (cell, pototo, gg) = doop
        .get_mouse_selection(cc, selected_unit.team, game, grey)
        .await;
    game = gg;

    let mouse_world = match pototo {
        Pototo::Normal(t) => t,
        Pototo::EndTurn => {
            //End the turn. Ok because we are not int he middle of anything.
            //return LoopRes::EndTurn;
            //unreachable!();
            return (game, LoopRes::Deselect);
        }
    };
    let target_cell = mouse_world;

    //This is the cell the user selected from the pool of available moves for the unit
    let CellSelection::MoveSelection(_, ss, _) = cell else {
        unreachable!()
    };

    //If we just clicked on ourselves, just deselect.
    if target_cell == unwrapped_selected_unit {
        return (game, LoopRes::Deselect);
    }

    let contains = ss.is_set(target_cell.sub(&unwrapped_selected_unit));

    //If we select a friendly unit quick swap
    if let Some(target) = game
        .factions
        .relative(selected_unit.team)
        .this_team
        .find_slow(&target_cell)
    {
        let tt = target.position;
        if !contains {
            //it should be impossible for a unit to move onto a friendly
            //assert!(!contains);
            return (game, LoopRes::Select(selected_unit.with(tt)));
        }
    }

    //If we select an enemy unit quick swap
    if let Some(target) = game
        .factions
        .relative(selected_unit.team)
        .that_team
        .find_slow(&target_cell)
    {
        let tt = target.position;
        if selected_unit.team != team || !contains {
            //If we select an enemy unit thats outside of our units range.
            return (game, LoopRes::Select(selected_unit.with(tt).not()));
        }
    }

    //If we selected an empty space, deselect.
    if !contains {
        return (game, LoopRes::Deselect);
    }

    //If we are trying to move an enemy piece, deselect.
    if selected_unit.team != team {
        return (game, LoopRes::Deselect);
    }

    // If we are trying to move a piece while in the middle of another
    // piece move, deselect.
    if let Some(e) = have_moved {
        if unwrapped_selected_unit != e.the_move.moveto {
            return (game, LoopRes::Deselect);
        }
    }

    //At this point all re-selecting of units based off of the input has occured.
    //We definately want to act on the action the user took on the selected unit.

    {
        if let Some(e) = have_moved.take() {
            let meta = e
                .the_move
                .clone()
                .into_attack(target_cell)
                .animate(selected_unit.team, &mut game, doop)
                .await
                .apply(selected_unit.team, &mut game);

            let effect = e.effect.combine(meta);

            (
                game,
                LoopRes::EndTurn((
                    moves::ActualMove {
                        original: e.the_move.original,
                        moveto: e.the_move.moveto,
                        attackto: target_cell,
                    },
                    effect,
                )),
            )
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

            let mp = move_build::MovePhase {
                original: p,
                moveto: target_cell,
            };

            let effect = mp
                .animate(selected_unit.team, &mut game, doop)
                .await
                .apply(selected_unit.team, &mut game);

            {
                *have_moved = Some(selection::HaveMoved {
                    the_move: mp,
                    effect,
                });
                (game, LoopRes::Select(selected_unit.with(c).with_team(team)))
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
            position: Axial::from_arr(a),
            typ: Type::Warrior { powerup },
            has_powerup: false,
        })
        .collect();

    //player
    let dogs = [[d, -d], [-d, 0], [0, d]];
    let dogs = dogs
        .into_iter()
        .map(|a| UnitData {
            position: Axial::from_arr(a),
            typ: Type::Warrior { powerup },
            has_powerup: false,
        })
        .collect();

    let powerups = vec![]; //vec![[1, 1], [1, -2], [-2, 1]];

    let world = Box::leak(Box::new(board::MyWorld::new()));

    let fog = world.get_game_cells().clone();
    //let fog = BitField::new();

    let mut k = GameState {
        factions: Factions {
            dogs: Tribe { units: dogs },
            cats: Tribe { units: cats },
        },
        env: Environment {
            land: BitField::from_iter([]),
            forest: BitField::from_iter([]),
            fog,
            powerups: powerups.into_iter().map(Axial::from_arr).collect(),
        },
        world,
    };

    for a in k.factions.cats.iter().chain(k.factions.dogs.iter()) {
        move_build::compute_fog(a.position, &mut k.env).apply(a.position, &mut k.env);
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

pub async fn main_logic(mut game: GameState, mut doop: WorkerManager) {
    let mut game_history = selection::MoveLog::new();

    //Loop over each team!
    'game_loop: for team in ActiveTeam::Dogs.iter() {
        if let Some(g) = game.game_is_over(team) {
            console_dbg!("Game over=", g);
            break 'game_loop;
        }

        //Add AIIIIII.
        if team == ActiveTeam::Cats {
            //{
            game = doop.send_popup("AI Thinking", team, game).await;
            let the_move = ai::iterative_deepening(&mut game, team);
            game = doop.send_popup("", team, game).await;

            let kk = the_move.as_move();

            let effect_m = kk
                .animate(team, &mut game, &mut doop)
                .await
                .apply(team, &mut game);

            let effect_a = kk
                .into_attack(the_move.attackto)
                .animate(team, &mut game, &mut doop)
                .await
                .apply(team, &mut game);

            game_history.push((the_move, effect_m.combine(effect_a)));

            continue;
        }

        let (a, b);
        (game, a, b) = handle_player(game, &mut doop, team, &mut game_history).await;
        game_history.push((a, b));

        ai::absolute_evaluate(&mut game, true);
    }

    //console_dbg!(share::save(&game_history));
}

async fn handle_player(
    mut game: GameState,
    doop: &mut WorkerManager,
    team: ActiveTeam,
    move_log: &mut selection::MoveLog,
) -> (GameState, moves::ActualMove, move_build::CombinedEffect) {
    //doop.send_popup("haha", team, game.clone()).await;

    let mut extra_attack = None;
    //Keep allowing the user to select units
    loop {
        //Loop until the user clicks on a selectable unit in their team.
        let mut selected_unit = loop {
            let data;
            (data, game) = doop.get_mouse_no_selection(team, game).await;

            let cell = match data {
                Pototo::Normal(a) => a,
                Pototo::EndTurn => {
                    if extra_attack.is_none() {
                        assert!(move_log.inner.len() >= 2, "Not enough moves to undo");
                        log!("undoing turn!!!");
                        let (a, e) = move_log.inner.pop().unwrap();
                        a.as_extra().undo(&e.extra_effect, &mut game);
                        a.as_move().undo(team.not(), &e.move_effect, &mut game);

                        let (a, e) = move_log.inner.pop().unwrap();
                        a.as_extra().undo(&e.extra_effect, &mut game);
                        a.as_move().undo(team, &e.move_effect, &mut game);
                    }
                    continue;
                }
            };
            //let game = game.view_mut(team_index);

            if let Some(unit) = game.factions.relative(team).this_team.find_slow(&cell) {
                break SelectType {
                    warrior: unit.position,
                    team,
                };
            }
            if let Some(unit) = game.factions.relative(team).that_team.find_slow(&cell) {
                break SelectType {
                    warrior: unit.position,
                    team: team.not(),
                };
            }
        };

        //Keep showing the selected unit's options and keep handling the users selections
        //Until the unit is deselected.
        loop {
            let res;
            (game, res) = reselect_loop(doop, game, team, &mut extra_attack, selected_unit).await;

            let a = match res {
                LoopRes::EndTurn((a, b)) => {
                    return (game, a, b);
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
