#[macro_use] extern crate glium;
extern crate image;
extern crate nalgebra;
extern crate tobj;

use std::io::Read;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use nalgebra::{Vector3, Vector4, UnitQuaternion, Matrix3, Matrix4, Transpose, PerspectiveMatrix3, Isometry3, Point3};
use std::mem;

#[derive(Copy, Clone)]
struct Vertex {
  position: [f32; 3],
  normal:   [f32; 3],
  texture:  [f32; 2],
}

implement_vertex!(Vertex, position, normal, texture);

#[derive(Copy, Clone)]
struct Transform {
  pos: Vector3<f32>,
  rot: UnitQuaternion<f32>,
  scale: Vector3<f32>,
}

impl Transform {
  fn apply(&self, v: &Vertex) -> Vertex {
    let pos = self.as_matrix() * Vector4::new(v.position[0],
                                              v.position[1],
                                              v.position[2],
                                              1.0);
    
    Vertex {
      position: [pos.x, pos.y, pos.z],
      // TODO: Transform normal too? (Maybe just rotate? I have no idea)
      normal: v.normal,
      texture: v.texture,
    }
  }

  fn as_matrix(&self) -> Matrix4<f32> {
    let mut model: Matrix4<f32> = nalgebra::one();
    model *= Matrix4::new(self.scale.x, 0.0, 0.0, 0.0,
                          0.0, self.scale.y, 0.0, 0.0,
                          0.0, 0.0, self.scale.z, 0.0,
                          0.0, 0.0, 0.0,               1.0).transpose();
    model *= nalgebra::to_homogeneous(self.rot.to_rotation_matrix().submatrix()).transpose();
    model *= Matrix4::new(1.0, 0.0, 0.0, self.pos.x,
                          0.0, 1.0, 0.0, self.pos.y,
                          0.0, 0.0, 1.0, self.pos.z,
                          0.0, 0.0, 0.0, 1.0).transpose();
    model
  }
}

struct Renderable {
  vertices: Vec<Vertex>,
  transform: Transform,
}

const VERTEX_SHADER_SRC: &'static str = r##"
#version 140

in vec3 position;
in vec3 normal;

out vec3 fragNormal;
out vec3 fragVert;

uniform mat4 projectionMatrix;
uniform mat4 modelViewMatrix;

void main() {
  fragNormal = normal;
  fragVert = position;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}
"##;


const FRAGMENT_SHADER_SRC: &'static str = r##"
#version 140

in vec3 fragNormal;
in vec3 fragVert;

out vec4 color;

uniform vec3 lightPosition;
uniform mat3 normalMatrix;
uniform mat4 modelMatrix;

const vec3 lightColor = vec3(1.0, 1.0, 1.0);
const vec4 surfaceColor = vec4(1.0, 0.0, 0.0, 1.0);

void main() {
  vec3 normal = normalize(normalMatrix * fragNormal);
  vec3 fragPosition = vec3(modelMatrix * vec4(fragVert, 0.0));
  vec3 surfaceToLight = lightPosition - fragPosition;

  float brightness = dot(normal, surfaceToLight) / (length(surfaceToLight) * length(normal));
  brightness = clamp(brightness, 0, 1);
 
  color = vec4(brightness * lightColor * surfaceColor.rgb, surfaceColor.a);
}
"##;

