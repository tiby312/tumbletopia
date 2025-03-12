// pub const OFFSETS: [[i16; 3]; 6] = [
//     [0, 1, -1],
//     [1, 0, -1],
//     [1, -1, 0],
//     [0, -1, 1],
//     [-1, 0, 1],
//     [-1, 1, 0],
// ];

pub type CoordNum = i8;

pub const OFFSETS: [[CoordNum; 3]; 6] = [
    [1, 0, -1],
    [1, -1, 0],
    [0, -1, 1],
    [-1, 0, 1],
    [-1, 1, 0],
    [0, 1, -1],
];

pub const DIAG_OFFSETS: [[CoordNum; 3]; 6] = [
    [2, -1, -1],
    [1, -2, 1],
    [-1, -1, 2],
    [-2, 1, 1],
    [-1, 2, -1],
    [1, 1, -2],
];

// var cube_diagonal_vectors = [
//     Cube(+2, -1, -1), Cube(+1, -2, +1), Cube(-1, -1, +2),
//     Cube(-2, +1, +1), Cube(-1, +2, -1), Cube(+1, +1, -2),
// ]

// pub enum MovDir{
//     Inner(HDir),
//     Outer(Outer)
// }
// pub enum Outer{
//     Aligned(HDir),
//     UnAligned(HDir)
// }

//TODO use this
#[derive(Copy, Clone, Default, Hash, Debug, PartialEq, Eq)]
pub enum HDir {
    #[default]
    BottomRight = 0,
    Bottom = 1,
    BottomLeft = 2,
    TopLeft = 3,
    Top = 4,
    TopRight = 5,
}

const ALL: [HDir; 6] = [
    HDir::BottomRight,
    HDir::Bottom,
    HDir::BottomLeft,
    HDir::TopLeft,
    HDir::Top,
    HDir::TopRight,
];

impl HDir {
    pub fn all() -> impl Iterator<Item = HDir> {
        ALL.iter().copied()

        //(0..6).map(HDir::from)
    }
    pub fn rotate_180(&self) -> HDir {
        match self{
            HDir::BottomRight => HDir::TopLeft,
            HDir::Bottom => HDir::Top,
            HDir::BottomLeft => HDir::TopRight,
            HDir::TopLeft => HDir::BottomRight,
            HDir::Top => HDir::Bottom,
            HDir::TopRight => HDir::BottomLeft,
        }
    }
    pub fn rotate60_right(&self) -> HDir {
        // 0->4
        // 1->5
        // 2->0
        // 3->1
        // 4->2
        // 5->3

        HDir::from((*self as u8 + 1) % 6)
    }

    pub fn rotate60_left(&self) -> HDir {
        // 0->2
        // 1->3
        // 2->4
        // 3->5
        // 4->0
        // 5->1

        HDir::from((*self as u8 + 5) % 6)
    }

