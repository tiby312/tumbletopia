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
    fn to_relative(&self) -> GridCoord {
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
                let num = self
                    .moves
                    .iter()
                    .take(self.num_moves as usize)
                    .filter(|&&a| a == m)
                    .count();
                //if num % 3 == 0 || num % 3==1  {
                if num%2 ==0{
                    MoveUnit(1)
                } else {
                    MoveUnit(2)
                }
            }
            _ => MoveUnit(1),
        }
        //MoveUnit(1)
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
pub struct MoveUnit(pub usize);
impl MoveUnit {
    fn add(self, a: MoveUnit) -> Self {
        MoveUnit(self.0 + a.0)
    }
    fn sub(self, a: MoveUnit) -> Self {
        MoveUnit(self.0 - a.0)
    }
}

//Represents all the legal moves for a specific piece.
pub struct PossibleMoves {
    //Has the end coord,path from current, and the remainder cost to get there.
    //cells that are the furthest away will have a move unit of zero.
    moves: Vec<(GridCoord, Path, MoveUnit)>,
    start: GridCoord,
}
impl PossibleMoves {
    pub fn new<K: MoveStrategy>(coord: GridCoord, remaining_moves: MoveUnit) -> Self {
        let mut p = PossibleMoves {
            moves: vec![],
            start: coord,
        };
        p.explore_path::<K>(Path::new(), remaining_moves);
        p
    }

    pub fn iter_coords(&self) -> impl Iterator<Item = &GridCoord> {
        self.moves.iter().map(|a| &a.0)
    }

    fn explore_path<K: MoveStrategy>(&mut self, current_path: Path, remaining_moves: MoveUnit) {
        if remaining_moves.0 == 0 {
            return;
        }

        let curr_pos = current_path.get_end_coord(self.start);

        for a in K::adjacent() {
            let target_pos = curr_pos.advance(a);

            //how much it would cost to move to this square
            let cost = terrain_cost(target_pos).add(current_path.move_cost(a));

            //can't afford to move to this square.
            if remaining_moves.0 < cost.0 {
                continue;
            }

            //subtract move cost
            let rr = remaining_moves.sub(cost);

            if !self.consider(&current_path, a, rr) {
                continue;
            }

            self.explore_path::<K>(current_path.add(a).unwrap(), rr)
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

fn terrain_cost(a: GridCoord) -> MoveUnit {
    MoveUnit(0)
}
