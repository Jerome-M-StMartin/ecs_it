//Jerome M. St.Martin
//June 22, 2022

//-----------------------------------------------------------------------------
//---------------------- Provides Thread-Safe Access to -----------------------
//-------------------------- an Inner Arc<Storage> ----------------------------
//------------------------------ Until Dropped --------------------------------
//-----------------------------------------------------------------------------

use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use super::Storage;
use super::super::{Component, Entity};

///What you get when you ask the ECS for access to a Storage via req_read_access().
///These should NOT be held long-term. Do your work then allow this struct to drop, else
///you will starve all other threads seeking write-access to the thing this guards.
#[derive(Debug)]
pub struct ImmutableStorageGuard<T: Component> {
    guarded: Arc<Storage<T>>,
}

impl<T> ImmutableStorageGuard<T> where T: Component {
    pub(crate) fn new(guarded: Arc<Storage<T>>) -> Self {
        guarded.init_read_access();
        ImmutableStorageGuard {
            guarded,
        }
    }

    pub fn get(&self, e: &Entity) -> Option<&T> {
        self
            .guarded
            .unsafe_borrow()
            .get(e)
            
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.guarded
            .unsafe_borrow()
            .values()
    }

    ///Favor using iter() or get() if at all possible.
    pub fn raw(&self) -> &HashMap<Entity, T> {
        self.guarded.unsafe_borrow()
    }
}

///What you get when you ask the ECS for access to a Storage via req_write_access().
///These should NOT be held long-term. Do your work then allow this struct to drop, else
///you will starve all other threads seeking write-access to the thing this guards.
#[derive(Debug)]
pub struct MutableStorageGuard<T: Component> {
    guarded: Arc<Storage<T>>,
}

impl<T> MutableStorageGuard<T> where T: Component {
    pub(crate) fn new(guarded: Arc<Storage<T>>) -> Self {
        guarded.init_write_access();
        MutableStorageGuard { 
            guarded,
        }
    }

    pub fn entry(&mut self, e: Entity) -> Entry<'_, Entity, T> {
        self
            .guarded
            .unsafe_borrow_mut()
            .entry(e)
    }

    ///User should perefer .entry() over this, the std Entry API is great.
    pub fn get_mut(&self, e: &Entity) -> Option<&mut T> {
        self
            .guarded
            .unsafe_borrow_mut()
            .get_mut(e)
    }

    pub fn insert(&mut self, e: Entity, c: T) -> Option<T> {
        self
            .guarded
            .unsafe_borrow_mut()
            .insert(e, c)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self
            .guarded
            .unsafe_borrow_mut()
            .values_mut()
    }

    pub fn raw_mut(&self) -> &mut HashMap<Entity, T> {
        self.guarded.unsafe_borrow_mut()
    }

    pub fn remove(&mut self, e: &Entity) -> Option<T> {
        self
            .guarded
            .unsafe_borrow_mut()
            .remove(e)
    }
}

impl<T> Drop for ImmutableStorageGuard<T> where T: Component {
    fn drop(&mut self) {
        self.guarded.drop_read_access();
    }
}

impl<T> Drop for MutableStorageGuard<T> where T: Component {
    fn drop(&mut self) {
        self.guarded.drop_write_access();
    }
}

