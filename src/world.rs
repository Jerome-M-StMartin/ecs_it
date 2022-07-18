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

///The core of the library; must instantiate (via World::new()).
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

    ///Inserts a "blank" Entity into the World. You need to call
    ///add_component() to allow this Entity to do/be anything of
    ///substance. Returns the entity ID, which is a usize, which
    ///is type-aliased as "Entity" in this library.
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

    ///When entities "die" or otherwise need to be removed from the game world,
    ///this is the fn to call. See: World::ecs_maintain()
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
        fn maintain_storage<T>(world: &World, entity: &Entity)
        where
            T: Component,
        {
            let mut mut_guard = world.req_write_guard::<T>();
            mut_guard.remove(entity);
        }

        let mut maint_fn_guard = self.maintenance_fns.lock().expect(MAINTENANCE_FN_POISON);

        maint_fn_guard.push(Box::new(maintain_storage::<T>));
    }

    ///Adds a component of type T to the passed-in entityr; replaces and returns
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

    ///Must be called every once and a while, depending on how often Entities
    ///are being "killed" in your game. If you don't call this, all Component
    ///data attached to killed entities will live in memory forever. In other
    ///words, if you don't call this you'll have a memory leak.
    ///
    ///You can call it every frame, but it mutably acceses ALL storages,
    ///iteratively, so no other System can be reaching into the ECS at the
    ///time. If only a few Entities are killed per second or minute of runtime,
    ///you can write some logic to call this once every few seconds or so and
    ///that would probably be fine.
    ///
    ///This should probably be called at the end of a game tick(), or maybe at
    ///the start of a game tick(). Anywhere but right in the middle, because
    ///you'll operate on garbage data in your Systems. This won't be a
    ///"problem" per-se, but it will result in wasted CPU cycles.
    ///
    ///# Example
    ///```
    /// use ecs_it::{
    ///     world::World,
    ///     Component,
    /// };
    ///
    /// #[derive(Debug)]
    /// struct DummyComponent {
    ///     dummy_data: usize,
    /// }
    /// impl Component for DummyComponent {}
    ///
    /// let world = World::new();
    /// world.register_component::<DummyComponent>();
    ///
    /// let ent1 = world.create_entity();
    /// world.add_component(ent1, DummyComponent { dummy_data: 1337 });
    ///
    /// let ent2 = world.create_entity();
    /// world.add_component(ent2, DummyComponent { dummy_data: 9001 });
    ///
    /// { //scope-in to drop the guard when required
    ///     let guard = world.req_read_guard::<DummyComponent>();
    ///
    ///     //This should print two DummyComponents, one for each entitiy.
    ///     for component in guard.iter() {
    ///         println!("{:?}", component);
    ///     }
    ///
    ///     world.rm_entity(ent1);
    ///
    ///     //Despite removing an Entity, this will still print two DummyComponents
    ///     //which is erroneous behaviour. To prevent this, call ecs_maintain()
    ///     //after each call to rm_entity() but prior to using any Component data
    ///     //which may have been associated with the removed Entity.
    ///     for component in guard.iter() {
    ///         println!("{:?}", component);
    ///     }
    ///
    ///     //Drop the guard first, because maintain_ecs() grabs all of them.
    ///     //This is done by letting the guard fall out of scope in the next
    ///     //line.
    /// }
    ///
    /// world.maintain_ecs();
    ///
    /// //re-acquire the guard
    /// let guard = world.req_read_guard::<DummyComponent>();
    ///
    /// //This should only print the DummyComponent associated with the living
    /// //Entity "ent2". All components associated with dead Entities is now
    /// //dropped from memory.
    /// for component in guard.iter() {
    ///     println!("{:?}", component);
    /// }
    ///```
    pub fn maintain_ecs(&self) {
        let maint_fns = self.maintenance_fns.lock().expect(MAINTENANCE_FN_POISON);

        let entities_guard = self.entities.lock().expect(ENTITIES_POISON);

        let dead_ent_inter = entities_guard.dead_iter();
        let zipped = dead_ent_inter.zip(maint_fns.iter());

        for (entity, f) in zipped {
            f(&self, entity);
        }
    }

    ///Use to get thread-safe read-access to a single ECS Storage.
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

    ///Use to get thread-safe write-access to a single ECS Storage.
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