fn main() {
  let obj = tobj::load_obj(&Path::new("teapot.obj"));
  let (models, materials) = obj.unwrap();

  let mut vertices: Vec<Vertex> = vec![];
  // let indices: Vec<u32> = vec![];

  {
    let model = &models[0];
    println!("model.name = {}", model.name);

    let mesh = &model.mesh;
    assert!(mesh.positions.len() % 3 == 0);

    // indices = mesh.indices.clone();
    
    let mut positions: Vec<Vector3<f32>> = vec![];
    for f in 0..mesh.positions.len() / 3 {
      let position = Vector3::new(mesh.positions[3 * f],
                                  mesh.positions[3 * f + 1],
                                  mesh.positions[3 * f + 2]);
      positions.push(position);
    }

    for f in 0..mesh.indices.len() / 3 {
      let face = [positions[mesh.indices[3 * f] as usize],
                  positions[mesh.indices[3 * f + 1] as usize],
                  positions[mesh.indices[3 * f + 2] as usize]];
      let normal = nalgebra::cross(&(face[1] - face[0]),
                                   &(face[2] - face[0]));

      for n in 0..3 {
        let p = face[n];
        vertices.push(Vertex {
          position: [p.x, p.y, p.z],
          normal: [normal.x, normal.y, normal.z],
          texture: [0.0; 2],
        });
      }
    }
  }

  use glium::{DisplayBuild, Surface};
  let display = glium::glutin::WindowBuilder::new()
    .with_depth_buffer(24)
    .build_glium().unwrap();

  let mut transform = Transform {
    pos: Vector3::new(0.0, -0.5, 0.0),
    rot: UnitQuaternion::from_axisangle(nalgebra::Unit::new(&Vector3::new(0.0, 1.0, 0.0)),
                                        0.0),
    scale: Vector3::new(0.5, 0.5, 0.5),
  };

  let positions = glium::VertexBuffer::new(&display, &vertices).unwrap();
  // let normals   = glium::VertexBuffer::new(&display, &indices).unwrap();
  // let indices   = glium::index::IndexBuffer::new(&display, glium::index::PrimitiveType::TrianglesList, &indices).unwrap();
  let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
  
  let program = glium::Program::from_source(&display,
                                            &VERTEX_SHADER_SRC,
                                            &FRAGMENT_SHADER_SRC,
                                            None).unwrap();

  let params = glium::DrawParameters {
    depth: glium::Depth {
      test: glium::draw_parameters::DepthTest::IfLess,
      write: true,
      .. Default::default()
    },
    .. Default::default()
  };
  
  let mut t: f32 = 0.0;
  let mut x: f32 = 0.0;
  let mut y: f32 = 0.0;
  
  loop {
    let mut target     = display.draw();
    target.clear_color_and_depth((0.2, 0.2, 0.2, 1.0), 1.0);
    
    let (width,height) = target.get_dimensions();

    t += 1.0;

    // transform.rot = UnitQuaternion::from_axisangle(nalgebra::Unit::new(&Vector3::new(0.0, 1.0, 0.0)),
    //                                                (x*2.0 / width as f32) * 3.141);
    // transform.pos.x = x*5.0 / width as f32;
    // transform.pos.z = y*-5.0 / width as f32;
    // let scale = t/500.0;
    // transform.scale = Vector3::new(scale, scale, scale);
    
    let projection_mat: Matrix4<f32> = {
      let ratio = height as f32 / width as f32;
      let fov: f32 = 3.141592 / 3.0;
      let zfar = 100.0;
      let znear = 1.0;

      if cfg!(feature="nalgebra_view") {
        let mut m = PerspectiveMatrix3::new(ratio, fov, znear, zfar).to_matrix();
        m = nalgebra::transpose(&m);
        m = nalgebra::inverse(&m).unwrap();
        m
      } else {
        let f = 1.0 / (fov / 2.0).tan();
        Matrix4::new(f *     ratio   ,    0.0,              0.0              ,   0.0,
                     0.0             ,     f ,              0.0              ,   0.0,
                     0.0             ,    0.0,  (zfar+znear)/(zfar-znear)    ,   1.0,
                     0.0             ,    0.0, -(2.0*zfar*znear)/(zfar-znear),   0.0)

      }
    };

    let radius = 3.0;
    // let cam = Vector3::new((x/100.0).sin()*radius, 0.0, (x/100.0).cos()*radius);
    let cam = Vector3::new((t/100.0).sin()*radius, 0.0, (t/100.0).cos()*radius);
    // let direction = transform.pos-cam;
    let direction = transform.pos-cam;
    println!("cam:       {:?}", cam);
    println!("direction: {:?}", direction);
    
    let view_mat = view_matrix(&cam,
                               &direction, 
                               &Vector3::new(0.0, 1.0, 0.0));
    let model_mat = transform.as_matrix();
    let model_view_mat = model_mat * view_mat;
    let normal_mat = nalgebra::transpose(&nalgebra::inverse(&matrix3_from_matrix4(&(model_mat))).unwrap());
    
    //let light = Vector3::<f32>::new(-2.0, 0.0, 0.0);
    let light = Vector3::<f32>::new(2.0, 1.0, 3.0);
    
    let uniforms = uniform! {
      modelMatrix: matrix4_as_array(&model_mat),
      projectionMatrix: matrix4_as_array(&projection_mat),
      viewMatrix: matrix4_as_array(&view_mat),
      
      modelViewMatrix: matrix4_as_array(&model_view_mat),
      normalMatrix: matrix3_as_array(&normal_mat),
      lightPosition: [light.x, light.y, light.z],
    };
    
    target.draw((&positions),
                &indices,
                &program,
                &uniforms,
                &params)
      .unwrap();
    
    target.finish().unwrap();

    for ev in display.poll_events() {
      println!("{:?}", ev);
      match ev {
        glium::glutin::Event::Closed => return,
        glium::glutin::Event::MouseMoved(xx,yy) => {
          x = xx as f32;
          y = yy as f32;
        },
        _ => (),
      }
    }
  }
}

