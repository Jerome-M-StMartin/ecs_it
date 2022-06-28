//Jerome M. St.Martin
//June 20, 2022

//-----------------------------------------------------------------------------
//-------------------------- Entity Builder Pattern ---------------------------
//-----------------------------------------------------------------------------

use std::any::Any;

use super::super::world::World;
use super::super::Entity;

///User-facing Builder Pattern object. Use this to make new Entities.
pub struct EntityBuilder {
    //components: Vec<Box<dyn Any>>,
    entity: Entity,
}

impl EntityBuilder {
    pub fn new(ecs: &World) -> EntityBuilder {
        EntityBuilder {
            ////components: Vec::new(),
            entity: ecs.init_entity(),
        }
    }

    pub fn with<T: 'static + Any>(mut self, ecs: &World, component: T) -> EntityBuilder {
        ecs.register_component::<T>();
        ecs.add_component(self.entity, component);
        self
    }

    pub fn build(self, ecs: &World) -> Entity {
        //let entity = ecs.init_entity();

        /*for boxed_component in self.components {
            println!("Adding component...\n\r");
            ecs.add_component(entity, c);
        }*/

        self.entity
    }
}
