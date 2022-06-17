//Jerome M. St.Martin
//June 15, 2022

use std::any::TypeId;
use std::marker::PhantomData;
use std::sync::{Arc, Condvar, Mutex};

use super::{
    world::World,
    resource::Resource,
};

///Abstraction Sequence:
///AccessGuard structs contain Accessor structs which contain AccessorState structs.
///Confusing, I know, but when used these abstractions are obfuscated so you won't
///need to interact with them directly. You query the ECS World, and it spits out
///an AccessGuard which you call further functions on directly as if it were a
///Storage or Resource, similar to the standard library's Mutex and MutexGuard.

//pub mod storage_guard;
//pub mod resource_guard;

///Used internally to guarantee safe concurrent access to Storages and Resources.
#[derive(Debug)]
pub struct Accessor {
    type_id: TypeId, //Type of the thing this controls access to.
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
pub struct AccessGuard<'a>(Arc<Accessor>, PhantomData<&'a ()>);

impl<'a> AccessGuard<'a> {
    pub(super) fn new(accessor: Arc<Accessor>) -> Self {
        AccessGuard(accessor.clone(), PhantomData)
    }

    fn get(&self, world: &'static World) -> &Box<dyn Resource> {
        const READ_ERR_MSG: &str = "StorageAccessGuard mutex poisoned before read.";

        //While read access is NOT allowed, wait until the calling thread is notified on the
        //condvar. Once the condvar (cvar) is notified, the calling thread is awoken,
        //the lock for the mutex (mtx) is acquired, and execution of this function continues.
        let mut accessor_state: std::sync::MutexGuard<'_, AccessorState> = self
            .reader_cvar
            .wait_while(self.mtx.lock().expect(READ_ERR_MSG), |acc_state: &mut AccessorState| {
                !acc_state.read_allowed
            })
            .expect(READ_ERR_MSG);

        //accessor_state.read_allowed = true; It will already be true at this point.
        accessor_state.write_allowed = false;
        accessor_state.readers += 1;

        world.read(self.type_id, &self)
    }

    fn get_mut(&self, world: &'static mut World) -> &mut Box<dyn Resource> {
        const WRITE_ERR_MSG: &str = "StorageAccessGuard mutex poisoned before write.";

        /*While write access is NOT allowed, wait until the calling thread is notified on the
         * condvar. Once the condvar is notified, the calling thread is awoken,
         * the lock for the mutex is acquired, and the execution of this function continues.*/
        let mut accessor_state: std::sync::MutexGuard<'_, AccessorState> = self
            .writer_cvar
            .wait_while(self.mtx.lock().expect(WRITE_ERR_MSG), |acc_state: &mut AccessorState| {
                !acc_state.write_allowed
            })
            .expect(WRITE_ERR_MSG);

        accessor_state.read_allowed = false;
        accessor_state.write_allowed = false;

        world.write(self.type_id, &self)
    }
}

impl<'a> std::ops::Deref for AccessGuard<'a> {
    type Target = Accessor;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Drop for AccessGuard<'a> {
    fn drop(&mut self) {

        let mut access_state = self
            .mtx
            .lock()
            .expect("AccessGuard Mutex poisoned before .drop()");

        match (access_state.write_allowed, access_state.read_allowed) {
            (false, false) => {
                //This AccessGuard was giving exclusive Write access,
                //so it is now safe to allow any type of access.
                access_state.write_allowed = true;
                access_state.read_allowed = true;
            },

            (false, true) => {
                //This AccessGuard was granding non-exclusive Read access,
                //so the reader count must be decremented.
                access_state.readers -= 1;

                if access_state.readers == 0 {
                    //Write access is allowed again, since there is no one with access currently.
                    access_state.write_allowed = true;

                    //To avoid writer starvation, notify a writer first whenever write access
                    //is available, which is now, when no current readers exist.
                    self.writer_cvar.notify_one(); 
                }
            },

            (w, r) => {
                panic!("This Condvar configuration should not be possible: ({}, {})", w, r)
            },
        }

        //Notify all the readers to hopefully read in parallel, assuming the duration of
        //their read access is significantly & sufficiently longer than the time required
        //to get through all the control structures (Mutexes, etc) to acquire read access.
        self.reader_cvar.notify_all();
    }
}
