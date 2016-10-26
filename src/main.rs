#[macro_use] extern crate glium;
extern crate image;
extern crate nalgebra;
extern crate tobj;

use std::collections::HashMap;
use std::path::Path;
use nalgebra::{Vector3, Vector4, UnitQuaternion, Matrix3, Matrix4, Transpose, PerspectiveMatrix3, Isometry3, Point3};
use nalgebra as na;

mod storage_manager;

#[derive(Copy, Clone)]
struct Transform {
  pos:   Vector3<f32>,
  rot:   UnitQuaternion<f32>,
  scale: Vector3<f32>,
}

impl Transform {
  fn as_matrix(&self) -> Matrix4<f32> {
    let mut model: Matrix4<f32> = na::one();
    model *= Matrix4::new(self.scale.x, 0.0,          0.0,          0.0,
                          0.0,          self.scale.y, 0.0,          0.0,
                          0.0,          0.0,          self.scale.z, 0.0,
                          0.0,          0.0,          0.0,          1.0);
    model *= na::to_homogeneous(self.rot.to_rotation_matrix().submatrix());
    model *= Matrix4::new(1.0, 0.0, 0.0, self.pos.x,
                          0.0, 1.0, 0.0, self.pos.y,
                          0.0, 0.0, 1.0, self.pos.z,
                          0.0, 0.0, 0.0, 1.0);
    model
  }
}


#[derive(Clone, Copy)]
struct Vertex { position: [f32; 3] }
implement_vertex!(Vertex, position);

impl From<Vector3<f32>> for Vertex {
  fn from(v: Vector3<f32>) -> Self {
    Vertex {
      position: [v.x, v.y, v.z]
    }    
  }
}

#[derive(Clone, Copy)]
struct Normal { normal: [f32; 3] }
implement_vertex!(Normal, normal);

impl From<Vector3<f32>> for Normal {
  fn from(v: Vector3<f32>) -> Self {
    Normal {
      normal: [v.x, v.y, v.z]
    }
  }
}

struct Mesh {
  vertices: Vec<Vertex>,
  indices:  Vec<u32>,
  normals:  Vec<Normal>,
}

impl Mesh {
  fn load_from_file<P: AsRef<Path>>(path: P) -> Self {
    let obj = tobj::load_obj(path.as_ref());
    let (models, _materials) = obj.unwrap();

    let model = &models[0];
    println!("model.name = {}", model.name);

    let mesh = &model.mesh;
    assert!(mesh.positions.len() % 3 == 0);

    let indices = mesh.indices.clone();

    let mut vertices = Vec::with_capacity(mesh.positions.len()/3);
    for f in 0..mesh.positions.len() / 3 {
      let position = Vector3::new(mesh.positions[3 * f],
                                  mesh.positions[3 * f + 1],
                                  mesh.positions[3 * f + 2]);
      vertices.push(position);
    }


    let mut normals = vec![na::zero(); vertices.len()];

    if mesh.normals.len() > 0 {
      println!("Got normals in obj file");
      for f in 0..mesh.normals.len() / 3 {
        let normal = Vector3::new(mesh.normals[3 * f],
                                  mesh.normals[3 * f + 1],
                                  mesh.normals[3 * f + 2]);
        normals[f] = normal;
      }
    } else {
      println!("Calculating our own normals :-(");
      // Go over all Tris and calculate normals ourselves
      for f in 0..indices.len()/3 {
        let idx1 = indices[3*f] as usize;
        let idx2 = indices[3*f+1] as usize;
        let idx3 = indices[3*f+2] as usize;

        let v1 = vertices[idx1];
        let v2 = vertices[idx2];
        let v3 = vertices[idx3];
        let normal  = na::normalize(&na::cross(&(v2-v1), &(v3-v1)));

        normals[idx1] = normal;
        normals[idx2] = normal;
        normals[idx3] = normal;
      }
    }

    println!("vertices.len: {}", vertices.len());
    println!("indices.len: {}", indices.len());
    println!("normals.len: {}", normals.len());

    Mesh {
      vertices: vertices.into_iter().map(Vertex::from).collect(),
      indices:  indices,
      normals:  normals.into_iter().map(Normal::from).collect(),
    }
  }

