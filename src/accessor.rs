//Jerome M. St.Martin
//June 15, 2022

use std::{
    any::{Any, TypeId},
    sync::{Arc, Condvar, Mutex},
};

use super::storage::Storage; //Arc<UnsafeCell<Vec<Option<Box<dyn Any>>>>>

///Abstraction Sequence:
///AccessGuard structs contain Accessor structs which contain AccessorState structs.

///Used internally to guarantee safe concurrent access to Storages and Resources.
#[derive(Debug)]
pub struct Accessor {
    pub(crate) type_id: TypeId,
    pub(crate) mtx: Mutex<AccessorState>,
    pub(crate) reader_cvar: Condvar,
    pub(crate) writer_cvar: Condvar,
}

impl Accessor {
    pub(super) fn new(type_id: TypeId) -> Self {
        Accessor {
            type_id,
            mtx: Mutex::new(AccessorState {
                readers: 0,
                read_allowed: true,
                write_allowed: true,
                writers_waiting: 0,
            }),
            reader_cvar: Condvar::new(),
            writer_cvar: Condvar::new(),
        }
    }
}

///Internal to Accessor structs.
#[derive(Debug)]
pub(crate) struct AccessorState {
    pub readers: u16, // num of currently reading readers, NOT waiting/slept readers
    pub read_allowed: bool,
    pub write_allowed: bool,
    pub writers_waiting: u16, //slept writers, NOT current writers (which is always 0..1)
}

///What you get when you ask the ECS for access to a Storage or Resource via req_access().
///These should NOT be held long-term. Do your work then allow this struct to drop, else
///you will starve all other threads seeking write-access to the thing this guards.
#[derive(Debug)]
pub struct AccessGuard {
    accessor: Arc<Accessor>,
    val: Arc<Storage>,
}

impl AccessGuard {
    pub(super) fn new(accessor: Arc<Accessor>,
                      val: Arc<Storage>) -> Self {

        AccessGuard {
            accessor,
            val,
        }
    }

    pub (crate) fn val<T: 'static>(&self) -> &Vec<Option<Box<dyn Any>>> {
        const READ_ERR_MSG: &str = "Accessor mtx found poisoned in AccessGuard.val().";

        assert_eq!(self.accessor.type_id, TypeId::of::<T>());

        //While write access is NOT allowed, wait until the calling thread is
        //notified on the condvar. Once the condvar is notified, the calling
        //thread is awoken, the lock for the mutex is acquired, and execution
        //of this function continues.
        let mut accessor_state: std::sync::MutexGuard<'_, AccessorState> = self
            .accessor
            .reader_cvar
            .wait_while(self.accessor.mtx.lock().expect(READ_ERR_MSG), |acc_state: &mut AccessorState| {
                !acc_state.read_allowed
            })
            .expect(READ_ERR_MSG);

        accessor_state.write_allowed = false;
        accessor_state.readers += 1;

        unsafe {
            &*self.val.get()
        }
    }

    pub(crate) fn val_mut<T: 'static>(&self) -> &mut Vec<Option<Box<dyn Any>>> {
        const WRITE_ERR_MSG: &str = "Accessor mtx found poisoned in AccessGuard.val_mut().";

        assert_eq!(self.accessor.type_id, TypeId::of::<T>());
        
        let mut accessor_state: std::sync::MutexGuard<'_, AccessorState> = self
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
            &mut *self.val.get()
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
impl Drop for AccessGuard {
    fn drop(&mut self) {

        let mut accessor_state = self
            .accessor
            .mtx
            .lock()
            .expect("AccessGuard Mutex poisoned before .drop()");

        match (accessor_state.write_allowed, accessor_state.read_allowed) {
            (false, false) => {
                //This AccessGuard was giving exclusive Write access, so it is
                //now safe to allow any type of access.
                accessor_state.write_allowed = true;
                accessor_state.read_allowed = true;
            },

            (false, true) => {
                //This AccessGuard was granting non-exclusive Read access,
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
            self.accessor.writer_cvar.notify_one();
        } else {
            self.accessor.reader_cvar.notify_all();
        }
    }
}
