//Jerome M. St.Martin
//Feb 20, 2023

//-----------------------------------------------------------------------------
//------------------- Component Trait & TypeId Getability ---------------------
//-----------------------------------------------------------------------------

use ::std::any::TypeId;

pub trait Component: 'static + Sized + Send + Sync {
    fn type_id() -> TypeId {
        TypeId::of::<Self>()
    }
}

#[cfg(test)]
mod component_tests {
    use super::Component;
    use std::any::TypeId;

    #[test]
    fn test_0() {
        struct TestComponent {}
        impl Component for TestComponent {}

        assert_eq!(TestComponent::type_id(), TypeId::of::<TestComponent>());
    }
}
