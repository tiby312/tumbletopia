use crate::grids::GridMatrix;

use super::*;
pub fn get_world_rect(view_projection: &Matrix4<f32>, grid: &GridMatrix) -> [[i16; 2]; 2] {
    let k = 1.0;
    let a = clip_to_world([k, k], view_projection);
    let b = clip_to_world([-k, -k], view_projection);
    let c = clip_to_world([-k, k], view_projection);
    let d = clip_to_world([k, -k], view_projection);

    let mut r = axgeom::Rect::new(0.0, 0.0, 0.0, 0.0);
    r.grow_to_fit_point(a.into());
    r.grow_to_fit_point(b.into());
    r.grow_to_fit_point(c.into());
    r.grow_to_fit_point(d.into());

    let a = grid.world_to_hex([r.x.start, r.y.start].into()).0;
    let b = grid.world_to_hex([r.x.end, r.y.end].into()).0;

    [[a[0], b[0] + 1], [a[1], b[1] + 1]]
}

pub fn clip_to_world(clip: [f32; 2], view_projection: &Matrix4<f32>) -> [f32; 2] {
    use matrix::*;
    let [clip_x, clip_y] = clip;
    let startc = [clip_x, clip_y, -0.9];
    let endc = [clip_x, clip_y, 0.999];

    let matrix = view_projection.inverse().generate();

    let a = matrix.transform_point(startc.into());
    let b = matrix.transform_point(endc.into());

    let v = b - a;
    let ray = collision::Ray::new(a, v);

    let p = cgmath::Point3::new(0.0, 0.0, 0.0);
    let up = cgmath::Vector3::new(0.0, 0.0, -1.0);

    let plane = collision::Plane::from_point_normal(p, up);
    use collision::Continuous;

    if let Some(point) = plane.intersection(&ray) {
        [point.x, point.y]
    } else {
        [300.0, -80.0]
    }
}

pub fn view_matrix(camera: [f32; 2], zoom: f32, rot: f32) -> cgmath::Matrix4<f32> {
    //TODO pass in the point to zoom in and rotate from!!!!!!

    //world coordinates when viewed with this camera is:
    //x leftdown
    //y right down
    //z+ into the sky (-z into the worlds ground)

    use matrix::*;

    use cgmath::*;

    let start_zoom = 800.0;

    let cam = Point3::new(0.0, 0.0, 0.0);
    let dir = Point3::new(-1.0, -1.0, -2.0);
    let up = Vector3::new(0.0, 0.0, 1.0);
    let g = cgmath::Matrix4::look_at(cam, dir, up).inverse();

    let rot = z_rotation(rot);
    let zoom = translation(0.0, 0.0, start_zoom + zoom);
    let camera = translation(camera[0], camera[1], 0.0)
        .chain(rot)
        .chain(g)
        .chain(zoom);

    camera.inverse().generate()
}

pub fn projection(dim: [f32; 2]) -> matrix::Perspective {
    //https://www.gamedev.net/forums/topic/558921-calculating-the-field-of-view/
    //https://docs.unity3d.com/Manual/FrustumSizeAtDistance.html

    let near = 10.0;
    let far = 2000.0;

    let fov_factor = 0.002;
    let dd = dim[1]; //.min(1200.0);
    let frustum_height = dd * fov_factor;

    let fov = 2.0 * (frustum_height * 0.5 / near).atan();
    matrix::perspective(fov /*0.4*/, dim[0] / dim[1], near, far)
}

// #[derive(Copy, Clone)]
// pub struct ViewProjection {
//     pub offset: [f32; 2],
//     pub dim: [f32; 2],
//     pub zoom: f32,
//     pub rot: f32,
// }
// impl matrix::Inverse for ViewProjection {
//     type Neg = cgmath::Matrix4<f32>;

//     fn inverse(self) -> Self::Neg {
//         self.generate().inverse()
//     }
// }
// impl matrix::MyMatrix for ViewProjection {
//     fn generate(self) -> cgmath::Matrix4<f32> {
//         use matrix::*;

//         projection(self.dim)
//             .chain(view_matrix(self.offset, self.zoom, self.rot))
//             .generate()
//     }
// }

// pub fn view_projection(offset: [f32; 2], dim: [f32; 2], zoom: f32, rot: f32) -> ViewProjection {
//     ViewProjection {
//         offset,
//         dim,
//         zoom,
//         rot,
//     }
// }
