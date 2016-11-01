#[macro_use] extern crate glium;
pub extern crate nalgebra;

pub use nalgebra as na;
pub use nalgebra::{Vector3, Vector4, UnitQuaternion, Matrix3, Matrix4, Transpose, PerspectiveMatrix3, Isometry3, Point3};

mod geometry;
pub use geometry::*;

mod resources;
pub use resources::*;

mod components;
pub use components::*;

mod world;
pub use world::*;

#[derive(Debug, Copy, Clone)]
pub struct TimeDelta(f32);

impl TimeDelta {
  pub fn as_seconds(&self) -> f32 {
    self.0 / 1000.0
  }

  pub fn as_millis(&self) -> f32 {
    self.0
  }
}

use glium as gl;
impl GameObject {
  pub fn draw<S: gl::Surface>(&self,
                              target: &mut S,
                              resources: &ResourceManager,
                              world_uniforms: &WorldUniforms) {
    if let Some(ref mesh) = self.components.geometry() {
      let view_mat       = world_uniforms.view_matrix;
      let model_mat      = self.transform.as_matrix();
      let model_view_mat = view_mat * model_mat;
      let normal_mat     = na::inverse(&matrix3_from_matrix4(&(model_mat))).unwrap();

      let uniforms = uniform! {
        modelMatrix:      model_mat.as_uniform(),
        projectionMatrix: world_uniforms.projection_matrix.as_uniform(),
        viewMatrix:       view_mat.as_uniform(),
        modelViewMatrix:  model_view_mat.as_uniform(),
        normalMatrix:     normal_mat.as_uniform(),
        lightPosition:    world_uniforms.light_position.as_uniform(),
      };

      // TODO: Pull out somewhere
      let params = gl::DrawParameters {
        depth: gl::Depth {
          test: gl::draw_parameters::DepthTest::IfLess,
          write: true,
          .. Default::default()
        },
        .. Default::default()
      };

      let ref buffers = resources.meshes[&mesh.geometry];
      let ref program = resources.programs[&mesh.program];
      target.draw((&buffers.positions, &buffers.normals),
                  &buffers.indices,
                  program,
                  &uniforms,
                  &params)
        .unwrap();
    }
  }
}

pub struct WorldUniforms {
  pub projection_matrix: Matrix4<f32>,
  pub view_matrix:       Matrix4<f32>,
  pub light_position:    Point3<f32>,
}

pub fn matrix3_from_matrix4<T: Copy>(m: &Matrix4<T>) -> Matrix3<T> {
  Matrix3::new(m[(0,0)], m[(1,0)], m[(2,0)],
               m[(0,1)], m[(1,1)], m[(2,1)],
               m[(0,2)], m[(1,2)], m[(2,2)])
}

pub fn quat_rotate<T: Copy+na::BaseFloat>(angle: T, axis: &Vector3<T>) -> UnitQuaternion<T> {
  UnitQuaternion::from_axisangle(na::Unit::new(axis), angle)
}

pub trait AsUniform<T>
  where Self: Sized+AsRef<T>, T: Clone {
  fn as_uniform(&self) -> T {
    let t: &T = self.as_ref();
    (*t).clone()
  }
}

impl<T: Copy> AsUniform<[[T; 3]; 3]> for Matrix3<T> { }
impl<T: Copy> AsUniform<[[T; 4]; 4]> for Matrix4<T> { }
impl<T: Copy> AsUniform<[T; 3]>      for Point3<T>  { }

