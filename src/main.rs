#[macro_use] extern crate glium;
extern crate kaffeesahne;

use std::f32::consts;
use glium as gl;
use kaffeesahne::*;
use std::time::{Duration,Instant};

fn main() {
  use glium::DisplayBuild;
  let display = gl::glutin::WindowBuilder::new()
    .with_depth_buffer(24)
    // .with_multisampling(8)
    .build_glium().unwrap();

  let mut world = World::new(&display);

  {
    let mut resources = &mut world.resources;
    resources.load_obj(&display, "terrain", "terrain.obj");
    resources.load_obj(&display, "light", "light.obj");
    resources.load_obj(&display, "hollow_cube", "hollow_cube.obj");
    resources.load_obj(&display, "teapot", "teapot.obj");
    resources.load_obj(&display, "cube", "toruscube.obj");
    resources.make_axis_object(&display, "axis");

    resources.compile_shader(&display,
                             "basic",
                             "src/shaders/basic.vertex.glsl",
                             "src/shaders/basic.fragment.glsl");
    // TODO: Move to RenderSystem
    resources.compile_shader(&display,
                             "picking",
                             "src/shaders/picking.vertex.glsl",
                             "src/shaders/picking.fragment.glsl");
    resources.compile_shader(&display,
                             "axis",
                             "src/shaders/axis.vertex.glsl",
                             "src/shaders/axis.fragment.glsl");
  }

  let terrain = world.entities.new_entity();
  {
    world.entities.set_position(terrain, Vector3::new(0.0, 0.0, 0.0));
    world.entities.add_geometry(terrain, Geometry {
      geometry: "terrain",
      program:  "basic",
    });
    world.entities.set_pickable(terrain, true);
  }

  world.light = na::Point3::new(1.0, 1.0, 0.0);
  {
    let light = world.entities.new_entity();
    let position = Position(world.light.to_vector());
    world.entities.set_position(light, position);
    world.entities.set_pickable(light, true);
    world.entities.add_geometry(light, Geometry {
      geometry: "light",
      program:  "basic",
    });
    world.entities.set_scale(light, Scale(na::one::<na::Vector3<f32>>()*0.05));
  }


  {
    let cube = world.entities.new_entity();
    world.entities.set_position(cube, Vector3::new(0.0, 0.75, 0.0));
    world.entities.add_geometry(cube, Geometry {
      geometry: "cube",
      program:  "basic",
    });
    world.entities.set_pickable(cube, true);
    world.entities.velocities.insert(cube, Velocity {
      linear: na::zero(),
      angular: Rotation(quat_rotate(2.0*consts::PI/8.0, na::Unit::new(&Vector3::new(0.0, 1.0, 0.0)))),
    });
    world.entities.bobs.insert(cube, Bob::new(Millis(1000.0), Vector3::new(0.0, 0.5, 0.0)));
  }

  let camera = world.entities.new_entity();
  world.entities.add_camera(camera, Camera {
    target: Point3::new(0.0, 0.0, 0.0),
    tracking: None,
  });
  world.entities.set_position(camera, Position(Vector3::new(0.5, 2.0, -3.0)));

  let ms_per_update = Duration::new(0, 1000000000/60);
  let mut previous = Instant::now();
  let mut lag = Duration::new(0, 00);

  loop {
    let now = Instant::now();
    lag += now - previous;
    previous = now;

    while lag >= ms_per_update {
      world.update(ms_per_update.into());
      lag -= ms_per_update;
    }

    world.handle_events(display.poll_events());

    let mut target = display.draw();
    world.draw(&mut target);
    target.finish().unwrap();
  }
}
