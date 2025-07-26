pub mod ai;
pub mod board;

pub mod main_logic;
pub mod mesh;
pub mod move_build;
pub mod moves;
pub mod unit;
use board::MyWorld;
pub use hex::Axial;
use mesh::small_mesh::SmallMesh;
use move_build::NormalMoveEffect;
pub use moves::Coordinate;
use serde::Deserialize;
use serde::Serialize;
pub use unit::GameState;
pub use unit::Team;

const NUM_STACK_HEIGHTS: usize = 7;

fn get_index(height: StackHeight, team: Team) -> usize {
    let height = height.to_num();
    assert!(height >= 0 && height <= 6, "uhoh:{}", height);
    let k = (height) as usize + NUM_STACK_HEIGHTS * ((team.value() + 1) as usize);
    assert!(k < NUM_STACK_HEIGHTS * 3);
    k
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Zobrist {
    inner: Vec<[u64; NUM_STACK_HEIGHTS * 3]>,
    white_to_move: u64,
    pass: u64,
}

#[derive(Hash, Copy, Clone, PartialEq, Eq, Debug)]
pub struct Key {
    key: u64,
}

impl Key {
    pub fn from_scratch(base: &Zobrist, game: &GameState, world: &MyWorld, team: Team) -> Key {
        let mut k = Key { key: 0 };

        for index in world.get_game_cells().inner.iter_ones() {
            match game.factions.get_cell_inner(index) {
                unit::GameCell::Piece(unit::Piece {
                    height: stack_height,
                    team: t,
                    ..
                }) => {
                    k.key ^= base.inner[index][get_index(*stack_height, *t)];
                }
                unit::GameCell::Empty => {}
            }
        }

        if let Team::White = team {
            k.key ^= base.white_to_move
        }

        k
    }
    pub fn move_update(
        &mut self,
        base: &Zobrist,
        m: NormalMove,
        team: Team,
        effect: &NormalMoveEffect,
    ) {
        if let Team::White = team {
            self.key ^= base.white_to_move
        }
        if m.is_pass() {
            self.key ^= base.pass;
        } else {
            if let Some(a) = effect.destroyed_unit {
                //panic!();
                //xor out what piece was there
                self.key ^= base.inner[m.coord.0][get_index(a.height, a.team)];
            }

            //xor in the new piece
            self.key ^= base.inner[m.coord.0][get_index(m.stack, team)];
        }
    }

    pub fn move_undo(
        &mut self,
        base: &Zobrist,
        m: NormalMove,
        team: Team,
        effect: &NormalMoveEffect,
    ) {
        if m.is_pass() {
            self.key ^= base.pass;
        } else {
            //xor out the new piece
            self.key ^= base.inner[m.coord.0][get_index(m.stack, team)];

            if let Some(a) = effect.destroyed_unit {
                //xor in what piece was there
                self.key ^= base.inner[m.coord.0][get_index(a.height, a.team)];
            }
        }

        if let Team::White = team {
            self.key ^= base.white_to_move
        }
    }
}

//const FOO:Zobrist=get_zobrist();

// #[test]
// fn test_zobrist() {
//     let world = &board::MyWorld::load_from_string("bb-t-bbsrd-s----s--").unwrap();
//     let mut game = world.starting_state.clone();

//     let base = Zobrist::new();

//     let mut k = Key::from_scratch(&base, &game, world, Team::White);

//     let a = Axial::from_letter_coord('B', 2, world.radius as i8);
//     let m = Coordinate(a.to_index());
//     let m = NormalMove { coord: m };
//     let team = Team::White;
//     let effect = m.apply(team, &mut game, &SmallMesh::new(), world, None);

//     //dbg!(game.tactical.into_string(world));
//     let orig = k.clone();
//     k.move_update(&base, m.clone(), team, &effect);
//     k.move_undo(&base, m, team, &effect);

//     assert_eq!(orig, k);
//     //panic!();
// }

impl Zobrist {
    pub fn new() -> Zobrist {
        //https://www.browserling.com/tools/random-bin
        use rand_chacha::rand_core::RngCore;
        use rand_chacha::rand_core::SeedableRng;
        let mut rng = rand_chacha::ChaCha12Rng::seed_from_u64(0x42);

        let inner = (0..board::TABLE_SIZE)
            .map(|_| std::array::from_fn(|_| rng.next_u64()))
            .collect();

        Zobrist {
            inner,
            pass: rng.next_u64(),
            white_to_move: rng.next_u64(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
macro_rules! log {
    ($($tt:tt)*) => {
        gloo_console::log!(format!($($tt)*))
    };
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! log {
    ($($tt:tt)*) => {
        //println!($($tt)*)
    };
}

pub(crate) use log;

use crate::move_build::GenericMove;
use crate::move_build::LighthouseMove;
use crate::move_build::LighthouseMoveEffect;
use crate::move_build::NormalMove;
use crate::unit::StackHeight;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Slot {
    Player,
    Ai,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum GameType {
    SinglePlayer(String),
    PassPlay(String),
    AIBattle(String),
    MapEditor(String),
    Replay(String),
    Game(Slot, Slot, Team, String),
}

pub mod share {
    #[derive(Debug)]
    pub struct LoadError;

    use super::*;
    pub fn load(s: &str) -> Result<JustMoveLog, LoadError> {
        use base64::prelude::*;
        let k = BASE64_STANDARD_NO_PAD.decode(s).map_err(|_| LoadError)?;
        let k = miniz_oxide::inflate::decompress_to_vec(&k).map_err(|_| LoadError)?;
        Ok(postcard::from_bytes(&k).map_err(|_| LoadError)?)
    }
    pub fn save(game_history: &JustMoveLog) -> String {
        use base64::prelude::*;

        let k = postcard::to_allocvec(game_history).unwrap();

        let k = miniz_oxide::deflate::compress_to_vec(&k, 10);
        BASE64_STANDARD_NO_PAD.encode(k)
    }
}

//This is for saving/loading.
#[derive(Deserialize, Serialize)]
pub struct JustMoveLog {
    pub inner: Vec<GenericMove<NormalMove, LighthouseMove>>,
}


pub trait CanPass{
    fn is_pass(&self)->bool;
}


// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct HistoryOneMoveBasic{
//     pub r: GenericMove<(NormalMove, NormalMoveEffect), (LighthouseMove, LighthouseMoveEffect)>,
// }


impl CanPass for HistoryOneMove{
    fn is_pass(&self)->bool{
        match &self.r{
            GenericMove::Normal(o) => o.0.is_pass(),
            GenericMove::Lighthouse(_) => false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HistoryOneMove {
    pub r: GenericMove<(NormalMove, NormalMoveEffect), (LighthouseMove, LighthouseMoveEffect)>,
    pub fe: unit::LastSeenObjectsAllEffect,
}

//Need to keep effect so you can undo all the way to the start.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MoveHistory<T> {
    pub inner: Vec<T>,
}

impl<T> Default for MoveHistory<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> MoveHistory<T> {
    pub fn into_string(&self, world: &MyWorld) -> String {
        // use std::fmt::Write;

        // let mut s = String::new();
        // for (index, e) in self.inner.iter() {
        //     write!(s, "{:?}", world.format(&index.coord),).unwrap();

        //     if e.destroyed_unit.is_some() {
        //         write!(s, "x").unwrap();
        //     }
        //     write!(s, " ").unwrap();
        // }

        // s
        todo!();
    }

    pub fn new() -> Self {
        MoveHistory { inner: vec![] }
    }
    pub fn into_just_move(self) -> JustMoveLog {
        // JustMoveLog {
        //     inner: self.inner.into_iter().map(|a| a.0).collect(),
        // }
        todo!()
    }

    pub fn push_normal(&mut self, o: T) {
        self.inner.push(o);
    }
}
