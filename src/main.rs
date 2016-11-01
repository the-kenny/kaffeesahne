#[macro_use] extern crate glium;
extern crate image;
extern crate opengl;

use glium as gl;
use opengl::*;

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
      components: vec![geometry].into(),
      transform: transform,
    }
  };

  let cube = {
    let geometry = Mesh {
      geometry: "cube",
      program:  "basic",
    };

    let bob = Bob::new(Vector3::new(0.0, 0.10, 0.0), 1000.0, 0.0);

    let scale = 0.1;
    let transform = Transform {
      pos: Vector3::new(0.0, 0.0, 0.0),
      rot: na::one(),
      scale: Vector3::new(1.0, 1.0, 1.0)*scale,
    };

    let mut components = ComponentStore::new();
    components.add(geometry);
    // components.add(bob);
    GameObject {
      components: components,
      transform: transform,
    }
  };


  let mut x: f32 = 0.0;
  let mut y: f32 = 0.0;

  let mut world = World::new();
  world.components = vec![terrain, cube];

  loop {
    let mut target = display.draw();
    target.clear_color_and_depth((0.2, 0.2, 0.2, 1.0), 1.0);

    let (width,height) = target.get_dimensions();

    world.update();

    // Camera
    // let cam_radius = (x / width as f32) * 3.0;
    // println!("cam_radius: {}", cam_radius);
    let cam_radius = 3.0;
    let cam = Point3::new(0.0, 1.5, 3.0);
    let cam_target = world.components[0].transform.pos.to_point();

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

    // for object in &mut objects {
    //   object.update(t);

    //   // TODO: Move to GameObject / Rotation-Component
    //   object.transform.rot = quat_rotate(t/200.0, &Vector3::new(0.0, 1.0, 0.0));
    // }

    for object in world.components.iter() {
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
        _ => (),
      }
    }
  }
}
