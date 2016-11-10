use nalgebra as na;
use nalgebra::{Vector3, UnitQuaternion, Matrix4};

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


#[derive(Clone, Copy)]
pub struct Vertex { position: [f32; 3] }
implement_vertex!(Vertex, position);

impl From<Vector3<f32>> for Vertex {
  fn from(v: Vector3<f32>) -> Self {
    Vertex {
      position: [v.x, v.y, v.z]
    }    
  }
}

#[derive(Clone, Copy)]
pub struct Normal { normal: [f32; 3] }
implement_vertex!(Normal, normal);

impl From<Vector3<f32>> for Normal {
  fn from(v: Vector3<f32>) -> Self {
    Normal {
      normal: [v.x, v.y, v.z]
    }
  }
}

