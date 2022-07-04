//Jerome M. St.Martin
//June 22, 2022

//-----------------------------------------------------------------------------
//---------------------- Provides Thread-Safe Access to -----------------------
//-------------------------- an Inner Arc<Storage> ----------------------------
//------------------------------ Until Dropped --------------------------------
//-----------------------------------------------------------------------------

use std::{
    any::Any,
    sync::Arc,
};

use super::Storage;
use super::super::Entity;

const USAGE_ERR: &str = "A StorageGuard cannot grant both immutable and mutable access!";
type InnerStorage = Vec<Option<Box<dyn Any>>>;

///What you get when you ask the ECS for access to a Storage via req_read_access().
///These should NOT be held long-term. Do your work then allow this struct to drop, else
///you will starve all other threads seeking write-access to the thing this guards.
#[derive(Debug)]
pub struct ImmutableStorageGuard {
    guarded: Arc<Storage>,
}

impl ImmutableStorageGuard {
    pub(crate) fn new(guarded: Arc<Storage>) -> Self {

        ImmutableStorageGuard {
            guarded,
        }
    }

    pub fn get(&self, e: Entity) -> &Option<Box<dyn Any>> {
        &self
            .guarded
            .unsafe_borrow()
            [e]
            
    }

    pub fn iter(&self) -> impl Iterator<Item = &Option<Box<dyn Any>>> {
        self.guarded
            .unsafe_borrow()
            .iter()
    }

    ///Favor using iter() or get() if at all possible.
    pub fn raw(&self) -> &Vec<Option<Box<dyn Any>>> {
        self.guarded.unsafe_borrow()
    }
}

///What you get when you ask the ECS for access to a Storage via req_write_access().
///These should NOT be held long-term. Do your work then allow this struct to drop, else
///you will starve all other threads seeking write-access to the thing this guards.
#[derive(Debug)]
pub struct MutableStorageGuard {
    guarded: Arc<Storage>,
}

impl MutableStorageGuard {
    pub(crate) fn new(guarded: Arc<Storage>) -> Self {

        MutableStorageGuard { 
            guarded,
        }
    }

    pub fn get_mut(&self, e: Entity) -> &mut Option<Box<dyn Any>> {
        &mut self
            .guarded
            .unsafe_borrow_mut()
            [e]
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Option<Box<dyn Any>>> {
        self
            .guarded
            .unsafe_borrow_mut()
            .iter_mut()
    }

    pub fn raw_mut(&self) -> &mut Vec<Option<Box<dyn Any>>> {
        self.guarded.unsafe_borrow_mut()
    }
}

///Writer-Prioritized Concurrent Access:
///
///Implementations of Drop for Immutable/MutableStorageGuards are half of
///the implementation of the above goal.
///
///This implementation should, assuming my logic is sound and correctly
///implemented, eliminate the possibility of starvation for writers. Readers,
///on the other hand, can VERY EASILY be starved if writers are continuously
///requesting access. This is an intentional trade-off: the use case for this
///ECS is turn-based video games, where reads occur every tick, but writes
///occur only corresponding with user input.
///
///NOTE: This implementation does NOT guarantee that all readers will read the
///result of every write. Many sequential writes may occur without any reads
///in-between.
impl Drop for ImmutableStorageGuard {
    fn drop(&mut self) {
        let mut accessor_state = self
            .guarded
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
            self.guarded.accessor.writer_cvar.notify_one();
        } else {
            self.guarded.accessor.reader_cvar.notify_all();
        }
    }
}

impl Drop for MutableStorageGuard {
    fn drop(&mut self) {
            let mut accessor_state = self
            .guarded
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
            self.guarded.accessor.writer_cvar.notify_one();
        } else {
            self.guarded.accessor.reader_cvar.notify_all();
        }
    }
}

/*
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
            }

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
            }

            (w, r) => {
                panic!(
                    "This Condvar configuration should not be possible: ({}, {})",
                    w, r
                )
            }
        }

        //Writer prioritization:
        if accessor_state.writers_waiting > 0 {
            self.storage.accessor.writer_cvar.notify_one();
        } else {
            self.storage.accessor.reader_cvar.notify_all();
        }
    }
}
*/
