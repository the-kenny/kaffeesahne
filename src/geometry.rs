use nalgebra as na;
use nalgebra::{Vector3, UnitQuaternion, Matrix4};

// TODO: We might want to get rid of Transform as an intermediate for
// render()
#[derive(Debug, Copy, Clone)]
pub struct Transform {
  pub pos:   Vector3<f32>,
  pub rot:   UnitQuaternion<f32>,
  pub scale: Vector3<f32>,
}

impl Transform {
  pub fn as_matrix(&self) -> Matrix4<f32> {
    let mut model = Matrix4::new(1.0, 0.0, 0.0, self.pos.x,
                                 0.0, 1.0, 0.0, self.pos.y,
                                 0.0, 0.0, 1.0, self.pos.z,
                                 0.0, 0.0, 0.0, 1.0);
    model *= na::to_homogeneous(self.rot.to_rotation_matrix().submatrix());
    model *= Matrix4::new(self.scale.x, 0.0,          0.0,          0.0,
                          0.0,          self.scale.y, 0.0,          0.0,
                          0.0,          0.0,          self.scale.z, 0.0,
                          0.0,          0.0,          0.0,          1.0);
    model
  }
}

impl Default for Transform {
  fn default() -> Self {
    Transform {
      pos: na::zero(),
      rot: na::one(),
      scale: na::one(),
    }
  }
}

macro_rules! implement_from {
  ( $t:ident, $inner:ident ) => {
    impl From<Vector3<f32>> for $t {
      fn from(v: Vector3<f32>) -> Self {
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
