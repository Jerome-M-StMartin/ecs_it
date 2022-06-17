//Jerome M. St.Martin
//June 15, 2022

use std::{
    any::{Any, TypeId},
    sync::{Arc, Condvar, Mutex},
    cell::UnsafeCell,
};

use super::Storage;

///Abstraction Sequence:
///AccessGuard structs contain Accessor structs which contain AccessorState structs.

///Used internally to guarantee safe concurrent access to Storages and Resources.
#[derive(Debug)]
pub struct Accessor {
    type_id: TypeId,
    mtx: Mutex<AccessorState>,
    reader_cvar: Condvar,
    writer_cvar: Condvar,
}

impl Accessor {
    pub(super) fn new(type_id: TypeId) -> Self {
        Accessor {
            type_id,
            mtx: Mutex::new(AccessorState {
                readers: 0,
                read_allowed: true,
                write_allowed: true,
            }),
            reader_cvar: Condvar::new(),
            writer_cvar: Condvar::new(),
        }
    }
}

///Internal to Accessor structs.
#[derive(Debug)]
struct AccessorState {
    pub readers: u8,
    pub read_allowed: bool,
    pub write_allowed: bool,
}

///What you get when you ask the ECS for access to a Storage or Resource via req_access().
///These should NOT be held long-term. Do your work then allow this struct to drop, else
///you will starve all other threads seeking write-access to the thing this guards.
#[derive(Debug)]
pub struct AccessGuard {
    accessor: Arc<Accessor>,
    val: Arc<UnsafeCell<Vec<Option<Box<dyn Any>>>>>,
}

impl AccessGuard {
    pub(super) fn new(accessor: Arc<Accessor>,
                      val: Storage) -> Self {

        AccessGuard {
            accessor,
            val,
        }
    }

    fn val<T: 'static>(&self) -> &Vec<Option<Box<dyn Any>>> {
        const READ_ERR_MSG: &str = "StorageAccessGuard mutex poisoned before read.";

        assert_eq!(self.accessor.type_id, TypeId::of::<T>());

        //While read access is NOT allowed, wait until the calling thread is notified on the
        //condvar. Once the condvar (cvar) is notified, the calling thread is awoken,
        //the lock for the mutex (mtx) is acquired, and execution of this function continues.
        let mut accessor_state: std::sync::MutexGuard<'_, AccessorState> = self
            .accessor
            .reader_cvar
            .wait_while(self.accessor.mtx.lock().expect(READ_ERR_MSG), |acc_state: &mut AccessorState| {
                !acc_state.read_allowed
            })
            .expect(READ_ERR_MSG);

        //accessor_state.read_allowed = true; It will already be true at this point.
        accessor_state.write_allowed = false;
        accessor_state.readers += 1;

        unsafe {
            &*self.val.get()
        }
    }

    fn val_mut<T: 'static>(&self) -> &mut Vec<Option<Box<dyn Any>>> {
        const WRITE_ERR_MSG: &str = "StorageAccessGuard mutex poisoned before write.";

        assert_eq!(self.accessor.type_id, TypeId::of::<T>());

        /*While write access is NOT allowed, wait until the calling thread is notified on the
         * condvar. Once the condvar is notified, the calling thread is awoken,
         * the lock for the mutex is acquired, and the execution of this function continues.*/
        let mut accessor_state: std::sync::MutexGuard<'_, AccessorState> = self
            .accessor
            .writer_cvar
            .wait_while(self.accessor.mtx.lock().expect(WRITE_ERR_MSG), |acc_state: &mut AccessorState| {
                !acc_state.write_allowed
            })
            .expect(WRITE_ERR_MSG);

        accessor_state.read_allowed = false;
        accessor_state.write_allowed = false;

        unsafe {
            &mut *self.val.get()
        }
    }
}

impl Drop for AccessGuard {
    fn drop(&mut self) {

        let mut access_state = self
            .accessor
            .mtx
            .lock()
            .expect("AccessGuard Mutex poisoned before .drop()");

        match (access_state.write_allowed, access_state.read_allowed) {
            (false, false) => {
                //This AccessGuard was giving exclusive Write access,
                //so it is now safe to allow any type of access, but
                //a writer should probably be notified first to avoid
                //writer starvation.
                access_state.write_allowed = true;
                access_state.read_allowed = true;
                
                //Notify a writer, if one exists.
                self.accessor.writer_cvar.notify_one();
            },

            (false, true) => {
                //This AccessGuard was granting non-exclusive Read access,
                //so the reader count must be decremented.
                access_state.readers -= 1;

                if access_state.readers == 0 {
                    //There are no current readers, so write access is allowed again.
                    access_state.write_allowed = true;

                    //To avoid writer starvation, notify a writer first whenever write access
                    //is available, which is now, when no current readers exist.
                    self.accessor.writer_cvar.notify_one(); 

                } else {
                    //One or more threads currently have read access.

                    //Notify all the readers to hopefully read in parallel, assuming the duration of
                    //their read access is significantly & sufficiently longer than the time required
                    //to get through all the control structures (Mutexes, etc) to acquire read access.
                    self.accessor.reader_cvar.notify_all();
                }
            },

            (w, r) => {
                panic!("This Condvar configuration should not be possible: ({}, {})", w, r)
            },
        }
    }
}
