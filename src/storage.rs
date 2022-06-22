//Jerome M. St.Martin
//June 21, 2020

//-----------------------------------------------------------------------------
//-------------------------- ECS Component Storages ---------------------------
//-----------------------------------------------------------------------------

use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub(crate) struct Storage {
    pub(crate) component_type: TypeId,
    pub(crate) inner: UnsafeCell<Vec<Option<Box<dyn Any>>>>,
}

impl Storage {
    pub fn new(component_type: TypeId) -> Self {
        
        let mut new_vec = Vec::new();
        new_vec.fill_with(|| { None });

        Storage {
            component_type,
            inner: UnsafeCell::new(new_vec),
        }
    }
}

impl Deref for Storage {
    type Target = UnsafeCell<Vec<Option<Box<dyn Any>>>>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Storage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