  fn draw<S, U>(&self,
                program: &glium::Program,
                frame: &mut S,
                mesh: &BufferedMesh,
                uniforms: &U) -> ()
    where S: glium::Surface, U: glium::uniforms::Uniforms {
    // let positions = glium::VertexBuffer::new(display, &self.vertices).unwrap();
    // let normals   = glium::VertexBuffer::new(display, &self.normals).unwrap();
    // let indices   = glium::index::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &self.indices).unwrap();

    let params = glium::DrawParameters {
      depth: glium::Depth {
        test: glium::draw_parameters::DepthTest::IfLess,
        write: true,
        .. Default::default()
      },
      .. Default::default()
    };

    frame.draw((&mesh.positions, &mesh.normals),
               &mesh.indices,
               &program,
               uniforms,
               &params)
      .unwrap();
  }
}

struct BufferedMesh {
  positions: glium::VertexBuffer<Vertex>,
  normals:   glium::VertexBuffer<Normal>,
  indices:   glium::index::IndexBuffer<u32>,
}

struct ResourceManager {
  meshes:   HashMap<&'static str, BufferedMesh>,
  programs: HashMap<&'static str, glium::Program>,
}

impl ResourceManager {
  fn new() -> Self {
    ResourceManager {
      meshes:   HashMap::new(),
      programs: HashMap::new(),
    }
  }

  fn compile_shader<P: AsRef<Path>>(&mut self,
                                    display: &glium::Display,
                                    name: &'static str,
                                    vertex: P,
                                    fragment: P) {
    use std::fs::File;
    use std::io::Read;
    let vertex_src = {
      let mut f = File::open(vertex).unwrap();
      let mut src = String::new();
      f.read_to_string(&mut src).unwrap();
      src
    };

    let fragment_src = {
      let mut f = File::open(fragment).unwrap();
      let mut src = String::new();
      f.read_to_string(&mut src).unwrap();
      src
    };

    let program = glium::Program::from_source(display,
                                              &vertex_src,
                                              &fragment_src,
                                              None).unwrap();
    self.programs.insert(name, program);
  }

  fn load_obj<P: AsRef<Path>>(&mut self,
                              display: &glium::Display,
                              name: &'static str,
                              path: P) {
    let obj = tobj::load_obj(path.as_ref());
    let (models, _materials) = obj.unwrap();

    // TODO: Use model.name for our name
    let model = &models[0];
    println!("model.name = {}", model.name);

    let mesh = &model.mesh;
    assert!(mesh.positions.len() % 3 == 0);

    let indices = mesh.indices.clone();

    let mut vertices = Vec::with_capacity(mesh.positions.len()/3);
    for f in 0..mesh.positions.len() / 3 {
      let position = Vector3::new(mesh.positions[3 * f],
                                  mesh.positions[3 * f + 1],
                                  mesh.positions[3 * f + 2]);
      vertices.push(position);
    }


    let mut normals = vec![na::zero(); vertices.len()];

    if mesh.normals.len() > 0 {
      println!("Got normals in obj file");
      for f in 0..mesh.normals.len() / 3 {
        let normal = Vector3::new(mesh.normals[3 * f],
                                  mesh.normals[3 * f + 1],
                                  mesh.normals[3 * f + 2]);
        normals[f] = normal;
      }
    } else {
      println!("Calculating our own normals :-(");
      // Go over all Tris and calculate normals ourselves
      for f in 0..indices.len()/3 {
        let idx1 = indices[3*f] as usize;
        let idx2 = indices[3*f+1] as usize;
        let idx3 = indices[3*f+2] as usize;

        let v1 = vertices[idx1];
        let v2 = vertices[idx2];
        let v3 = vertices[idx3];
        let normal  = na::normalize(&na::cross(&(v2-v1), &(v3-v1)));

        normals[idx1] = normal;
        normals[idx2] = normal;
        normals[idx3] = normal;
      }
    }

    println!("vertices.len: {}", vertices.len());
    println!("indices.len: {}", indices.len());
    println!("normals.len: {}", normals.len());

    let vertices: Vec<_> = vertices.into_iter().map(Vertex::from).collect();
    let normals:   Vec<_> = normals.into_iter().map(Normal::from).collect();
    
    let positions = glium::VertexBuffer::new(display, &vertices).unwrap();
    let normals   = glium::VertexBuffer::new(display, &normals).unwrap();
    let indices   = glium::index::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &indices).unwrap();

