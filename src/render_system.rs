use glium as gl;
use glium::backend::Facade;

use super::*;

#[derive(Debug, Clone, Copy)]
#[allow(non_snake_case)]
struct Uniforms {
  pickingId:         u32,
  _padding1:         [u32; 3],
  modelMatrix:       [[f32; 4]; 4],
  normalMatrix:      [[f32; 4]; 4],
  // Material:          &'a Material,
  // diffuseTexture:    &'a gl::texture::SrgbTexture2d,
  // hasDiffuseTexture: bool,

  viewMatrix:        [[f32; 4]; 4],
  projectionMatrix:  [[f32; 4]; 4],
  lightPosition:     [f32; 3],
  _padding2:         u32,
  cameraPosition:    [f32; 3],
}

implement_uniform_block!(Uniforms,
                         pickingId,
                         modelMatrix,
                         normalMatrix,
                         viewMatrix,
                         projectionMatrix,
                         lightPosition,
                         cameraPosition);

pub struct RenderSystem {
  empty_texture: gl::texture::SrgbTexture2d,
  uniform_buffer: gl::uniforms::UniformBuffer<Uniforms>,
  pub render_wireframe: bool,
}

impl RenderSystem {
  pub fn new<F: Facade>(f: &F) -> Self {
    RenderSystem {
      empty_texture: gl::texture::SrgbTexture2d::empty(f, 0, 0).unwrap(),
      uniform_buffer: gl::uniforms::UniformBuffer::empty_dynamic(f).unwrap(),
      render_wireframe: false,
    }
  }

  pub fn render<S, PS>(&mut self,
                       manager: &EntityManager,
                       surface: &mut S,
                       picking_surface: &mut PS,
                       // TODO: Pass via `World`
                       resources: &ResourceManager,
                       world_uniforms: &WorldUniforms)
    where S: gl::Surface, PS: gl::Surface {
    // TODO: Pull out somewhere
    let mut params = gl::DrawParameters {
      depth: gl::Depth {
        test: gl::draw_parameters::DepthTest::IfLess,
        write: true,
        ..Default::default()
      },
      ..Default::default()
    };

    if self.render_wireframe {
      params.polygon_mode = gl::PolygonMode::Line;
    }
    
    // Clear Buffers
    picking_surface.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
    let picking_program = &resources.programs["picking"];

    surface.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

    // Update the `world` uniforms (once per frame)
    {
      let mut x = self.uniform_buffer.map();
      // x.viewMatrix       = world_uniforms.camera_matrix.as_uniform();
      // x.projectionMatrix = world_uniforms.projection_matrix.as_uniform();
      // x.lightPosition    = world_uniforms.light_position.as_uniform();
      // x.cameraPosition   = world_uniforms.camera_position.as_uniform();
      x.viewMatrix       = world_uniforms.camera_matrix.as_uniform();
      x.projectionMatrix = world_uniforms.projection_matrix.as_uniform();
      x.lightPosition    = world_uniforms.light_position.as_uniform();
      x.cameraPosition   = world_uniforms.camera_position.as_uniform();
    }
    
    // Iterate over all entities with geometries
    for entity in EntityManager::entity_iter(&manager.entities, FLAG_GEOMETRY) {
      let g = manager.geometries[entity];
      let flags = manager.entities[entity];
      
      let p = manager.positions[entity];
      let pickable_id = if flags.contains(FLAG_PICKABLE) {
        Some(manager.pickables[entity])
      } else {
        None
      };
      

      let model_mat = {
        let mut m = p.as_matrix();
        if flags.contains(FLAG_ROTATION) {
          m *= manager.rotations[entity].as_matrix();
        }
        if flags.contains(FLAG_SCALE) {
          m *= manager.scales[entity].as_matrix();
        }
        m
      };


      let normal_mat = model_mat; // No idea why this doesn't need inverse()

      let ref program = resources.programs[&g.program];

      for (_name, mesh) in resources.meshes[&g.geometry].meshes.iter() {
        let texture = mesh.texture.as_ref().and_then(|name| {
          resources.textures.get(name)
        }).unwrap_or(&self.empty_texture);

        // Update `model` uniforms, once per draw-call
        {
          let mut x = self.uniform_buffer.map();
          x.pickingId = if flags.contains(FLAG_PICKABLE) {
            manager.pickables[entity].0
          } else {
            0
          };
          x.modelMatrix = model_mat.as_uniform();
          x.normalMatrix = normal_mat.as_uniform();
        }

        let uniforms = uniform! {
          Uniforms:          &*self.uniform_buffer,
          Material:          &mesh.material,
          diffuseTexture:    texture,
          hasDiffuseTexture: mesh.texture.is_some(),
        };

        surface.draw((&mesh.positions, &mesh.normals),
                     &mesh.indices,
                     program,
                     &uniforms,
                     &params)
          .unwrap();
        
        if pickable_id.is_some() {
          picking_surface.draw(&mesh.positions,
                               &mesh.indices,
                               &picking_program,
                               &uniforms,
                               &params)
            .unwrap();
        }
      }
    }


    // Render axis system
    {
      let uniforms = uniform! {
        projectionMatrix: world_uniforms.projection_matrix.as_uniform(),
        viewMatrix:       world_uniforms.camera_matrix.as_uniform(),
      };

      let ref buffers = resources.meshes["axis"].meshes["axis"];
      let ref program = resources.programs["axis"];
      surface.draw(&buffers.positions,
                   &buffers.indices,
                   program,
                   &uniforms,
                   &gl::DrawParameters::default())
        .unwrap();
    }
  }
}
