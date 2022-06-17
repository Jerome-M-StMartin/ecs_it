//Jerome M. St.Martin
//June 15, 2022

use std::{
    collections::HashMap,
    any::TypeId, //TypeId::of<T>() -> TypeId;
    sync::{Arc, Mutex},
};

use super::{
    accessor::{Accessor, AccessGuard},
    resource::Resource,
    //storage::Storage, //Storages are Resources
};

pub struct World {
    accessors: Mutex<HashMap<TypeId, Arc<Accessor>>>,
    resource_indices: Mutex<HashMap<TypeId, usize>>, //Values are indices of 'resources' vec.
    resources: Vec<Box<dyn Resource>>,
}

impl World {

    ///Use this to gain thread-safe access to a single ECS Storage or Resource.
    ///When you need access to multiple Storages and/or Resources (such as when
    ///you're running a sufficiently complex System) use req_multi_access().
    pub fn req_access(&self, type_id: TypeId) -> AccessGuard {

        //Acquire Lock
        let mut accessors = self
            .accessors
            .lock()
            .expect("Mutex found poisoned during world.req_access()");

        //Acquire Lock
        let mut resource_indices = self
            .resource_indices
            .lock()
            .expect("Mutex found poisoned during world.req_access()");
    
        //Create accessor if this is a new resource type. Make handle to it either way.
        let accessor_arc: Arc<Accessor> = accessors.entry(type_id)
            .or_insert(Arc::new(Accessor::new(type_id)))
            .clone();    

        //Create index/key if this is a new resource type.
        let _ = resource_indices
            .entry(type_id)
            .or_insert(self.resources.len());

        AccessGuard::new(accessor_arc)
    }

    ///Use this to gain thread-safe access to multiple ECS Storages and/or
    ///Resources simultaneously. If you need access to only one Storage or
    ///Resource, req_access() should be preferred.
    pub fn req_multi_access(&self, id_vec: Vec<TypeId>) -> Vec<AccessGuard> {

        let mut guards: Vec<AccessGuard> = Vec::new();

        //Acquire Lock
        let mut accessors = self
            .accessors
            .lock()
            .expect("Mutex found poisoned during world.req_multi_access()");

        //Acquire Lock
        let mut resource_indices = self
            .resource_indices
            .lock()
            .expect("Mutex found poisoned during world.req_multi_access()");

        for type_id in id_vec {
            //Create accessor if this is a new resource type. Else grab it.
            let accessor_arc = accessors
                .entry(type_id)
                .or_insert(Arc::new(Accessor::new(type_id)))
                .clone();

            //Assign index if this is a new resource type.
            let _ = resource_indices
                .entry(type_id)
                .or_insert(self.resources.len());

            guards.push(AccessGuard::new(accessor_arc));
        }
        
        guards
    }

    pub(crate) fn read<'a>(&'static self,
                           type_id: TypeId,
                           _guard: &'a AccessGuard<'a>) -> &'a Box<dyn Resource> {
    
        let resource_indices = self
            .resource_indices
            .lock()
            .expect("Mutex found poisoned during world.read()");

        let resource_index = resource_indices
            .get(&type_id)
            .unwrap();

        &self.resources[*resource_index]
    }

    pub(crate) fn write<'a>(&'static mut self,
                            type_id: TypeId,
                            _guard: &'a AccessGuard<'a>) -> &'a mut Box<dyn Resource> {
        
        let resource_indices = self
            .resource_indices
            .lock()
            .expect("Mutex found poisoned during world.read()");

        let resource_index = resource_indices
            .get(&type_id)
            .unwrap();

        &mut self.resources[*resource_index]
    }
}