    self.meshes.insert(name, BufferedMesh {
      positions: positions,
      normals: normals,
      indices: indices,
    });
  }
}

// enum ComponentType {
//   Mesh,
// }

struct WorldUniforms {
  projection_matrix: Matrix4<f32>,
  view_matrix:       Matrix4<f32>,
  light_position:    [f32; 3],  // TODO
}

struct GameObject {
  mesh: (&'static str, &'static str),       // TODO
  transform: Transform,
}

impl GameObject {
  fn draw<S: glium::Surface>(&self,
                             target: &mut S,
                             resources: &ResourceManager,
                             world_uniforms: &WorldUniforms) {

    let view_mat = world_uniforms.view_matrix;
    let model_mat = self.transform.as_matrix();
    let model_view_mat = view_mat * model_mat;
    let normal_mat = na::inverse(&matrix3_from_matrix4(&(model_mat))).unwrap();
    let projection_mat = world_uniforms.projection_matrix;

    let uniforms = uniform! {
      modelMatrix:      model_mat.as_uniform(),
      projectionMatrix: projection_mat.as_uniform(),
      viewMatrix:       view_mat.as_uniform(),
      modelViewMatrix:  model_view_mat.as_uniform(),
      normalMatrix:     normal_mat.as_uniform(),
      lightPosition:    world_uniforms.light_position,
    };

    // TODO: Pull out somewhere
    let params = glium::DrawParameters {
      depth: glium::Depth {
        test: glium::draw_parameters::DepthTest::IfLess,
        write: true,
        .. Default::default()
      },
      .. Default::default()
    };

    let ref mesh = resources.meshes[&self.mesh.0];
    let ref program = resources.programs[&self.mesh.1];
    target.draw((&mesh.positions, &mesh.normals),
                &mesh.indices,
                program,
                &uniforms,
                &params)
      .unwrap();
  }
}

// impl GameObject {
//   fn draw(&self, surface: &glium::Surface, ) {

//   }
// }

