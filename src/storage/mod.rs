//Jerome M. St.Martin
//June 21, 2022

//-----------------------------------------------------------------------------
//-------------------------- ECS Component Storages ---------------------------
//-----------------------------------------------------------------------------

use std::cell::UnsafeCell;

use super::Component;
use accessor::{Accessor, AccessorState};
pub use storage_guard::{ImmutableStorageGuard, MutableStorageGuard};

mod accessor;
mod storage_guard;

pub type InnerStorage<T> = Vec<Option<T>>;

///Entities are keys to a map which holds the indices where any key-entity's,
///relevant component is stored in the Vec.
#[derive(Debug)]
pub(crate) struct Storage<T> {
    accessor: Accessor,
    inner: UnsafeCell<InnerStorage<T>>,
}

unsafe impl<T> Sync for Storage<T> where T: Component {}

impl<T> Storage<T>
where
    T: Component,
{
    pub(crate) fn new() -> Self {
        Storage {
            accessor: Accessor::new(),
            inner: UnsafeCell::new(Vec::new()),
        }
    }

    ///Called internally whenever a ImmutStorageGuard is instantiated.
    pub(super) fn init_read_access(&self) {
        const READ_ERR_MSG: &str = "Accessor mtx found poisoned";

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
    }

    ///Called internally whenever a MutStorageGuard is instantiated.
    pub(super) fn init_write_access(&self) {
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
    }

    ///Called internally only by ImmutableStorageGuard API.
    pub(super) fn unsafe_borrow(&self) -> &InnerStorage<T> {
        unsafe { &*self.inner.get() }
    }

    ///Called internally only by MutableStorageGuard API.
    pub(super) fn unsafe_borrow_mut(&self) -> &mut InnerStorage<T> {
        unsafe { &mut *self.inner.get() }
    }

    ///Writer-Prioritized Concurrent Access:
    ///
    ///Called when a _StorageGuard is dropped, and elsewhere internally.
    ///
    ///These implementations should, assuming my logic is sound and correctly
    ///implemented, eliminate the possibility of starvation for writers. Readers,
    ///on the other hand, can VERY EASILY be starved if writers are continuously
    ///requesting access. This is an intentional trade-off: the use case for this
    ///ECS is turn-based video games, where reads occur every tick, but writes
    ///occur only corresponding with user input.
    ///
    ///NOTE: This implementation does NOT guarantee that all readers will read the
    ///result of every write. Many sequential writes may occur without any reads
    ///in-between.
    pub(super) fn drop_read_access(&self) {
        let mut accessor_state = self
            .accessor
            .mtx
            .lock()
            .expect("StorageGuard Mutex poisoned before .drop()");

        //This StorageGuard was granting non-exclusive Read access,
        //so the reader count must be decremented.
        accessor_state.readers -= 1;

        if accessor_state.readers == 0 {
            //There are no current readers, so write access is allowed.
            accessor_state.write_allowed = true;

            //Note: read_allowed is not and SHOULD NOT BE set to false
            //here, because it is possible to reach 0 readers before
            //the entire pool of notified readers have had a chance to
            //read. By leaving read_allowed set to true, it gives these
            //"late" readers a chance to race for the lock.
            //
            //Furthermore, and most importantly, setting read_allowed to
            //false at this point introduces the possibility of an
            //erronious reader lockout where there are no readers nor
            //writers yet read_allowed is set to false. This would
            //self-correct once a writer drops, but until that point
            //behaviour would be incorrect.
        }

        //Writer prioritization:
        if accessor_state.writers_waiting > 0 {
            self.accessor.writer_cvar.notify_one();
        } else {
            self.accessor.reader_cvar.notify_all();
        }
    }

    pub(super) fn drop_write_access(&self) {
        let mut accessor_state = self
            .accessor
            .mtx
            .lock()
            .expect("StorageGuard Mutex poisoned before .drop()");

        //This StorageGuard was giving exclusive Write access, so it is
        //now safe to allow any type of access.
        accessor_state.write_allowed = true;
        accessor_state.read_allowed = true;

        //Writer prioritization:
        if accessor_state.writers_waiting > 0 {
            self.accessor.writer_cvar.notify_one();
        } else {
            self.accessor.reader_cvar.notify_all();
        }
    }
}
