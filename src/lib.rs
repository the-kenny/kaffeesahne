#[macro_use] extern crate glium;
pub extern crate nalgebra;

pub use nalgebra as na;
pub use nalgebra::{Vector3, Vector4, UnitQuaternion, Matrix3, Matrix4, Transpose, PerspectiveMatrix3, Isometry3, Point3};
use std::time::Duration;

mod geometry;
pub use geometry::*;

mod resources;
pub use resources::*;

mod components;
pub use components::*;

mod world;
pub use world::*;

#[derive(Debug, Copy, Clone)]
pub struct Millis(pub f32);

impl Millis {
  pub fn as_seconds(&self) -> f32 {
    self.0 / 1000.0
  }

  pub fn as_millis(&self) -> f32 {
    self.0
  }
}

impl From<Duration> for Millis {
  fn from(dur: Duration) -> Self {
    Millis(dur.as_secs() as f32 / 1000.0
           +
           dur.subsec_nanos() as f32 / 1000000.0)
  }
}

pub fn matrix3_from_matrix4<T: Copy>(m: &Matrix4<T>) -> Matrix3<T> {
  Matrix3::new(m[(0,0)], m[(1,0)], m[(2,0)],
               m[(0,1)], m[(1,1)], m[(2,1)],
               m[(0,2)], m[(1,2)], m[(2,2)])
}

pub fn quat_rotate<T: Copy+na::BaseFloat>(angle: T, axis: na::Unit<na::Vector3<T>>)
                                          -> na::UnitQuaternion<T> {
  UnitQuaternion::from_axisangle(axis, angle)
}

pub trait AsUniform<T>
  where Self: Sized+AsRef<T>, T: Clone {
  fn as_uniform(&self) -> T {
    let t: &T = self.as_ref();
    t.clone()
  }
}

impl<T: Copy> AsUniform<[[T; 3]; 3]> for Matrix3<T> { }
impl<T: Copy> AsUniform<[[T; 4]; 4]> for Matrix4<T> { }
impl<T: Copy> AsUniform<[T; 3]>      for Point3<T>  { }