use std::env;
fn main() {
  let filename = env::args().skip(1).next().unwrap();

  use glium::{DisplayBuild, Surface};
  let display = glium::glutin::WindowBuilder::new()
    .with_depth_buffer(24)
    .build_glium().unwrap();

  let mut resources = ResourceManager::new();
  resources.load_obj(&display, "object", filename);
  resources.compile_shader(&display,
                           "object",
                           "src/shaders/basic.vertex.glsl",
                           "src/shaders/basic.fragment.glsl");

  let mut t: f32 = 0.0;
  let mut x: f32 = 0.0;
  let mut y: f32 = 0.0;
  let mut draw_object = true;

  loop {
    let mut target = display.draw();
    target.clear_color_and_depth((0.2, 0.2, 0.2, 1.0), 1.0);

    let (width,height) = target.get_dimensions();

    t += 1.0;

    let params = glium::DrawParameters {
      depth: glium::Depth {
        test: glium::draw_parameters::DepthTest::IfLess,
        write: true,
        .. Default::default()
      },
      .. Default::default()
    };

    // Camera
    // let cam_radius = (x / width as f32) * 3.0;
    // println!("cam_radius: {}", cam_radius);
    let cam_radius = 3.0;
    let cam = Point3::new(0.0, 1.5, 3.0);
    let cam_target = Point3::new(0.0, 0.0, 0.0);
    
    // let light = Vector3::<f32>::new(-2.0, 1.0, 3.0);
    let light = Vector3::<f32>::new(1.5, (t/50.0).sin()*2.0, (t/50.0).cos()*2.0);
    
    let view_mat: Matrix4<f32> = na::to_homogeneous(
      &Isometry3::look_at_rh(&cam,
                             &cam_target,
                             &Vector3::new(0.0, 1.0, 0.0)));

    let projection_mat: Matrix4<f32> = {
      let ratio    = width as f32 / height as f32;
      let fov: f32 = 3.141592 / 3.0;
      let zfar     = 1024.0;
      let znear    = 0.1;

      PerspectiveMatrix3::new(ratio, fov, znear, zfar).to_matrix()
    };

    // let light = cam;

    // Unit Vectors
    //    {
    //       let vertex_src = r##"
    // #version 140

    // in vec3 position;

    // flat out vec3 direction;

    // uniform mat4 projectionMatrix;
    // uniform mat4 modelViewMatrix;

    // void main() {
    //   direction = position;
    //   gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
    // }
    // "##;

    //       let fragment_src = r##"
    // #version 140

    // flat in vec3 direction;
    // out vec4 color;

    // void main() {
    //   color = vec4(direction, 1.0);
    // }
    // "##;

    //       let program = glium::Program::from_source(&display,
    //                                                 &vertex_src,
    //                                                 &fragment_src,
    //                                                 None).unwrap();

    //       for dir in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]].into_iter() {
    //         let vertices: Vec<Vertex> = [[0.0; 3], *dir].into_iter()
    //           .map(|d| Vertex {
    //             position: *d,
    //             normal:  [0.0; 3],
    //             texture: [0.0; 2]
    //           }).collect();

    //         let positions = glium::VertexBuffer::new(&display, &vertices).unwrap();
    //         let indices = glium::index::NoIndices(glium::index::PrimitiveType::LinesList);

    //         let model_mat: Matrix4<f32> = (Transform {
    //           pos: Vector3::new(0.0, 0.0, 0.0),
    //           rot: na::one(),
    //           scale: na::one(),
    //         }).as_matrix();

    //         let model_view_mat: Matrix4<f32> = view_mat * model_mat;

    //         let uniforms = uniform! {
    //           modelMatrix:      (model_mat).as_uniform(),
    //           projectionMatrix: (projection_mat).as_uniform(),
    //           viewMatrix:       (view_mat).as_uniform(),

    //           modelViewMatrix:  (model_view_mat).as_uniform(),
    //         };

    //         target.draw((&positions),
    //                     &indices,
    //                     &program,
    //                     &uniforms,
    //                     &params)
    //           .unwrap();
    //       }
    //     }

    // Object
    if draw_object {
      // let mut scale = x / width as f32;
      // if !scale.is_normal() || scale < 0.01 {
      //   scale = 1.0;
      // }

      let scale = 0.5;

      let mut transform = Transform {
        pos: Vector3::new(0.0, -0.5, 0.0),
        rot: UnitQuaternion::from_axisangle(na::Unit::new(&Vector3::new(0.0, 1.0, 0.0)),
                                            0.0),
        scale: na::one::<Vector3<f32>>()*scale,
      };

      transform.rot = quat_rotate(t/200.0, &Vector3::new(0.0, 1.0, 0.0));
      // transform.pos.x = x*5.0 / width as f32;
      // transform.pos.z = y*5.0 / width as f32;
      // let scale = t/500.0;
      // transform.scale = Vector3::new(scale, scale, scale);


      let object = GameObject {
        mesh: ("object", "object"),
        transform: transform,
      };
      
      let model_mat = transform.as_matrix();
      let model_view_mat = view_mat * model_mat;
      let normal_mat = na::inverse(&matrix3_from_matrix4(&(model_mat))).unwrap();

      let uniforms = WorldUniforms {
        projection_matrix: projection_mat,
        view_matrix:       view_mat,
        light_position:    [light.x, light.y, light.z], // TODO
      };
      
      object.draw(&mut target,
                  &resources,
                  &uniforms);
    }

    target.finish().unwrap();

    for ev in display.poll_events() {
      println!("{:?}", ev);
      use glium::glutin::*;
      match ev {
        Event::Closed => return,
        Event::MouseMoved(xx,yy) => {
          x = xx as f32;
          y = yy as f32;
        },
        Event::MouseInput(ElementState::Pressed, MouseButton::Left) => {
          draw_object = !draw_object;
        },
        Event::MouseInput(_, MouseButton::Right) => {
          t = 0.0;
        }
        _ => (),
      }
    }
  }
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

// fn matrix3_as_array<T: Copy+Clone>(m: &Matrix3<T>) -> [[T; 3]; 3] {
//   m.as_ref().clone()
// }


fn matrix3_from_matrix4<T: Copy>(m: &Matrix4<T>) -> Matrix3<T> {
  Matrix3::new(m[(0,0)], m[(1,0)], m[(2,0)],
               m[(0,1)], m[(1,1)], m[(2,1)],
               m[(0,2)], m[(1,2)], m[(2,2)])
}

fn quat_rotate<T: Copy+na::BaseFloat>(angle: T, axis: &Vector3<T>) -> UnitQuaternion<T> {
  UnitQuaternion::from_axisangle(na::Unit::new(axis), angle)
}
