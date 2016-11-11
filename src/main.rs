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
    .build_glium().unwrap();

  let mut world = World::new(&display);

  let mut resources = ResourceManager::new();
  resources.load_obj(&display, "terrain", "terrain.obj");
  resources.load_obj(&display, "teapot", "teapot.obj");
  resources.load_obj(&display, "cube", "cube.obj");
  resources.compile_shader(&display,
                           "basic",
                           "src/shaders/basic.vertex.glsl",
                           "src/shaders/basic.fragment.glsl");
  // TODO: Move to RenderSystem
  resources.compile_shader(&display,
                           "picking",
                           "src/shaders/picking.vertex.glsl",
                           "src/shaders/picking.fragment.glsl");

  let terrain = world.entities.new_entity();
  {
    let position = Position(Vector3::new(0.0, 0.0, 0.0));
    world.entities.set_position(terrain, position);
    world.entities.add_geometry(terrain, Geometry {
      geometry: "terrain",
      program:  "basic",
    });
    world.entities.set_scale(terrain, Scale(na::one::<na::Vector3<f32>>()*0.5));
    world.entities.set_pickable(terrain, true);
    world.entities.velocities.insert(terrain, Velocity {
      linear: Vector3::new(0.0, 0.0, 0.0),
      angular: Rotation(quat_rotate(2.0*consts::PI/8.0, na::Unit::new(&Vector3::new(0.0, 1.0, 0.0)))),
    });
  }

  // let teapot = world.entities.new_entity();
  // {
  //   let position = Position(Vector3::new(0.0, 0.0, 0.0));
  //   world.entities.set_position(teapot, position);
  //   world.entities.add_geometry(teapot, Geometry {
  //     geometry: "teapot",
  //     program:  "basic",
  //   });
  //   world.entities.set_scale(teapot, Scale(na::one::<na::Vector3<f32>>()*0.5));
  //   world.entities.set_pickable(teapot, true);
  //   world.entities.velocities.insert(teapot, Velocity {
  //     linear: Vector3::new(0.0, 0.0, 0.0),
  //     angular: Rotation(quat_rotate(2.0*consts::PI/8.0, na::Unit::new(&Vector3::new(0.0, 1.0, 0.0)))),
  //   });
  // }


  // // TODO: 256 entities get too slow. Octree?
  // for i in 0..128 {
  //   let cube = world.entities.new_entity();
  //   let position = Position(Vector3::new((16.0 - i as f32)*0.25, 1.0, 0.0));
  //   world.entities.set_position(cube, position);
  //   world.entities.add_geometry(cube, Geometry {
  //     geometry: "cube",
  //     program:  "basic",
  //   });
  //   world.entities.set_scale(cube, Scale(na::one::<na::Vector3<f32>>()*0.1));
  //   world.entities.set_pickable(cube, true);
  //   world.entities.velocities.insert(cube, Velocity {
  //     linear: Vector3::new(0.0, 0.0, 0.0),
  //     angular: Rotation(quat_rotate(2.0*consts::PI/8.0, na::Unit::new(&Vector3::new(0.0, 1.0, 0.0)))),
  //   });
  // }

  world.light = na::Point3::new(-1.0, 0.5, 0.0);
  {
    let cube = world.entities.new_entity();
    let position = Position(world.light.to_vector());
    world.entities.set_position(cube, position);
    world.entities.add_geometry(cube, Geometry {
      geometry: "cube",
      program:  "basic",
    });
    world.entities.set_scale(cube, Scale(na::one::<na::Vector3<f32>>()*0.05));
  }


  let camera = world.entities.new_entity();
  world.entities.add_camera(camera, Camera {
    target: Point3::new(0.0, 0.0, 0.0),
    tracking: None,
  });
  world.entities.set_position(camera, Position(Vector3::new(0.0, 1.5, -3.0)));

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

    let mut target = display.draw();
    world.draw(&mut target, &resources);
    target.finish().unwrap();

    // TODO: How to handle these events? Picking with vsync still fucks up
    for ev in display.poll_events() {
      use glium::glutin::*;
      match ev {
        Event::Closed => return,
        Event::MouseMoved(x,y) => {
          println!("{:?}", world.render_system.pick);
          world.render_system.pick_position = Some((x as u32, y as u32));
        }
        _ => (),
      }
    }
  }
}
