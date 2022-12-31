use webgl_matrix::prelude::*;

pub trait Inverse {
    type Neg: MyMatrix;
    fn inverse(self) -> Self::Neg;
    
}

impl MyMatrix for [f32;16]{
    fn generate(self) -> [f32; 16] {
        self
    }
}
pub trait MyMatrix {
    fn generate(self) -> [f32; 16];

    fn chain<K: MyMatrix>(self, other: K) -> Chain<Self, K>
    where
        Self: Sized,
    {
        Chain { a: self, b: other }
    }

    fn calc_inverse(self)->[f32;16] where Self:Sized{
        let mut g=self.generate();
        g.inverse();
        g
    }
}

pub struct Chain<A, B> {
    a: A,
    b: B,
}
impl<A: MyMatrix + Inverse, B: MyMatrix + Inverse> Inverse for Chain<A, B> {
    type Neg = Chain<B::Neg, A::Neg>;
    fn inverse(self) -> Self::Neg {
        Chain {
            a: self.b.inverse(),
            b: self.a.inverse(),
        }
    }
}
impl<A: MyMatrix, B: MyMatrix> MyMatrix for Chain<A, B> {
    fn generate(self) -> [f32; 16] {
        let mut a = self.a.generate();
        let b = self.b.generate();
        a.mul(&b);
        a
    }
}
pub struct Scale {
    pub tx: f32,
    pub ty: f32,
    pub tz: f32,
}

impl Inverse for Scale {
    type Neg = Self;
    fn inverse(self) -> Self::Neg {
        Scale {
            tx: 1.0 / self.tx,
            ty: 1.0 / self.ty,
            tz: self.tz,
        }
    }
}
impl MyMatrix for Scale {
    fn generate(self) -> [f32; 16] {
        let x = self.tx;
        let y = self.ty;
        let z = self.tz;
        [x, 0., 0., 0., 0., y, 0., 0., 0., 0., z, 0., 0., 0., 0., 1.0]
    }
}

pub struct XRot {
    pub angle_rad: f32,
}
impl Inverse for XRot {
    type Neg = Self;
    fn inverse(self) -> Self::Neg {
        XRot {
            angle_rad: -self.angle_rad,
        }
    }
}
impl MyMatrix for XRot {
    fn generate(self) -> [f32; 16] {
        let c = self.angle_rad.cos();
        let s = self.angle_rad.sin();

        [1., 0., 0., 0., 0., c, s, 0., 0., -s, c, 0., 0., 0., 0., 1.]
    }
}

pub struct YRot {
    pub angle_rad: f32,
}
impl Inverse for YRot {
    type Neg = Self;
    fn inverse(self) -> Self::Neg {
        YRot {
            angle_rad: -self.angle_rad,
        }
    }
}
impl MyMatrix for YRot {
    fn generate(self) -> [f32; 16] {
        let c = self.angle_rad.cos();
        let s = self.angle_rad.sin();

        [c, 0., -s, 0., 0., 1., 0., 0., s, 0., c, 0., 0., 0., 0., 1.]
    }
}

pub struct ZRot {
    pub angle_rad: f32,
}
impl Inverse for ZRot {
    type Neg = Self;
    fn inverse(self) -> Self::Neg {
        ZRot {
            angle_rad: -self.angle_rad,
        }
    }
}
impl MyMatrix for ZRot {
    fn generate(self) -> [f32; 16] {
        let c = self.angle_rad.cos();
        let s = self.angle_rad.sin();

        [c, s, 0., 0., -s, c, 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.]
    }
}

pub fn x_rotation(angle_rad: f32) -> XRot {
    XRot { angle_rad }
}
pub fn y_rotation(angle_rad: f32) -> YRot {
    YRot { angle_rad }
}
pub fn z_rotation(angle_rad: f32) -> ZRot {
    ZRot { angle_rad }
}

pub fn scale(x: f32, y: f32, z: f32) -> Scale {
    Scale {
        tx: x,
        ty: y,
        tz: z,
    }
}
pub fn translation(tx: f32, ty: f32, tz: f32) -> Translation {
    Translation { tx, ty, tz }
}
pub struct Translation {
    tx: f32,
    ty: f32,
    tz: f32,
}

impl Inverse for Translation {
    type Neg = Self;
    fn inverse(self) -> Self::Neg {
        Translation {
            tx: -self.tx,
            ty: -self.ty,
            tz: -self.tz,
        }
    }
}
impl MyMatrix for Translation {
    fn generate(self) -> [f32; 16] {
        let tx = self.tx;
        let ty = self.ty;
        let tz = self.tz;
        [
            1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., tx, ty, tz, 1.,
        ]
    }
}
