//Jerome M. St.Martin
//June 20, 2022

//-----------------------------------------------------------------------------
//------------------------------- ECS Entities --------------------------------
//-----------------------------------------------------------------------------

use std::{
    any::Any,
    collections::HashSet,
};

use rand::random;

use super::{Entity, World};

///Internal; for safely generating and tracking unique Entity IDs.
pub struct Entities {
    //Invariant:
    //The intersection of living and dead entities is the null set.
    num_entities: usize,
    living_entities: HashSet<Entity>,
    dead_entities: Vec<Entity>,
}

///User-facing Builder Pattern object. Use this to make new Entities.
pub struct EntityBuilder {
    components: Vec<Box<dyn Any>>,
}

impl Entities {
    pub(crate) fn new() -> Entities {
        Entities {
            num_entities: 0,
            living_entities: HashSet::new(),
            dead_entities: Vec::new(),
        }
    }

    pub(crate) fn new_entity_id(&mut self) -> Entity {
        let entity_id = self.get_next_id();
        self.living_entities.insert(entity_id);

        entity_id
    }

    fn get_next_id(&mut self) -> Entity {
        let mut new_id: usize = random();

        if let Some(id) = self.dead_entities.pop() {
            new_id = id;

        } else {
            while self.living_entities.contains(&new_id) {
                new_id = random();
            }
        }

        new_id
    }
}

impl EntityBuilder {
    pub fn new() -> EntityBuilder {
        EntityBuilder {
            components: Vec::new(),
        }
    }

    pub fn with<T: 'static, Any>(mut self, ecs: &World, component: T) -> EntityBuilder {
        self.components.push(Box::new(component));
        self
    }

    pub fn build(self, ecs: &World) {
        let entity = ecs.init_entity();

        for c in self.components {
            ecs.add_component(entity, c);
        }
    }
}
