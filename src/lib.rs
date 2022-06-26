//Jerome M. St.Martin
//June 15, 2022

//Goal: See if it's remotely viable to implement a minimal ECS library from scratch,
//      for use in terminal-emulated roguelikes.
//      Minimal probably means no bitmasks, but DOES include thread-safe access.

mod entity;
mod storage;
mod world;

pub(crate) const MAX_COMPONENTS: usize = 64;

pub(crate) type Entity = usize;

#[cfg(test)]
mod tests {
    
    //Must run 'cargo test -- --nocapture' to allow printing of time elapsed

    use super::world::World;
    use std::time::Instant;

    struct TestComponent {
        val: usize,
    }

    #[test]
    fn create_new_entity() {
        let now = Instant::now();

        let w = World::new(3);
        let entity0: usize = w.init_entity();
        let entity1: usize = w.init_entity();
        let entity2: usize = w.init_entity();

        assert_eq!(entity0, 0);
        assert_eq!(entity1, 1);
        assert_eq!(entity2, 2);

        println!("Time Elapsed during create_new_entity(): {}", now.elapsed().as_nanos());
    }

    #[test]
    fn add_component() {
        let w = World::new(1);
        let entity0: usize;
        let mut now = Instant::now();
        {
            w.register_component::<TestComponent>();
            println!("Time to register component: {}", now.elapsed().as_nanos());

            now = Instant::now();
            entity0 = w.init_entity();
            println!("Time to init entity: {}", now.elapsed().as_nanos());
        }
        now = Instant::now();
        w.add_component(entity0, TestComponent { val: 42, });
        println!("Time to add component(): {}", now.elapsed().as_nanos());
    }
}
