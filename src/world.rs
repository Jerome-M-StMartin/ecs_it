//Jerome M. St.Martin
//June 15, 2022

use std::{
    any::{Any, TypeId}, //TypeId::of<T>() -> TypeId;
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use super::{
    entity::Entities,
    storage::{Storage, ImmutableStorageGuard, MutableStorageGuard},
    Entity, //usize
};

const STORAGE_POISON: &str = "storages mtx found poisoned in world.rs";

///The core of the library; must instantiate.

pub struct World {
    //Arc<World>
    pub(crate) num_entities_estimate: usize, //used as init size for Storage vecs
    pub(crate) entities: Mutex<Entities>,
    storages: Mutex<HashMap<TypeId, Arc<Storage>>>,
}

impl World {
    /// Provide a non-zero estimate of the total number of entities you wish to
    /// instantiate at any given time. It's used as the initialization length
    /// of Storage vecs.
    ///
    ///This estimate should probably not exceed a few thousand - if you need an
    ///ECS that can handle that volume in a performant manner I suggest 'specs',
    ///which provides mutliple underlying storage types, while this library uses
    ///only Vecs. So, for example, if you have a component associated with only
    ///a few Entities, say one Entity, but have 1,000 living Entities, there
    ///will exist a Vec with 999 'None' elements and 1 'Some' element.
    pub fn new(num_entities_estimate: usize) -> Self {
        assert!(num_entities_estimate > 0); //should also prob. not exceed ~10,000

        World {
            num_entities_estimate,
            entities: Mutex::new(Entities::new()),
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
            let storage_guard = self.req_access_while_map_locked(&storage_map_guard, type_id);
            let storage: &mut Vec<Option<Box<dyn Any>>> = storage_guard.raw_mut();

            //Now that we have write access to this storage,
            //increase its length by 1, so the new Entity
            //MAY have a component placed here.
            storage.push(None);
        }

        id
    }

    /// Clones all existing Entities into an UNSORTED Vec, then returns an
    /// iterator over that Vec; does not consume the underlying data structure.
    ///
    /// Reminder: an Entity is just a usize - nothing more.
    ///
    ///# Example
    ///```
    /// use ecs_it::world::World;
    ///
    /// let world = World::new(5);
    ///
    /// for _ in 0..5 {
    ///     world.create_entity();
    /// }
    ///
    /// for (i, ent) in world.entity_iter().enumerate() {
    ///     println!("i: {}, entity: {}", i, ent);
    /// }
    ///```
    pub fn entity_iter(&self) -> impl Iterator<Item = Entity> {
        let entities_guard: MutexGuard<Entities> = self
            .entities
            .lock()
            .expect("Entities mtx found poisoned in world.rs");
        
        entities_guard.vec().into_iter()
    }

    ///Component types must be registered with the ECS before they are accessed.
    ///
    /// ## Panics
    ///
    /// Panics if you register the same component type twice.
    pub fn register_component<T: 'static + Any>(&self) {
        let type_id = TypeId::of::<T>();

        let mut storages_guard: MutexGuard<'_, HashMap<TypeId, Arc<Storage>>> = self
            .storages
            .lock()
            .expect(STORAGE_POISON);

        if storages_guard.contains_key(&type_id) {
            panic!("attempted to register the same component type twice");
        }

        let should_be_none = storages_guard.insert(
            type_id,
            Arc::new(Storage::new(type_id, self.num_entities_estimate)),
        );

        assert!(should_be_none.is_none());
    }

    ///Adds a component of type T to the passed-in entity, replaces and returns
    ///the T that was already here, if any.
    ///
    ///Note: In the case that the passed-in entity has an existing component
    ///of this type, this does NOT re-allocate memory. The new component is
    ///placed into the old component's Box<>.
    pub fn add_component<T: 'static + Any>(&self, ent: Entity, comp: T) -> Option<T> {
        let guard = self.req_write_guard::<T>(); //This may block.

        let storage: &mut Vec<Option<Box<dyn Any>>> = guard.raw_mut();
        let entity_slot: &mut Option<Box<dyn Any>> = &mut storage[ent];

        //To avoid unneccesary memory allocations, we re-use the box in this
        //entity_slot if it already exists. Else we allocate a new box.
        if entity_slot.is_some() {
            let mut box_to_reuse: Box<dyn Any> = entity_slot.take().unwrap();
            let old_comp: &mut T = &mut *box_to_reuse.downcast_mut::<T>().unwrap();
            let old_comp_raw: T = std::mem::replace(old_comp, comp); //old_comp is discarded
            let _: &mut Box<dyn Any + 'static> = entity_slot.insert(box_to_reuse);
            
            Some(old_comp_raw)
        } else {
            let _ = entity_slot.insert(Box::new(comp));
            None
        }
    }

    ///Removes the component of the type T from this entity and returns it.
    ///If this component type didn't exist on this entity, None is returned.
    pub fn rm_component<T: 'static>(&self, ent: Entity) -> Option<Box<dyn Any>> {
        let guard = self.req_write_guard::<T>(); //This may block.

        let storage: &mut Vec<Option<Box<dyn Any>>> = guard.raw_mut();
        let entity_slot: &mut Option<Box<dyn Any>> = &mut storage[ent];

        entity_slot.take()
    }

    ///Use to get thread-safe read-access to a single ECS Storage. If you need
    ///access to multiple Storages, prefer req_multi_read_guards.
    ///## Panics
    ///Panics if you call on an unregistered Component type, T.
    pub fn req_read_guard<T: 'static + Any>(&self) -> ImmutableStorageGuard {
        let type_id = TypeId::of::<T>();

        //Request an ImmutableStorageGuard; blocks until read-access is allowed.
        let storage = self
            .storages
            .lock()
            .expect(STORAGE_POISON)
            .get(&type_id)
            .unwrap_or_else(|| {
                panic!("Attempted to request access to unregistered component storage");
            })
            .clone();

        storage.init_read_access();
        ImmutableStorageGuard::new(storage)
    }

    ///Similar to req_read_guard() but returns Some(ImmutableStorageGuard) only
    ///if the passed in Entity has a Component of type T. Else returns None.
    pub fn req_read_guard_if<T: 'static + Any>(&self, ent: Entity) -> Option<ImmutableStorageGuard> {
        let type_id = TypeId::of::<T>();

        //Request an ImmutableStorageGuard; blocks until read-access is allowed.
        let storage = self
            .storages
            .lock()
            .expect(STORAGE_POISON)
            .get(&type_id)
            .unwrap_or_else(|| {
                panic!("Attempted to request access to uninitialized component storage");
            })
            .clone();
        
        {
            storage.init_read_access();
            let guard = ImmutableStorageGuard::new(storage);

            if guard.get(ent).is_some() {
                return Some(guard)
            }
        }

        None
    }

    ///Use to get thread-safe write-access to a single ECS Storage. If you need
    ///access to multiple Storages, prefer req_multi_write_guards().
    ///
    /// ## Panics
    /// Panics if you call on an unregistered Component type, T.
    pub fn req_write_guard<T: 'static + Any>(&self) -> MutableStorageGuard {
        let type_id = TypeId::of::<T>();

        let storage = self
            .storages
            .lock()
            .expect(STORAGE_POISON)
            .get(&type_id)
            .unwrap_or_else(|| {
                panic!("Attempted to request access to uninitialized component storage");
            })
            .clone();

        storage.init_write_access();
        MutableStorageGuard::new(storage)
    }

    ///Similar to req_write_guard() but returns Some(MutableStorageGuard) if
    ///the passed-in Entity has a Component of type T. Else returns None.
    pub fn req_write_guard_if<T: 'static + Any>(&self, ent: Entity) -> Option<MutableStorageGuard> {
        let type_id = TypeId::of::<T>();

        let storage = self
            .storages
            .lock()
            .expect(STORAGE_POISON)
            .get(&type_id)
            .unwrap_or_else(|| {
                panic!("Attempted to request access to uninitialized component storage");
            })
            .clone();
        
        {
            storage.init_write_access();
            let guard = MutableStorageGuard::new(storage);

            if guard.get_mut(ent).is_some() {
                return Some(guard)
            }
        }

        None
    }

    ///Use to getthread-safe read-access to multiple ECS Storages and/or
    ///Resources simultaneously. If you need access to only one Storage,
    ///req_read_guard() should be preferred.
    pub fn req_multi_read_guards(&self, id_vec: Vec<TypeId>) -> Vec<ImmutableStorageGuard> {
        let mut guards: Vec<ImmutableStorageGuard> = Vec::new();

        //Lock storages map until all requested Storages are acquired and returned.
        let storage_map_guard = self.storages.lock().expect(STORAGE_POISON);

        for type_id in id_vec {
            let storage = storage_map_guard
                .get(&type_id)
                .unwrap_or_else(|| {
                    panic!("Attempted to request access to uninitialized component storage");
                })
                .clone();

            storage.init_read_access();
            guards.push(ImmutableStorageGuard::new(storage));
        }

        guards
    }

    ///Use to get thread-safe write-access to multiple ECS Storages and/or
    ///Resources simultaneously. If you need access to only one Storage,
    ///req_write_guard() should be preferred.
    pub fn req_multi_write_guards(&self, id_vec: Vec<TypeId>) -> Vec<MutableStorageGuard> {
        let mut guards: Vec<MutableStorageGuard> = Vec::new();

        //Lock storages map until all requested Storages are acquired and returned.
        let storage_map_guard = self.storages.lock().expect(STORAGE_POISON);

        for type_id in id_vec {
            let storage = storage_map_guard
                .get(&type_id)
                .unwrap_or_else(|| {
                    panic!("Attempted to request access to uninitialized component storage");
                })
                .clone();

            storage.init_read_access(); 
            guards.push(MutableStorageGuard::new(storage));
        }

        guards
    }

    ///Used internally during entity initialization.
    fn req_access_while_map_locked<'a>(
        &'a self,
        storage_map_guard: &'a MutexGuard<'_, HashMap<TypeId, Arc<Storage>>>,
        type_id: &TypeId,
    ) -> MutableStorageGuard {
        let storage = storage_map_guard
            .get(type_id)
            .unwrap_or_else(|| {
                panic!("Attempted to request access to uninitialized component storage");
            })
            .clone();

        storage.init_write_access();
        MutableStorageGuard::new(storage)
    }
}
