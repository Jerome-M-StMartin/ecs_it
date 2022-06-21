//Jerome M. St.Martin
//June 15, 2022

use std::{
    collections::HashMap,
    any::{Any, TypeId}, //TypeId::of<T>() -> TypeId;
    sync::{Arc, Mutex},
    cell::UnsafeCell,
};

use super::{
    accessor::{Accessor, AccessGuard},
    Storage, //Arc<UnsafeCell<Vec<Option<Box<dyn Any>>>>>
    Entity, //usize
    MAX_COMPONENTS,
    entity::{Entities, EntityBuilder},
};

pub struct World { //Arc<World>
    pub(crate) entities: Mutex<Entities>,
    accessors: Mutex<HashMap<TypeId, Arc<Accessor>>>,
    storage_idxs: Mutex<HashMap<TypeId, usize>>, //Values are indices of 'storages' vec.
    storages: UnsafeCell<[Option<Storage>; MAX_COMPONENTS]>,
}

impl World {

    pub fn new() -> Self {

        const NONE: Option<Storage> = None;
        let storage_array: [Option<Storage>; MAX_COMPONENTS] = [NONE; MAX_COMPONENTS];

        World {
            entities: Mutex::new(Entities::new()),
            accessors: Mutex::new(HashMap::new()),
            storage_idxs: Mutex::new(HashMap::new()),
            storages: UnsafeCell::new(storage_array),
        }
    }

    pub fn build_entity() -> EntityBuilder {
        EntityBuilder::new()
    }

    ///Used internally by the Entity Builder Pattern, called when build() is called.
    pub(crate) fn init_entity(&self) -> Entity {
        let id = self
            .entities
            .lock()
            .expect("Entities mtx poisoned.")
            .new_entity_id();

        //TODO
        //Increase the length of all Storage vectors in ecs.storages
        
        id
    }
    
    ///Adds a component of type T, which must be Any, to the passed-in entity.
    ///The component's storage is lazily initialized herein, so this can be
    ///the first time the ECS learns about this component type without causing
    ///any issue.
    ///
    ///Note: In the case that the passed-in entity has an existing component
    ///of this type, this does NOT re-allocate memory. The new component is
    ///placed into the old component's box. :]
    pub fn add_component<T: 'static + Any>(&self, ent: Entity, comp: T) {

        let guard = self.req_access::<T>(); //This may block.

        let storage: &mut Vec<Option<Box<dyn Any>>> = guard.val_mut::<T>();
        let entity_slot: &mut Option<Box<dyn Any>> = &mut storage[ent];
        
        //To avoid unneccesary memory allocations, we re-use the box in this
        //entity_slot if it already exists. Else we allocate a new box.
        if entity_slot.is_some() {
            let mut box_to_reuse: Box<dyn Any> = entity_slot.take().unwrap();
            let old_comp: &mut T = &mut *box_to_reuse.downcast_mut::<T>().unwrap();
            let _: T = std::mem::replace(old_comp, comp); //old_comp is discarded
            let _: &mut Box<dyn Any + 'static> = entity_slot.insert(box_to_reuse);
        } else {
            let _ = entity_slot.insert(Box::new(comp));
        }
    }

    ///Removes the component of the type T from this entity and returns it.
    ///If this component type didn't exist on this entity, None is returned.
    pub fn rm_component<T: 'static>(&self, ent: Entity) -> Option<Box<dyn Any>> {

        let guard = self.req_access::<T>(); //This may block.

        let storage: &mut Vec<Option<Box<dyn Any>>> = guard.val_mut::<T>();
        let entity_slot: &mut Option<Box<dyn Any>> = &mut storage[ent];

        entity_slot.take()
    }

    ///Use this to gain thread-safe access to a single ECS Storage or Resource.
    ///When you need access to multiple Storages and/or Resources (such as when
    ///you're running a sufficiently complex System) use req_multi_access().
    ///
    ///This fn uses an unsafe block to access an UnsafeCell to allow for
    ///interior mutability across thread boundaries. This is required
    ///because this ECS uses lazy initialization of storages: each access
    ///request for a novel Storage type causes runtime initialization of
    ///that storage.
    pub fn req_access<T: 'static + Any>(&self) -> AccessGuard {
        
        let type_id = TypeId::of::<T>();

        //Acquire Lock
        let mut accessors = self
            .accessors
            .lock()
            .expect("Mutex found poisoned during world.req_access()");

        //Acquire Lock
        let mut storage_idxs = self
            .storage_idxs
            .lock()
            .expect("Mutex found poisoned during world.req_access()");
    
        //Create accessor if this is a new storage type. Clone it either way.
        let accessor_arc: Arc<Accessor> = accessors.entry(type_id)
            .or_insert(Arc::new(Accessor::new(type_id)))
            .clone();    

        //Create index/key if this is a new storage type.
        let num_components = storage_idxs.len();
        let storage_idx = storage_idxs
            .entry(type_id)
            .or_insert(num_components);

        //Clone the storage.
        let unsafe_ptr: *mut [Option<Storage>; MAX_COMPONENTS] = self.storages.get();
        let storage_arc = unsafe {
            let storage_arc: &mut [Option<Storage>; MAX_COMPONENTS] = &mut *unsafe_ptr;
            storage_arc[*storage_idx]
            .get_or_insert_with(|| {
                    let mut new_vec = Vec::with_capacity(MAX_COMPONENTS);
                    new_vec.fill_with(|| { None });
                    Arc::new(UnsafeCell::new(new_vec))
            })
            .clone()
        };

        AccessGuard::new(accessor_arc, storage_arc)
    }

    ///Use this to gain thread-safe access to multiple ECS Storages and/or
    ///Resources simultaneously. If you need access to only one Storage or
    ///Resource, req_access() should be preferred.
    ///
    ///This fn uses an unsafe block to access an UnsafeCell to allow for
    ///interior mutability across thread boundaries. This is required
    ///because this ECS uses lazy initialization of storages: each access
    ///request for a novel Storage type causes runtime initialization of
    ///that storage.
    pub fn req_multi_access(&self, id_vec: Vec<TypeId>) -> Vec<AccessGuard> {

        let mut guards: Vec<AccessGuard> = Vec::new();

        //Acquire Lock
        let mut accessors = self
            .accessors
            .lock()
            .expect("Mutex found poisoned during world.req_multi_access()");

        //Acquire Lock
        let mut storage_idxs = self
            .storage_idxs
            .lock()
            .expect("Mutex found poisoned during world.req_multi_access()");

        for type_id in id_vec {

            //Create accessor if this is a new storage type. Clone it either way.
            let accessor_arc = accessors
                .entry(type_id)
                .or_insert(Arc::new(Accessor::new(type_id)))
                .clone();

            //Create index/key if this is a new storage type. Get key either way.
            let num_components = storage_idxs.len();
            let storage_idx = storage_idxs
                .entry(type_id)
                .or_insert(num_components);

            //Create storage if this is a new type. Clone it either way.
            let unsafe_ptr: *mut [Option<Storage>; MAX_COMPONENTS] = self.storages.get();
            let storage_arc = unsafe {
                let storage_arc: &mut [Option<Storage>; MAX_COMPONENTS] = &mut *unsafe_ptr;
                storage_arc[*storage_idx]
                .get_or_insert_with(|| {
                        let mut new_vec = Vec::with_capacity(MAX_COMPONENTS);
                        new_vec.fill_with(|| { None });
                        Arc::new(UnsafeCell::new(new_vec))
                })
                .clone()
            };

            guards.push(AccessGuard::new(accessor_arc, storage_arc));
        }
        
        guards
    }
}
