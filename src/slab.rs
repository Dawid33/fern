use std::marker::PhantomData;
use memmap;

struct SlabAllocator<T> {

    t : PhantomData<*const T>,
}

impl<T> SlabAllocator<T> where T : Sized {
    pub fn new() -> Self {
        Self { t: Default::default() }
    }
    pub fn alloc() -> () {
        ()
    }
}

