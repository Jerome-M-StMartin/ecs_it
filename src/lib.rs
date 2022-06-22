//Jerome M. St.Martin
//June 15, 2022

//Goal: See if it's remotely viable to implement a minimal ECS library from scratch,
//      for use in terminal-emulated roguelikes.
//      Minimal probably means no bitmasks, but DOES include thread-safe access.

mod accessor;
mod entity;
mod storage;
mod world;

use world::*;
use accessor::*;

pub(crate) const MAX_COMPONENTS: usize = 64;

pub(crate) type Entity = usize;

#[cfg(test)]
mod tests {
    
    //Must run 'cargo test -- --nocapture' to allow printing of time elapsed

    use super::*;
    use std::time::Instant;

    struct TestComponent {
        val: usize,
    }
/*
    #[test]
    fn create_new_entity() {
        let now = Instant::now();

        let w = World::new();
        let entity0: usize = w.init_entity();
        let entity1: usize = w.init_entity();
        let entity2: usize = w.init_entity();

        assert_eq!(entity0, 0);
        assert_eq!(entity1, 1);
        assert_eq!(entity2, 2);

        println!("Time Elapsed during create_new_entity(): {}", now.elapsed().as_millis());
    }
*/
    #[test]
    fn add_component() {
        let now = Instant::now();

        let w = World::new();
        let entity0: usize = w.init_entity();
        let _entity1: usize = w.init_entity();
        let _entity2: usize = w.init_entity();

        w.add_component(entity0, TestComponent { val: 42, });

        println!("Time Elapsed during create_new_entity(): {}", now.elapsed().as_millis());
    }
}
