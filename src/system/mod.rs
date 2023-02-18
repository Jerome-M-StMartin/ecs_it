//Jerome M. St.Martin
//Dec. 4, 2022

//-----------------------------------------------------------------------------
//------------------------------ ECS System API -------------------------------
//-----------------------------------------------------------------------------

use std::fmt;

use crate::world::World;

pub trait System {
    fn run(self, world: &World) -> Result<(), ECSSystemError>;
}

//--- Error Type ---

#[derive(Debug)]
pub struct ECSSystemError(&'static str);

impl std::fmt::Display for ECSSystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ECSSystemError {}
