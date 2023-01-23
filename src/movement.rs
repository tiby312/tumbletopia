use super::*;

pub trait MoveStrategy {
    type It: IntoIterator<Item = Moves>;
    fn adjacent() -> Self::It;
}
pub struct WarriorMovement;
impl MoveStrategy for WarriorMovement {
    type It = std::array::IntoIter<Moves, 8>;
    fn adjacent() -> Self::It {
        use Moves::*;
        [Up, UpLeft, Left, DownLeft, Down, DownRight, Right, UpRight].into_iter()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Moves {
    Up,
    UpLeft,
    Left,
    DownLeft,
    Down,
    DownRight,
    Right,
    UpRight,
}
impl Moves {
    pub fn to_relative(&self) -> GridCoord {
        use Moves::*;
        GridCoord(match self {
            Up => [0, 1],
            UpLeft => [-1, 1],
            Left => [-1, 0],
            DownLeft => [-1, -1],
            Down => [0, -1],
            DownRight => [1, -1],
            Right => [1, 0],
            UpRight => [1, 1],
        })
    }
}
#[derive(Copy, Clone, Debug)]
pub struct Path {
    //TODO optimize this to be just one 64bit integer?
    //20 moves is just max possible moves
    moves: [Moves; 20],
    num_moves: u8,
}
impl Path {
    pub fn new() -> Self {
        Path {
            moves: [Moves::Up; 20],
            num_moves: 0,
        }
    }
    pub fn get_moves(&self) -> &[Moves] {
        &self.moves[0..self.num_moves as usize]
    }
    pub fn add(mut self, a: Moves) -> Option<Self> {
        if self.num_moves >= 20 {
            return None;
        }

        self.moves[self.num_moves as usize] = a;
        self.num_moves += 1;
        Some(self)
    }

    pub fn get_end_coord(&self, mut start: GridCoord) -> GridCoord {
        for m in self.moves.iter().take(self.num_moves as usize) {
            start = start.add(m.to_relative());
        }
        start
    }

    fn move_cost(&self, m: Moves) -> MoveUnit {
        use Moves::*;
        match m {
            UpLeft | DownLeft | UpRight | DownRight => {
                //MoveUnit(1)

                // if self.num_moves==0 {
                //     MoveUnit(1)
                // }else{
                //     MoveUnit(2)
                // }
                MoveUnit(3)
                // let num = self
                //     .moves
                //     .iter()
                //     .take(self.num_moves as usize)
                //     .filter(|&&a| a == m)
                //     .count();

                // //if num % 3 == 0 || num % 3==1  {
                // if num!=0 && num % 2 == 0 {

                //     MoveUnit(0)
                // } else {

                //     //Technically should have the penalty all the time.
                //     //But it looks better with this so roads work on corners for warriors (1 move unit)
                //     //if num>0{
                //     MoveUnit(2) //TODO 2 better?
                //     //}else{
                //     //    MoveUnit(0)
                //     //}
                // }
            }
            _ => MoveUnit(2),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct GridCoord(pub [i16; 2]);
impl GridCoord {
    fn advance(self, m: Moves) -> GridCoord {
        self.add(m.to_relative())
    }
    fn add(mut self, o: GridCoord) -> Self {
        self.0[0] += o.0[0];
        self.0[1] += o.0[1];
        self
    }
}

#[derive(Copy, Clone)]
pub struct MoveUnit(pub i8);
impl MoveUnit {
    fn add(self, a: MoveUnit) -> Self {
        MoveUnit(self.0 + a.0)
    }
    fn sub(self, a: MoveUnit) -> Self {
        MoveUnit(self.0 - a.0)
    }
}

pub trait Filter {
    fn filter(&self, a: &GridCoord) -> bool;
    fn chain<K: Filter>(self, other: K) -> Chain<Self, K>
    where
        Self: Sized,
    {
        Chain { a: self, b: other }
    }
}
pub struct Chain<A, B> {
    a: A,
    b: B,
}
impl<A: Filter, B: Filter> Filter for Chain<A, B> {
    fn filter(&self, a: &GridCoord) -> bool {
        self.a.filter(a) && self.b.filter(a)
    }
}

pub fn contains_coord<'a, I: Iterator<Item = &'a GridCoord>>(mut it: I, b: &GridCoord) -> bool {
    it.find(|a| *a == b).is_some()
}

//Represents all the legal moves for a specific piece.
pub struct PossibleMoves {
    //Has the end coord,path from current, and the remainder cost to get there.
    //cells that are the furthest away will have a move unit of zero.
    moves: Vec<(GridCoord, Path, MoveUnit)>,
    start: GridCoord,
}

impl PossibleMoves {
    pub fn new<K: MoveStrategy, F: Filter, M: MoveCost>(
        movement: &K,
        filter: &F,
        mo: &M,
        coord: GridCoord,
        remaining_moves: MoveUnit,
    ) -> Self {
        //A typical move costs 2, so scale everything as if it cost 1.
        let remaining_moves = MoveUnit(remaining_moves.0 * 2);
        let mut p = PossibleMoves {
            moves: vec![],
            start: coord,
        };
        p.explore_path(movement, filter, mo, Path::new(), remaining_moves);
        p
    }

    pub fn get_path(&self, g: GridCoord) -> Option<&Path> {
        self.moves.iter().find(|a| a.0 == g).map(|a| &a.1)
    }

    pub fn start(&self) -> &GridCoord {
        &self.start
    }

    pub fn iter_coords(&self) -> impl Iterator<Item = &GridCoord> {
        self.moves.iter().map(|a| &a.0)
    }

    fn explore_path<K: MoveStrategy, F: Filter, M: MoveCost>(
        &mut self,
        movement: &K,
        filter: &F,
        mo: &M,
        current_path: Path,
        mut remaining_moves: MoveUnit,
    ) {
        if remaining_moves.0 == 0 {
            return;
        }

        // 2-OG
        // warrior has 2 move points
        // warrior moves to grass and expends its 2 move points
        // warrior cant move anymore

        // 2-ORG
        // warrior has 2 move points
        // warrior moves to road on grass and expends 1 move point (2-1)
        // warrior has 1 move point.
        // warrior moves to grass and expends 2 move points.
        // warrior has -1 move points. can't move anymore.

        // 2-ORRG
        // warrior has 2 move points
        // warrior moves to road on grass and expends 1 move point (2-1)
        // warrior has 1 move point
        // warrior moves to road on grass and expends 1 move point?????
        // warrior has 0 move points. cant move anymore.

        let curr_pos = current_path.get_end_coord(self.start);

        //log!(format!("rem:{:?}",remaining_moves.0));
        for a in K::adjacent() {
            let target_pos = curr_pos.advance(a);

            if !filter.filter(&target_pos) {
                continue;
            }

            //We must have remaining moves to satisfy ALL move cost.
            // if remaining_moves.0<current_path.move_cost(a).0{
            //     continue;
            // }

            let move_cost = current_path.move_cost(a);
            // if move_cost.0>remaining_moves.0{
            //     move_cost.0=remaining_moves.0;
            // }
            //TODO road should HALF the cost?
            let cost = mo.foop(target_pos, move_cost);

            //as long as we have SOME remainv moves, we can go to this square even
            //if it is really expensive.
            if !(remaining_moves.0 > 0) {
                continue;
            }

            //subtract move cost
            let rr = remaining_moves.sub(cost);

            if !self.consider(&current_path, a, rr) {
                continue;
            }

            self.explore_path(movement, filter, mo, current_path.add(a).unwrap(), rr)
        }
    }

    fn consider(&mut self, path: &Path, m: Moves, cost: MoveUnit) -> bool {
        //if this move unit is greater than what we already have, replace it.
        //we found a quicker way to get to the same square.

        //if it is not quicker, imediately stop everything.
        let new_path = path.add(m).unwrap();
        let coord = new_path.get_end_coord(self.start);

        //we found a match now lets compare
        let index =
            if let Some((index, _)) = self.moves.iter().enumerate().find(|(_, a)| a.0 == coord) {
                index
            } else {
                self.moves.push((coord, new_path, cost));
                return true;
            };

        if cost.0 > self.moves[index].2 .0 {
            self.moves.push((coord, new_path, cost));
            self.moves.swap_remove(index);
            return true;
        }

        return false;
    }
}

// //normal terrain is 2.
// //road is 1.
// fn terrain_cost(a: GridCoord) -> MoveUnit {
//     MoveUnit(2)
// }
