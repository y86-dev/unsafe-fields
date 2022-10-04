#![feature(allocator_api, alloc_layout_extra)]
use core::{
    alloc::{Allocator, Layout},
    ops::{Index, IndexMut},
    ptr::{self, NonNull},
};
use unsafe_fields::{unsafe_fields, UnsafeField};

#[unsafe_fields]
pub(crate) struct MyVec<T> {
    #[unsafe_field]
    ptr: *mut T,
    #[unsafe_field]
    cap: usize,
    #[unsafe_field]
    len: usize,
}

impl<T> MyVec<T> {
    pub fn new() -> Self {
        unsafe {
            Self {
                ptr: UnsafeField::new(ptr::null_mut()),
                cap: UnsafeField::new(0),
                len: UnsafeField::new(0),
            }
        }
    }
    pub fn push(&mut self, value: T) {
        // This will panic or abort if we would allocate > isize::MAX bytes
        // or if the length increment would overflow for zero-sized types.
        if self.len.get_clone() == self.cap.get_clone() {
            self.reserve(self.len.get_clone());
        }
        unsafe {
            let end = self.ptr.get_clone().add(self.len.get_clone());
            ptr::write(end, value);
            *self.len.get_mut() += 1;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        todo!()
    }

    pub fn reserve(&mut self, amount: usize) {
        assert!(
            self.cap
                .get_clone()
                .checked_add(amount)
                .expect("integer overflow calculating new capacity")
                < isize::MAX as usize
        );
        if let Some(ptr) = NonNull::new(self.ptr.get_clone()) {
            unsafe {
                let old_layout = Layout::new::<T>()
                    .repeat(self.cap.get_clone())
                    .expect("Layout computation failed")
                    .0;
                *self.cap.get_mut() += amount;
                let new_layout = Layout::new::<T>()
                    .repeat(self.cap.get_clone())
                    .expect("Layout computation failed")
                    .0;
                std::alloc::Global
                    .grow(ptr.cast::<u8>(), old_layout, new_layout)
                    .expect("reallocation of MyVec failed.");
            }
        } else {
            unsafe {
                self.cap.set(amount);
                let layout = Layout::new::<T>()
                    .repeat(self.cap.get_clone())
                    .expect("Layout computation failed")
                    .0;
                self.ptr.set(
                    std::alloc::Global
                        .allocate(layout)
                        .expect("allocation for MyVec failed.")
                        .cast::<T>()
                        .as_ptr(),
                );
            }
        }
    }

    pub fn len(&self) -> usize {
        self.len.get_clone()
    }
}

impl<T> Index<usize> for MyVec<T> {
    type Output = T;
    fn index(&self, idx: usize) -> &Self::Output {
        assert!(idx < self.len());
        unsafe { &*self.ptr.get_clone().add(idx) }
    }
}

impl<T> IndexMut<usize> for MyVec<T> {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        assert!(idx < self.len());
        unsafe { &mut *self.ptr.get_clone().add(idx) }
    }
}

fn main() {
    let mut vec = MyVec::new();
    vec.push(1);
    vec.push(2);

    assert_eq!(vec.len(), 2);
    assert_eq!(vec[0], 1);

    assert_eq!(vec.pop(), Some(2));
    assert_eq!(vec.len(), 1);

    vec[0] = 7;
    assert_eq!(vec[0], 7);

    while let Some(x) = vec.pop() {
        println!("{x}");
    }
}
