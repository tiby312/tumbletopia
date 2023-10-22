use crate::movement::{Filter, FilterRes, GridCoord, HexDir};

// pub const OFFSETS: [[i16; 3]; 6] = [
//     [0, 1, -1],
//     [1, 0, -1],
//     [1, -1, 0],
//     [0, -1, 1],
//     [-1, 0, 1],
//     [-1, 1, 0],
// ];

pub const OFFSETS: [[i16; 3]; 6] = [
    [1, 0, -1],
    [1, -1, 0],
    [0, -1, 1],
    [-1, 0, 1],
    [-1, 1, 0],
    [0, 1, -1],
];

#[test]
fn what() {
    let k = Cube(OFFSETS[0]);

    let j = k.rotate_60_right().rotate_60_right();

    assert_eq!(j.0, [0, -1, 1]);
}

//TODO use this
#[derive(Copy, Clone, Default, Hash, Debug, PartialEq, Eq)]
pub enum Dir {
    #[default]
    BottomRight = 0,
    Bottom = 1,
    BottomLeft = 2,
    TopLeft = 3,
    Top = 4,
    TopRight = 5,
}
impl From<u8> for Dir {
    fn from(value: u8) -> Self {
        use Dir::*;
        match value {
            0 => BottomRight,
            1 => Bottom,
            2 => BottomLeft,
            3 => TopLeft,
            4 => Top,
            5 => TopRight,
            _ => unreachable!(),
        }
    }
}

pub(crate) const SQRT_3: f32 = 1.73205080757;

// https://www.redblobgames.com/grids/hexagons/#hex-to-pixel

// pub const HEX_PROJ_POINTY: cgmath::Matrix2<f32> =
//     cgmath::Matrix2::new(SQRT_3, 0.0, SQRT_3 / 2.0, 3.0 / 2.0);

pub const HEX_PROJ_FLAT: cgmath::Matrix2<f32> =
    cgmath::Matrix2::new(3.0 / 2.0, SQRT_3 / 2.0, 0.0, SQRT_3);

//q r s
#[derive(Copy, Clone, Debug)]
pub struct Cube(pub [i16; 3]);
impl Cube {
    // triplex & operator*=(const triplex &rhs)
    // {
    //     /*
    //      * (this->r + this->s * f) * (rhs.r + rhs.s * f)
    //      * = this->r * rhs.r + (this->r * rhs.s + this->s * rhs.r ) * f
    //      *   + this->s * rhs.s * f * f
    //      *
    //      * ... remembering that f * f = -3 ...
    //      *
    //      * = (this->r * rhs.r - 3 * this->s * rhs.s)
    //      *   + (this->r * rhs.s + this->s * rhs.r) * f
    //      */
    //     int new_r = this->r * rhs.r - 3 * this->s * rhs.s;
    //     int new_s = this->r * rhs.s + this->s * rhs.r;
    //     this->r = new_r; this->s = new_s;
    //     return *this;
    // }
    pub fn triplex(self, other: &Cube) -> Self {
        let this = &self.0;
        let other = &other.0;
        let new_q = this[0] * other[0] - 3 * this[1] * other[1];
        let new_r = this[0] * other[1] + this[1] * other[1];
        Cube::new(new_q, new_r)
    }

    pub fn new(q: i16, r: i16) -> Self {
        Cube([q, r, -q - r])
    }
    pub fn rotate_60_right(self) -> Cube {
        let [q, _, s] = self.0;
        Cube::new(-s, -q)
    }
    pub fn rotate_60_left(self) -> Cube {
        let [_, r, s] = self.0;
        Cube::new(-r, -s)
    }

