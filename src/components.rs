// use std::convert::TryFrom;
use super::geometry::*;
use nalgebra as na;

trait Tick {
  fn tick(&mut self, _parent: &mut GameObject, _t: f32) {}
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

// Visible Mesh
#[derive(Copy, Clone)]
pub struct Mesh {
  pub geometry: &'static str,
  pub program:  &'static str,
}

impl Tick for Mesh {}

// TODO
// impl TryFrom<ComponentSlot> for Mesh {
//   type Err = ();
//   fn try_from(slot: ComponentSlot) -> Result<Self, Self::Err> {
//     match slot {
//       ComponentSlot::Mesh(mesh) => Ok(mesh),
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

impl Tick for Bob {
  fn tick(&mut self, parent: &mut GameObject, t: f32) {
    parent.transform.pos += self.direction*((t+self.delta)/self.period).sin();
  }
}

macro_rules! count_items {
  ($name:ident) => { 1 };
  ($first:ident $($rest:ident),*) => {
    1 + count_items!($($rest),*)
  }
}

macro_rules! components {
  { $($component:ident), * } => {
    const NUM_COMPONENTS: usize = count_items!($($component )*);

    // The enum storing the components
    #[derive(Copy, Clone)]
    pub enum ComponentSlot {
      $(
        $component($component),
        )*
        Empty, 
    }

    // Enum used to index the slots
    #[derive(Copy, Clone)]
    enum ComponentSlotIndex {
      $( $component, )*
    }

    impl ComponentSlot {
      pub fn update(&mut self, parent: &mut GameObject, t: f32) {
        use self::ComponentSlot::*;
        match *self {
          Mesh(mut m)     => m.tick(parent, t),
          Bob(mut b)      => b.tick(parent, t),
          Empty           => ()
        };
      }
    }

    // Conversions for ComponentSlotIndex
    $(
      impl From<$component> for ComponentSlot {
        fn from(x: $component) -> Self {
          ComponentSlot::$component(x)
        }
      }
      )*

      
      impl<'a> From<&'a ComponentSlot> for ComponentSlotIndex {
        fn from(slot: &'a ComponentSlot) -> Self {
          use self::ComponentSlot::*;
          match *slot {
            Mesh(_) => ComponentSlotIndex::Mesh,
            Bob(_)      => ComponentSlotIndex::Bob,
            Empty       => unreachable!()
          }
        }
      }
  }
}

components! {
  Mesh,
  Bob
}

// Component Storage
#[derive(Copy, Clone)]
pub struct ComponentStore([ComponentSlot; NUM_COMPONENTS]);

impl ComponentStore {
  pub fn new() -> Self {
    ComponentStore([ComponentSlot::Empty; 2])
  }
  pub fn add<C: Into<ComponentSlot>>(&mut self, component: C) {
    let slot = component.into();
    let idx = ComponentSlotIndex::from(&slot);
    self[idx] = slot;
  }

  pub fn geometry(&self) -> Option<&Mesh> {
    match self[ComponentSlotIndex::Mesh] {
      ComponentSlot::Mesh(ref m) => Some(m),
      _ => None
    }
  }
}

use std::ops::{Index, IndexMut};
impl Index<ComponentSlotIndex> for ComponentStore {
  type Output = ComponentSlot;
  fn index(&self, idx: ComponentSlotIndex) -> &Self::Output {
    &self.0[idx as usize]
  }
}

impl IndexMut<ComponentSlotIndex> for ComponentStore {
  fn index_mut(&mut self, idx: ComponentSlotIndex) -> &mut Self::Output {
    &mut self.0[idx as usize]
  }
}

impl Default for ComponentStore {
  fn default() -> Self {
    Self::new()
  }
}

impl<C: Into<ComponentSlot>> From<Vec<C>> for ComponentStore {
  fn from(other: Vec<C>) -> Self {
    let mut store = Self::new();
    for c in other.into_iter() {
      store.add(c);
    }
    store
  }
}
