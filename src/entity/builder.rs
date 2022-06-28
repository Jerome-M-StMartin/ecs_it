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
    components: Vec<Box<dyn Any>>,
}

impl EntityBuilder {
    pub fn new() -> EntityBuilder {
        EntityBuilder {
            components: Vec::new(),
        }
    }

    pub fn with<T: 'static, Any>(mut self, ecs: &World, component: T) -> EntityBuilder {
        ecs.register_component::<T>();
        self.components.push(Box::new(component));
        self
    }

    pub fn build(self, ecs: &World) -> Entity {
        let entity = ecs.init_entity();

        for c in self.components {
            ecs.add_component(entity, c);
        }

        entity
    }
}
