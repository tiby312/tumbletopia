use board::MyWorld;
use mesh::small_mesh::SmallMesh;

use super::*;

//Keeps track of the last seen objects in darkness
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LastSeenObjects {
    pub state: GameState,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct CellDiff {
    old: GameCell<Piece>,
    pos: Coordinate,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LastSeenObjectsAll {
    pub fog: [LastSeenObjects; 2],
}

impl LastSeenObjectsAll {
    pub fn new(a: &GameState) -> Self {
        LastSeenObjectsAll {
            fog: std::array::from_fn(|_| LastSeenObjects { state: a.clone() }),
        }
    }
    pub fn apply(
        &mut self,
        game_after: &GameState,
        m: (&NormalMove, &NormalMoveEffect),
        world: &MyWorld,
    ) -> LastSeenObjectsAllEffect {
        let a = self.fog[0].apply(game_after, world, m, Team::White);
        let b = self.fog[1].apply(game_after, world, m, Team::Black);
        LastSeenObjectsAllEffect { diff: [a, b] }
    }
    pub fn undo(&mut self, ll: &LastSeenObjectsAllEffect) {
        self.fog[0].undo(&ll.diff[0]);
        self.fog[1].undo(&ll.diff[0]);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LastSeenObjectsAllEffect {
    diff: [LastSeenObjectsEffect; 2],
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LastSeenObjectsEffect {
    diff: Vec<CellDiff>,
}
impl LastSeenObjectsAllEffect {
    pub fn dummy() -> Self {
        Self {
            diff: std::array::from_fn(|_| LastSeenObjectsEffect { diff: vec![] }),
        }
    }
}

impl LastSeenObjects {
    pub fn undo(&mut self, ll: &LastSeenObjectsEffect) {
        for a in &ll.diff {
            self.state.factions.cells[a.pos.0] = a.old.clone();
        }
    }

    pub fn apply(
        &mut self,
        game_after: &GameState,
        world: &MyWorld,
        m: (&NormalMove, &NormalMoveEffect),
        team: Team,
    ) -> LastSeenObjectsEffect {
        //if we are adding a piece,
        //check if we can see it. If we can't, don't update last seen.

        
        let mut handle_this = vec![];
        let j = m.0.coord.0;
        if let Some(p) = m.1.captured_unit(&m.0, game_after) {
            if p.team == team {
                let r = if p.has_lighthouse {
                    LIGHTHOUSE_RANGE
                } else {
                    NORMAL_RANGE
                };

                for a in Axial::from_index(&j).to_cube().range(r) {
                    let x = a.ax.to_index();

                    match game_after.factions.cells[x] {
                        GameCell::Piece(p) => {
                            if p.team != team {
                                handle_this.push(x);
                            }
                        }
                        GameCell::Empty => {}
                    }
                }
            }
        }

        let mut diffs = vec![];

        for j in handle_this {
            diffs.push(CellDiff {
                old: self.state.factions.cells[j].clone(),
                pos: Coordinate(j),
            });
            self.state
                .factions
                .copy_cell_if_occupied(&game_after.factions, j);
            
        }

        LastSeenObjectsEffect { diff: diffs }
    }
}


#[must_use]
#[derive(Serialize, Deserialize, Hash, Ord, PartialOrd, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Team {
    White = 0,
    Black = 1,
    Neutral = 2,
}

impl std::ops::Not for Team {
    type Output = Team;

    fn not(self) -> Self::Output {
        Team::not(&self)
    }
}

impl std::ops::Neg for Team {
    type Output = Team;

    fn neg(self) -> Self::Output {
        self.not()
    }
}

impl std::ops::SubAssign<unit::Team> for i64 {
    fn sub_assign(&mut self, rhs: unit::Team) {
        *self -= rhs.value();
    }
}
impl std::ops::AddAssign<unit::Team> for i64 {
    fn add_assign(&mut self, rhs: unit::Team) {
        *self += rhs.value();
    }
}
impl std::ops::Add<unit::Team> for i64 {
    type Output = i64;

    fn add(self, rhs: unit::Team) -> Self::Output {
        self + rhs.value()
    }
}

impl std::ops::Sub<unit::Team> for i64 {
    type Output = i64;

    fn sub(self, rhs: unit::Team) -> Self::Output {
        self - rhs.value()
    }
}

impl<T> std::ops::IndexMut<Team> for [T] {
    fn index_mut(&mut self, index: Team) -> &mut Self::Output {
        match index {
            Team::White => &mut self[0],
            Team::Black => &mut self[1],
            _ => {
                unreachable!()
            }
        }
    }
}
impl<T> std::ops::Index<Team> for [T] {
    type Output = T;
    fn index(&self, idx: Team) -> &Self::Output {
        match idx {
            Team::White => &self[0],
            Team::Black => &self[1],
            _ => {
                unreachable!()
            }
        }
    }
}

impl Team {
    pub fn value(&self) -> i64 {
        match self {
            Team::White => 1,
            Team::Black => -1,
            Team::Neutral => 0,
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Team::White => 0,
            Team::Black => 1,
            Team::Neutral => unreachable!(),
        }
    }
    pub fn is_white(&self) -> bool {
        if let Team::White = self { true } else { false }
    }
    pub fn iter(&self) -> impl Iterator<Item = Self> {
        [*self, self.not()].into_iter().cycle()
    }
    pub fn not(&self) -> Self {
        match self {
            Team::White => Team::Black,
            Team::Black => Team::White,
            Team::Neutral => Team::Neutral,
        }
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]

pub struct GameStateTotal {
    //0 is white fog. 1 is black fog
    pub last_seen: LastSeenObjectsAll,
    pub tactical: GameState,
    pub history: MoveHistory<HistoryOneMove>,
}

//Additionally removes need to special case animation.
#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct GameState {
    pub factions: Tribe,
}

#[must_use]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameOver {
    WhiteWon,
    BlackWon,
    Tie,
}

pub const LIGHTHOUSE_RANGE: i8 = 2;
pub const NORMAL_RANGE: i8 = 1;

impl GameState {
    pub fn convert_to_playable(&self, world: &MyWorld, team_perspective: Team) -> GameState {
        let d = self.darkness(world, team_perspective);

        let mut gg = self.clone();
        for a in d.iter_mesh(Axial::zero()) {
            gg.factions.remove(a);
            gg.factions
                .add_cell(a, StackHeight::Stack6, Team::Neutral, true);
        }
        gg
    }

    pub fn darkness(&self, world: &MyWorld, team_perspective: Team) -> SmallMesh {
        let mut darkness = world.land.clone();
        for a in world.land.inner.iter_ones() {
            match self.factions.get_cell_inner(a) {
                &GameCell::Piece(p) => {
                    if p.team == team_perspective {
                        let r = if p.has_lighthouse {
                            LIGHTHOUSE_RANGE
                        } else {
                            NORMAL_RANGE
                        };
                        for j in Axial::from_index(&a).to_cube().range(r) {
                            darkness.set_coord(j.ax, false);
                        }
                    }
                }
                GameCell::Empty => {}
            }
        }

        darkness
    }

    pub fn new() -> GameState {
        GameState {
            factions: Tribe::new(),
        }
    }
    
    pub fn hash_me(&self) -> u64 {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::hash::DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
    pub fn game_is_over(
        &self,
        world: &board::MyWorld,
        _team: Team,
        history: &MoveHistory<HistoryOneMove>,
    ) -> Option<GameOver> {
        let score_data = self.score(world);

        let (a, b) = match &history.inner[..] {
            [.., a, b] => (a, b),
            _ => return None,
        };

        //if let (GenericMove::Normal(a), GenericMove::Normal(b)) = (&a.r, &b.r) {
        if a.r.0.is_pass() && b.r.0.is_pass() {
            //return None;
            if score_data.white > score_data.black {
                return Some(GameOver::WhiteWon);
            } else if score_data.white < score_data.black {
                return Some(GameOver::BlackWon);
            } else {
                return Some(GameOver::Tie);
            }
        }
        //}

        None
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ScoreData {
    pub white: usize,
    pub black: usize,
    pub neutral: usize,
}
impl GameState {
    pub fn score(&self, world: &MyWorld) -> ScoreData {
        //let total_num = world.get_game_cells().inner.count_ones();
        let mut neutral = 0;
        let game = self;
        let mut white_score = 0;
        let mut black_score = 0;
        for index in world.get_game_cells().inner.iter_ones() {
            let mut num_white = 0;
            let mut num_black = 0;
            for (_, rest) in game.factions.iter_end_points(world, index) {
                if let Some(e) = rest {
                    match e.piece.team {
                        Team::White => num_white += 1,
                        Team::Black => num_black += 1,
                        Team::Neutral => {}
                    }
                }
            }

            match game.factions.get_cell_inner(index) {
                GameCell::Piece(Piece { team: tt, .. }) => {
                    //let height = height as i8;
                    match tt {
                        Team::White => {
                            white_score += 1;
                            // if num_black >= height {
                            //     black_score += 1
                            // }
                        }
                        Team::Black => {
                            black_score += 1;
                            // if num_white >= height {
                            //     white_score += 1;
                            // }
                        }
                        Team::Neutral => {
                            neutral += 1;
                        }
                    }
                }
                GameCell::Empty => {
                    let ownership = num_white - num_black;

                    if ownership > 0 {
                        white_score += 1;
                    } else if ownership < 0 {
                        black_score += 1;
                    } else {
                        neutral += 1;
                    }
                }
            }
        }
        ScoreData {
            white: white_score,
            black: black_score,
            neutral,
        }
    }

}

#[derive(
    Default, Debug, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd, Hash, Copy, Clone,
)]
pub enum StackHeight {
    Stack0 = 0,
    #[default]
    Stack1 = 1,
    Stack2 = 2,
    Stack3 = 3,
    Stack4 = 4,
    Stack5 = 5,
    Stack6 = 6,
}
impl StackHeight {
    pub fn to_num(&self) -> i8 {
        *self as i8
    }
    pub fn from_num(num: i8) -> StackHeight {
        use StackHeight::*;
        match num {
            0 => Stack0,
            1 => Stack1,
            2 => Stack2,
            3 => Stack3,
            4 => Stack4,
            5 => Stack5,
            6 => Stack6,
            _ => unreachable!("Not a valid stack height:{}", num),
        }
    }
}

#[derive(PartialOrd, Ord, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Clone)]

pub struct Piece {
    pub height: StackHeight,
    pub team: Team,
    pub has_lighthouse: bool,
}

#[derive(Hash, Deserialize, Serialize, PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord)]

pub enum PieceType {
    Normal,
    Lighthouse,
}

#[derive(PartialOrd, Ord, Debug, Serialize, Deserialize, Default, Eq, PartialEq, Hash, Clone)]

pub enum GameCell<T> {
    Piece(T),
    #[default]
    Empty,
}

#[derive(Debug, Serialize, Deserialize, Default, Eq, PartialEq, Hash, Clone)]
pub struct Tribe {
    pub cells: Vec<GameCell<Piece>>,
}

pub fn ray(
    start: Axial,
    dd: hex::HDir,
    world: &board::MyWorld,
) -> (i8, impl Iterator<Item = isize> + use<'_>) {
    let stride = board::STRIDES[dd as usize] as isize;
    let dis = board::dis_to_hex_of_hexagon(start, dd, world.radius as i8);
    let mut index2 = start.to_index() as isize;

    debug_assert!(
        world.get_game_cells().inner[index2 as usize],
        "uhoh {:?}",
        world.format(&start)
    );
    (
        dis,
        (1..dis).map(move |_d| {
            index2 += stride;
            //assert!(index2 > 0);

            debug_assert!(
                world.get_game_cells().inner[index2 as usize],
                // "fail {}:{}:{:?}:{:?}",
                // d,
                // dis,
                // dd,
                // start.to_letter_coord(world.radius as i8)
            );
            index2
        }),
    )
}

#[derive(Debug, PartialEq, Eq)]
pub struct EndPoint {
    pub index: usize,
    pub piece: Piece,
}

impl Tribe {
    pub fn filter_los(&self, index: usize, world: &MyWorld) -> SmallMesh {
        let mut s = SmallMesh::new();

        for i in 0..6 {
            let dd = hex::HDir::from(i as u8);

            let stride = board::STRIDES[i] as isize;
            let dis =
                board::dis_to_hex_of_hexagon(Axial::from_index(&index), dd, world.radius as i8);
            let mut index2 = index as isize;

            for _ in 0..dis {
                index2 += stride;

                s.inner.set(index2 as usize, true);

                match self.get_cell_inner(index2 as usize) {
                    GameCell::Piece(_) => break,
                    GameCell::Empty => {}
                }
            }
        }

        s
    }
    pub fn iter_end_points<'a>(
        &'a self,
        world: &'a board::MyWorld,
        index: usize,
    ) -> impl Iterator<Item = (i8, Option<EndPoint>)> + use<'a> {
        assert!(
            world.get_game_cells().inner[index as usize],
            "uhoh {:?}",
            world.format(&Coordinate(index))
        );
        hex::HDir::all().map(move |dd| {
            let (dis, it) = ray(Axial::from_index(&index), dd, world);
            for (d, index2) in it.enumerate() {
                match self.get_cell_inner(index2 as usize) {
                    &GameCell::Piece(piece) => {
                        return (
                            d as i8 + 1,
                            Some(EndPoint {
                                index: index2 as usize,
                                piece,
                            }),
                        );
                    }
                    GameCell::Empty => {}
                }
            }

            (dis, None)
        })

        // core::array::from_fn(|i| {

        // })
    }

    pub fn new() -> Tribe {
        let cells: Vec<_> = (0..board::TABLE_SIZE).map(|_| GameCell::Empty).collect();
        assert_eq!(cells.len(), board::TABLE_SIZE);
        Tribe { cells }
    }

    pub fn remove(&mut self, a: Axial) {
        let a = a.to_index();
        self.remove_inner(a);
    }
    pub fn remove_inner(&mut self, a: usize) {
        self.cells[a] = GameCell::Empty;
    }
    
    pub fn copy_cell_if_occupied(&mut self, other: &Tribe, index: usize) {
        assert!(index != hex::PASS_MOVE_INDEX);

        match other.cells[index] {
            GameCell::Piece(o) => {
                self.cells[index] = GameCell::Piece(o);
            }
            GameCell::Empty => {}
        }
    }

    pub fn get_cell_inner(&self, index: usize) -> &GameCell<Piece> {
        assert!(index != hex::PASS_MOVE_INDEX);
        &self.cells[index]
        
    }
    pub fn get_cell(&self, a: Axial) -> &GameCell<Piece> {
        self.get_cell_inner(a.to_index())
    }


    pub fn add_cell_inner(
        &mut self,
        a: usize,
        stack: StackHeight,
        team: Team,
        has_lighthouse: bool,
    ) {
        self.cells[a] = GameCell::Piece(Piece {
            team,
            height: stack,
            has_lighthouse,
        });
        
    }
    pub fn add_cell(&mut self, a: Axial, stack: StackHeight, team: Team, has_lighthouse: bool) {
        let a = a.to_index();
        self.add_cell_inner(a, stack, team, has_lighthouse);
    }
}

#[derive(PartialOrd, Ord, Eq, PartialEq, Copy, Clone, Debug)]
pub enum UnitType {
    Mouse,
    Rabbit,
}

//need 8 layers of map
//each map needs

// pub fn parse_replay_string(s: &str, world: &MyWorld) -> Option<(Map, MoveHistory)> {
//     let mut s = s.split(":");

//     let map = s.next()?;

//     let Ok(map) = Map::load(&map, world) else {
//         return None;
//     };

//     let moves = s.next()?;

//     let (mut g, start_team) = GameStateTotal::new(world, &map);
//     let mut mh = MoveHistory::new();
//     for (f, team) in moves.split_terminator(',').zip(start_team.iter()) {
//         let m = ActualMove::from_str(f)?;

//         let effect = m.apply(team, &mut g.tactical, world);
//         g.update_fog(world, team);
//         mh.inner.push((m, effect));
//     }

//     Some((map, mh))
// }

pub fn replay_string(
    _moves: &MoveHistory<HistoryOneMove>,
    _world: &MyWorld,
) -> Result<String, std::fmt::Error> {
    let s = String::new();

    //let map_str = map.save(world).unwrap();
    //TODO update this!!!
    //todo!();
    // write!(&mut s, "{}:", map_str)?;

    // for m in moves.inner.iter() {
    //     let mut kk = String::new();
    //     m.0.as_text(&mut kk).unwrap();
    //     write!(&mut s, "{},", kk)?;
    // }

    Ok(s)
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Map {
    pub ice: SmallMesh,
    pub water: SmallMesh,
    pub forests: SmallMesh,
    pub white: SmallMesh,
    pub black: SmallMesh,
}

impl unit::Map {
    // pub fn load(s: &str, world: &MyWorld) -> Result<unit::Map, ()> {
    //     // use base64::prelude::*;
    //     // let k = BASE64_STANDARD_NO_PAD.decode(s).map_err(|_| LoadError)?;
    //     // let k = miniz_oxide::inflate::decompress_to_vec(&k).map_err(|_| LoadError)?;
    //     // Ok(postcard::from_bytes(&k).map_err(|_| LoadError)?)

    //     let mut ice = SmallMesh::new();
    //     let mut forests = SmallMesh::new();
    //     let mut mountains = SmallMesh::new();
    //     let mut white = SmallMesh::new();
    //     let mut black = SmallMesh::new();

    //     let mut s = s.chars();

    //     for a in world.get_game_cells().inner.iter_ones() {
    //         let Some(c) = s.next() else {
    //             return Err(());
    //         };

    //         match c {
    //             'i' => ice.inner.set(a, true),
    //             'f' => forests.inner.set(a, true),
    //             'w' => mountains.inner.set(a, true),
    //             '1' => white.inner.set(a, true),
    //             '2' => black.inner.set(a, true),
    //             '-' => continue,
    //             _ => return Err(()),
    //         }
    //     }

    //     Ok(unit::Map {
    //         ice,
    //         water: mountains,
    //         forests,
    //         white,
    //         black,
    //     })
    // }
    // pub fn save(&self, world: &MyWorld) -> Result<String, std::fmt::Error> {
    //     use std::fmt::Write;
    //     let mut s = String::new();

    //     //write!(&mut s,"m{}=[",world.radius)?;
    //     for a in world.get_game_cells().inner.iter_ones() {
    //         if self.ice.inner[a] {
    //             write!(&mut s, "i")?;
    //         } else if self.forests.inner[a] {
    //             write!(&mut s, "f")?;
    //         } else if self.water.inner[a] {
    //             write!(&mut s, "w")?;
    //         } else if self.white.inner[a] {
    //             write!(&mut s, "1")?;
    //         } else if self.black.inner[a] {
    //             write!(&mut s, "2")?;
    //         } else {
    //             write!(&mut s, "-")?;
    //         }
    //     }
    //     //write!(&mut s,"]")?;
    //     Ok(s)

    //     // use base64::prelude::*;

    //     // let k = postcard::to_allocvec(self).unwrap();

    //     // let k = miniz_oxide::deflate::compress_to_vec(&k, 10);
    //     // BASE64_STANDARD_NO_PAD.encode(k)
    // }
}

impl Map {
    // pub fn from_game_state(game: &GameState, world: &board::MyWorld) -> Option<Map> {
    //     let water = game.factions.ice.clone();

    //     let mut white = SmallMesh::new();
    //     let mut black = SmallMesh::new();
    //     let mut mountains = SmallMesh::new();
    //     let mut forests = SmallMesh::new();

    //     for a in world.get_game_cells().inner.iter_ones() {
    //         if let Some((height, team)) = game.factions.get_cell_inner(a) {
    //             match team {
    //                 Team::White => {
    //                     white.inner.set(a, true);
    //                 }
    //                 Team::Black => {
    //                     black.inner.set(a, true);
    //                 }
    //                 Team::Neutral => {
    //                     if height == 6 {
    //                         mountains.inner.set(a, true);
    //                     } else if height == 1 {
    //                         forests.inner.set(a, true);
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     Some(Map {
    //         ice: water,
    //         water: mountains,
    //         forests,
    //         white,
    //         black,
    //     })
    // }
}

// pub fn default_map(world: &board::MyWorld) -> Map {
//     //Map::load("----------w-------ww--m----------w------m----m------2m--ww--m-w------------ww-------f-----ww--wwm---m-m-w---ww--m---m1------w---m-----mmw------m--www-m-mm----w----------",world).unwrap()

//     //Map::load("----------if------iiffw----------i------w----w----i-2w--ii--w-i-------f----ii------f------ii--iiw-ffw-w-i---ii--w---w1------i---w-----wwi-----1w--iii-w-ww---fi-2-----ff-",world).unwrap()

//     //Map::load("wwwwwwwww-if--wwwwiiffw-wwwww----iw-www-w----i--2ww-2w--iiw-w-ww------f----wwwwff--f-----wwwwfiiw-ffw-w-www-ii--w---w1ww----i---w-ww1-wwi----www--iii-wwww---fiwwwwwwwwww",world).unwrap()
//     //Map::load("wwwwwwwww-i---iiwwii--i-iiwwi----ii-iww-i----i--2ww-2i--iii-i-ww-----------iwwi-----f----iiww-iii---i-i-iww-ii--i---i1ww----i---i-ww1-iii----wwi--iii--wwi----i-wwwwwwwww",world).unwrap()
//     //Map::load("----------i1--ii--ii--i-ii--i--2-ii-i", world).unwrap()
//     // let mut mountains = SmallMesh::new();
//     // let mut water = SmallMesh::new();
//     // let mut forests = SmallMesh::new();

//     // let mountains2 = [
//     //     [1, -3],
//     //     [1, 1],
//     //     [-5, 3],
//     //     [2, -1],
//     //     [-3, 3],
//     //     [-4, -2],
//     //     [-3, -2],
//     //     [-2, -2],
//     // ];

//     // for a in mountains2 {
//     //     mountains.add(Axial::from_arr(a));
//     // }

//     // let water2 = [[-2, 2], [-2, 1], [-4, 3], [3, -2], [4, -2], [5, -3]];

//     // for a in water2 {
//     //     water.add(Axial::from_arr(a));
//     // }

//     // forests.add(Axial::from_arr([0, 0]));

//     // let start1 = Axial { q: -1, r: 2 };
//     // let start2 = Axial { q: 0, r: -5 };

//     // Map {
//     //     water,
//     //     mountains,
//     //     forests,
//     //     start1,
//     //     start2,
//     // }
// }


