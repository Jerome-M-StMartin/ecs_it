//Jerome M. St.Martin
//June 21, 2020

//-----------------------------------------------------------------------------
//-------------------------- ECS Component Storages ---------------------------
//-----------------------------------------------------------------------------

use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
};

mod accessor;
mod storage_guard;

pub(crate) use {
    accessor::{Accessor, AccessorState},
    storage_guard::StorageGuard,
};

///Used internally to store components of a single type, ant to control both
///mutable and immutable access to said storage.
#[derive(Debug)]
pub(crate) struct Storage {
    pub(crate) component_type: TypeId,
    pub(crate) accessor: Accessor,
    //Deref inner's UnsafeCell IF AND ONLY IF you hold an AccessGuard granted by this Accessor.
    pub(crate) inner: UnsafeCell<Vec<Option<Box<dyn Any>>>>,
}

impl Storage {
    pub(crate) fn new(component_type: TypeId, num_ents_estimate: usize) -> Self {
        let mut new_vec = Vec::with_capacity(num_ents_estimate);
        new_vec.fill_with(|| None);

        Storage {
            component_type,
            accessor: Accessor::new(component_type),
            inner: UnsafeCell::new(new_vec),
        }
    }
}
