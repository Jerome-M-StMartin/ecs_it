//Jerome M. St.Martin
//June 15, 2022

use std::{
    any::{Any, TypeId},
    sync::{Arc, Condvar, Mutex},
    cell::UnsafeCell,
};

use super::Storage; //Arc<UnsafeCell<Vec<Option<Box<dyn Any>>>>>

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
    val: Storage, //Arc<UnsafeCell<Vec<Option<Box<dyn Any>>>>>
}

impl AccessGuard {
    pub(super) fn new(accessor: Arc<Accessor>,
                      val: Storage) -> Self {

        AccessGuard {
            accessor,
            val,
        }
    }

    pub (crate) fn val<T: 'static>(&self) -> &Vec<Option<Box<dyn Any>>> {
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

    pub(crate) fn val_mut<T: 'static>(&self) -> &mut Vec<Option<Box<dyn Any>>> {
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


///This implementation should, assuming my logic is sound and correctly implemented,
///eliminate the possibility of starvation for both readers and writers. It achieves
///this through implementing a "flip-flop" functionality, where each dropped writer
///wakes all readers, and no further readers are awoken until these readers drop.
///Once this pool of readers drop, ONE writer is awoken, after which the next pool
///of readers is awoken. So on and so forth. This is certainly not the most
///performant way, but it is safe from starvation.
///
///NOTE: This implementation does NOT guarantee that all readers will read the
///result of every write.
impl Drop for AccessGuard {
    fn drop(&mut self) {

        let mut access_state = self
            .accessor
            .mtx
            .lock()
            .expect("AccessGuard Mutex poisoned before .drop()");

        match (access_state.write_allowed, access_state.read_allowed) {
            (false, false) => {
                //This AccessGuard was giving exclusive Write access, so it is
                //now safe to allow any type of access; but in order to
                //implement a "flip-flop" style behavior, all readers should
                //be notified at this time.
                access_state.write_allowed = true;
                access_state.read_allowed = true;
                

                self.accessor.reader_cvar.notify_all();
            },

            (false, true) => {
                //This AccessGuard was granting non-exclusive Read access,
                //so the reader count must be decremented.
                access_state.readers -= 1;

                if access_state.readers == 0 {
                    //There are no current readers, so write access is allowed
                    //again, and to implement "flop-flop" style behavior,
                    //only a writer should be notified at this time.
                    access_state.write_allowed = true;
                    self.accessor.writer_cvar.notify_one(); 

                    //Note: read_allowed is not and SHOULD NOT BE set to false
                    //here, because it is possible to reach 0 readers before
                    //the entire pool of notified readers have had a chance to
                    //read. By leaving read_allowed set to true, it gives these
                    //"late" readers a chance to race for the lock with the
                    //writer that was just notified.
                    //
                    //Therefor, this implementation does NOT guarantee that all
                    //readers will read the result of every write, because it's
                    //possible for the notified writer to win the race for the
                    //lock, but only in the case where there are remaining awake
                    //readers when the reader count is set to 0.
                    //
                    //Furthermor, and most importantly, setting read_allowed to
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
    }
}