fn view_matrix(position: &Vector3<f32>,
               direction: &Vector3<f32>,
               up: &Vector3<f32>) -> Matrix4<f32> {
  let f = if cfg!(feature="nalgebra_view") {
    nalgebra::normalize(direction)*Vector3::new(0.0, 0.0, -1.0)
  } else {
    nalgebra::normalize(direction)
  };
  
  let s = Vector3::new(up.y * f.z - up.z * f.y,
                       up.z * f.x - up.x * f.z,
                       up.x * f.y - up.y * f.x);
  let s_norm = nalgebra::normalize(&s);

  let u = Vector3::new(f.y * s_norm.z - f.z * s_norm.y,
                       f.z * s_norm.x - f.x * s_norm.z,
                       f.x * s_norm.y - f.y * s_norm.x);

  let p = Vector3::new(-position.x * s_norm.x - position.y * s_norm.y - position.z * s_norm.z,
                       -position.x * u.x - position.y * u.y - position.z * u.z,
                       -position.x * f.x - position.y * f.y - position.z * f.z);

  Matrix4::new(s_norm.x, u.x, f.x, 0.0,
               s_norm.y, u.y, f.y, 0.0,
               s_norm.z, u.z, f.z, 0.0,
               p.x, p.y, p.z, 1.0)
}

fn matrix4_as_array<T: Copy+Clone>(m: &Matrix4<T>) -> [[T; 4]; 4] {
  [[m[(0,0)], m[(0,1)], m[(0,2)], m[(0,3)]],
   [m[(1,0)], m[(1,1)], m[(1,2)], m[(1,3)]],
   [m[(2,0)], m[(2,1)], m[(2,2)], m[(2,3)]],
   [m[(3,0)], m[(3,1)], m[(3,2)], m[(3,3)]]]
}

fn matrix3_as_array<T: Copy+Clone>(m: &Matrix3<T>) -> [[T; 3]; 3] {
  [[m[(0,0)], m[(0,1)], m[(0,2)]],
   [m[(1,0)], m[(1,1)], m[(1,2)]],
   [m[(2,0)], m[(2,1)], m[(2,2)]],
   ]
}


fn matrix3_from_matrix4<T: Copy>(m: &Matrix4<T>) -> Matrix3<T> {
  Matrix3::new(m[(0,0)], m[(0,1)], m[(0,2)],
               m[(1,0)], m[(1,1)], m[(1,2)],
               m[(2,0)], m[(2,1)], m[(2,2)])
}
