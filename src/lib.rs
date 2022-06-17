//Jerome M. St.Martin
//June 15, 2022

//Goal: See if it's remotely viable to implement a minimal ECS library from scratch,
//      for use in terminal-emulated roguelikes.
//      Minimal probably means no bitmasks, but DOES include thread-safe access.

mod accessor;
pub mod component;
pub mod resource;
pub(crate) mod storage;
mod world;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
