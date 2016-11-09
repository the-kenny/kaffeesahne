use std::collections::{BTreeMap, BTreeSet, HashMap};
use glium as gl;

use super::*;

type EntityId = u64;
type EntityMap<K,V> = BTreeMap<K,V>;

// Basic Object Attributes

#[derive(Debug, Copy, Clone)]
pub struct Position(pub Vector3<f32>);

#[derive(Debug, Copy, Clone)]
pub struct Scale(pub Vector3<f32>);

#[derive(Debug, Copy, Clone)]
pub struct Rotation(pub na::UnitQuaternion<f32>);

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
pub struct Pickable(pub u32);

#[derive(Debug)]
pub struct Camera {
  pub target: Point3<f32>,
  pub tracking: Option<EntityId>,
}

pub trait IComponent {
  fn cname(&self) -> Component;
}

macro_rules! components {
  {$($name:ident, )*} => {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Component {
      $($name, )*
    }

    $(impl IComponent for $name {
      fn cname(&self) -> Component { Component::$name }
    })*
  }
}

components! {
  Position,
  Scale,
  Rotation,
  Velocity,
  Geometry,
  Pickable,
  Camera,
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
}

impl EntityManager {
  pub fn entities(&self, component: Component) -> Vec<EntityId> {
    use self::Component::*;
    match component {
      Position => self.positions.keys().cloned().collect(),
      Scale    => self.scales.keys().cloned().collect(),
      Rotation => self.rotations.keys().cloned().collect(),
      Velocity => self.velocities.keys().cloned().collect(),
      Geometry => self.geometries.keys().cloned().collect(),
      Pickable => self.pickables.keys().cloned().collect(),
      Camera   => self.cameras.keys().cloned().collect(),
    }
  }

  pub fn new_entity(&mut self) -> EntityId {
    self.highest_id += 1;
    self.entities.insert(self.highest_id);
    self.highest_id
  }

  pub fn add_geometry(&mut self, entity: EntityId, g: Geometry) {
    self.geometries.insert(entity, g);
  }

  pub fn set_position(&mut self, entity: EntityId, p: Position) {
    self.positions.insert(entity, p);
  }

  pub fn add_camera(&mut self, entity: EntityId, camera: Camera) {
    self.cameras.insert(entity, camera);
  }

  pub fn set_scale(&mut self, entity: EntityId, scale: Scale) {
    self.scales.insert(entity, scale);
  }

  pub fn set_rotation(&mut self, entity: EntityId, rot: Rotation) {
    self.rotations.insert(entity, rot);
  }

  pub fn set_pickable(&mut self, entity: EntityId, enable: bool) {
    assert!(entity <= u32::max_value() as u64);
    if enable {
      self.pickables.insert(entity, Pickable(entity as u32));
    } else {
      self.pickables.remove(&entity);
    }
  }
}

struct Picking {
  texture: gl::texture::UnsignedTexture2d,
  depth: gl::framebuffer::DepthRenderBuffer,
  pbo: gl::texture::pixel_buffer::PixelBuffer<u32>,
  size: (u32, u32),
}

impl Picking {
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

  fn new<F: gl::backend::Facade+Sized>(display: &F, size: (u32, u32)) -> Self {
    let (color, depth) = Self::new_buffers(display, size);
    Picking {
      texture: color,
      depth: depth,
      pbo: gl::texture::pixel_buffer::PixelBuffer::new_empty(display, 1),
      size: size,
    }
  }

  fn prepare(&mut self, size: (u32, u32)) {
    if self.size != size {
      println!("Updating size from {:?} to {:?}", self.size, size);

      self.size = size;
      let (color, depth) = Self::new_buffers(self.pbo.get_context(), size);
      self.texture = color;
      self.depth = depth;
    }
  }

  fn get_surface(&self) -> gl::framebuffer::SimpleFrameBuffer {
    let facade = self.pbo.get_context();
    gl::framebuffer::SimpleFrameBuffer::with_depth_buffer(facade,
                                                          &self.texture,
                                                          &self.depth)
      .unwrap()
  }
}

use std::cell::RefCell;
pub struct RenderSystem {
  pub pick: Option<EntityId>,
  picking: RefCell<Picking>
}

impl RenderSystem {
  fn new<F: gl::backend::Facade>(display: &F) -> Self {
    let size = (800,600);

    RenderSystem {
      pick: None,
      picking: Picking::new(display, size).into()
    }
  }

