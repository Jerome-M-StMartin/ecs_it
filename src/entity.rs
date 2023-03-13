//Jerome M. St.Martin
//June 20, 2022

//-----------------------------------------------------------------------------
//------------------------------- ECS Entities --------------------------------
//-----------------------------------------------------------------------------

use std::sync::MutexGuard;

use super::{Entity, warehouse::Warehouse}; //Entity is just a usize

///Internal; for generating, controlling, and holding unique Entity IDs.
pub struct Entities {
    //Invariant:
    //The intersection of active and dead entities is the null set.
    num_entities: usize,
    dead_entities: Vec<Entity>, //This will never exceed the length of any Storage vec.
}

impl Entities {
    pub(crate) fn new() -> Entities {
        Entities {
            num_entities: 0,
            dead_entities: Vec::new(),
        }
    }

    ///If the entity exists, removes it and returns true;
    ///if the entity did not exist, returns false.
    ///Attempting to remove an Entity that doesn't exist won't panic.
    pub(crate) fn rm_entity(&mut self, ent: Entity, warehouse: &MutexGuard<Warehouse>) -> bool {
        if ent <= self.num_entities {
            
            //Fill all associated Component slots in Storages with None.
            todo!();

            self.dead_entities.push(ent);
            return true
        }

        false
    }

    ///Returns an available ID from a dead entity or makes a new one.
    pub(crate) fn new_entity(&mut self, warehouse: &MutexGuard<Warehouse>) -> Entity {
        let entity_id = self.get_next_id(warehouse);
        self.num_entities += 1;

        entity_id
    }

    fn get_next_id(&mut self, warehouse: &MutexGuard<Warehouse>) -> Entity {
        //If there is an available ID from a dead Entity, use it so we don't
        //have to increase the size of the Storage vecs, and so an unused
        //index of the Storages can be used.
        if let Some(id) = self.dead_entities.pop() {
            return id;
        }

        //If there is no dead Entity ID available, make one. This ID will
        //always be equal to the length of any Storage before adding an
        //idx for this new Entity. a_storage.length() == self.num_entites.
        let mut new_id: usize = self.num_entities;

        //Tell the Warehouse to trigger lazy lengthening of all Storage vecs.
        warehouse.lazy_lengthen(1);

        new_id
    }
}