    pub fn rotate(self, dir: HexDir) -> Cube {
        let k = self;
        match dir.dir {
            0 => k,
            1 => k.rotate_60_right(),
            2 => k.rotate_60_right().rotate_60_right(),
            3 => k.rotate_60_right().rotate_60_right().rotate_60_right(),
            4 => k.rotate_60_left().rotate_60_left(),
            5 => k.rotate_60_left(),
            _ => unreachable!(),
        }
    }
    pub fn rotate_back(self, dir: HexDir) -> Cube {
        let k = self;
        match dir.dir {
            0 => k,
            5 => k.rotate_60_right(),
            4 => k.rotate_60_right().rotate_60_right(),
            3 => k.rotate_60_right().rotate_60_right().rotate_60_right(),
            2 => k.rotate_60_left().rotate_60_left(),
            1 => k.rotate_60_left(),
            _ => unreachable!(),
        }
    }
    pub fn round(frac: [f32; 3]) -> Cube {
        let mut q = frac[0].round() as i16;
        let mut r = frac[1].round() as i16;
        let mut s = frac[2].round() as i16;

        let q_diff = (q as f32 - frac[0]).abs();
        let r_diff = (r as f32 - frac[1]).abs();
        let s_diff = (s as f32 - frac[2]).abs();

        if q_diff > r_diff && q_diff > s_diff {
            q = -r - s
        } else if r_diff > s_diff {
            r = -q - s
        } else {
            s = -q - r
        }
        return Cube([q, r, s]);
    }

    pub const fn to_axial(&self) -> GridCoord {
        GridCoord([self.0[0], self.0[1]])
    }

    pub fn neighbour(&self, dir: Dir) -> Cube {
        self.add(Cube::direction(dir))
    }
    pub fn direction(dir: Dir) -> Cube {
        Cube(OFFSETS[dir as usize])
    }
    pub fn add(mut self, other: Cube) -> Cube {
        let a = &mut self.0;
        let b = other.0;
        a[0] += b[0];
        a[1] += b[1];
        a[2] += b[2];

        self
    }

    pub fn rays(&self, start: i16, end: i16, ff: impl Filter + Copy) -> impl Iterator<Item = Cube> {
        let o = *self;
        OFFSETS.iter().flat_map(move |&i| {
            (1..end)
                .map(move |a| (a, o.add(Cube(i).scale(a))))
                .take_while(move |(_, o)| ff.filter(&o.to_axial()) == FilterRes::Accept)
                .filter(move |(a, _)| *a >= start)
                .map(|(_, a)| a)
        })
    }
    pub fn ring(&self, n: i16) -> impl Iterator<Item = Cube> + Clone {
        let mut hex = self.add(Cube::direction(Dir::BottomLeft).scale(n));

        (0..6)
            .flat_map(move |i| std::iter::repeat(i).take(n as usize))
            .map(move |i| {
                let h = hex;
                hex = hex.neighbour(i.into());
                h
            })
    }

    pub fn scale(self, n: i16) -> Cube {
        let a = self.0;
        Cube(a.map(|a| a * n))
    }

    pub fn range(&self, n: i16) -> impl Iterator<Item = Cube> + Clone {
        let o = *self;
        (-n..n + 1)
            .flat_map(move |q| ((-n).max(-q - n)..n.min(-q + n) + 1).map(move |r| (q, r)))
            .map(move |(q, r)| {
                let s = -q - r;
                o.add(Cube([q, r, s]))
            })
    }

    //TODO implement using ring??
    pub fn neighbours(&self) -> impl Iterator<Item = Cube> + Clone {
        let k = self.clone();
        OFFSETS.iter().map(move |a| {
            k.add(Cube(*a))
            // let mut a = a.clone();
            // for (a, b) in a.iter_mut().zip(k.iter()) {
            //     *a += b;
            // }
            // Cube(a)
        })
    }

    pub fn dist(&self, other: &Cube) -> i16 {
        let b = other.0;
        let a = self.0;
        // https://www.redblobgames.com/grids/hexagons/#distances-cube
        ((b[0] - a[0]).abs() + (b[1] - a[1]).abs() + (b[2] - a[2]).abs()) / 2
    }
}
