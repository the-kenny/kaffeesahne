use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use super::*;

type EntityId = u64;

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
  Camera,
}


#[derive(Default)]
pub struct EntityManager {
  pub entities:   BTreeSet<EntityId>,
  highest_id:     EntityId,

  pub positions:  BTreeMap<EntityId, Position>,
  pub scales:     BTreeMap<EntityId, Scale>,
  pub rotations:  BTreeMap<EntityId, Rotation>,
  pub velocities: BTreeMap<EntityId, Velocity>,
  pub geometries: BTreeMap<EntityId, Geometry>,
  pub cameras:    BTreeMap<EntityId, Camera>,
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
}

#[derive(Default)]
struct RenderSystem;

use glium as gl;
impl RenderSystem {
  fn render<S: gl::Surface>(&self,
                            manager: &EntityManager,
                            surface: &mut S,
                            // TODO: Pass via `World`
                            resources: &ResourceManager,
                            world_uniforms: &WorldUniforms) {
    for entity in manager.entities.iter() {
      if let (Some(g), Some(p)) = (manager.geometries.get(entity),
                                   manager.positions.get(entity)) {
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
      }
    }
  }
}

#[derive(Default)]
struct VelocitySystem;
impl VelocitySystem {
  // TODO: Move to trait
  fn run(&self, manager: &mut EntityManager, delta: TimeDelta) {
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
          Vacant(entry)   => {
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
  fn run(&self, manager: &mut EntityManager, _delta: TimeDelta) {
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
  render_system: RenderSystem,

  time: Instant,
}

impl World {
  pub fn new() -> Self {
    World {
      entities: EntityManager::default(),
      velocity_system: VelocitySystem,
      camera_system: CameraSystem,
      render_system: RenderSystem,
      time: Instant::now(),
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

  pub fn update(&mut self) {
    let now = Instant::now();
    let delta: TimeDelta = {
      let delta = now - self.time;
      TimeDelta(delta.as_secs() as f32 * 1000.0
                +
                delta.subsec_nanos() as f32 / 1000000.0)
    };

    self.velocity_system.run(&mut self.entities, delta);
    self.camera_system.run(&mut self.entities, delta);

    self.time = now;
  }

  pub fn draw<S: gl::Surface>(&self,
                              surface: &mut S,
                              // TODO: Pass via `World`
                              resources: &ResourceManager) {
    let surface_size = surface.get_dimensions();
    let world_uniforms = self.uniforms(surface_size);
    self.render_system.render(&self.entities,
                              surface,
                              resources,
                              &world_uniforms);
  }
}
