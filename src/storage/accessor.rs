//Jerome M. St.Martin
//June 15, 2022

//-----------------------------------------------------------------------------
//-------------- Controls Access to Storages' Inner UnsafeCell ----------------
//-----------------------------------------------------------------------------

use std::{
    any::TypeId,
    sync::{Condvar, Mutex},
};

///Abstraction Sequence:
///StorageGuard structs contain Accessor structs which contain AccessorState structs.

///Used internally to guarantee safe concurrent access to Storages.
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
pub struct AccessorState {
    pub readers: u16, // num of currently reading readers, NOT waiting/slept readers
    pub read_allowed: bool,
    pub write_allowed: bool,
    pub writers_waiting: u16, //slept writers, NOT current writers (which is always 0..1)
}
