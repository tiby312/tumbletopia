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

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum GameType {
    SinglePlayer(String),
    PassPlay(String),
    AIBattle(String),
    MapEditor(String),
    Replay(String),
}

pub mod share {
    #[derive(Debug)]
    pub struct LoadError;

    //use std::ptr::metadata;

    use board::MyWorld;
    use mesh::small_mesh::SmallMesh;

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

    impl unit::Map {
        pub fn load(s: &str, world: &MyWorld) -> Result<unit::Map, LoadError> {
            // use base64::prelude::*;
            // let k = BASE64_STANDARD_NO_PAD.decode(s).map_err(|_| LoadError)?;
            // let k = miniz_oxide::inflate::decompress_to_vec(&k).map_err(|_| LoadError)?;
            // Ok(postcard::from_bytes(&k).map_err(|_| LoadError)?)

            let mut water = SmallMesh::new();
            let mut forests = SmallMesh::new();
            let mut mountains = SmallMesh::new();
            let mut start1 = None;
            let mut start2 = None;

            let mut s = s.chars();

            for a in world.get_game_cells().inner.iter_ones() {
                let Some(c) = s.next() else {
                    return Err(LoadError);
                };

                match c {
                    'w' => water.inner.set(a, true),
                    'f' => forests.inner.set(a, true),
                    'm' => mountains.inner.set(a, true),
                    '1' => start1 = Some(mesh::small_mesh::inverse(a)),
                    '2' => start2 = Some(mesh::small_mesh::inverse(a)),
                    '-' => continue,
                    _ => return Err(LoadError),
                }
            }

            let Some(start1) = start1 else {
                return Err(LoadError);
            };

            let Some(start2) = start2 else {
                return Err(LoadError);
            };

            Ok(unit::Map {
                water,
                mountains,
                forests,
                start1,
                start2,
            })
        }
        pub fn save(&self, world: &MyWorld) -> Result<String, std::fmt::Error> {
            use std::fmt::Write;
            let mut s = String::new();

            for a in world.get_game_cells().inner.iter_ones() {
                if self.water.inner[a] {
                    write!(&mut s, "w")?;
                } else if self.forests.inner[a] {
                    write!(&mut s, "f")?;
                } else if self.mountains.inner[a] {
                    write!(&mut s, "m")?;
                } else if mesh::small_mesh::conv(self.start1) == a {
                    write!(&mut s, "1")?;
                } else if mesh::small_mesh::conv(self.start2) == a {
                    write!(&mut s, "2")?;
                } else {
                    write!(&mut s, "-")?;
                }
            }
            Ok(s)

            // use base64::prelude::*;

            // let k = postcard::to_allocvec(self).unwrap();

            // let k = miniz_oxide::deflate::compress_to_vec(&k, 10);
            // BASE64_STANDARD_NO_PAD.encode(k)
        }
    }
}

//This is for saving/loading.
#[derive(Deserialize, Serialize)]
pub struct JustMoveLog {
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
    pub fn into_just_move(self) -> JustMoveLog {
        JustMoveLog {
            inner: self.inner.into_iter().map(|a| a.0).collect(),
        }
    }

    pub fn push(&mut self, o: (moves::ActualMove, move_build::MoveEffect)) {
        self.inner.push(o);
    }
}
