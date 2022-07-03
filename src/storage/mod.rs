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

use accessor::{Accessor, AccessorState};
pub use storage_guard::{ImmutableStorageGuard, MutableStorageGuard};

///Used internally to store components of a single type, ant to control both
///mutable and immutable access to said storage.
#[derive(Debug)]
pub(crate) struct Storage {
    pub(crate) component_type: TypeId,
    accessor: Accessor,
    //Deref inner's UnsafeCell IF AND ONLY IF you hold an AccessGuard granted by this Accessor.
    inner: UnsafeCell<Vec<Option<Box<dyn Any>>>>,
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

    pub(super) fn access_inner(&self) -> &Vec<Option<Box<dyn Any>>> {
        const READ_ERR_MSG: &str = "Accessor mtx found poisoned in StorageGuard.val().";

        //While write access is NOT allowed, wait until the calling thread is
        //notified on the condvar. Once the condvar is notified, the calling
        //thread is awoken, the lock for the mutex is acquired, and execution
        //of this function continues.
        let mut accessor_state: std::sync::MutexGuard<'_, AccessorState> = self
            .accessor
            .reader_cvar
            .wait_while(
                self.accessor.mtx.lock().expect(READ_ERR_MSG),
                |acc_state: &mut AccessorState| !acc_state.read_allowed,
            )
            .expect(READ_ERR_MSG);

        accessor_state.write_allowed = false;
        accessor_state.readers += 1;

        unsafe { &*self.inner.get() }
    }

    pub(super) fn access_inner_mut(&self) -> &mut Vec<Option<Box<dyn Any>>> {
        const WRITE_ERR_MSG: &str = "Accessor mtx found poisoned in StorageGuard.val_mut().";

        let mut accessor_state: std::sync::MutexGuard<'_, AccessorState> =
            self.accessor.mtx.lock().expect(WRITE_ERR_MSG);

        accessor_state.writers_waiting += 1;

        //While write access is NOT allowed, wait until the calling thread is
        //notified on the condvar. Once the condvar is notified, the calling
        //thread is awoken, the lock for the mutex is acquired, and execution
        //of this function continues.
        accessor_state = self
            .accessor
            .writer_cvar
            .wait_while(accessor_state, |acc_state: &mut AccessorState| {
                !acc_state.write_allowed
            })
            .expect(WRITE_ERR_MSG);

        accessor_state.read_allowed = false;
        accessor_state.write_allowed = false;
        accessor_state.writers_waiting -= 1;

        unsafe { &mut *self.inner.get() }
    }
}
