//Jerome M. St.Martin
//June 20, 2022

//-----------------------------------------------------------------------------
//------------------------------- ECS Entities --------------------------------
//-----------------------------------------------------------------------------

use std::collections::HashSet;
use rand::random;

use super::Entity;

pub mod builder;

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

        entity_id
    }

    fn get_next_id(&mut self) -> Entity {
        let mut new_id: usize = random();

        if let Some(id) = self.dead_entities.pop() {
            new_id = id;

        } else {
            while self.active_entities.contains(&new_id) {
                new_id = random();
            }
        }

        new_id
    }
}

