use nalgebra as na;
use glium as gl;

use super::components::*;
use super::render_system::*;
use super::{Millis, ResourceManager};

pub struct World {
  pub resources: ResourceManager,
  pub entities: EntityManager,

  // TODO: Make an Entity
  pub light:          na::Point3<f32>,
  pub mouse_position: Option<(u32, u32)>,

  render_system:   RenderSystem,
  picking_system:  PickingSystem,
}

impl World {
  pub fn new<F: gl::backend::Facade+Sized>(display: &F) -> Self {
    World {
      resources: ResourceManager::new(),
      entities: EntityManager::default(),

      render_system: RenderSystem::new(display),
      picking_system: PickingSystem::new(display, (800,600)),

      light: na::Point3::new(0.0, 0.0, 0.0),
      mouse_position: None,
    }
  }

  fn current_camera(&self) -> Option<EntityId> {
    self.entities.cameras.iter().next().map(|(id, _)| *id)
  }

  fn uniforms(&self, (width,height): (u32, u32)) -> WorldUniforms {
    let camera = self.current_camera()
      .expect("Scene doesn't contain a camera!");

    let camera_position = self.entities.positions[&camera].0.as_point();
    let camera_mat: na::Matrix4<f32> = na::to_homogeneous(
      &na::Isometry3::look_at_rh(&camera_position,
                                 &self.entities.cameras[&camera].target,
                                 &na::Vector3::new(0.0, 1.0, 0.0)));

    // Something is wrong here - perspective doesn't look right
    let projection_mat = {
      let ratio    = width as f32 / height as f32;
      let fov: f32 = 3.141592 / (360.0 / 75.0);
      let (znear, zfar) = (0.1, 1024.0);
      na::PerspectiveMatrix3::new(ratio, fov, znear, zfar).to_matrix()
    };

    WorldUniforms {
      projection_matrix: projection_mat,
      light_position:    self.light,
      camera_matrix:     camera_mat,
      camera_position:   *camera_position,
    }
  }

  pub fn toggle_wireframe(&mut self) {
    self.render_system.render_wireframe = !self.render_system.render_wireframe;
  }

  pub fn update(&mut self, delta: Millis) {
    // Update picked entity
    self.entities.picked_entity = self.mouse_position.and_then(|_| {
      self.picking_system.read_picking_buffer()
    });

    BobSystem::run(&mut self.entities, delta);
    VelocitySystem::run(&mut self.entities, delta);
    CameraSystem::run(&mut self.entities, delta);
  }

  pub fn draw<S>(&mut self, surface: &mut S)
    where S: gl::Surface {
    let surface_size = surface.get_dimensions();

    // Update PickingSystem's dimensions
    self.picking_system.prepare(surface_size);
    let mut picking_surface = self.picking_system.get_surface();

    let world_uniforms = self.uniforms(surface_size);

    self.render_system.render(&self.entities,
                              surface,
                              &mut picking_surface,
                              &self.resources,
                              &world_uniforms);

    if let Some(pos) = self.mouse_position {
      self.picking_system.update(pos);
    }
  }

  pub fn handle_events<I>(&mut self, events: I)
    where I: Iterator<Item=gl::glutin::Event> {
    use glium::glutin::*;
    for ev in events {
      match ev {
        Event::Closed => return,
        Event::MouseInput(ElementState::Pressed, _) => {
          // camera_idx = (camera_idx+1) % camera_positions.len();
          // let pos = camera_positions[camera_idx];
          // println!("camera: {:?}", pos);
          // world.entities.set_position(camera, pos);

          if let Some(e) = self.entities.picked_entity {
            println!("Entity: {}", e);
          }
        },
        Event::MouseMoved(x,y) => {
          self.mouse_position = Some((x as u32, y as u32));
        },
        Event::Focused(false) => {
          self.mouse_position = None;
        },
        // Toggle wireframe with `w`
        Event::KeyboardInput(ElementState::Pressed, 25, _) => {
          self.toggle_wireframe();
        }
        _ => (),
      }
    }
  }
}
