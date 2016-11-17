use nalgebra as na;

macro_rules! implement_from {
  ( $t:ident, $inner:ident ) => {
    impl From<na::Vector3<f32>> for $t {
      fn from(v: na::Vector3<f32>) -> Self {
        $t::new(v.x, v.y, v.z)
      }
    }

    impl From<(f32, f32, f32)> for $t {
      fn from((x, y, z): (f32, f32, f32)) -> Self {
        $t::new(x, y, z)
      }
    }

    impl $t {
      fn new(x: f32, y: f32, z: f32) -> Self {
        $t { $inner: [x, y, z] }
      }
    }
  };
}

#[derive(Clone, Copy)]
pub struct Vertex { position: [f32; 3] }
implement_vertex!(Vertex, position);
implement_from!(Vertex, position);

#[derive(Clone, Copy)]
pub struct Normal { normal: [f32; 3] }
implement_vertex!(Normal, normal);
implement_from!(Normal, normal);
