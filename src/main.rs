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

  let mut world = World::new();

  let mut resources = ResourceManager::new();
  resources.load_obj(&display, "terrain", "terrain.obj");
  resources.load_obj(&display, "cube", "cube.obj");
  resources.compile_shader(&display,
                           "basic",
                           "src/shaders/basic.vertex.glsl",
                           "src/shaders/basic.fragment.glsl");

  let terrain = world.entities.new_entity();
  {
    let geometry = Geometry {
      geometry: "terrain",
      program:  "basic",
    };
    world.entities.add_geometry(terrain, geometry);

    let scale = 0.5;
    let position = Position(Vector3::new(0.0, 0.0, 0.0));
    world.entities.set_position(terrain, position);
    world.entities.set_scale(terrain, Scale(na::one::<na::Vector3<f32>>()*scale));
  }

  let camera = world.entities.new_entity();
  world.entities.add_camera(camera, Camera {
    target: Point3::new(0.0, 0.0, 0.0),
    tracking: None,
  });
  world.entities.set_position(camera, Position(Vector3::new(0.0, 1.5, 3.0)));

  loop {
    let mut target = display.draw();
    target.clear_color_and_depth((0.2, 0.2, 0.2, 1.0), 1.0);

    world.update();
    world.draw(&mut target, &resources);

    target.finish().unwrap();

    for ev in display.poll_events() {
      println!("{:?}", ev);
      use glium::glutin::*;
      match ev {
        Event::Closed => return,
        _ => (),
      }
    }
  }
}
