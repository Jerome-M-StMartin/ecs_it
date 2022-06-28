//Jerome M. St.Martin
//June 20, 2022

//-----------------------------------------------------------------------------
//------------------------------- ECS Entities --------------------------------
//-----------------------------------------------------------------------------

use std::collections::HashSet;

use super::Entity;

//pub mod builder;

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

    ///This returns a boolean corresponding to whether the entity existed or not.
    ///If it existed, it was removed and this will return true, else false.
    ///Attempting to remove an Entity that doesn't exist won't panic.
    pub(crate) fn rm_entity(&mut self, ent: Entity) -> bool {
        //Panics if ent doesn't exist.
        if let Some(entity_to_rm) = self.active_entities.take(&ent) {
            self.dead_entities.push(entity_to_rm);
            return true;
        }

        false
    }

    fn get_next_id(&mut self) -> Entity {
        let mut new_id: usize = self.num_entities;

        if let Some(id) = self.dead_entities.pop() {
            new_id = id;
        }

        new_id
    }
}
