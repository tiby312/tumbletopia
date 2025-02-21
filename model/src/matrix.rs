use cgmath::Transform;

pub trait Inverse: MyMatrix {
    type Neg: MyMatrix + Inverse;
    fn inverse(self) -> Self::Neg;
}

impl MyMatrix for cgmath::Matrix4<f32> {
    fn generate(self) -> cgmath::Matrix4<f32> {
        self
    }
}

impl Inverse for cgmath::Matrix4<f32> {
    type Neg = Self;
    fn inverse(self) -> Self::Neg {
        self.inverse_transform().unwrap()
    }
}
pub trait MyMatrix {
    fn generate(self) -> cgmath::Matrix4<f32>;

    fn chain<K: MyMatrix>(self, other: K) -> Chain<Self, K>
    where
        Self: Sized,
    {
        Chain { a: self, b: other }
    }
}

#[derive(Copy, Clone, Debug)]
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
    fn generate(self) -> cgmath::Matrix4<f32> {
        let a = self.a.generate();
        let b = self.b.generate();
        a * b
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Perspective {
    field_of_view_rad: f32,
    aspect: f32,
    near: f32,
    far: f32,
}

impl MyMatrix for Perspective {
    fn generate(self) -> cgmath::Matrix4<f32> {
        //let rr=100.0;
        //cgmath::ortho(-rr,rr, -rr, rr, self.near, self.far)
        cgmath::perspective(
            cgmath::Rad(self.field_of_view_rad),
            self.aspect,
            self.near,
            self.far,
        )
    }
}

impl Inverse for Perspective {
    type Neg = cgmath::Matrix4<f32>;
    fn inverse(self) -> Self::Neg {
        self.generate().inverse_transform().unwrap()
    }
}
pub fn perspective(field_of_view_rad: f32, aspect: f32, near: f32, far: f32) -> Perspective {
    Perspective {
        field_of_view_rad,
        aspect,
        near,
        far,
    }
}

#[derive(Copy, Clone, Debug)]
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
            tz: 1.0 / self.tz,
        }
    }
}
impl MyMatrix for Scale {
    fn generate(self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::new(
            self.tx, 0., 0., 0., 0., self.ty, 0., 0., 0., 0., self.tz, 0., 0., 0., 0., 1.0,
        )
    }
}

#[derive(Copy, Clone, Debug)]
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
    fn generate(self) -> cgmath::Matrix4<f32> {
        let c = self.angle_rad.cos();
        let s = self.angle_rad.sin();

        cgmath::Matrix4::new(1., 0., 0., 0., 0., c, s, 0., 0., -s, c, 0., 0., 0., 0., 1.)
    }
}

#[derive(Copy, Clone, Debug)]
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
    fn generate(self) -> cgmath::Matrix4<f32> {
        let c = self.angle_rad.cos();
        let s = self.angle_rad.sin();

        cgmath::Matrix4::new(c, 0., -s, 0., 0., 1., 0., 0., s, 0., c, 0., 0., 0., 0., 1.)
    }
}

#[derive(Copy, Clone, Debug)]
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
    fn generate(self) -> cgmath::Matrix4<f32> {
        let c = self.angle_rad.cos();
        let s = self.angle_rad.sin();

        cgmath::Matrix4::new(c, s, 0., 0., -s, c, 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.)
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

#[derive(Clone, Debug)]
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
    fn generate(self) -> cgmath::Matrix4<f32> {
        let tx = self.tx;
        let ty = self.ty;
        let tz = self.tz;
        cgmath::Matrix4::new(
            1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., tx, ty, tz, 1.,
        )
    }
}
