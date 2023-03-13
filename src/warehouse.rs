//Jerome M. St.Martin
//Feb 12, 2023

//-----------------------------------------------------------------------------
//------------------- Warehouse: What Stores the Storages  --------------------
//-----------------------------------------------------------------------------

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}}
};

use super::{
    Entity,
    component::Component,
    storage::{ImmutableStorageGuard, MutableStorageGuard, Storage},
};

///Container for all Storages in the ECS World; lives in an Arc.
pub struct Warehouse {
    //Invariants:
    //1.) each storage has the same length (underlying vec I mean)
    //2.) capacity == the length of the storages
    capacity: usize, //Exact length of all Storage vecs, not # of storages.
    dead_entities: Vec<Entity>,
    dirty_flag: usize, //Increments to indicate to Storages that they're dirty.
    storages: HashMap<TypeId, StorageBox>,
    //pub(crate) maintenance_functions: Vec<Box<dyn Fn(&World, &Entity)>>,
}

impl Warehouse {
    pub(crate) fn new() -> Self {
        Warehouse {
            capacity: 0,
            dead_entities: Vec::new(),
            storages: HashMap::new(),
        }
    }
    
    ///TODO
    pub fn checkout_storage<T: Component>(&self) -> ImmutableStorageGuard<T> {
        self.capacity_check::<T>();
        let type_id = TypeId::of::<T>();

        if let Some(storage_box) = self.storages.get(&type_id) {
            let arc = storage_box.clone_storage_arc();
            return ImmutableStorageGuard::new(arc);
        } else {
            panic!("Failed to find Storage<T>. Did you forget to register a Component?");
        }
    }

    ///TODO
    pub fn checkout_storage_mut<T: Component>(&self) -> MutableStorageGuard<T> {
        self.capacity_check::<T>();
        let type_id = TypeId::of::<T>();

        if let Some(storage_box) = self.storages.get(&type_id) {
            let arc = storage_box.clone_storage_arc();
            return MutableStorageGuard::new(arc);
        } else {
            panic!("Failed to find Storage<T>. Did you forget to register a Component?");
        }
    }

    //Whenever one or more new Entity IDs are created, call this.
    pub(crate) fn lazy_lengthen(&self, num_new_ents: usize) {
        self.capacity += num_new_ents;
    }

    pub(crate) fn notify_of_dead_entity(&self, ent: Entity) {
        self.dead_entities.push(ent);
    }

    pub(crate) fn notify_of_dead_entities(&self, ents: Vec<Entity>) {
        self.dead_entities.append(&mut ents);
    }

    fn maintain_storage<T: Component>(&self) {
        self.capacity_check::<T>();
        self.bring_out_the_dead::<T>();
    }

    //Call any time a Storage is being accessed/checked-out to guarantee:
    //Invariant: Warehouse.capacity == Storage<*>.capacity == num Entity IDs
    fn capacity_check<T: Component>(&self) {
        let type_id = TypeId::of::<T>();

        if let Some(storage_box) = self.storages.get(&type_id) {
            let storage_cap = storage_box.capacity.load(Ordering::SeqCst);
            let warehouse_cap = self.capacity;

            assert!(storage_cap <= warehouse_cap,
                    "There is no correct state where a storage's capacity
                     should exceed the warehouse's capacity.");

            let new_entity_count = warehouse_cap - storage_cap;
            if new_entity_count > 0 {
                //0.) Acquire mutable access to the storage vec
                let mut guard = MutableStorageGuard::<T>::new(storage_box.clone_storage_arc());

                //1.) lengthen storage vec
                guard.register_new_entities(new_entity_count);

                //2.) increment storage capacity
                storage_box.capacity.fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    fn bring_out_the_dead<T: Component>(&self) {
        for ent in self.dead_entities.into_iter() {
            /*
             * Uh oh, this is highly complex. Instead of triggering some fn
             * with a dirty flag, I should implement a task system, where
             * the logic is supplied by a closure.
             *
             * MaintenanceTask {}
             *
             */
            todo!()
        }
    }
}

///Used internally to provide abstraction over generically typed Storages
///to allow storing any kind of Storage<T>. i.e. Implements polymorphism over
///all Storage types.
///
///Additionally, these are what own the the Arcs that own each Storage,
///allowing for thread-safe ownership of subsets of Storages rather than
///requiring a continuous lock on the entire Warehouse.
///
///Invariant:
///- capacity must be equal to warehouse.capacity before any read/write.
#[derive(Debug)]
pub(crate) struct StorageBox<'a> {
    pub(crate) boxed: Arc<dyn Any + Send + Sync + 'static>,
    pub(crate) capacity: AtomicUsize,
    pub(crate) maintenance_tasks: Mutex<Vec<Box<dyn Fn() + 'a>>>,
}

impl<'a> StorageBox<'a> {
    pub(crate) fn clone_storage_arc<T: Component>(&self) -> Arc<Storage<T>> {
        let arc_any = self.boxed.clone();
        arc_any.downcast::<Storage<T>>().unwrap_or_else(|e| {
            panic!("{:?}", e);
        })
    }
}

///Used interally
///TODO
struct MaintenanceTask {
    logic: dyn Fn(),
}

impl MaintenanceTask {
    fn new(logic: dyn Fn()) -> Self {
        MaintenanceTask { logic, }
    }

    fn run(self) {
        let f = self.logic;
        f()
    }
}


