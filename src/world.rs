//Jerome M. St.Martin
//June 15, 2022

use std::{
    any::TypeId, //TypeId::of<T>() -> TypeId;
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use super::{
    entity::Entities,
    storage::{ImmutableStorageGuard, MutableStorageGuard, Storage, StorageBox},
    Component,
    Entity, //usize
};

const STORAGE_POISON: &str = "storages mtx found poisoned in world.rs";
const ENTITIES_POISON: &str = "Entities mtx found poisoned in world.rs";
const MAINTENANCE_FN_POISON: &str = "maintenance_fns mtx found poisoned in world.rs";

///The core of the library; must instantiate.
pub struct World {
    //Arc<World>
    pub(crate) entities: Mutex<Entities>,
    storages: Mutex<HashMap<TypeId, StorageBox>>,
    maintenance_fns: Mutex<Vec<Box<dyn Fn(&World, &Entity)>>>,
}

impl World {
    pub fn new() -> Self {
        World {
            entities: Mutex::new(Entities::new()),
            storages: Mutex::new(HashMap::new()),
            maintenance_fns: Mutex::new(Vec::new()),
        }
    }

    pub fn create_entity(&self) -> Entity {
        let id = self
            .entities
            .lock()
            .expect("entities mtx found poisoned in World::init_entity()")
            .new_entity_id();

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
    /// let world = World::new();
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
        let entities_guard: MutexGuard<Entities> = self.entities.lock().expect(ENTITIES_POISON);

        entities_guard.vec().into_iter()
    }

    pub fn rm_entity(&self, e: Entity) {
        self.entities.lock().expect(ENTITIES_POISON).rm_entity(e);
    }

    ///Component types must be registered with the ECS before use. This fn also
    ///creates an FnMut() based for each registered component, which is used
    ///internally to maintain the ecs. (This is why world.maintain() must be
    ///called periodically.)
    ///
    /// ## Panics
    /// Panics if you register the same component type twice.
    pub fn register_component<T: Component>(&self) {
        let type_id = TypeId::of::<T>();

        let mut storages_guard: MutexGuard<'_, HashMap<TypeId, StorageBox>> =
            self.storages.lock().expect(STORAGE_POISON);

        if storages_guard.contains_key(&type_id) {
            panic!("attempted to register the same component type twice");
        }

        let should_be_none = storages_guard.insert(
            type_id,
            StorageBox {
                boxed: Arc::new(Storage::<T>::new()),
            },
        );

        assert!(should_be_none.is_none());

        //Generate Fn to be called in world.maintain() & store it in World
        fn maintain_storage<T>(world: &World, entity: &Entity) where T: Component {
            let mut mut_guard = world.req_write_guard::<T>();
            mut_guard.remove(entity);
        }

        let mut maint_fn_guard = self
            .maintenance_fns
            .lock()
            .expect(MAINTENANCE_FN_POISON);

        maint_fn_guard.push(Box::new(maintain_storage::<T>));
    }

    ///Adds a component of type T to the passed-in entity, replaces and returns
    ///the T that was already here, if any.
    pub fn add_component<T: Component>(&self, ent: Entity, comp: T) -> Option<T> {
        let mut storage_guard = self.req_write_guard::<T>(); //This may block.

        //'Attatch' component to ent
        let old_component = storage_guard.insert(ent, comp);
        old_component
    }

    ///Removes the component of the type T from this entity and returns it.
    ///If this component type didn't exist on this entity, None is returned.
    pub fn rm_component<T: Component>(&self, ent: &Entity) -> Option<T> {
        let mut storage_guard = self.req_write_guard::<T>(); //This may block.

        storage_guard.remove(ent)
    }

    //TODO: Docs
    pub fn maintain_ecs(&self) {
        let maint_fns = self
            .maintenance_fns
            .lock()
            .expect(MAINTENANCE_FN_POISON);

        let entities_guard = self
            .entities
            .lock()
            .expect(ENTITIES_POISON);

        let dead_ent_inter = entities_guard.dead_iter();
        let zipped = dead_ent_inter.zip(maint_fns.iter());

        for (entity, f) in zipped {
            f(&self, entity);
        }
    }

    ///Use to get thread-safe read-access to a single ECS Storage. If you need
    ///access to multiple Storages, prefer req_multi_read_guards.
    ///## Panics
    ///Panics if you call on an unregistered Component type, T.
    pub fn req_read_guard<T: Component>(&self) -> ImmutableStorageGuard<T> {
        let type_id = TypeId::of::<T>();

        //Request an ImmutableStorageGuard; blocks until read-access is allowed.
        let storage_arc = self
            .storages
            .lock()
            .expect(STORAGE_POISON)
            .get(&type_id)
            .unwrap_or_else(|| {
                panic!("Attempted to request access to unregistered component storage");
            })
            .clone_storage();

        ImmutableStorageGuard::new(storage_arc)
    }

    ///Similar to req_read_guard() but returns Some(ImmutableStorageGuard) only
    ///if the passed in Entity has a Component of type T. Else returns None.
    pub fn req_read_guard_if<T: Component>(
        &self,
        ent: &Entity,
    ) -> Option<ImmutableStorageGuard<T>> {
        let type_id = TypeId::of::<T>();

        //Request an ImmutableStorageGuard; blocks until read-access is allowed.
        let storage_arc = self
            .storages
            .lock()
            .expect(STORAGE_POISON)
            .get(&type_id)
            .unwrap_or_else(|| {
                panic!("Attempted to request access to uninitialized component storage");
            })
            .clone_storage();

        {
            let guard = ImmutableStorageGuard::new(storage_arc);

            if guard.get(ent).is_some() {
                return Some(guard);
            }
        }

        None
    }

    ///Use to get thread-safe write-access to a single ECS Storage. If you need
    ///access to multiple Storages, prefer req_multi_write_guards().
    ///
    /// ## Panics
    /// Panics if you call on an unregistered Component type, T.
    pub fn req_write_guard<T: Component>(&self) -> MutableStorageGuard<T> {
        let type_id = TypeId::of::<T>();

        let storage_arc = self
            .storages
            .lock()
            .expect(STORAGE_POISON)
            .get(&type_id)
            .unwrap_or_else(|| {
                panic!("Attempted to request access to uninitialized component storage");
            })
            .clone_storage();

        MutableStorageGuard::new(storage_arc)
    }

    ///Similar to req_write_guard() but returns Some(MutableStorageGuard) if
    ///the passed-in Entity has a Component of type T. Else returns None.
    pub fn req_write_guard_if<T: Component>(&self, ent: &Entity) -> Option<MutableStorageGuard<T>> {
        let type_id = TypeId::of::<T>();

        let storage_arc = self
            .storages
            .lock()
            .expect(STORAGE_POISON)
            .get(&type_id)
            .unwrap_or_else(|| {
                panic!("Attempted to request access to uninitialized component storage");
            })
            .clone_storage();

        {
            let guard = MutableStorageGuard::new(storage_arc);

            if guard.get_mut(ent).is_some() {
                return Some(guard);
            }
        }

        None
    }
}
