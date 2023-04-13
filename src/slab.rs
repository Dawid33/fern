use memmap;
use std::marker::PhantomData;

struct SlabAllocator<T> {
    t: PhantomData<*const T>,
}

impl<T> SlabAllocator<T>
where
    T: Sized,
{
    pub fn new() -> Self {
        Self {
            t: Default::default(),
        }
    }
    pub fn alloc() -> () {
        ()
    }
}
