//Jerome M. St.Martin
//June 15, 2022

use std::{
    collections::HashMap,
    any::{Any, TypeId}, //TypeId::of<T>() -> TypeId;
    sync::{Arc, Mutex, MutexGuard},
    cell::UnsafeCell,
};

use super::{
    accessor::{Accessor, AccessGuard, AccessorState},
    storage::Storage,
    Entity, //usize
    MAX_COMPONENTS,
    entity::{Entities, builder::EntityBuilder},
};

pub struct World { //Arc<World>
    pub(crate) entities: Mutex<Entities>,
    accessors: Mutex<HashMap<TypeId, Arc<Accessor>>>,
    storage_idxs: Mutex<HashMap<TypeId, usize>>, //Values are indices of 'storages' vec.
    storages: UnsafeCell<[Option<Arc<Storage>>; MAX_COMPONENTS]>,
}

impl World {

    pub fn new() -> Self {

        const NONE: Option<Arc<Storage>> = None;
        let storage_array: [Option<Arc<Storage>>; MAX_COMPONENTS] = [NONE; MAX_COMPONENTS];

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

    ///Used internally by the Entity Builder Pattern, called in build().
    pub(crate) fn init_entity(&self) -> Entity {
        let id = self
            .entities
            .lock()
            .expect("antities mtx found poisoned in World.init_entity()")
            .new_entity_id();

        let accessor_map_guard: MutexGuard<'_, HashMap<TypeId, Arc<Accessor>>> = self
            .accessors
            .lock()
            .expect("accessors mtx found poisoned in World.init_entity()");

        let _storage_idxs_guard = self
            .accessors
            .lock()
            .expect("storage_idxs mtx found poisoned in World.init_entity()");

        //Borrow the storage and turn it into an iterator
        let unsafe_ptr = self.storages.get();
        let storage_iter = unsafe {
            let array: &[Option<Arc<Storage>>; MAX_COMPONENTS] = &*unsafe_ptr;
            let iter: std::slice::Iter<'_, Option<Arc<Storage>>> = array.as_slice().iter();
            iter
        };

        //Increase the length of all Storage vectors in ecs.storages
        for s in storage_iter {
            if let Some(storage) = s {
                //Acquire the Condvar's associated Mutex for this storage and
                //get write access.
                let type_id = storage.component_type;
                let accessor: Arc<Accessor> = accessor_map_guard
                    .get(&type_id)
                    .unwrap() //panic desired if accessor not found here
                    .clone();
                
                const ERR_MSG: &str = "Accessor mtx found poisoned in world.init_entity()";

                let mut accessor_state: MutexGuard<'_, AccessorState> = accessor
                    .mtx
                    .lock()
                    .expect(ERR_MSG);

                accessor_state.writers_waiting += 1;

                accessor_state = accessor
                    .writer_cvar
                    .wait_while(accessor.mtx.lock().expect(ERR_MSG),
                    |acc_state: &mut AccessorState| {
                        !acc_state.write_allowed
                    })
                    .expect(ERR_MSG);

                    accessor_state.read_allowed = false;
                    accessor_state.write_allowed = false;
                    accessor_state.writers_waiting -= 1;

                //Now that we have write access to this storage,
                //increase its length by 1, so the new Entity
                //MAY have a component placed here.
                //Awful lot of work to do something so small...
                let unsafe_ptr: *mut Vec<Option<Box<dyn Any>>> = storage.inner.get();
                unsafe {
                    let storage_vec: &mut Vec<Option<Box<dyn Any>>> = &mut *unsafe_ptr;
                    storage_vec.push(None);
                }

            } else {
                //Once we hit a None, there should be no further Some's
                //because component vecs are populated into the storage
                //array beginning at the head, and component types
                //cannot be removed from the ECS at runtime.
                break;
            }
        }
        
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
        let num_component_types = storage_idxs.len();
        let storage_idx = storage_idxs
            .entry(type_id)
            .or_insert(num_component_types);

        //Clone the storage.
        let unsafe_ptr: *mut [Option<Arc<Storage>>; MAX_COMPONENTS] = self.storages.get();
        let storage_arc = unsafe {
            let storages: &mut [Option<Arc<Storage>>; MAX_COMPONENTS] = &mut *unsafe_ptr;
            storages[*storage_idx]
            .get_or_insert_with(|| {
                    Arc::new(Storage::new(type_id))
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
            let unsafe_ptr: *mut [Option<Arc<Storage>>; MAX_COMPONENTS] = self.storages.get();
            let storage_arc = unsafe {
                let storages: &mut [Option<Arc<Storage>>; MAX_COMPONENTS] = &mut *unsafe_ptr;
                storages[*storage_idx]
                .get_or_insert_with(|| {
                        Arc::new(Storage::new(type_id))
                })
                .clone()
            };

            guards.push(AccessGuard::new(accessor_arc, storage_arc));
        }
        
        guards
    }
}
