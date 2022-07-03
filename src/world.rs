//Jerome M. St.Martin
//June 15, 2022

use std::{
    any::{Any, TypeId}, //TypeId::of<T>() -> TypeId;
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, MutexGuard},
};

use super::{
    entity::Entities,
    storage::{Storage, ImmutableStorageGuard, MutableStorageGuard},
    Entity, //usize
};

const STORAGE_POISON: &str = "storages mtx found poisoned in world.rs";

///The root of the library; must instantiate. Provide a non-zero estimate of
///the total number of entities you wish to instantiate at any given time.
///It's used as the initialization length of Storage vecs.
///
///This estimate should probably not exceed a few thousand - if you need an
///ECS that can handle that volume in a performant manner I suggest 'specs'.
pub struct World {
    //Arc<World>
    pub(crate) num_entities_estimate: usize, //used as init size for Storage vecs.
    pub(crate) entities: Mutex<Entities>,
    anti_deadlock_set: Mutex<HashSet<TypeId>>, //tracks which Storages have living AccessGuards
    storages: Mutex<HashMap<TypeId, Arc<Storage>>>,
}

impl<'a> World {
    pub fn new(num_entities_estimate: usize) -> Self {
        assert!(num_entities_estimate > 0); //should also prob. not exceed ~10,000

        World {
            num_entities_estimate,
            entities: Mutex::new(Entities::new()),
            anti_deadlock_set: Mutex::new(HashSet::new()),
            storages: Mutex::new(HashMap::new()),
        }
    }

    pub fn create_entity(&self) -> Entity {
        self.init_entity()
    }

    pub(crate) fn init_entity(&self) -> Entity {
        let id = self
            .entities
            .lock()
            .expect("entities mtx found poisoned in World::init_entity()")
            .new_entity_id();

        let storage_map_guard = self.storages.lock().expect(STORAGE_POISON);

        let storage_map_keys = storage_map_guard.keys();

        //Increase the length of all Storage vectors in ecs.storages
        for type_id in storage_map_keys {
            let storage_guard = World::req_access_while_map_locked(&storage_map_guard, type_id);
            let storage: &mut Vec<Option<Box<dyn Any>>> = storage_guard.raw_mut();

            //Now that we have write access to this storage,
            //increase its length by 1, so the new Entity
            //MAY have a component placed here.
            storage.push(None);
        }

        id
    }

    ///Component types must be registered with the ECS before they are used
    ///or otherwise accessed. Failing to do so will cause a panic.
    pub fn register_component<T: 'static + Any>(&self) {
        let type_id = TypeId::of::<T>();
        let mut storages_guard: MutexGuard<'_, HashMap<TypeId, Arc<Storage>>> =
            self.storages.lock().expect(STORAGE_POISON);

        if storages_guard.contains_key(&type_id) {
            //This component has already been registered.
            return;
        }

        let should_be_none = storages_guard.insert(
            type_id,
            Arc::new(Storage::new(type_id, self.num_entities_estimate)),
        );

        assert!(should_be_none.is_none());
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
        let guard = self.req_write_access::<T>(); //This may block.

        let storage: &mut Vec<Option<Box<dyn Any>>> = guard.raw_mut();
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
        let guard = self.req_write_access::<T>(); //This may block.

        let storage: &mut Vec<Option<Box<dyn Any>>> = guard.raw_mut();
        let entity_slot: &mut Option<Box<dyn Any>> = &mut storage[ent];

        entity_slot.take()
    }

    ///Use to get thread-safe read-access to a single ECS Storage or Resource.
    ///When you need access to multiple Storages and/or Resources (such as when
    ///you're running a sufficiently complex System) use req_multi_access().
    ///
    ///This fn uses an unsafe block to access an UnsafeCell to allow for
    ///interior mutability across thread boundaries. This is required
    ///because this ECS uses lazy initialization of storages: each access
    ///request for a novel Storage type causes runtime initialization of
    ///that storage.
    pub fn req_read_access<T: 'static + Any>(&self) -> ImmutableStorageGuard {
        let type_id = TypeId::of::<T>();

        let storage_arc = self
            .storages
            .lock()
            .expect(STORAGE_POISON)
            .get(&type_id)
            .unwrap_or_else(|| {
                panic!("Attempted to request access to uninitialized component storage");
            })
            .clone();

        ImmutableStorageGuard::new(storage_arc)
    }

    ///Use to get thread-safe write-access to a single ECS Storage or Resource.
    ///When you need access to multiple Storages and/or Resources (such as when
    ///you're running a sufficiently complex System) use req_multi_access().
    ///
    ///This fn uses an unsafe block to access an UnsafeCell to allow for
    ///interior mutability across thread boundaries. This is required
    ///because this ECS uses lazy initialization of storages: each access
    ///request for a novel Storage type causes runtime initialization of
    ///that storage.
    pub fn req_write_access<T: 'static + Any>(&self) -> MutableStorageGuard {
        let type_id = TypeId::of::<T>();

        let storage_arc = self
            .storages
            .lock()
            .expect(STORAGE_POISON)
            .get(&type_id)
            .unwrap_or_else(|| {
                panic!("Attempted to request access to uninitialized component storage");
            })
            .clone();

        MutableStorageGuard::new(storage_arc)
    }

    ///Use to getthread-safe read-access to multiple ECS Storages and/or
    ///Resources simultaneously. If you need access to only one Storage or
    ///Resource, req_access() should be preferred.
    ///
    ///This fn uses an unsafe block to access an UnsafeCell to allow for
    ///interior mutability across thread boundaries. This is required
    ///because this ECS uses lazy initialization of storages: each access
    ///request for a novel Storage type causes runtime initialization of
    ///that storage.
    pub fn req_multi_read_access(&self, id_vec: Vec<TypeId>) -> Vec<ImmutableStorageGuard> {
        let mut guards: Vec<ImmutableStorageGuard> = Vec::new();

        //Lock storages map until all requested Storages are acquired and returned.
        let storage_map_guard = self.storages.lock().expect(STORAGE_POISON);

        for type_id in id_vec {
            let storage_arc = storage_map_guard
                .get(&type_id)
                .unwrap_or_else(|| {
                    panic!("Attempted to request access to uninitialized component storage");
                })
                .clone();

            guards.push(ImmutableStorageGuard::new(storage_arc));
        }

        guards
    }

    ///Use to get thread-safe write-access to multiple ECS Storages and/or
    ///Resources simultaneously. If you need access to only one Storage or
    ///Resource, req_access() should be preferred.
    ///
    ///This fn uses an unsafe block to access an UnsafeCell to allow for
    ///interior mutability across thread boundaries. This is required
    ///because this ECS uses lazy initialization of storages: each access
    ///request for a novel Storage type causes runtime initialization of
    ///that storage.
    pub fn req_multi_write_access(&self, id_vec: Vec<TypeId>) -> Vec<MutableStorageGuard> {
        let mut guards: Vec<MutableStorageGuard> = Vec::new();

        //Lock storages map until all requested Storages are acquired and returned.
        let storage_map_guard = self.storages.lock().expect(STORAGE_POISON);

        for type_id in id_vec {
            let storage_arc = storage_map_guard
                .get(&type_id)
                .unwrap_or_else(|| {
                    panic!("Attempted to request access to uninitialized component storage");
                })
                .clone();

            guards.push(MutableStorageGuard::new(storage_arc));
        }

        guards
    }

    ///Used internally during entity initialization.
    fn req_access_while_map_locked(
        storage_map_guard: &MutexGuard<'_, HashMap<TypeId, Arc<Storage>>>,
        type_id: &TypeId,
    ) -> MutableStorageGuard {
        let storage_arc = storage_map_guard
            .get(type_id)
            .unwrap_or_else(|| {
                panic!("Attempted to request access to uninitialized component storage");
            })
            .clone();

        MutableStorageGuard::new(storage_arc)
    }
}