    pub const fn to_relative(&self) -> Axial {
        Cube::from_arr(OFFSETS[*self as usize]).to_axial()
    }
}
impl From<u8> for HDir {
    fn from(value: u8) -> Self {
        use HDir::*;
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

pub(crate) const SQRT_3: f32 = 1.732_050_8;

// https://www.redblobgames.com/grids/hexagons/#hex-to-pixel

// pub const HEX_PROJ_POINTY: cgmath::Matrix2<f32> =
//     cgmath::Matrix2::new(SQRT_3, 0.0, SQRT_3 / 2.0, 3.0 / 2.0);

pub const HEX_PROJ_FLAT: cgmath::Matrix2<f32> =
    cgmath::Matrix2::new(3.0 / 2.0, SQRT_3 / 2.0, 0.0, SQRT_3);

//q r s
#[derive(Copy, Clone, Debug)]
pub struct Cube {
    pub ax: Axial,
    pub s: CoordNum,
}

impl From<Cube> for Axial {
    fn from(value: Cube) -> Self {
        value.ax
    }
}

impl From<Axial> for Cube {
    fn from(value: Axial) -> Self {
        Cube::new(value.q, value.r)
    }
}

impl std::ops::Deref for Cube {
    type Target = Axial;
    fn deref(&self) -> &Self::Target {
        &self.ax
    }
}

impl Cube {
    pub fn s(&self) -> CoordNum {
        self.s
    }

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
    // pub fn triplex(self, other: &Cube) -> Self {
    //     let this = &self;
    //     let other = &other;
    //     let new_q = this.q() * other.q() - 3 * this.r() * other.r();
    //     let new_r = this.q() * other.r() + this.r() * other.r();
    //     Cube::new(new_q, new_r)
    // }

    pub const fn from_arr([q, r, s]: [CoordNum; 3]) -> Self {
        Cube {
            ax: Axial { q, r },
            s,
        }
    }
    pub const fn new(q: CoordNum, r: CoordNum) -> Self {
        Cube::from_arr([q, r, -q - r])
    }
    pub fn rotate_60_right(self) -> Cube {
        let Cube {
            ax: Axial { q, .. },
            s,
        } = self;
        Cube::new(-s, -q)
    }
    pub fn rotate_60_left(self) -> Cube {
        let Cube {
            ax: Axial { r, .. },
            s,
        } = self;
        Cube::new(-r, -s)
    }

    pub fn rotate(self, dir: HDir) -> Cube {
        let k = self;
        match dir as u8 {
            0 => k,
            1 => k.rotate_60_right(),
            2 => k.rotate_60_right().rotate_60_right(),
            3 => k.rotate_60_right().rotate_60_right().rotate_60_right(),
            4 => k.rotate_60_left().rotate_60_left(),
            5 => k.rotate_60_left(),
            _ => unreachable!(),
        }
    }
    // pub fn rotate_back(self, dir: HexDir) -> Cube {
    //     let k = self;
    //     match dir.dir {
    //         0 => k,
    //         5 => k.rotate_60_right(),
    //         4 => k.rotate_60_right().rotate_60_right(),
    //         3 => k.rotate_60_right().rotate_60_right().rotate_60_right(),
    //         2 => k.rotate_60_left().rotate_60_left(),
    //         1 => k.rotate_60_left(),
    //         _ => unreachable!(),
    //     }
    // }
    pub fn round(frac: [f32; 3]) -> Cube {
        let mut q = frac[0].round() as CoordNum;
        let mut r = frac[1].round() as CoordNum;
        let mut s = frac[2].round() as CoordNum;

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
        Cube::from_arr([q, r, s])
    }

    pub const fn to_axial(&self) -> Axial {
        self.ax
    }
    pub fn ray_from_vector(&self, v: Cube) -> impl Iterator<Item = Cube> {
        let mut c = *self;
        std::iter::repeat_with(move || {
            c = c.add(v);
            c
        })
    }
    pub fn ray(&self, dir: HDir) -> impl Iterator<Item = (Cube, Cube)> {
        let mut c = *self;
        std::iter::repeat_with(move || {
            let cc = c;
            let k = c.neighbour(dir);
            c = k;
            (cc, k)
        })
    }
    pub fn neighbour(&self, dir: HDir) -> Cube {
        self.add(Cube::direction(dir))
    }
    pub fn direction(dir: HDir) -> Cube {
        Cube::from_arr(OFFSETS[dir as usize])
    }
    pub fn add(mut self, other: Cube) -> Cube {
        self.ax.q += other.ax.q;
        self.ax.r += other.ax.r;
        self.s += other.s;

        self
    }
    pub fn sub(mut self, other: Cube) -> Cube {
        self.ax.q -= other.ax.q;
        self.ax.r -= other.ax.r;
        self.s -= other.s;

        self
    }

    // pub fn rays(&self, start: i16, end: i16, ff: impl Filter + Copy) -> impl Iterator<Item = Cube> {
    //     let o = *self;
    //     OFFSETS.iter().flat_map(move |&i| {
    //         (1..end)
    //             .map(move |a| (a, o.add(Cube::from_arr(i).scale(a))))
    //             .take_while(move |(_, o)| ff.filter(&o.to_axial()) == FilterRes::Accept)
    //             .filter(move |(a, _)| *a >= start)
    //             .map(|(_, a)| a)
    //     })
    // }

    //clockwise
    pub fn ring(&self, n: CoordNum) -> impl Iterator<Item = Cube> + Clone {
        let mut hex = self.add(Cube::direction(HDir::Top).scale(n));

        let k = std::iter::repeat(()).take(n as usize);

        (0..6)
            .flat_map(move |i| k.clone().map(move |_| i))
            .map(move |i| {
                let h = hex;
                hex = hex.neighbour(i.into());

                h
            })
    }

    pub fn scale(mut self, n: CoordNum) -> Cube {
        self.ax.q *= n;
        self.ax.r *= n;
        self.s *= n;
        self
    }

    pub fn range(&self, n: CoordNum) -> impl Iterator<Item = Cube> + Clone {
        let o = *self;
        (-n..n + 1)
            .flat_map(move |q| ((-n).max(-q - n)..n.min(-q + n) + 1).map(move |r| (q, r)))
            .map(move |(q, r)| {
                let s = -q - r;
                o.add(Cube::from_arr([q, r, s]))
            })
    }

    pub fn neighbours2(&self) -> [Cube; 6] {
        let k = self.clone();
        OFFSETS.map(move |a| k.add(Cube::from_arr(a)))
    }

    //TODO implement using ring??
    pub fn neighbours(&self) -> impl Iterator<Item = Cube> + Clone {
        self.ring(1)

        // let k = self.clone();
        // OFFSETS.iter().map(move |a| {
        //     k.add(Cube(*a))
        //     // let mut a = a.clone();
        //     // for (a, b) in a.iter_mut().zip(k.iter()) {
        //     //     *a += b;
        //     // }
        //     // Cube(a)
        // })
    }

    pub fn dist(&self, other: &Cube) -> CoordNum {
        let b = other;
        let a = self;
        // https://www.redblobgames.com/grids/hexagons/#distances-cube
        ((b.q() - a.q()).abs() + (b.r() - a.r()).abs() + (b.s() - a.s()).abs()) / 2
    }
}

use serde::Deserialize;
use serde::Serialize;
#[derive(
    Deserialize, Serialize, Hash, Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord,
)]
#[must_use]
pub struct Axial {
    pub q: CoordNum,
    pub r: CoordNum,
}

