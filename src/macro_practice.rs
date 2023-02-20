
use core::any::Any;
use core::marker::PhantomData;

#[derive(Debug)]
struct Guard<T> {
    inner: PhantomData<T>,
}
impl<T> Guard<T> {
    fn new() -> Self {
        Guard {
            inner: PhantomData::<T>,
            //inner: Vec::with_capacity(0),
        }
    }
}

fn test<T>() -> Guard<T> { Guard::<T>::new() }

struct StorageBox {
    pub boxed: Box<dyn Any + 'static>,
}

struct TupleBuilder {
    vec: Vec<StorageBox>,
}

impl TupleBuilder {
    fn new() -> Self {
        TupleBuilder {
            vec: Vec::new(),
        }
    }
    
    fn with(self, sb: StorageBox) -> Self {
        self.vec.push(sb);
    }
    
    fn build(self) -> tuple {
        
    }
}

//Goal: Outputs zipped Iterator of $(Storages<$T>)+
// use itertools::zip! to create final output
macro_rules! warehouse_fetch {
    //Accepts as parameters 1..n Storage<T> types
    ($( $storage_n:ty ), +) => {{
        //Init
        let mut guards = Vec::new();
        
        //Fetch n storages
        $(  
            let guard = test::<$storage_n>();
            guards.push(guard);
        )+
        
        //Convert vec into tuple for output
        let mut tuple = ();
        
        //return output
        tuple
    }}
}


fn main() {
    //println!("{:?}", ().push(Guard::<usize>::new()).push(Guard::<u8>::new()));
    //println!("{:?}", warehouse_fetch!(usize, u8, u16, i32));
}

//====================================================================================
//====================================================================================
//====================================================================================
//Feb 18, 23

use core::any::Any;
use core::marker::PhantomData;

#[derive(Debug)]
struct Guard<T> {
    inner: PhantomData<T>,
}
impl<T> Guard<T> {
    fn new() -> Self {
        Guard {
            inner: PhantomData::<T>,
            //inner: Vec::with_capacity(0),
        }
    }
}

fn test<T>() -> Guard<T> { Guard::<T>::new() }

struct StorageBox {
    pub boxed: Box<dyn Any + 'static>,
}

struct WarehouseIteratorBuilder {
    storages: Vec<&'static StorageBox>,
}

impl WarehouseIteratorBuilder {
    fn new() -> Self {
        WarehouseIteratorBuilder {
            storages: Vec::new(),
        }
    }
    
    fn with(self, sb: StorageBox) -> Self {
        self.storages.push(sb);
    }
    
    //Immutable iter, TODO:build_mut()
    fn build(self) -> Iterator {
    
        for storage_box in self.storages {
            storage_box.clone
            let mut iter: Iterator = self.storage[0].iter();
            for idx in self.storage.length..0 {
                let storage_n = self.storages[idx];
                iter = iter.zip(storage_n.iter());
            }
        }
        // return an iterator like:
        // (StorageGuard<T0>, StorageGuard<T1>, ..., StorageGuard<Tn>)
        iter.flatten()
    }
}

//Goal: Outputs zipped Iterator of $(Storages<$T>)+
// use itertools::zip! to create final output
macro_rules! warehouse_fetch {
    //Accepts as parameters 1..n Storage<T> types
    ($( $storage_n:ty ), +) => {{
        //Init
        let mut guards = Vec::new();
        
        //Fetch n storages
        $(  
            let guard = test::<$storage_n>();
            guards.push(guard);
        )+
        
        //Convert vec into tuple for output
        let mut tuple = ();
        
        //return output
        tuple
    }}
}


fn main() {
    //println!("{:?}", ().push(Guard::<usize>::new()).push(Guard::<u8>::new()));
    //println!("{:?}", warehouse_fetch!(usize, u8, u16, i32));
}
