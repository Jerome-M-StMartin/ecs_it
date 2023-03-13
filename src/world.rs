//Jerome M. St.Martin
//June 15, 2022

use std::{
    any::TypeId, //TypeId::of<T>() -> TypeId;
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use super::{
    component::Component,
    entity::Entities,
    storage::{ImmutableStorageGuard, MutableStorageGuard, Storage},
    warehouse::{StorageBox, Warehouse},
    Entity, //usize
};

const ENTITIES_POISON: &str = "Entities mtx found poisoned in world.rs";
const MAINTENANCE_FN_POISON: &str = "maintenance_fns mtx found poisoned in world.rs";
const WAREHOUSE_POISON: &str = "warehouse mtx found poisoned in world.rs";

///The core of the library; must instantiate (via World::new()).
pub struct World {
    //Arc<World>
    pub(crate) entities: Mutex<Entities>,
    warehouse: Mutex<Warehouse>,
    //storages: Mutex<HashMap<TypeId, StorageBox>>,
    //maintenance_fns: Mutex<Vec<Box<dyn Fn(&World, &Entity)>>>,
}

impl World {
    pub fn new() -> Self {
        World {
            entities: Mutex::new(Entities::new()),
            warehouse: Mutex::new(Warehouse::new()),
            //storages: Mutex::new(HashMap::new()),
            //maintenance_fns: Mutex::new(Vec::new()),
        }
    }

    ///All compoent storages are held in the warehouse.
    ///TODO: further docs & focused doctest
    pub fn open_warehouse(&self) -> MutexGuard<Warehouse> {
        let warehouse_guard = self.warehouse.lock().expect(WAREHOUSE_POISON);
        warehouse_guard
    }

    ///Inserts a "blank" Entity into the World. You need to call
    ///add_component() to allow this Entity to do/be anything of
    ///substance. Returns the entity ID, which is a usize, which
    ///is type-aliased as "Entity" in this library.
    pub fn create_entity(&self) -> Entity {
        todo!()
    }   

    ///When entities "die" or otherwise need to be removed from the game world,
    ///this is the fn to call. See: World::maintain_ecs()
    pub fn rm_entity(&self, e: Entity) {
        self.entities.lock().expect(ENTITIES_POISON).rm_entity(e);
        todo!()
    }

    ///Component types must be registered with the ECS before use. This fn also
    ///creates an FnMut() for each registered component, which is used
    ///internally to maintain the ecs. (This is why world.maintain_ecs() must be
    ///called periodically.)
    ///
    /// ## Panics
    /// Panics if you register the same component type twice.
    pub fn register_component<T: Component>(&self) {
        let type_id = TypeId::of::<T>();

        let warehouse = self.open_warehouse();
        todo!()

        /*
        if warehouse_guard.storages.contains_key(&type_id) {
            panic!("attempted to register the same component type twice");
        }

        let should_be_none = warehouse_guard.storages.insert(
            type_id,
            StorageBox {
                boxed: Arc::new(Storage::<T>::new()),
            },
        );

        assert!(should_be_none.is_none());

        //Generate Fn to be called in world.maintain_ecs() & store it in World
        fn maintain_storage<T>(world: &World, entity: &Entity)
        where
            T: Component,
        {
            let mut mut_guard = world.req_write_guard::<T>();
            mut_guard.remove(entity);
        }

        let mut warehouse_guard = self.warehouse.lock().expect(MAINTENANCE_FN_POISON);
        let mut maint_fns = warehouse_guard.maintenance_functions;

        maint_fns.push(Box::new(maintain_storage::<T>));
        */
    }

    ///Adds a component of type T to the passed-in entity; replaces and returns
    ///the T that was already here, if any.
    pub fn add_component<T: Component>(&self, ent: Entity, comp: T) -> Option<T> {
        todo!()
    }

    ///Removes the component of the type T from this entity and returns it.
    ///If this component type didn't exist on this entity, None is returned.
    pub fn rm_component<T: Component>(&self, ent: &Entity) -> Option<T> {
        todo!()
    }

    ///Must be called every once and a while, depending on how often Entities
    ///are being "killed" in your game. If you don't call this, all Component
    ///data attached to killed entities will live in memory forever. In other
    ///words, if you don't call this you'll have a memory leak.
    ///
    ///You can call it every frame, but it mutably acceses ALL storages,
    ///iteratively, so no other System can reach into the ECS at the time. If
    ///only a few Entities are killed per second or minute of runtime, you can
    ///write some logic to call this once every few seconds or so and that
    ///would probably be fine.
    ///
    ///This should be called at the end of a game tick(), or maybe at the
    ///start of a game tick(). Anywhere but right in the middle, because
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
    /// //Entity "ent2". All components associated with dead Entities are now
    /// //dropped from memory.
    /// for component in guard.iter() {
    ///     println!("{:?}", component);
    /// }
    ///```
    pub fn maintain_ecs(&self) {
        todo!()
        /*
        let warehouse_guard = self.warehouse.lock().expect(MAINTENANCE_FN_POISON);
        let maint_fns = warehouse_guard.maintenance_functions;

        let entities_guard = self.entities.lock().expect(ENTITIES_POISON);

        //let dead_ent_iter = entities_guard.dead_entities_iter();
        //let zipped = dead_ent_iter.zip(maint_fns.iter());

        //TODO: Verify that this zip is what I want... is each f guaranteed
        //      to be correctly paired with its associated entity?
        //Later Me: positive that this was wrong. Going to need to change
        //to work with vec Storages anyway.
        /*for (entity, f) in zipped {
            f(&self, entity);
        }*/
        */
    }

    /*
    ///Use to get thread-safe read-access to a single ECS Storage.
    ///## Panics
    ///Panics if you call on an unregistered Component type, T.
    pub fn req_read_guard<T: Component>(&self) -> ImmutableStorageGuard<T> {
        //Acquire lock on the Warehouse
        let warehouse = self.warehouse.lock().expect(WAREHOUSE_POISON);

        //Clone the Arc<> that owns the Storage we're reading from
        let storage_arc = warehouse.checkout_storage::<T>().unwrap();

        //Instantiate and return a guard holding the Arc
        ImmutableStorageGuard::new(storage_arc)
    }

    ///Use to get thread-safe write-access to a single ECS Storage.
    /// ## Panics
    /// Panics if you call on an unregistered Component type, T.
    pub fn req_write_guard<T: Component>(&self) -> MutableStorageGuard<T> {
        //Acquire lock on the Warehouse
        let warehouse = self.warehouse.lock().expect(WAREHOUSE_POISON);

        //Clone the Arc<> that owns the Storage we're reading from
        let storage_arc = warehouse.checkout_storage::<T>().unwrap();

        //Instantiate and return a guard holding the Arc
        MutableStorageGuard::new(storage_arc)
    }
    */

    /*
    ///TODO: Change API on World to only have one way to get StorageGuards,
    ///something like: Warehouse<T0, T1, T2, ...>() -> (Storage<T0>, ...) {...}
    ///
    ///Seems like this will require a macro, if there is a way to pass an
    ///arbitrary number of generic types to a macro...let's find out.
    ///
    ///Fuck it, let's keep the verbose way where the user must do things
    ///in the following order:
    ///1.) acquire Warehouse lock
    ///2.) in series, acquire every StorageGuard you need
    ///3.) release Warehouse lock
    ///
    ///
    ///... Through this use,
    ///you prevent deadlocks that may occur while two threads wait to acquire
    ///one or more StorageGuards currently held be the other, in the case
    ///where either thread needs multiple StorageGuards simultaneously.

    pub fn access_warehouse(&self) -> MutexGuard<Warehouse> {
        //TODO: doctests
        let warehouse_guard = self.warehouse.lock().expect(WAREHOUSE_POISON);

        warehouse_guard
    }*/
}

/*
#[derive(Debug)]
pub struct ManyGuard<'a> {
    guarded: MutexGuard<'a, HashMap<TypeId, StorageBox>>,
}

impl<'a> ManyGuard<'a> {
    pub(crate) fn new(guarded: MutexGuard<'a, HashMap<TypeId, StorageBox>>) -> Self {
        ManyGuard { guarded }
    }

    pub fn req_read_guard<T: Component>(&self, world: &World) -> ImmutableStorageGuard<T> {
        world.req_read_guard()
    }

    pub fn req_read_guard_if<T: Component>(
        &self,
        world: &World,
        ent: &Entity,
    ) -> Option<ImmutableStorageGuard<T>> {
        world.req_read_guard_if(ent)
    }

    pub fn req_write_guard<T: Component>(&self, world: &World) -> MutableStorageGuard<T> {
        world.req_write_guard()
    }

    pub fn req_write_guard_if<T: Component>(
        &self,
        world: &World,
        ent: &Entity,
    ) -> Option<MutableStorageGuard<T>> {
        world.req_write_guard_if(ent)
    }
}*/