const LETTER_COORDINATES: [char; 26] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

const fn inverse(index: usize) -> Axial {
    let x = index / 16;
    let y = index % 16;
    Axial::from_arr([(x as isize - 8) as i8, (y as isize - 8) as i8])
}

const fn conv2(a: Axial) -> usize {
    let Axial { q, r } = a;
    //     let ind=x/7+y%7;
    //     // -3 -2 -1 0 1 2 3
    //     // -6 -5 -4 -3 -2 -1 0 1 2 3 4 5 6
    // ind as usize
    ((q as isize + 8) * 16 + (r as isize + 8)) as usize

    // TABLE
    //     .iter()
    //     .enumerate()
    //     .find(|(_, x)| **x == a.0)
    //     .expect("Could not find the coord in table")
    //     .0
}

pub trait HexDraw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, radius: i8) -> Result<(), std::fmt::Error>;
}

impl HexDraw for Axial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, radius: i8) -> Result<(), std::fmt::Error> {
        match self.to_letter_coord(radius) {
            TextMove::Move(a, b) => {
                write!(f, "{}{}", a, b)
            }
            TextMove::Pass => {
                write!(f, "pp")
            }
        }
    }
}

impl<H: HexDraw> HexDraw for Vec<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>, radius: i8) -> Result<(), std::fmt::Error> {
        write!(f, "{}", "[")?;
        for a in self.iter() {
            a.fmt(f, radius)?;
            write!(f, "{}", ",")?;
        }
        write!(f, "{}", "]")
    }
}

