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
//! Storage - A vector of Components of a specific type.
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
//!
//! let example_component = ExampleComponent { example_data: 7331 };
//!
//! /*
//! * Initialize the ECS World. The new() fn takes one argument, which is an
//! * ESTIMATE of the number of Entities that will exist in your program at
//! * any given time. Internally, Storages contain a Vec, and this estimate
//! * is used when initializing these vectors to allocate space to support
//! * the estimated number of entities. This is not critical, as the vectors
//! * can re-allocate themselves more space at runtime, but not having to
//! * perform this re-allocation will provide a small performance benefit.
//! * The estimate MUST be an integer greater than Zero.
//! */
//! 
//! let num_entities_estimate: usize = 100;
//! let world = ecs_it::world::World::new(num_entities_estimate);
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
//! let the_removed_component: Option<Box<dyn Any>> = world.rm_component::<ExampleComponent>(entity);
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
//!
//! let world = world::World::new(100);
//! world.register_component::<ExampleComponent>();
//!
//! /*
//! There are only two functions you need to know in order to gain either
//! immutable or mutable access to any given Storage, which will return a vector
//! over all the Components that currently exist attached to some Entity. The
//! index of any given Component in the Vec matches the Entity ID of its parent
//! Entity.
//! */
//!
//! /*
//! Grab a StorageGuard from the ECS. The interface for this object allows
//! thread-safe concurrent access to its underlying storage in a BLOCKING
//! manner.
//! */
//!
//! //For immutable access you call .val() on a StorageGuard.
//! {
//!     let storage_guard = world.req_access::<ExampleComponent>();
//!     let storage: &Vec<Option<Box<dyn Any>>> = storage_guard.val();
//! }
//!
//! //For mutable access do you call .val_mut() on a StorageGuard.
//! {
//!     let storage_guard = world.req_access::<ExampleComponent>();
//!     let storage: &mut Vec<Option<Box<dyn Any>>> = storage_guard.val_mut();
//! }
//!         
//! /*
//! The scoping brackets used above are to force the StorageGuard to be dropped.
//! This is how the crate ensures safe concurrent access to Storages. As long as
//! a StorageGuard exists that has granted mutable access, no other StorageGuard
//! will grant any access. If a StorageGuard exists that has granted immutable
//! access, any number of other StorageGuards will also grant immutable access,
//! but none will grant mutable access.
//!
//! Because of this, you never have to worry about keeping track of what access
//! was granted where -- simply drop StorageGuards when you no longer need
//! access to that storage (either manually or by simply allowing the guard to
//! fall out of scope).
//! */
//!```

mod entity;
mod storage;
pub mod world;

pub(crate) const MAX_COMPONENTS: usize = 64;

pub(crate) type Entity = usize;

#[cfg(test)]
mod tests {

    //Must run 'cargo test -- --nocapture' to allow printing of time elapsed

    use std::any::Any;
    use super::world::World;
    use std::time::Instant;

    struct TestComponent {
        val: usize,
    }

    #[test]
    fn create_raw_entity() {
        let now = Instant::now();

        let w = World::new(3);
        let entity0: usize = w.create_entity();
        let entity1: usize = w.create_entity();
        let entity2: usize = w.create_entity();

        assert_eq!(entity0, 0);
        assert_eq!(entity1, 1);
        assert_eq!(entity2, 2);

        println!(
            "Time Elapsed during create_raw_entity(): {}",
            now.elapsed().as_nanos()
        );
    }

    #[test]
    fn add_component() {
        let w = World::new(1);
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
        w.add_component(entity0, TestComponent { val: 42 });
        println!("Time to add component(): {}", now.elapsed().as_nanos());
    }
}
