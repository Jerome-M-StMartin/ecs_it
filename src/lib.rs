//Jerome M. St.Martin
//June 15, 2022

//Goal: See if it's remotely viable to implement a minimal ECS library from scratch,
//      for use in terminal-emulated roguelikes.
//      Minimal probably means no bitmasks, but DOES include thread-safe access.

use std::{
    sync::Arc,
    cell::UnsafeCell,
    any::Any,
};

mod accessor;
//pub mod component;
//pub mod resource;
//pub(crate) mod storage;
mod world;

//Inner Vec of Storage type must be initialized to be the # of components,
//or the max # of components I guess would be fine, which is 64, currently.
pub(crate) type Storage = Arc<UnsafeCell<Vec<Option<Box<dyn Any>>>>>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
