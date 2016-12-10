use std::collections::{BTreeMap, BTreeSet};
use glium as gl;
use std::f32::consts;

use super::*;

pub type EntityId = u32;
type EntityMap<K,V> = BTreeMap<K,V>;

pub trait AsMatrix {
  fn as_matrix(&self) -> Matrix4<f32>;
}

// Basic Object Attributes

#[derive(Debug, Copy, Clone)]
pub struct Position(pub Vector3<f32>);

impl AsMatrix for Position {
  fn as_matrix(&self) -> Matrix4<f32> {
    Matrix4::new(1.0, 0.0, 0.0, self.0.x,
                 0.0, 1.0, 0.0, self.0.y,
                 0.0, 0.0, 1.0, self.0.z,
                 0.0, 0.0, 0.0, 1.0)
  }
}

impl From<na::Vector3<f32>> for Position {
  fn from(o: na::Vector3<f32>) -> Self {
    Position(o)
  }
}

#[derive(Debug, Copy, Clone)]
pub struct Scale(pub Vector3<f32>);

impl AsMatrix for Scale {
  fn as_matrix(&self) -> Matrix4<f32> {
    Matrix4::new(self.0.x, 0.0,      0.0,      0.0,
                 0.0,      self.0.y, 0.0,      0.0,
                 0.0,      0.0,      self.0.z, 0.0,
                 0.0,      0.0,      0.0,      1.0)
  }
}

impl From<na::Vector3<f32>> for Scale {
  fn from(o: na::Vector3<f32>) -> Self {
    Scale(o)
  }
}

#[derive(Debug, Copy, Clone)]
pub struct Rotation(pub na::UnitQuaternion<f32>);

impl AsMatrix for Rotation {
  fn as_matrix(&self) -> Matrix4<f32> {
    na::to_homogeneous(self.0.to_rotation_matrix().submatrix())
  }
}

#[derive(Debug, Copy, Clone)]
pub struct Velocity {
  pub angular: Rotation,
  pub linear:  Vector3<f32>
}

// More Complex Object Attributes

#[derive(Debug, Copy, Clone)]
pub struct Geometry {
  pub geometry: &'static str,
  pub program:  &'static str,
}

#[derive(Debug, Copy, Clone)]
pub struct Pickable(pub EntityId);

#[derive(Debug)]
pub struct Camera {
  pub target: Point3<f32>,
  pub tracking: Option<EntityId>,
}

#[derive(Default)]
pub struct EntityManager {
  pub entities:   BTreeSet<EntityId>,
  highest_id:     EntityId,

  pub positions:  EntityMap<EntityId, Position>,
  pub scales:     EntityMap<EntityId, Scale>,
  pub rotations:  EntityMap<EntityId, Rotation>,
  pub velocities: EntityMap<EntityId, Velocity>,
  pub geometries: EntityMap<EntityId, Geometry>,
  pub pickables:  EntityMap<EntityId, Pickable>,
  pub cameras:    EntityMap<EntityId, Camera>,
  pub bobs:       EntityMap<EntityId, Bob>,

  pub picked_entity: Option<EntityId>,
}

impl EntityManager {
  pub fn new_entity(&mut self) -> EntityId {
    self.highest_id += 1;
    self.entities.insert(self.highest_id);
    self.highest_id
  }

  pub fn delete_entity(&mut self, _entity: EntityId) {
    unimplemented!();
  }

  pub fn add_geometry(&mut self, entity: EntityId, g: Geometry) {
    self.geometries.insert(entity, g);
  }

  pub fn set_position<P: Into<Position>>(&mut self, entity: EntityId, p: P) {
    self.positions.insert(entity, p.into());
  }

  pub fn add_camera(&mut self, entity: EntityId, camera: Camera) {
    self.cameras.insert(entity, camera);
  }

  pub fn set_scale<S: Into<Scale>>(&mut self, entity: EntityId, scale: S) {
    self.scales.insert(entity, scale.into());
  }

  pub fn set_rotation(&mut self, entity: EntityId, rot: Rotation) {
    self.rotations.insert(entity, rot);
  }

  pub fn set_pickable(&mut self, entity: EntityId, enable: bool) {
    if enable {
      self.pickables.insert(entity, Pickable(entity));
    } else {
      self.pickables.remove(&entity);
    }
  }
}

pub struct PickingSystem {
  texture: gl::texture::UnsignedTexture2d,
  depth:   gl::framebuffer::DepthRenderBuffer,
  pbo:     gl::texture::pixel_buffer::PixelBuffer<u32>,
  size:    (u32, u32),
}

