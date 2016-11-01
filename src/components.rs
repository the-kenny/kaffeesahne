// use std::convert::TryFrom;
use super::geometry::*;
use super::TimeDelta;
use nalgebra as na;

pub trait Tick {
  fn tick(&mut self, _parent: &mut GameObject, _delta: TimeDelta) {}
}

#[derive(Clone)]
pub struct GameObject {
  pub components: ComponentStore,
  pub transform: Transform,
}

impl GameObject {
  pub fn update(&mut self, delta: TimeDelta) {
    let mut new = self.clone();
    for component in self.components.0.iter_mut() {
      component.update(&mut new, delta);
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
#[derive(Copy, Clone, Debug)]
pub struct Bob {
  pub direction: na::Vector3<f32>,
  pub period: f32,
  value: f32,
}

impl Bob {
  pub fn new(direction: na::Vector3<f32>, period: f32, delta: f32) -> Self {
    Bob {
      direction: direction,
      period: period,
      value: delta,
    }
  }
}

impl Tick for Bob {
  fn tick(&mut self, parent: &mut GameObject, delta: TimeDelta) {
    // TODO: Handle initial position correctly
    use std::f32;
    // TODO: store only values from 0..1 in self.value. Overflows.
    self.value += delta.as_millis();
    let val = self.value/self.period;
    // TODO: Broken without vsync. Need to generate "diffs" instead of
    // applying the changes directly. Or force a range on `delta`
    parent.transform.pos += self.direction*(val*2.0*f32::consts::PI).cos();
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
    #[derive(Clone, Copy)]
    pub enum ComponentSlot {
      $(
        $component($component),
        )*
        Empty, 
    }

    // Enum used to index the slots
    #[derive(Clone)]
    enum ComponentSlotIndex {
      $( $component, )*
    }

    impl ComponentSlot {
      pub fn update(&mut self, parent: &mut GameObject, delta: TimeDelta) {
        use self::ComponentSlot::*;
        match *self {
          Mesh(ref mut m)     => m.tick(parent, delta),
          Bob(ref mut b)      => b.tick(parent, delta),
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
#[derive(Clone)]
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
