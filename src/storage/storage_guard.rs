//Jerome M. St.Martin
//June 22, 2022

//-----------------------------------------------------------------------------
//---------------------- Provides Thread-Safe Access to -----------------------
//-------------------------- an Inner Arc<Storage> ----------------------------
//------------------------------ Until Dropped --------------------------------
//-----------------------------------------------------------------------------

use std::{
    any::{Any, TypeId},
    sync::Arc,
};

use super::{
    Storage,
    accessor::AccessorState
};

///What you get when you ask the ECS for access to a Storage or Resource via req_access().
///These should NOT be held long-term. Do your work then allow this struct to drop, else
///you will starve all other threads seeking write-access to the thing this guards.
#[derive(Debug)]
pub struct StorageGuard {
    storage: Arc<Storage>,
}

impl StorageGuard {
    pub(crate) fn new(storage: Arc<Storage>) -> Self {
        StorageGuard {
            storage,
        }
    }

    pub(crate) fn val/*<T: 'static>*/(&self) -> &Vec<Option<Box<dyn Any>>> {
        const READ_ERR_MSG: &str = "Accessor mtx found poisoned in StorageGuard.val().";

        //assert_eq!(self.storage.accessor.type_id, TypeId::of::<T>());

        //While write access is NOT allowed, wait until the calling thread is
        //notified on the condvar. Once the condvar is notified, the calling
        //thread is awoken, the lock for the mutex is acquired, and execution
        //of this function continues.
        let mut accessor_state: std::sync::MutexGuard<'_, AccessorState> = self
            .storage
            .accessor
            .reader_cvar
            .wait_while(self.storage.accessor.mtx.lock().expect(READ_ERR_MSG), |acc_state: &mut AccessorState| {
                !acc_state.read_allowed
            })
            .expect(READ_ERR_MSG);

        accessor_state.write_allowed = false;
        accessor_state.readers += 1;

        unsafe {
            &*self.storage.inner.get()
        }
    }

    pub(crate) fn val_mut/*<T: 'static>*/(&self) -> &mut Vec<Option<Box<dyn Any>>> {
        const WRITE_ERR_MSG: &str = "Accessor mtx found poisoned in StorageGuard.val_mut().";
        
        //assert_eq!(self.storage.accessor.type_id, TypeId::of::<T>());
        
        let mut accessor_state: std::sync::MutexGuard<'_, AccessorState> = self
            .storage
            .accessor
            .mtx
            .lock()
            .expect(WRITE_ERR_MSG);

        accessor_state.writers_waiting += 1;

        //While write access is NOT allowed, wait until the calling thread is
        //notified on the condvar. Once the condvar is notified, the calling
        //thread is awoken, the lock for the mutex is acquired, and execution
        //of this function continues.
        accessor_state = self
            .storage
            .accessor
            .writer_cvar
            .wait_while(accessor_state, |acc_state: &mut AccessorState| {
                !acc_state.write_allowed
            })
            .expect(WRITE_ERR_MSG);

        accessor_state.read_allowed = false;
        accessor_state.write_allowed = false;
        accessor_state.writers_waiting -= 1;

        unsafe {
            &mut *self.storage.inner.get()
        }
    }
}


///Writer-Prioritized Concurrent Access:
///
///This implementation should, assuming my logic is sound and correctly
///implemented, eliminate the possibility of starvation for writers. Readers,
///on the other hand, can VERY EASILY be starved if writers are continuously
///requesting access. This is an intentional trade-off: the use case for this
///ECS is turn-based video games, where reads for rendering purposes occurr
///every tick, but writes occurr only corresponding with user input.
///
///NOTE: This implementation does NOT guarantee that all readers will read the
///result of every write. Many sequential writes may occur without any reads
///in-between.
impl Drop for StorageGuard {
    fn drop(&mut self) {

        let mut accessor_state = self
            .storage
            .accessor
            .mtx
            .lock()
            .expect("StorageGuard Mutex poisoned before .drop()");

        match (accessor_state.write_allowed, accessor_state.read_allowed) {
            (false, false) => {
                //This StorageGuard was giving exclusive Write access, so it is
                //now safe to allow any type of access.
                accessor_state.write_allowed = true;
                accessor_state.read_allowed = true;
            },

            (false, true) => {
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
            },

            (w, r) => {
                panic!("This Condvar configuration should not be possible: ({}, {})", w, r)
            },
        }

        //Writer prioritization:
        if accessor_state.writers_waiting > 0 {
            self.storage.accessor.writer_cvar.notify_one();
        } else {
            self.storage.accessor.reader_cvar.notify_all();
        }
    }
}
/*
impl Deref for StorageGuard {
    type Target = Arc<Storage>;
    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

impl DerefMut for StorageGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.storage
    }
}
*/
