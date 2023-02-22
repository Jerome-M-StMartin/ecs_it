//Jerome M. St.Martin
//June 20, 2022

//-----------------------------------------------------------------------------
//------------------------------- ECS Entities --------------------------------
//-----------------------------------------------------------------------------

//use std::collections::{hash_set::Iter, HashSet};

use super::Entity; //Entity is just a usize

///Internal; for generating, controlling, and holding unique Entity IDs.
pub struct Entities {
    //Invariant:
    //The intersection of active and dead entities is the null set.
    num_entities: usize,
    //active_entities: HashSet<Entity>,
    dead_entities: Vec<Entity>, //This will never exceed the length of any Storage vec.
}

impl Entities {
    pub(crate) fn new() -> Entities {
        Entities {
            num_entities: 0,
            //active_entities: HashSet::new(), //Not sure I need this anymore.
            dead_entities: Vec::new(),
        }
    }

    ///Returns an available ID from a dead entity or makes a new one.
    pub(crate) fn new_entity_id(&mut self) -> Entity {
        let entity_id = self.get_next_id();
        //self.active_entities.insert(entity_id);
        self.num_entities += 1;

        entity_id
    }

    ///If the entity exists, removes it and returns true;
    ///if the entity did not exist, returns false.
    ///Attempting to remove an Entity that doesn't exist won't panic.
    pub(crate) fn rm_entity(&mut self, ent: Entity) -> bool {
        //TODO: new way w/ vec storages

        /*if let Some(entity_to_rm) = self.active_entities.take(&ent) {
            self.dead_entities.push(entity_to_rm);
            return true;
        }
        */
        false
    }

    fn get_next_id(&mut self) -> Entity {
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

        //Tell Storages to increase inner vec.length by one.
        //If possible... it would be ideal to do this lazily so we don't have
        //to request write access to every Storage concurrently.
        //Lazy Storage.len++ Procedure Attempt #1:
        //1.) Create an atomic usize, new_ents, which is checked before any
        //attempt to access any storage, regardless of read or write intent.
        //2.) On either req_read_access() or req_write_access() calls:
        //If new_ents > 0, we do a req_write_access() and do len++.
        //Problem: How would we know which storages have done len++ and which
        //have not yet? This way would require each Storage to have its own
        //new_ents, which still requires a bunch of concurrent write locks.
        //
        //Lazy Storage.len++ Procedure Attempt #2:
        //1.) Make num_entities an atomic.
        //2.) On acquiring read/write storage lock:
        //  If storage.len < num_entities:
        //      req_write_access();
        //      len += (num_entities - len);

        new_id
    }

    /* Not sure any of these are needed. Try without. 2/12/2023
    pub(crate) fn living_entities_iter(&self) -> Iter<'_, Entity> {
        self.active_entities.iter()
    }

    pub(crate) fn dead_entities_iter(&self) -> std::slice::Iter<'_, Entity> {
        self.dead_entities.iter()
    }*/

    /* Don't think I need this functionality,
     * If I do, rewrite this in a less garbo way plz
    pub(crate) fn vec(&self) -> Vec<Entity> {
        let mut vec = Vec::with_capacity(self.active_entities.len());
        let iter = self.active_entities.iter();

        for &ent in iter {
            vec.push(ent);
        }

        vec
    }
    */
}