impl PickingSystem {
  pub fn new<F: gl::backend::Facade+Sized>(display: &F, size: (u32, u32)) -> Self {
    let (color, depth) = Self::new_buffers(display, size);
    PickingSystem {
      texture: color,
      depth: depth,
      pbo: gl::texture::pixel_buffer::PixelBuffer::new_empty(display, 1),
      size: size,
    }
  }

  pub fn prepare(&mut self, size: (u32, u32)) {
    if self.size != size {
      println!("Updating size from {:?} to {:?}", self.size, size);

      let (color, depth) = Self::new_buffers(self.pbo.get_context(), size);
      self.texture = color;
      self.depth = depth;
      self.size = size;
    }
  }

  pub fn get_surface(&self) -> gl::framebuffer::SimpleFrameBuffer {
    let facade = self.pbo.get_context();
    gl::framebuffer::SimpleFrameBuffer::with_depth_buffer(facade,
                                                          &self.texture,
                                                          &self.depth)
      .unwrap()
  }

  fn new_buffers<F: gl::backend::Facade>(display: &F, size: (u32, u32))
                                         -> (gl::texture::UnsignedTexture2d, gl::framebuffer::DepthRenderBuffer) {
    let color = gl::texture::UnsignedTexture2d::empty_with_format(display,
                                                                  gl::texture::UncompressedUintFormat::U32,
                                                                  gl::texture::MipmapsOption::NoMipmap,
                                                                  size.0, size.1).unwrap();
    let depth = gl::framebuffer::DepthRenderBuffer::new(display,
                                                        gl::texture::DepthFormat::F32,
                                                        size.0, size.1).unwrap();

    (color, depth)
  }


  pub fn read_picking_buffer(&mut self) -> Option<EntityId> {
    // Copy the picking_vbo into main memory and read its value
    self.pbo.read().ok().and_then(|px| {
      if px[0] > 0 { Some(px[0]) } else { None }
    })
  }

  pub fn update(&self, position: (u32, u32)) {
    if position.0 <= self.size.0 && position.1 <= self.size.1 {
      let rect = gl::Rect {
        left: position.0 as u32,
        bottom: self.size.1 - position.1 as u32 - 1,
        width: 1,
        height: 1
      };
      self.texture.main_level()
        .first_layer()
        .into_image(None).unwrap()
        .raw_read_to_pixel_buffer(&rect, &self.pbo);
    }
  }
}

pub struct WorldUniforms {
  pub projection_matrix: na::Matrix4<f32>,
  pub light_position:    na::Point3<f32>,
  pub camera_matrix:     na::Matrix4<f32>,
  pub camera_position:   na::Point3<f32>,
}

#[derive(Default)]
pub struct VelocitySystem;
impl VelocitySystem {
  pub fn run(manager: &mut EntityManager, delta: Millis) {
    for (entity, velocity) in manager.velocities.iter() {
      let delta = delta.as_seconds();
      // Update component.position
      manager.positions.get_mut(&entity).unwrap().0 += velocity.linear * delta;

      // Update component.rotation
      let angle = velocity.angular.0.angle()*delta;
      let axis  = velocity.angular.0.axis().unwrap();

      use std::collections::btree_map::Entry::*;
      match manager.rotations.entry(*entity) {
        Vacant(entry) => {
          entry.insert(Rotation(quat_rotate(angle, axis)));
        },
        Occupied(entry) => {
          entry.into_mut().0 *= UnitQuaternion::from_axisangle(axis, angle);
        },
      };
    }
  }
}

pub struct CameraSystem;
impl CameraSystem {
  pub fn run(manager: &mut EntityManager, _delta: Millis) {
    for (_, camera) in manager.cameras.iter_mut() {
      if let Some(target) = camera.tracking {
        camera.target = manager.positions[&target].0.to_point();
      }
    }
  }
}

#[derive(Debug)]
pub struct Bob {
  pub period: Millis,
  pub direction: Vector3<f32>,
  pub state: Millis,
}

impl Bob {
  pub fn new(period: Millis, direction: Vector3<f32>) -> Self {
    Bob {
      period: period,
      direction: direction,
      state: Millis(0.0),
    }
  }
}

pub struct BobSystem;
impl BobSystem {
  pub fn run(manager: &mut EntityManager, delta: Millis) {
    for (entity, mut bob) in manager.bobs.iter_mut() {
      // Update new Bob state
      bob.state += delta;
      if bob.state.as_millis() >= bob.period.as_millis() {
        bob.state -= bob.period;
      }
      // Calculate current position in Sine curve
      let sine = ((bob.state.as_millis() / bob.period.as_millis()) * 2.0 * consts::PI).sin();
      // Calculate position-delta (direction*sine scaled by delta-t)
      let td = delta.as_millis() / bob.period.as_millis();
      let pd = sine * td * bob.direction;
      manager.positions.get_mut(&entity).unwrap().0 += pd;
    }
  }
}
