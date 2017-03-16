#[macro_use] extern crate glium;
pub extern crate nalgebra;
extern crate alga;

#[macro_use] extern crate newtype_derive;
#[macro_use] extern crate custom_derive;

#[macro_use] extern crate bitflags;

use std::fmt::Debug;
use std::cmp::PartialEq;

pub use nalgebra as na;
pub use nalgebra::{Vector3, Vector4, UnitQuaternion, Matrix3, Matrix4};
use std::time::Duration;

mod geometry;
pub use geometry::*;

mod resources;
pub use resources::*;

mod components;
pub use components::*;

mod render_system;
pub use render_system::*;

mod world;
pub use world::*;

custom_derive! {
  #[derive(Debug, Copy, Clone,
           NewtypeAdd, NewtypeSub,
           NewtypeAddAssign, NewtypeSubAssign)]
  pub struct Millis(pub f32);
}

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

pub fn matrix3_from_matrix4<T>(m: &Matrix4<T>) -> Matrix3<T>
  where T: Copy + PartialEq + Debug + 'static {
  Matrix3::new(m[(0,0)], m[(1,0)], m[(2,0)],
               m[(0,1)], m[(1,1)], m[(2,1)],
               m[(0,2)], m[(1,2)], m[(2,2)])
}

pub fn quat_rotate<T>(angle: T, axis: na::Unit<na::Vector3<T>>) -> na::UnitQuaternion<T>
  where T: Copy + alga::general::Real {
  na::UnitQuaternion::from_axis_angle(&axis, angle)
}

pub trait AsUniform<T: Clone> {
  fn as_uniform(&self) -> T;
}

impl<T: Sized + AsRef<T> + Clone> AsUniform<T> for T {
  fn as_uniform(&self) -> T {
    let t: &T = self.as_ref();
    t.clone()
  }
}

impl<S: Copy + Debug + PartialEq + 'static> AsUniform<[[S; 3]; 3]> for Matrix3<S> {
  fn as_uniform(&self) -> [[S; 3]; 3] {
    let s = self.transpose();
    [[s[(0,0)], s[(0,1)], s[(0,2)]],
     [s[(1,0)], s[(1,1)], s[(1,2)]],
     [s[(2,0)], s[(2,1)], s[(2,2)]]]
  }
}

impl<S: Copy + Debug + PartialEq + 'static> AsUniform<[[S; 4]; 4]> for Matrix4<S> {
  fn as_uniform(&self) -> [[S; 4]; 4] {
    let s = self.transpose();
    [[s[(0,0)], s[(0,1)], s[(0,2)], s[(0,3)]],
     [s[(1,0)], s[(1,1)], s[(1,2)], s[(1,3)]],
     [s[(2,0)], s[(2,1)], s[(2,2)], s[(2,3)]],
     [s[(3,0)], s[(3,1)], s[(3,2)], s[(3,3)]]]
  }
}

impl<S: Copy + Debug + PartialEq + 'static> AsUniform<[S; 3]> for Vector3<S>  {
  fn as_uniform(&self) -> [S; 3] {
    [self.x, self.y, self.z]
  }
}

