//Jerome M. St.Martin
//Feb 12, 2023

//-----------------------------------------------------------------------------
//------------------- Warehouse: What Stores the Storages  --------------------
//-----------------------------------------------------------------------------

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use super::{
    component::Component,
    storage::{ImmutableStorageGuard, MutableStorageGuard, Storage},
    world::World,
    Entity,
};

///Container for all Storages in the ECS World; lives in an Arc.
pub struct Warehouse {
    //Invariants:
    //1.) each storage has the same length (underlying vec I mean)
    //2.) capacity == the length of the storages
    pub(crate) capacity: usize, //Exact length of all Storage vecs, not # of storages.
    storages: HashMap<TypeId, StorageBox>,
    pub(crate) maintenance_functions: Vec<Box<dyn Fn(&World, &Entity)>>,
}

impl Warehouse {
    pub(crate) fn new() -> Self {
        Warehouse {
            capacity: 0,
            storages: HashMap::new(),
            maintenance_functions: Vec::new(),
        }
    }

    pub fn checkout_storage<T: Component>(&self) -> ImmutableStorageGuard<T> {
        let type_id = TypeId::of::<T>();

        if let Some(storage_box) = self.storages.get(&type_id) {
            let arc = storage_box.clone_storage_arc();
            return ImmutableStorageGuard::new(arc);
        } else {
            panic!("Failed to find Storage<T>. Did you forget to register a Component?");
        }
    }

    pub fn checkout_storage_mut<T: Component>(&self) -> MutableStorageGuard<T> {
        let type_id = TypeId::of::<T>();

        if let Some(storage_box) = self.storages.get(&type_id) {
            let arc = storage_box.clone_storage_arc();
            return MutableStorageGuard::new(arc);
        } else {
            panic!("Failed to find Storage<T>. Did you forget to register a Component?");
        }
    }
}

///Used internally to provide abstraction over generically typed Storages
///to allow storing any kind of Storage<T>. i.e. Implements polymorphism over
///all Storage types.
///
///Additionally, these are what own the the Arcs that own each Storage,
///allowing for thread-safe ownership of subsets of Storages rather than
///requiring a continuous lock on the entire Warehouse.
#[derive(Debug)]
pub(crate) struct StorageBox {
    pub(crate) boxed: Arc<dyn Any + Send + Sync + 'static>,
}

impl StorageBox {
    pub(crate) fn clone_storage_arc<T: Component>(&self) -> Arc<Storage<T>> {
        let arc_any = self.boxed.clone();
        arc_any.downcast::<Storage<T>>().unwrap_or_else(|e| {
            panic!("{:?}", e);
        })
    }
}

/*wtf is this? Is this used? Where? Why?
  Seems like an old version before I came up with StorageBox.
  Or perhaps I wanted a way to queue a lazy Component removal?
  Seems like that may be valuable... idk, later me problem.
pub(crate) trait AnyStorage {
    fn rm_component(&self, e: &Entity);
}*/
