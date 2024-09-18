pub mod ai;
pub mod board;

pub mod main_logic;
pub mod mesh;
pub mod move_build;
pub mod moves;
pub mod unit;
pub use hex::Axial;
pub use moves::ActualMove;
use serde::Deserialize;
use serde::Serialize;
pub use unit::ActiveTeam;
pub use unit::GameState;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameType {
    SinglePlayer,
    PassPlay,
    AIBattle,
    Replay(String),
}

pub mod share {

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
    pub seed: board::WorldSeed,
    pub inner: Vec<moves::ActualMove>,
}


//Need to keep effect so you can undo all the way to the start.
pub struct MoveHistory {
    pub inner: Vec<(moves::ActualMove, move_build::MoveEffect)>,
}


impl Default for MoveHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveHistory {
    pub fn new() -> Self {
        MoveHistory { inner: vec![] }
    }
    pub fn into_just_move(self, seed: board::WorldSeed) -> JustMoveLog {
        JustMoveLog {
            seed,
            inner: self.inner.into_iter().map(|a| a.0).collect(),
        }
    }

    pub fn push(&mut self, o: (moves::ActualMove, move_build::MoveEffect)) {
        self.inner.push(o);
    }
}
