//Jerome M. St.Martin
//June 15, 2022

use std::{
    collections::HashMap,
    any::TypeId, //TypeId::of<T>() -> TypeId;
    sync::{Arc, Mutex},
    cell::UnsafeCell,
};

use super::{
    accessor::{Accessor, AccessGuard},
    Storage,
};

const MAX_COMPONENTS: usize = 64;

pub struct World { //Arc<World>
    accessors: Mutex<HashMap<TypeId, Arc<Accessor>>>,
    storage_indices: Mutex<HashMap<TypeId, usize>>, //Values are indices of 'storages' vec.
    storages: UnsafeCell<[Option<Storage>; MAX_COMPONENTS]>, //This means there can be no more than 64 component types.
}

impl World {

    ///Use this to gain thread-safe access to a single ECS Storage or Resource.
    ///When you need access to multiple Storages and/or Resources (such as when
    ///you're running a sufficiently complex System) use req_multi_access().
    pub fn req_access<T: 'static>(&self, type_id: TypeId) -> AccessGuard {
        
        assert_eq!(type_id, TypeId::of::<T>());

        //Acquire Lock
        let mut accessors = self
            .accessors
            .lock()
            .expect("Mutex found poisoned during world.req_access()");

        //Acquire Lock
        let mut storage_indices = self
            .storage_indices
            .lock()
            .expect("Mutex found poisoned during world.req_access()");
    
        //Create accessor if this is a new storage type. Clone it either way.
        let accessor_arc: Arc<Accessor> = accessors.entry(type_id)
            .or_insert(Arc::new(Accessor::new(type_id)))
            .clone();    

        //Create index/key if this is a new storage type.
        let num_components = storage_indices.len();
        let storage_idx = storage_indices
            .entry(type_id)
            .or_insert(num_components);

        //Clone the storage.
        let unsafe_ptr: *mut [Option<Storage>; MAX_COMPONENTS] = self.storages.get();
        let storage_arc = unsafe {
                let storage_arc: &mut [Option<Storage>; MAX_COMPONENTS] = &mut *unsafe_ptr;
                storage_arc[*storage_idx]
                .get_or_insert_with(|| {
                        Arc::new(UnsafeCell::new(Vec::with_capacity(MAX_COMPONENTS)))
                })
                .clone()
            };

        AccessGuard::new(accessor_arc, storage_arc)
    }
/*
    ///Use this to gain thread-safe access to multiple ECS Storages and/or
    ///Resources simultaneously. If you need access to only one Storage or
    ///Resource, req_access() should be preferred.
    pub fn req_multi_access(&self, id_vec: Vec<TypeId>) -> Vec<AccessGuard<T>> {

        let mut guards: Vec<AccessGuard> = Vec::new();

        //Acquire Lock
        let mut accessors = self
            .accessors
            .lock()
            .expect("Mutex found poisoned during world.req_multi_access()");

        //Acquire Lock
        let mut storage_indices = self
            .storage_indices
            .lock()
            .expect("Mutex found poisoned during world.req_multi_access()");

        for type_id in id_vec {

            //Create accessor if this is a new storage type. Clone it either way.
            let accessor_arc = accessors
                .entry(type_id)
                .or_insert(Arc::new(Accessor::new(type_id)))
                .clone();

            //Assign index if this is a new storage type.
            let storage_index = storage_indices
                .entry(type_id)
                .or_insert(self.storages.len());

            //Clone the storage.
            let storage_arc = self
                .storages[*storage_index]
                .clone();

            guards.push(AccessGuard::new(accessor_arc, storage_arc));
        }
        
        guards
    }
*/
    /*
    ///Grants immutable access to the Resource mapped to passed-in TypeId,
    ///for as long as the lifetime of the passed-in AccessGuard.
    pub(crate) fn read<'a>(&'static self,
                           type_id: TypeId,
                           _guard: &'a AccessGuard<'a>) -> &'a Box<&dyn Resource> {
    
        let resource_indices = self
            .resource_indices
            .lock()
            .expect("Mutex found poisoned during world.read()");

        let resource_index = resource_indices
            .get(&type_id)
            .unwrap();

        let mut_ptr: &dyn Resource = self.resources[*resource_index].get() as &dyn Resource;

        &Box::new(mut_ptr)
    }
    
    ///Grants mutable access to the Resource mapped to the passed-in TypeId,
    ///for as long as the lifetime of the passed-in AccessGuard.
    pub(crate) fn write<'a>(&'static self,
                            type_id: TypeId,
                            _guard: &'a AccessGuard<'a>) -> &'a mut Box<dyn Resource> {
        
        let resource_indices = self
            .resource_indices
            .lock()
            .expect("Mutex found poisoned during world.read()");

        let resource_index = resource_indices
            .get(&type_id)
            .unwrap();

        self.resources[*resource_index]
    }*/
}
