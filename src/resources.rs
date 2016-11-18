extern crate tobj;

use std::collections::HashMap;
use glium as gl;
use nalgebra as na;
use super::geometry::*;

use std::path::Path;
use std::fmt;

pub struct BufferedMesh {
  pub positions: gl::VertexBuffer<Vertex>,
  pub normals:   gl::VertexBuffer<Normal>,
  pub indices:   gl::index::IndexBuffer<u32>,
  pub material:  gl::uniforms::UniformBuffer<Material>,
}

pub struct ResourceManager {
  pub meshes:    HashMap<&'static str, BufferedMesh>,
  pub programs:  HashMap<&'static str, gl::Program>,
}

impl ResourceManager {
  pub fn new() -> Self {
    ResourceManager {
      meshes:    HashMap::new(),
      programs:  HashMap::new(),
    }
  }
  
  pub fn compile_shader<P: AsRef<Path>>(&mut self,
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

  pub fn load_obj<P>(&mut self,
                     display: &gl::Display,
                     name: &'static str,
                     path: P)
    where P: AsRef<Path>+fmt::Display {
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
      let position = na::Vector3::new(mesh.positions[3 * f],
                                      mesh.positions[3 * f + 1],
                                      mesh.positions[3 * f + 2]);
      vertices.push(position);
    }


    let mut normals = vec![na::zero(); vertices.len()];

    if mesh.normals.len() > 0 {
      println!("Got normals in obj file");
      for f in 0..mesh.normals.len() / 3 {
        let normal = na::Vector3::new(mesh.normals[3 * f],
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

    let material = gl::uniforms::UniformBuffer::new(display, Material {
      surfaceColor: (1.0, 0.5, 0.0, 1.0),
    }).unwrap();

    self.meshes.insert(name, BufferedMesh {
      positions: positions,
      normals: normals,
      indices: indices,
      material: material,
    });
  }

  pub fn make_axis_object<F: gl::backend::Facade>(&mut self, display: &F, name: &'static str) {
    let vertices = [(0.0, 0.0, 0.0).into(),
                    (1.0, 0.0, 0.0).into(),
                    (0.0, 1.0, 0.0).into(),
                    (0.0, 0.0, 1.0).into()];
    let positions = gl::VertexBuffer::new(display, &vertices).unwrap();
    let normals = gl::VertexBuffer::empty(display, 0).unwrap();
    // let indices   = gl::index::NoIndices(gl::index::PrimitiveType::LinesList);
    let indices   = gl::index::IndexBuffer::new(display,
                                                gl::index::PrimitiveType::LinesList,
                                                &[0,1, 0,2, 0,3]).unwrap();

    self.meshes.insert(name, BufferedMesh {
      positions: positions,
      normals: normals,
      indices: indices,
      material: gl::uniforms::UniformBuffer::empty(display).unwrap(),
    });

  }
}

