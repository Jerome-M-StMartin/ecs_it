//Jerome M. St.Martin
//June 20, 2022

//-----------------------------------------------------------------------------
//------------------------------- ECS Entities --------------------------------
//-----------------------------------------------------------------------------

use std::collections::{hash_set::Iter, HashSet};

use super::Entity;

///Internal; generating, controlling, and  holding unique Entity IDs.
pub struct Entities {
    //Invariant:
    //The intersection of active and dead entities is the null set.
    num_entities: usize,
    active_entities: HashSet<Entity>,
    dead_entities: Vec<Entity>,
}

impl Entities {
    pub(crate) fn new() -> Entities {
        Entities {
            num_entities: 0,
            active_entities: HashSet::new(),
            dead_entities: Vec::new(),
        }
    }

    pub(crate) fn new_entity_id(&mut self) -> Entity {
        let entity_id = self.get_next_id();
        self.active_entities.insert(entity_id);
        self.num_entities += 1;

        entity_id
    }

    ///If the entity exists, removes it and returns true;
    ///if the entity did not exist, returns false.
    ///Attempting to remove an Entity that doesn't exist won't panic.
    pub(crate) fn rm_entity(&mut self, ent: Entity) -> bool {
        if let Some(entity_to_rm) = self.active_entities.take(&ent) {
            self.dead_entities.push(entity_to_rm);
            return true;
        }

        false
    }

    pub(crate) fn living_entities_iter(&self) -> Iter<'_, Entity> {
        self.active_entities.iter()
    }

    pub(crate) fn dead_entities_iter(&self) -> std::slice::Iter<'_, Entity> {
        self.dead_entities.iter()
    }

    pub(crate) fn vec(&self) -> Vec<Entity> {
        let mut vec = Vec::with_capacity(self.active_entities.len());
        let iter = self.active_entities.iter();

        for &ent in iter {
            vec.push(ent);
        }

        vec
    }

    fn get_next_id(&mut self) -> Entity {
        let mut new_id: usize = self.num_entities;

        if let Some(id) = self.dead_entities.pop() {
            new_id = id;
        }

        new_id
    }
}
