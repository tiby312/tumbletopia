pub mod ai;
pub mod board;
pub mod grids;

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

impl JustMoveLog {
    // pub fn deserialize(buffer: Vec<u8>) -> JustMoveLog {
    //     use byteorder::{BigEndian, ReadBytesExt};
    //     use std::io::Cursor;
    //     let mut rdr = Cursor::new(buffer);
    //     let ver = rdr.read_u32::<BigEndian>().unwrap();
    //     assert_eq!(ver, 0);
    //     let num = rdr.read_u32::<BigEndian>().unwrap();

    //     let mut ret = vec![];
    //     for _ in 0..num {
    //         let vals = [
    //             rdr.read_i16::<BigEndian>().unwrap(),
    //             rdr.read_i16::<BigEndian>().unwrap(),
    //             rdr.read_i16::<BigEndian>().unwrap(),
    //             rdr.read_i16::<BigEndian>().unwrap(),
    //             rdr.read_i16::<BigEndian>().unwrap(),
    //             rdr.read_i16::<BigEndian>().unwrap(),
    //         ];

    //         ret.push(moves::ActualMove {
    //             original: Axial {
    //                 q: vals[0],
    //                 r: vals[1],
    //             },
    //             moveto: Axial {
    //                 q: vals[2],
    //                 r: vals[3],
    //             },
    //             attackto: Axial {
    //                 q: vals[4],
    //                 r: vals[5],
    //             },
    //         });
    //     }
    //     JustMoveLog { inner: ret }
    // }
    // pub fn serialize(&self) -> Vec<u8> {
    //     let o = &self.inner;
    //     use byteorder::{BigEndian, WriteBytesExt};

    //     let mut wtr = vec![];

    //     let version = 0;
    //     wtr.write_u32::<BigEndian>(version).unwrap();

    //     wtr.write_u32::<BigEndian>(o.len().try_into().unwrap())
    //         .unwrap();

    //     for a in o.iter() {
    //         wtr.write_i16::<BigEndian>(a.original.q).unwrap();
    //         wtr.write_i16::<BigEndian>(a.original.r).unwrap();
    //         wtr.write_i16::<BigEndian>(a.moveto.q).unwrap();
    //         wtr.write_i16::<BigEndian>(a.moveto.r).unwrap();
    //         wtr.write_i16::<BigEndian>(a.attackto.q).unwrap();
    //         wtr.write_i16::<BigEndian>(a.attackto.r).unwrap();
    //     }
    //     wtr
    // }
}

//Need to keep effect so you can undo all the way to the start.
pub struct MoveHistory {
    pub inner: Vec<(moves::ActualMove, move_build::MoveEffect)>,
}
// impl Default for MoveHistory {
//     fn default() -> Self {
//         Self::new()
//     }
// }

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
