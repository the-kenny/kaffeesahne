use glium as gl;
use std::f32::consts;

use super::*;

pub type EntityId = u32;

const MAX_ENTITIES: usize = 4096;

bitflags! {
  // TODO: Add a macro for all of these
  pub flags ComponentFlags: u64 {
    const FLAG_NONE     = 1 << 1,
    const FLAG_POSITION = 1 << 3,
    const FLAG_SCALE    = 1 << 4,
    const FLAG_ROTATION = 1 << 5,
    const FLAG_VELOCITY = 1 << 6,
    const FLAG_GEOMETRY = 1 << 7,
    const FLAG_PICKABLE = 1 << 8,
    const FLAG_CAMERA   = 1 << 9,
    const FLAG_BOB      = 1 << 10,
  }
}

impl Default for ComponentFlags {
  fn default() -> Self { FLAG_NONE }
}

pub struct EntityArray<T: Default>([T; MAX_ENTITIES]);

impl<T: Default> Default for EntityArray<T> {
  fn default() -> Self {
    let mut arr: [T; MAX_ENTITIES];
    unsafe {
      use std::{ptr, mem};
      arr = mem::uninitialized();
      for elem in &mut arr[..] {
        ptr::write(elem, Default::default())
      }
    }
    EntityArray(arr)
  }
}

pub type ComponentArray<T> = EntityArray<T>;
pub type FlagsArray = EntityArray<ComponentFlags>;

use std::ops::{Index, IndexMut, RangeFull};

impl<T: Default> Index<EntityId> for EntityArray<T> {
  type Output = T;
  fn index(&self, entity: EntityId) -> &Self::Output {
    self.0.index(entity as usize)
  }
}

impl<T: Default> IndexMut<EntityId> for EntityArray<T> {
    fn index_mut<'a>(&'a mut self, entity: EntityId) -> &'a mut T {
      self.0.index_mut(entity as usize)
    }
}

impl<'a, T: Default> Index<RangeFull> for EntityArray<T> {
  type Output = [T];
  fn index(&self, _idx: RangeFull) -> &Self::Output {
    self.0.index(..)
  }
}


pub struct EntityIterator<'a> {
  // TODO: use a range for `idx`
  idx: EntityId,
  entities: &'a [ComponentFlags],
  flags: ComponentFlags,
}

impl<'a> Iterator for EntityIterator<'a> {
  type Item = EntityId;
  fn next(&mut self) -> Option<Self::Item> {
    // TODO: Rewrite via iterators
    while (self.idx as usize) < self.entities.len() {
      if self.entities[self.idx as usize].contains(self.flags) {
        let idx = self.idx;
        self.idx += 1;
        return Some(idx)
      } else {
        self.idx += 1;
        continue;
      }
    }
    None
  }
}

pub trait AsMatrix {
  fn as_matrix(&self) -> Matrix4<f32>;
}

// Basic Object Attributes

#[derive(Debug, Copy, Clone)]
pub struct Position(pub Vector3<f32>);

impl Default for Position {
  fn default() -> Self {
    Position(na::zero())
  }
}

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

impl Default for Scale {
  fn default() -> Self {
    Scale(Vector3::new(1.0, 1.0, 1.0))
  }
}

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

impl Default for Rotation {
  fn default() -> Self {
    Rotation(na::UnitQuaternion::identity())
  }
}

impl AsMatrix for Rotation {
  fn as_matrix(&self) -> Matrix4<f32> {
    self.0.to_rotation_matrix()
      //.submatrix()
      .to_homogeneous()
  }
}

#[derive(Debug, Copy, Clone)]
pub struct Velocity {
  pub angular: Rotation,
  pub linear:  Vector3<f32>
}

impl Default for Velocity {
  fn default() -> Self {
    Velocity {
      angular: Rotation::default(),
      linear: na::zero(),
    }
  }
}

// More Complex Object Attributes

#[derive(Debug, Copy, Clone)]
pub struct Geometry {
  pub geometry: &'static str,
  pub program:  &'static str,
}


impl Default for Geometry {
  fn default() -> Self {
    Geometry {
      geometry: "UNINITIALIZED",
      program: "UNINITIALIZED",
    }
  }
}

#[derive(Debug, Copy, Clone)]
pub struct Pickable(pub EntityId);

impl Default for Pickable {
  fn default() -> Self { Pickable(0) }
}

#[derive(Debug)]
pub struct Camera {
  pub target: Vector3<f32>,
  pub tracking: Option<EntityId>,
}

