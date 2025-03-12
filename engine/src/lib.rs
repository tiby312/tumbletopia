pub mod ai;
pub mod board;

pub mod main_logic;
pub mod mesh;
pub mod move_build;
pub mod moves;
pub mod unit;
use board::MyWorld;
pub use hex::Axial;
use move_build::MoveEffect;
pub use moves::ActualMove;
use serde::Deserialize;
use serde::Serialize;
pub use unit::GameState;
pub use unit::Team;

fn get_index(height: u8, team: Team) -> usize {
    assert!(height > 0 && height <= 6);
    let k = (height - 1) as usize + 6 * ((team.value() + 1) as usize);
    assert!(k < 6 * 3);
    k
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Zobrist {
    inner: Vec<[u64; 6 * 3]>,
}

#[derive(Hash, Copy, Clone, PartialEq, Eq, Debug)]
pub struct Key {
    key: u64,
}

impl Key {
    pub fn from_scratch(base: &Zobrist, game: &GameState, world: &MyWorld) -> Key {
        let mut k = Key { key: 0 };

        for index in world.get_game_cells().inner.iter_ones() {
            if let Some((h, t)) = game.factions.get_cell_inner(index) {
                k.key ^= base.inner[index][get_index(h, t)];
            }
        }
        k
    }
    pub fn move_update(&mut self, base: &Zobrist, m: ActualMove, team: Team, effect: &MoveEffect) {
        if let Some(a) = effect.destroyed_unit {
            //panic!();
            //xor out what piece was there
            self.key ^= base.inner[m.moveto][get_index(a.0, a.1)];
        }

        //xor in the new piece
        self.key ^= base.inner[m.moveto][get_index(effect.height, team)];
    }

    pub fn move_undo(&mut self, base: &Zobrist, m: ActualMove, team: Team, effect: &MoveEffect) {
        //xor out the new piece
        self.key ^= base.inner[m.moveto][get_index(effect.height, team)];

        if let Some(a) = effect.destroyed_unit {
            //xor in what piece was there
            self.key ^= base.inner[m.moveto][get_index(a.0, a.1)];
        }
    }
}

//const FOO:Zobrist=get_zobrist();

#[test]
fn test_zobrist() {
    let world = &board::MyWorld::load_from_string("bb-t-bbsrd-s----s--");
    let mut game = world.starting_state.clone();

    let base = Zobrist::new();

    let mut k = Key::from_scratch(&base, &game.tactical, world);

    let a = Axial::from_letter_coord('B', 2, world.radius as i8);
    let m = ActualMove {
        moveto: a.to_index(),
    };

    let team = Team::White;
    let effect = m.apply(team, &mut game.tactical, &game.fog[0], world, None);

    //dbg!(game.tactical.into_string(world));
    let orig = k.clone();
    k.move_update(&base, m.clone(), team, &effect);
    k.move_undo(&base, m, team, &effect);

    assert_eq!(orig, k);
    //panic!();
}

impl Zobrist {
    pub fn new() -> Zobrist {
        //https://www.browserling.com/tools/random-bin
        use rand_chacha::rand_core::RngCore;
        use rand_chacha::rand_core::SeedableRng;
        let mut rng = rand_chacha::ChaCha12Rng::seed_from_u64(0x42);

        let inner = (0..board::TABLE_SIZE)
            .map(|_| std::array::from_fn(|i| rng.next_u64()))
            .collect();

        Zobrist { inner }
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
    pub inner: Vec<moves::ActualMove>,
}

//Need to keep effect so you can undo all the way to the start.
#[derive(Serialize, Deserialize, Debug, Clone)]
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
