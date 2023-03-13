//Jerome M. St.Martin
//June 22, 2022

//-----------------------------------------------------------------------------
//---------------------- Provides Thread-Safe Access to -----------------------
//-------------------------- an Inner Arc<Storage> ----------------------------
//------------------------------ Until Dropped --------------------------------
//-----------------------------------------------------------------------------

use std::sync::Arc;

use super::super::{component::Component, Entity};
use super::{InnerStorage, Storage};

///What you get when you ask the ECS for access to a Storage via req_read_access().
///These should NOT be held long-term. Do your work then allow this struct to drop, else
///you will starve all threads seeking write-access to the Storage this guards.
#[derive(Debug)]
pub struct ImmutableStorageGuard<T: Component> {
    guarded: Arc<Storage<T>>,
}

impl<T> ImmutableStorageGuard<T>
where
    T: Component,
{
    pub(crate) fn new(guarded: Arc<Storage<T>>) -> Self {
        guarded.init_read_access();
        ImmutableStorageGuard { guarded }
    }

    pub fn iter(&self) -> std::slice::Iter<Option<T>> {
        self.guarded.unsafe_borrow().iter()
    }

    ///Favor using iter() or get() if at all possible.
    pub fn raw(&self) -> &InnerStorage<T> {
        self.guarded.unsafe_borrow()
    }
}

impl<'a, T: Component> IntoIterator for &'a ImmutableStorageGuard<T> {
    type Item = &'a Option<T>;
    type IntoIter = std::slice::Iter<'a, Option<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> Drop for ImmutableStorageGuard<T>
where
    T: Component,
{
    fn drop(&mut self) {
        self.guarded.drop_read_access();
    }
}

///What you get when you ask the ECS for access to a Storage via req_write_access().
///These should NOT be held long-term. Do your work then allow this struct to drop, else
///you will starve all other threads seeking write-access to the Storage this guards.
#[derive(Debug)]
pub struct MutableStorageGuard<T: Component> {
    guarded: Arc<Storage<T>>,
}

impl<T> MutableStorageGuard<T>
where
    T: Component,
{
    pub(crate) fn new(guarded: Arc<Storage<T>>) -> Self {
        guarded.init_write_access();
        MutableStorageGuard { guarded }
    }

    ///Associates Component c with Entity e.
    ///If e already had a c, that old c is returned in an option.
    ///Assumptions:
    ///Entity e already exists and Component c is registered in the ecs world.
    pub fn insert(&mut self, e: Entity, c: T) -> Option<T> {
        let storage = self.guarded.unsafe_borrow_mut();
        let old_component: Option<T> = storage[e].replace(c); //In superposition of Some/None.
        old_component //Regardless of if it's Some/None, it will be correct to return it.
    }

    ///TODO
    pub fn iter_mut(&mut self) -> std::slice::IterMut<Option<T>> {
        self.guarded.unsafe_borrow_mut().iter_mut()
    }

    ///Favor using other fns in this API over this if at all possible.
    pub fn raw_mut(&mut self) -> &InnerStorage<T> {
        self.guarded.unsafe_borrow_mut()
    }

    ///If Entity e has a component T, this returns Some(T),
    ///removing it from the Storage.
    ///If there was no such T,
    ///nothing happens and this returns None.
    pub fn remove(&mut self, e: Entity) -> Option<T> {
        let storage = self.guarded.unsafe_borrow_mut();
        let component: Option<T> = storage[e].take();
        component
    }
    
    ///Used internally to lengthen the storage vec to make room for a new entity.
    pub(crate) fn register_new_entities(&mut self, num_new_entities: usize) {
       let storage = self.guarded.unsafe_borrow_mut();
       for _ in 0..num_new_entities {
           storage.push(None);
       }
    }
}

impl<'a, T: Component> IntoIterator for &'a mut MutableStorageGuard<T> {
    type Item = &'a mut Option<T>;
    type IntoIter = std::slice::IterMut<'a, Option<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> Drop for MutableStorageGuard<T>
where
    T: Component,
{
    fn drop(&mut self) {
        self.guarded.drop_write_access();
    }
}
