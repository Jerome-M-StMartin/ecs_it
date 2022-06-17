//Jerome M. St.Martin
//June 15, 2022

pub trait Resource {
    fn new() -> Self where Self: Sized;
}
