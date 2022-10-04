pub use unsafe_fields_internal::unsafe_fields;

use core::ptr::{addr_of, addr_of_mut};

#[repr(transparent)]
pub struct UnsafeField<T: ?Sized> {
    value: T,
}

impl<T: ?Sized> UnsafeField<T> {
    pub unsafe fn new(value: T) -> Self
    where
        T: Sized,
    {
        Self { value }
    }

    pub unsafe fn get(&self) -> &T {
        &self.value
    }

    pub unsafe fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub unsafe fn raw_get(this: *const Self) -> *const T {
        addr_of!((*this).value)
    }

    pub unsafe fn raw_get_mut(this: *mut Self) -> *mut T {
        addr_of_mut!((*this).value)
    }

    pub unsafe fn set(&mut self, value: T)
    where
        T: Sized,
    {
        self.value = value;
    }

    pub fn get_clone(&self) -> T
    where
        T: Clone,
    {
        self.value.clone()
    }
}