  fn render<S: gl::Surface>(&self,
                            manager: &EntityManager,
                            surface: &mut S,
                            // TODO: Pass via `World`
                            resources: &ResourceManager,
                            world_uniforms: &WorldUniforms) {
    use glium::Surface;
    let mut picking = self.picking.borrow_mut();
    picking.prepare(surface.get_dimensions());
    let mut picking_surface = picking.get_surface();
    let picking_program = &resources.programs["picking"];

    // Clear Buffers
    surface.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
    picking_surface.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

    // Iterate over all entities with geometries
    for (entity, g) in manager.geometries.iter() {
      let p = manager.positions[entity];
      let pickable_id = manager.pickables.get(entity);

      let model_mat = {
        let rot = manager.rotations.get(entity).map(|x| *x).unwrap_or(Rotation(na::one()));
        let sca = manager.scales.get(entity).map(|x| *x).unwrap_or(Scale(na::one()));
        (Transform {
          pos: p.0,
          rot: rot.0,
          scale: sca.0,
        }).as_matrix()
      };

      let view_mat = world_uniforms.view_matrix;
      let normal_mat = na::inverse(&matrix3_from_matrix4(&(model_mat))).unwrap();

      let uniforms = uniform! {
        picking_id:       pickable_id.map(|&Pickable(id)| id).unwrap_or(0),
        modelMatrix:      model_mat.as_uniform(),
        projectionMatrix: world_uniforms.projection_matrix.as_uniform(),
        viewMatrix:       view_mat.as_uniform(),
        modelViewMatrix:  (view_mat * model_mat).as_uniform(),
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

      let ref buffers = resources.meshes[&g.geometry];
      let ref program = resources.programs[&g.program];
      surface.draw((&buffers.positions, &buffers.normals),
                   &buffers.indices,
                   program,
                   &uniforms,
                   &params)
        .unwrap();

      if pickable_id.is_some() {
        picking_surface.draw(&buffers.positions,
                             &buffers.indices,
                             &picking_program,
                             &uniforms,
                             &params)
          .unwrap();
      }
    }
  }

  fn update_picked_object(&mut self) {
    // Copy the picking_vbo into main memory and read its value
    // TODO
    self.pick = self.picking.borrow().pbo.read().ok().and_then(|px| {
      if px[0] > 0 { Some(px[0] as u64) } else { None }
    });
  }

  pub fn pick(&self, position: (u32, u32)) {
    let picking = self.picking.borrow();

    let (x,y) = position;
    let rect = gl::Rect {
      left: x as u32,
      bottom: picking.size.1 - y as u32 - 1,
      width: 1,
      height: 1
    };
    picking.texture.main_level()
      .first_layer()
      .into_image(None).unwrap()
      .raw_read_to_pixel_buffer(&rect, &picking.pbo);
  }
}

#[derive(Default)]
struct VelocitySystem;
impl VelocitySystem {
  // TODO: Move to trait
  fn run(&self, manager: &mut EntityManager, delta: Millis) {
    for entity in manager.entities(Component::Velocity) {
      if let Some(velocity) = manager.velocities.get(&entity) {
        let delta = delta.as_seconds();
        // Update component.position
        manager.positions.get_mut(&entity).unwrap().0 += velocity.linear * delta;

        // Update component.rotation
        let angle = velocity.angular.0.angle()*delta;
        let axis  = velocity.angular.0.axis().unwrap();

        use std::collections::btree_map::Entry::*;
        match manager.rotations.entry(entity) {
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
}

struct CameraSystem;
impl CameraSystem {
  fn run(&self, manager: &mut EntityManager, _delta: Millis) {
    for camera in manager.entities(Component::Camera) {
      let camera = manager.cameras.get_mut(&camera).unwrap();
      if let Some(target) = camera.tracking {
        camera.target = manager.positions[&target].0.to_point();
      }
    }
  }
}

pub struct World {
  pub entities: EntityManager,
  velocity_system: VelocitySystem,
  camera_system: CameraSystem,
  pub render_system: RenderSystem, // TODO
}

impl World {
  pub fn new<F: gl::backend::Facade+Sized>(display: &F) -> Self {
    let render_system = RenderSystem::new(display);

    World {
      entities: EntityManager::default(),
      velocity_system: VelocitySystem,
      camera_system: CameraSystem,
      render_system: render_system,
    }
  }

  fn current_camera(&self) -> Option<EntityId> {
    self.entities.entities(Component::Camera).into_iter().next()
  }

  fn uniforms(&self, (width,height): (u32, u32)) -> WorldUniforms {
    let camera = self.current_camera()
      .expect("Scene doesn't contain a camera!");
    let light = Vector3::<f32>::new(-3.0, 1.0, 3.0);

    let view_mat: Matrix4<f32> = na::to_homogeneous(
      &Isometry3::look_at_rh(&self.entities.positions[&camera].0.as_point(),
                             &self.entities.cameras[&camera].target,
                             &Vector3::new(0.0, 1.0, 0.0)));

    // Something is wrong hree - perspective doesn't look right
    let projection_mat: Matrix4<f32> = {
      let ratio    = width as f32 / height as f32;
      let fov: f32 = 3.141592 / 3.0;
      let zfar     = 1024.0;
      let znear    = 0.1;

      PerspectiveMatrix3::new(ratio, fov, znear, zfar).to_matrix()
    };

    WorldUniforms {
      projection_matrix: projection_mat,
      view_matrix:       view_mat,
      light_position:    Point3::new(light.x, light.y, light.z),
    }
  }

  pub fn update(&mut self, delta: Millis) {
    self.render_system.update_picked_object();
    self.velocity_system.run(&mut self.entities, delta);
    self.camera_system.run(&mut self.entities, delta);
  }

  pub fn draw<S>(&self,
                 surface: &mut S,
                 // TODO: Pass via `World`
                 resources: &ResourceManager)
    where S: gl::Surface {
    let surface_size = surface.get_dimensions();
    let world_uniforms = self.uniforms(surface_size);

    self.render_system.render(&self.entities,
                              surface,
                              resources,
                              &world_uniforms);
  }
}
