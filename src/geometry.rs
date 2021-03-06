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

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
  pub position: [f32; 3],
  pub uv: [f32; 2],
}
implement_vertex!(Vertex, position, uv);
// implement_from!(Vertex, position);

impl From<na::Vector3<f32>> for Vertex {
  fn from(v: na::Vector3<f32>) -> Self {
    Vertex::new(v.x, v.y, v.z)
  }
}

impl From<(f32, f32, f32)> for Vertex {
  fn from((x, y, z): (f32, f32, f32)) -> Self {
    Vertex::new(x, y, z)
  }
}

impl Vertex {
  fn new(x: f32, y: f32, z: f32) -> Self {
    Vertex {
      position: [x, y, z],
      uv: [0.0; 2],
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct Normal { normal: [f32; 3] }
implement_vertex!(Normal, normal);
implement_from!(Normal, normal);

#[derive(Debug, Clone, Copy)]
pub struct Material {
  pub ambient:   [f32; 4], // Fourth element is needed for padding
  pub diffuse:   [f32; 4],
  pub specular:  [f32; 4],
  pub shininess: f32,
}

impl Default for Material {
  fn default() -> Self {
    Material {
      ambient: [0.0; 4],
      diffuse: [0.0; 4],
      specular: [0.0; 4],
      shininess: 0.0,
    }
  }
}

implement_uniform_block!(Material,
                         ambient,
                         diffuse,
                         specular,
                         shininess);