impl Default for Camera {
  fn default() -> Self {
    Camera {
      target: na::Vector3::new(0.0, 0.0, 0.0),
      tracking: None
    }
  }
}

#[derive(Default)]
pub struct EntityManager {
  highest_id:     EntityId,

  pub entities: FlagsArray,

  pub positions:  ComponentArray<Position>,
  pub scales:     ComponentArray<Scale>,
  pub rotations:  ComponentArray<Rotation>,
  pub velocities: ComponentArray<Velocity>,
  pub geometries: ComponentArray<Geometry>,
  pub pickables:  ComponentArray<Pickable>,
  pub cameras:    ComponentArray<Camera>,
  pub bobs:       ComponentArray<Bob>,

  pub picked_entity: Option<EntityId>,
}

impl EntityManager {
  pub fn entity_iter<'a>(flags: &'a FlagsArray, flag: ComponentFlags) -> EntityIterator<'a> {
    EntityIterator {
      idx: 0,
      entities: &flags[..],
      flags: flag,
    }
  }

  pub fn new_entity(&mut self) -> EntityId {
    // TODO: Return first entity which has FLAG_NONE set when we run out of IDs
    self.highest_id += 1;
    assert!(self.highest_id < (self.entities.0.len() as EntityId));
    self.entities[self.highest_id] = FLAG_NONE;
    self.highest_id
  }

  pub fn delete_entity(&mut self, _entity: EntityId) {
    unimplemented!();
  }

  pub fn add_geometry(&mut self, entity: EntityId, g: Geometry) {
    self.geometries[entity] = g;
    self.entities[entity].insert(FLAG_GEOMETRY);
  }

  pub fn set_position<P: Into<Position>>(&mut self, entity: EntityId, p: P) {
    self.positions[entity] = p.into();
    self.entities[entity].insert(FLAG_POSITION);
  }

  pub fn add_camera(&mut self, entity: EntityId, camera: Camera) {
    self.cameras[entity] = camera;
    self.entities[entity].insert(FLAG_CAMERA);
  }

  pub fn set_scale<S: Into<Scale>>(&mut self, entity: EntityId, scale: S) {
    self.scales[entity] = scale.into();
    self.entities[entity].insert(FLAG_SCALE);
  }

  pub fn set_rotation(&mut self, entity: EntityId, rot: Rotation) {
    self.rotations[entity] = rot;
    self.entities[entity].insert(FLAG_ROTATION);
  }

  pub fn set_pickable(&mut self, entity: EntityId, enable: bool) {
    if enable {
      self.pickables[entity] = Pickable(entity);
      self.entities[entity].insert(FLAG_PICKABLE);
    } else {
      self.entities[entity].remove(FLAG_PICKABLE);
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
  pub light_position:    na::Vector3<f32>,
  pub camera_matrix:     na::Matrix4<f32>,
  pub camera_position:   na::Vector3<f32>,
}

#[derive(Default)]
pub struct VelocitySystem;
impl VelocitySystem {
  pub fn run(manager: &mut EntityManager, delta: Millis) {
    for entity in EntityManager::entity_iter(&manager.entities, FLAG_VELOCITY) {
      let velocity = manager.velocities[entity];
      let delta = delta.as_seconds();
      // Update component.position
      manager.positions[entity].0 += velocity.linear * delta;

      // Update component.rotation
      if manager.entities[entity].contains(FLAG_ROTATION) {
        let angle = velocity.angular.0.angle()*delta;
        let axis  = velocity.angular.0.axis().unwrap();

        manager.rotations[entity].0 *= UnitQuaternion::from_axis_angle(&axis, angle)
      } else if velocity.angular.0.angle() != 0.0 {
        // If rotation isn't enabled log a warning
        // TODO: Rate-limit
        println!("Entity {}: Got angular rotation but FLAG_ROTATION isn't set", entity);
      }
    }
  }
}

pub struct CameraSystem;
impl CameraSystem {
  pub fn run(manager: &mut EntityManager, _delta: Millis) {
    for entity in EntityManager::entity_iter(&manager.entities, FLAG_CAMERA) {
      let ref mut camera = manager.cameras[entity];

      if let Some(target) = camera.tracking {
        camera.target = manager.positions[target].0;
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

impl Default for Bob {
  fn default() -> Self {
    Bob::new(Millis(0.0), na::zero())
  }
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
    for entity in EntityManager::entity_iter(&manager.entities, FLAG_BOB) {
      let ref mut bob = manager.bobs[entity];

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
      manager.positions[entity].0 += pd;
    }
  }
}
