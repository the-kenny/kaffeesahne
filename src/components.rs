// use std::convert::TryFrom;
use super::geometry::*;
use nalgebra as na;

trait Tick {
  fn tick(&mut self, _parent: &mut GameObject, t: f32) {}
}

#[derive(Copy, Clone)]
pub struct GameObject {
  pub components: ComponentStore,
  pub transform: Transform,
}

impl GameObject {
  pub fn update(&mut self, t: f32) {
    let mut new = self.clone();
    for component in self.components.0.iter_mut() {
      component.update(&mut new, t);
    }
    self.transform = new.transform;
  }
}

// Visible Geometry
#[derive(Copy, Clone)]
pub struct Mesh {
  pub geometry: &'static str,
  pub program:  &'static str,
}

impl Tick for Mesh {}

// TODO: Macro
impl Into<ComponentSlot> for Mesh {
  fn into(self) -> ComponentSlot {
    ComponentSlot::Geometry(self)
  }
}

// TODO
// impl TryFrom<ComponentSlot> for Mesh {
//   type Err = ();
//   fn try_from(slot: ComponentSlot) -> Result<Self, Self::Err> {
//     match slot {
//       ComponentSlot::Geometry(mesh) => Ok(mesh),
//       _ => Err(())
//     }
//   }
// }

// Sine Wave Translation
#[derive(Copy, Clone)]
pub struct Bob {
  pub direction: na::Vector3<f32>,
  pub period: f32,
  pub delta: f32,
}

impl Into<ComponentSlot> for Bob {
  fn into(self) -> ComponentSlot {
    ComponentSlot::Bob(self)
  }
}

impl Tick for Bob {
  fn tick(&mut self, parent: &mut GameObject, t: f32) {
    parent.transform.pos += self.direction*((t+self.delta)/self.period).sin();
  }
}

#[derive(Copy, Clone)]
pub enum ComponentSlot {
  Geometry(Mesh),
  Bob(Bob),      
  Empty,         
}

impl ComponentSlot {
  pub fn update(&mut self, parent: &mut GameObject, t: f32) {
    use self::ComponentSlot::*;
    match *self {
      Geometry(mut m) => m.tick(parent, t),
      Bob(mut b)      => b.tick(parent, t),
      Empty           => ()
    };
  }
}

// Component Storage
#[derive(Copy, Clone)]
pub struct ComponentStore([ComponentSlot; 2]);

impl ComponentStore {
  pub fn new() -> Self {
    ComponentStore([ComponentSlot::Empty; 2])
  }
  
  pub fn add<C: Into<ComponentSlot>>(&mut self, component: C) {
    use self::ComponentSlot::*;
    let slot = component.into();
    let idx = match slot {
      Geometry(_) => 0,
      Bob(_) =>      1,
      Empty => unreachable!()
    };
    self.0[idx] = slot;
  }

  pub fn geometry(&self) -> Option<&Mesh> {
    match self.0[0] {           // TODO
      ComponentSlot::Geometry(ref m) => Some(m),
      _ => None
    }
  }
}

// use std::ops::{Index, IndexMut};
// impl Index<ComponentType> for ComponentStore {
//   type Output = ComponentSlot;
//   fn index(&self, idx: ComponentType) -> &Self::Output {
//     &self.0[idx as usize]
//   }
// }

// impl IndexMut<ComponentType> for ComponentStore {
//   fn index_mut(&mut self, idx: ComponentType) -> &mut Self::Output {
//     &mut self.0[idx as usize]
//   }
// }

// impl Default for ComponentStore {
//   fn default() -> Self {
//     ComponentStore([Component::Empty; 2])
//   }
// }

impl<C: Into<ComponentSlot>> From<Vec<C>> for ComponentStore {
  fn from(other: Vec<C>) -> Self {
    let mut store = Self::new();
    for c in other.into_iter() {
      store.add(c);
    }
    store
  }
}

