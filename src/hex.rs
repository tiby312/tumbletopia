use crate::movement::GridCoord;

pub const OFFSETS: [[i16; 3]; 6] = [
    [0, 1, -1],
    [1, 0, -1],
    [1, -1, 0],
    [0, -1, 1],
    [-1, 0, 1],
    [-1, 1, 0],
];



const SQRT_3:f32=1.73205080757;

// https://www.redblobgames.com/grids/hexagons/#hex-to-pixel



pub const HEX_PROJ_POINTY:cgmath::Matrix2<f32>=cgmath::Matrix2::new(SQRT_3,0.0,SQRT_3/2.0,3.0/2.0);

pub const HEX_PROJ_FLAT:cgmath::Matrix2<f32>=cgmath::Matrix2::new(3.0/2.0,SQRT_3/2.0,0.0,SQRT_3);


//This is world
pub fn world()->Vec<Cube>{
    Cube([0,5,0]).range(5).collect()
}


//q r s
#[derive(Copy,Clone)]
pub struct Cube(pub [i16; 3]);
impl Cube {
    pub fn to_axial(&self)->GridCoord{
        GridCoord([self.0[0],self.0[1]])
    }
    
    pub fn neighbour(&self,dir:i16)->Cube{
        self.add(Cube::direction(dir))
    }
    pub fn direction(dir:i16)->Cube{
        Cube(OFFSETS[dir as usize])
    }
    pub fn add(mut self,other:Cube)->Cube{
        let a=&mut self.0;
        let b=other.0;
        for (a,b) in a.iter_mut().zip(b.iter()){
            *a+=b;
        }
        self
    }
    pub fn ring(&self,n:i16)->impl Iterator<Item=Cube>{
        let mut hex=self.add(Cube::direction(4).scale(n));
        
        (0..6).flat_map(move |i|{
            (0..n).map(move |_|{
                let h=hex;
                hex=hex.neighbour(i);
                h
            })
        })
    }

    pub fn scale(self,n:i16)->Cube{
        let a=self.0;
        Cube(a.map(|a|a*n))
    }

    pub fn range(&self,n:i16)->impl Iterator<Item=Cube>{
        let o=*self;
        (-n..n+1).flat_map(move |q|{
            ((-n).max(-q-n)..n.min(-q+n)+1).map(move |r|{
                let s=-q-r;
                o.add(Cube([q,r,s]))
            })
        })
    }
    pub fn neighbours(&self) -> impl Iterator<Item = Cube> {
        let k = self.0.clone();
        OFFSETS.iter().map(move |a| {
            let mut a = a.clone();
            for (a, b) in a.iter_mut().zip(k.iter()) {
                *a += b;
            }
            Cube(a)
        })
    }

    pub fn dist(&self, other: &Cube) -> i16 {
        let b = other.0;
        let a = self.0;
        // https://www.redblobgames.com/grids/hexagons/#distances-cube
        ((b[0] - a[0]).abs() + (b[1] - a[1]).abs() + (b[2] - a[2]).abs()) / 2
    }
}
