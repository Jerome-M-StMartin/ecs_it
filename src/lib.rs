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
mod world;

use world::*;
use accessor::*;

pub(crate) const MAX_COMPONENTS: usize = 64;

pub(crate) type Storage = Arc<UnsafeCell<Vec<Option<Box<dyn Any>>>>>;
pub(crate) type Entity = usize;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