pub fn disp<H: HexDraw>(a: &H, radius: i8) -> Displayer<H> {
    Displayer { ax: a, radius }
}

pub struct Displayer<'a, H> {
    ax: &'a H,
    radius: i8,
}
// impl<H: HexDraw> std::fmt::Display for Displayer<H> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         self.ax.fmt(f, self.radius)
//     }
// }
impl<H: HexDraw> std::fmt::Debug for Displayer<'_, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.ax.fmt(f, self.radius)
    }
}

pub const PASS_MOVE: Axial = Axial { q: -5, r: -5 };
pub const PASS_MOVE_INDEX: usize = const { PASS_MOVE.to_index() };

#[derive(Debug, Clone)]
pub enum TextMove {
    Pass,
    Move(char, i8),
}

impl Axial {
    pub fn disp(&self, radius: i8) -> Displayer<Axial> {
        Displayer { ax: self, radius }
    }
    pub const fn from_index(index: usize) -> Axial {
        inverse(index)
    }
    pub const fn to_index(&self) -> usize {
        conv2(*self)
    }

    pub fn from_letter_coord(foo: char, num: i8, radius: i8) -> Axial {
        let r = num - radius;

        let (index, _) = LETTER_COORDINATES
            .iter()
            .enumerate()
            .find(|(_, x)| **x == foo)
            .unwrap();

        let q = index as i8 - (r + radius) + 1;

        Axial { q, r }
    }
    pub fn to_letter_coord(&self, radius: i8) -> TextMove {
        if *self == PASS_MOVE {
            return TextMove::Pass;
        }
        let k = self.to_cube();

        let number = k.r + radius;
        let letter = LETTER_COORDINATES[(k.q + number - 1) as usize];
        TextMove::Move(letter, number)
    }
    pub fn mul(&self, dis: i8) -> Axial {
        Axial {
            q: self.q * dis,
            r: self.r * dis,
        }
    }
    pub fn index(&self) -> i32 {
        ((self.q as i32) << 16) | self.r as i32
    }
    pub fn q(&self) -> CoordNum {
        self.q
    }
    pub fn r(&self) -> CoordNum {
        self.r
    }
    pub const fn from_arr([q, r]: [CoordNum; 2]) -> Self {
        Axial { q, r }
    }
    pub fn zero() -> Axial {
        Axial { q: 0, r: 0 }
    }
    pub fn dir_to(&self, other: &Axial) -> HDir {
        let mut offset = other.sub(self);

        offset.q = offset.q.clamp(-1, 1);
        offset.r = offset.r.clamp(-1, 1);

        // assert!(offset.0[0].abs() <= 1);
        // assert!(offset.0[1].abs() <= 1);
        let Cube {
            ax: Axial { q, r },
            s,
        } = offset.to_cube();

        OFFSETS
            .iter()
            .enumerate()
            .find(|(_, x)| **x == [q, r, s])
            .map(|(i, _)| HDir::from(i as u8))
            .unwrap()
    }
    pub fn to_cube(self) -> Cube {
        let a = self;
        Cube::from_arr([a.q, a.r, -a.q - a.r])
    }

    pub fn advance(self, m: HDir) -> Axial {
        self.add(m.to_relative())
    }
    pub fn back(self, m: HDir) -> Axial {
        self.sub(&m.to_relative())
    }
    pub fn sub(mut self, o: &Axial) -> Self {
        self.q -= o.q;
        self.r -= o.r;
        self
    }
    pub const fn add(mut self, o: Axial) -> Self {
        self.q += o.q;
        self.r += o.r;
        self
    }
}
