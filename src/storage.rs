//Jerome M. St.Martin
//June 15, 2022

use std::ops::{Deref, DerefMut};

use super::component::Component;

type Entity = u32;

pub(crate) struct VecStorage<T: Component>(Vec<Option<T>>);

impl<T> VecStorage<T> where T: Component {
    fn new() -> Self {
        VecStorage(Vec::new())
    }

    fn query(&self, e: Entity) -> &Option<T> {
        &self[e as usize]
    }

    fn mut_query(&mut self, e: Entity) -> &mut Option<T> {
        &mut self[e as usize]
    }
}

impl<T> Deref for VecStorage<T> where T: Component {
    type Target = Vec<Option<T>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for VecStorage<T> where T: Component {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
