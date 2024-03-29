//Jerome M. St.Martin
//June 15, 2022

//! # Key Words / Legend:
//!
//! ECS - Entity-Component-System Architecture
//!
//! Entity - A usize which represents an in-diegesis 'thing' in the game.
//!
//! Component - A struct associated with a specific Entity.
//!
//! Storage - A collection of Components of a specific type.
//!
//! System - Logic that operates over one or more Components within one or more Storages.
//!
//! ## Summary
//!
//! This crate provides a very simple, thread-safe ECS which allows concurrent queries and
//! mutations of Component Storages in a performant, blocking manner.
//!
//! By performant, I mean that this crate doesn't use spin-loops to cause a thread to wait for
//! storage access. Internally it uses Mutex-Condvar pairs to provide functionality similar to a
//! RwLock over each Storage individually. Threads are put to sleep while they wait via the
//! rust std Condvar API.
//!
//! Given that each thread may access some subset of Storages, parallel access is possible if
//! and only if the intersection of sets being accessed at any given moment between two or more
//! threads is the null set.
//!
//! There is no built-in System API. Implementing Systems is left up to the user of this crate.
//!
//! Usage of this crate boils down to calling ecs_it::World::new(...), registering all components,
//! then requesting access to storages which results in being handed a StorageGuard struct. The
//! API on this StorageGuard provides BLOCKING access to its guarded Storage, and dropping this
//! StorageGuard (as it falls out of scope) triggers unlocking/concurrency logic for its underlying
//! Storage. You never have to explicitly lock or unlock any Mutex or whatever.
//!
//! The World is a container for the various Storages and Entities you create, and it should be
//! stored in an Arc<> for shared ownership between threads that need access to the ECS.
//!
//! Components can be any struct which is 'static + Any.
//!
//! # Demonstration of Use:
//!
//!```
//! use ecs_it::*;
//! use std::any::Any;
//!
//! // Define a Struct which will be a Component:
//! struct ExampleComponent {
//!     example_data: usize,
//! }
//! impl Default for ExampleComponent {
//!     fn default() -> Self {
//!         ExampleComponent { example_data: 0 }
//!     }
//! }
//! impl Component for ExampleComponent {}
//!
//! let example_component = ExampleComponent { example_data: 7331 };
//!
//! // Initialize the ECS World.
//! let world = ecs_it::world::World::new();
//!
//! /*
//! * IMPORTANT:
//! * You MUST register all components before creating any entities, and
//! * certainly before adding any components to any entities.
//! */
//! world.register_component::<ExampleComponent>();
//!
//! // Entity Creation:
//! let entity = world.create_entity();
//! world.add_component(entity, example_component);
//!
//! // You may remove Components from an Entity via the following:
//! let the_removed_component: Option<ExampleComponent> = world.rm_component::<ExampleComponent>(&entity);
//!
//!```
//!
//! # How to query the ECS for existing Storages/Components:
//!```
//! use std::any::Any;
//! use ecs_it::*;
//!
//! struct ExampleComponent {
//!     example_data: usize,
//! }
//! impl Default for ExampleComponent {
//!     fn default() -> Self {
//!         ExampleComponent { example_data: 0 }
//!     }
//! }
//! impl Component for ExampleComponent {}
//!
//! let world = world::World::new();
//! world.register_component::<ExampleComponent>();
//! let my_entity = world.create_entity();
//!
//! /*
//! First, via req_read_guard() or req_write_guard():
//! Grab a StorageGuard from the ECS. Requesting either type of StorageGuard
//! is a BLOCKING call, which allows only one exclusive MutableStorageGuard or
//! many ImmutableStorageGuards to exist at any given time.
//! */
//!
//! /*
//! For immutable access you can call one of the following methods implemented
//! on ImmutableStorageGuards:
//!     get(e: Entity)
//!     iter()
//!     raw()
//! */
//! {
//!     let storage_guard = world.req_read_guard::<ExampleComponent>();
//!     let a_component: Option<&ExampleComponent> = storage_guard.get(&my_entity);
//!     let component_iter = storage_guard.iter();
//! }
//!
//! /*
//! For mutable access you can call one of the following methods implemented
//! on MutableStorageGuards:
//!     get_mut(e: Entity)
//!     iter_mut()
//!     raw_mut()
//! */
//! {
//!     let mut storage_guard = world.req_write_guard::<ExampleComponent>();
//!     let a_component: Option<&mut ExampleComponent> = storage_guard.get_mut(&my_entity);
//!     let component_iter_mut = storage_guard.iter_mut();
//! }
//!         
//! /*
//! The scoping brackets used above are to force StorageGuards to be dropped.
//! This is how the crate ensures safe concurrent access to Storages -- you
//! can only instantiate *StorageGuards if there are no MutableStorageGuards
//! already existing for a given Storage. You can instantiate any number of
//! ImmutableStorageGuards for the same Storage if and only if there are no
//! MutableStorageGuards already.
//!
//! Because of this, you never have to worry about keeping track of what access
//! was granted where -- simply drop StorageGuards when you no longer need
//! access to that storage (by simply allowing the guard to fall out of scope).
//! */
//!```

//use std::any::Any;

mod entity;
mod storage;
pub mod world;

pub type Entity = usize;

pub trait Component: 'static + Sized + Send + Sync {}

#[cfg(test)]
mod tests {

    //Must run 'cargo test -- --nocapture' to allow printing of time elapsed

    use super::world::World;
    use super::Component;
    use std::time::Instant;

    struct TestComponent {
        _val: usize,
    }
    impl Component for TestComponent {}
    impl Default for TestComponent {
        fn default() -> Self {
            TestComponent { _val: 0 }
        }
    }

    #[test]
    fn entity_tests() {
        let w = World::new();
        let entity0: usize = w.create_entity();
        let entity1: usize = w.create_entity();
        let entity2: usize = w.create_entity();

        assert_eq!(entity0, 0);
        assert_eq!(entity1, 1);
        assert_eq!(entity2, 2);

        for (i, ent) in w.entity_iter().enumerate() {
            println!("i: {}, ent: {}", i, ent);
        }
    }

    #[test]
    fn add_component() {
        let w = World::new();
        let entity0: usize;
        let mut now = Instant::now();
        {
            w.register_component::<TestComponent>();
            println!("Time to register component: {}", now.elapsed().as_nanos());

            now = Instant::now();
            entity0 = w.create_entity();
            println!("Time to init entity: {}", now.elapsed().as_nanos());
        }
        now = Instant::now();
        w.add_component(entity0, TestComponent { _val: 42 });
        println!("Time to add component(): {}", now.elapsed().as_nanos());
    }
}
