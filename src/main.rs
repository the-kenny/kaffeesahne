#[macro_use] extern crate glium;
extern crate image;
extern crate nalgebra;
extern crate tobj;

use std::collections::HashMap;
use std::path::Path;
use nalgebra::{Vector3, Vector4, UnitQuaternion, Matrix3, Matrix4, Transpose, PerspectiveMatrix3, Isometry3, Point3};
use nalgebra as na;
use glium as gl;

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

impl Default for Transform {
  fn default() -> Self {
    Transform {
      pos: na::zero(),
      rot: na::one(),
      scale: na::one(),
    }
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

struct BufferedMesh {
  positions: gl::VertexBuffer<Vertex>,
  normals:   gl::VertexBuffer<Normal>,
  indices:   gl::index::IndexBuffer<u32>,
}

struct ResourceManager {
  meshes:   HashMap<&'static str, BufferedMesh>,
  programs: HashMap<&'static str, gl::Program>,
}

impl ResourceManager {
  fn new() -> Self {
    ResourceManager {
      meshes:   HashMap::new(),
      programs: HashMap::new(),
    }
  }

  fn compile_shader<P: AsRef<Path>>(&mut self,
                                    display: &gl::Display,
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

    let program = gl::Program::from_source(display,
                                           &vertex_src,
                                           &fragment_src,
                                           None).unwrap();
    self.programs.insert(name, program);
  }

  fn load_obj<P>(&mut self,
                 display: &gl::Display,
                 name: &'static str,
                 path: P)
    where P: AsRef<Path>+std::fmt::Display {
    println!("Loading {} from {}", name, path);
    
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
    let normals:  Vec<_> = normals.into_iter().map(Normal::from).collect();
    
    let positions = gl::VertexBuffer::new(display, &vertices).unwrap();
    let normals   = gl::VertexBuffer::new(display, &normals).unwrap();
    let indices   = gl::index::IndexBuffer::new(display, gl::index::PrimitiveType::TrianglesList, &indices).unwrap();

    self.meshes.insert(name, BufferedMesh {
      positions: positions,
      normals: normals,
      indices: indices,
    });
  }
}

struct WorldUniforms {
  projection_matrix: Matrix4<f32>,
  view_matrix:       Matrix4<f32>,
  light_position:    Point3<f32>,
}

#[derive(Copy, Clone)]
struct Mesh {
  geometry: &'static str,
  program:  &'static str,
}

impl UpdateComponent for Mesh {}

trait UpdateComponent {
  fn update(&mut self, _parent: &mut GameObject, t: f32) {}
}

#[derive(Copy, Clone)]
struct Bob {
  direction: Vector3<f32>,
  period: f32,
  delta: f32,
}

impl UpdateComponent for Bob {
  fn update(&mut self, parent: &mut GameObject, t: f32) {
    parent.transform.pos += self.direction*((t+self.delta)/self.period).sin();
  }
}

#[derive(Copy, Clone)]
enum Component {
  Geometry(Mesh),               // Index 0
  Bob(Bob),                     // Index 1
  Empty,                        // Empty Marker
}

impl UpdateComponent for Component {
  fn update(&mut self, parent: &mut GameObject, t: f32) {
    use Component::*;
    match *self {
      Geometry(ref mut geometry) => geometry.update(parent, t),
      Bob(ref mut bob) => bob.update(parent, t),
      _ => (),
    }
  }
}

// TODO: Keeping those numbers in sync is annoying
#[derive(Copy, Clone)]
enum ComponentType {
  Geometry = 0,
  Bob = 1,
}

#[derive(Copy, Clone)]
struct ComponentStore([Component; 2]);

use std::ops::{Index, IndexMut};
impl Index<ComponentType> for ComponentStore {
  type Output = Component;
  fn index(&self, idx: ComponentType) -> &Self::Output {
    &self.0[idx as usize]
  }
}

impl IndexMut<ComponentType> for ComponentStore {
  fn index_mut(&mut self, idx: ComponentType) -> &mut Self::Output {
    &mut self.0[idx as usize]
  }
}


impl Default for ComponentStore {
  fn default() -> Self {
    ComponentStore([Component::Empty; 2])
  }
}

impl From<Vec<Component>> for ComponentStore {
  fn from(other: Vec<Component>) -> Self {
    let mut store = Self::default();
    for c in other.into_iter() {
      let idx = match c {
        Component::Geometry(_) => Some(ComponentType::Geometry),
        Component::Bob(_)      => Some(ComponentType::Bob),
        Component::Empty       => None,
      };
      idx.map(|idx| store[idx] = c);
    }
    store
  }
}

#[derive(Copy, Clone)]
struct GameObject {
  components: ComponentStore,
  transform: Transform,
}

impl GameObject {
  fn update(&mut self, t: f32) {
    let mut new = self.clone();
    for component in self.components.0.iter_mut() {
      component.update(&mut new, t);
    }
    self.transform = new.transform;
  }
}

impl GameObject {
  fn draw<S: gl::Surface>(&self,
                          target: &mut S,
                          resources: &ResourceManager,
                          world_uniforms: &WorldUniforms) {
    if let Component::Geometry(ref mesh) = self.components[ComponentType::Geometry] {
      let view_mat       = world_uniforms.view_matrix;
      let model_mat      = self.transform.as_matrix();
      let model_view_mat = view_mat * model_mat;
      let normal_mat     = na::inverse(&matrix3_from_matrix4(&(model_mat))).unwrap();

      let world_uniforms = uniform! {
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
                  &world_uniforms,
                  &params)
        .unwrap();
    }
  }
}
use std::env;
fn main() {
  use glium::{DisplayBuild, Surface};
  let display = gl::glutin::WindowBuilder::new()
    .with_depth_buffer(24)
    .build_glium().unwrap();

  let mut resources = ResourceManager::new();
  resources.load_obj(&display, "terrain", "terrain.obj");
  resources.load_obj(&display, "cube", "cube.obj");
  resources.compile_shader(&display,
                           "basic",
                           "src/shaders/basic.vertex.glsl",
                           "src/shaders/basic.fragment.glsl");

  let terrain = {
    let geometry = Mesh {
      geometry: "terrain",
      program:  "basic",
    };

    let scale = 0.5;
    let transform = Transform {
      pos: Vector3::new(0.0, -0.5, 0.0),
      rot: na::one(),
      scale: Vector3::new(1.0, 1.0, 1.0)*scale,
    };

    
    GameObject {
      components: vec![Component::Geometry(geometry)].into(),
      transform: transform,
    }
  };

  let mut objects = vec![terrain];

  // Generate some cubes
  for n in 0..50 {
    let geometry = Mesh {
      geometry: "cube",
      program:  "basic",
    };

    let n = n as f32;
    let bob = Bob {
      direction: Vector3::new(0.0, 0.10, 0.0),
      period: 20.0,
      delta: n*11.0,
    };

    let scale = 0.1;
    let transform = Transform {
      pos: Vector3::new(n*3.0 - 50.0, 0.5, 0.0),
      rot: na::one(),
      scale: Vector3::new(1.0, 1.0, 1.0)*scale,
    };

    let obj = GameObject {
      components: vec![Component::Geometry(geometry), Component::Bob(bob)].into(),
      transform: transform,
    };
    objects.push(obj);
  };
  
  let mut t: f32 = 0.0;
  let mut x: f32 = 0.0;
  let mut y: f32 = 0.0;

  loop {
    let mut target = display.draw();
    target.clear_color_and_depth((0.2, 0.2, 0.2, 1.0), 1.0);

    let (width,height) = target.get_dimensions();

    t += 1.0;

    // Camera
    // let cam_radius = (x / width as f32) * 3.0;
    // println!("cam_radius: {}", cam_radius);
    let cam_radius = 3.0;
    let cam = Point3::new(0.0, 1.5, 3.0);
    let cam_target = objects[0].transform.pos.to_point();
    
    let light = Vector3::<f32>::new(-3.0, 1.0, 3.0);
    // let light = Vector3::<f32>::new(1.5, (t/50.0).sin()*2.0, (t/50.0).cos()*2.0);
    
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

    let world_uniforms = WorldUniforms {
      projection_matrix: projection_mat,
      view_matrix:       view_mat,
      light_position:    Point3::new(light.x, light.y, light.z),
    };

    for object in &mut objects {
      object.update(t);
      
      // TODO: Move to GameObject / Rotation-Component
      object.transform.rot = quat_rotate(t/200.0, &Vector3::new(0.0, 1.0, 0.0));
    }
    
    for object in objects.iter() {
      object.draw(&mut target,
                  &resources,
                  &world_uniforms);
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
impl<T: Copy> AsUniform<[T; 3]>      for Point3<T>  { }

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
