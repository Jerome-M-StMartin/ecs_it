//Jerome M. St.Martin
//June 22, 2022

//-----------------------------------------------------------------------------
//---------------------- Provides Thread-Safe Access to -----------------------
//-------------------------- an Inner Arc<Storage> ----------------------------
//------------------------------ Until Dropped --------------------------------
//-----------------------------------------------------------------------------

use std::sync::Arc;

use super::super::{Component, Entity};
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

    /*Do I want this? Single component lookup is not the ECS way.
     * Commenting out for now to see if I can force use of .iter()
    pub fn get(&self, e: Entity) -> Option<&T> {
        let (map, vec) = self.guarded.unsafe_borrow();

        if let Some(component_idx) = map.get(&e) {
            let component: &T = &vec[*component_idx];
            return Some(component);
        };

        None
    }*/

    //pub fn iter(&self) -> impl Iterator<Item = &T> {
    pub fn iter(&self) -> impl Iterator<Item = &Option<T>> {
        self.guarded.unsafe_borrow().iter()
    }

    ///Favor using iter() or get() if at all possible.
    pub fn raw(&self) -> &InnerStorage<T> {
        self.guarded.unsafe_borrow()
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

    /*Do I want this? Single component lookup is not the ECS way.
     * Commenting out for now to see if I can force use of .iter()
    pub fn get_mut(&self, e: &Entity) -> Option<&mut T> {
        self.guarded.unsafe_borrow_mut().get_mut(e)
    }*/

    ///Associates Component c with Entity e. If e already had a c, then
    ///that old c is returned in an option.
    ///Assumptions:
    ///Entity e already exists and Component c is registered in the ecs world.
    pub fn insert(&mut self, e: Entity, c: T) -> Option<T> {
        let mut storage = self.guarded.unsafe_borrow_mut();
        //1.) Insert new ent/idx pair into map.
        //2.) Push component into Entity's associated slot in vec.
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Option<T>> {
        self.guarded.unsafe_borrow_mut().iter_mut()
    }

    ///Favor using other fns in this API over this if at all possible.
    pub fn raw_mut(&self) -> &InnerStorage<T> {
        self.guarded.unsafe_borrow_mut()
    }

    ///If Entity e has a component T, this returns Some(T),
    ///removing it from the Storage.
    ///If there was no such T,
    ///nothing happens and this returns None.
    pub fn remove(&mut self, e: &Entity) -> Option<T> {
        let storage = self.guarded.unsafe_borrow_mut();
        let map = //TODO

        //If this entity has a component of type T:
        if let Some(component_idx) = map.get(e) {
            let component: T = vec[*component_idx];
            let test = vec[*component_idx];
            return Some(component);
        }

        //Else:
        None
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

impl<T> Drop for MutableStorageGuard<T>
where
    T: Component,
{
    fn drop(&mut self) {
        self.guarded.drop_write_access();
    }
}
