use super::*;

mod seal {
    pub trait Sealed {}
    impl<T> Sealed for T {}
    impl<T> Sealed for [T] {}
}

mod seal2 {
    use super::*;

    pub trait Sealed {}
    impl<T> Sealed for MU<T> {}
    impl<T> Sealed for [MU<T>] {}
}

pub
trait UnsizedInit : seal::Sealed {
    type Uninit : ?Sized + UnsizedUninit<Init = Self>;
}

impl<T> UnsizedInit for T {
    type Uninit = MU<T>;
}

impl<T> UnsizedInit for [T] {
    type Uninit = [MU<T>];
}

pub
trait UnsizedUninit : seal2::Sealed {
    type Init : ?Sized + UnsizedInit<Uninit = Self>;

    unsafe
    fn assume_init_mut(self: &mut Self) -> &mut Self::Init;
}

impl<T> UnsizedUninit for MU<T> {
    type Init = T;

    unsafe
    fn assume_init_mut<'r>(self: &'r mut MU<T>) -> &'r mut T {
        #![deny(unconditional_recursion)]
        unsafe { self.assume_init_mut() }
    }
}

impl<T> UnsizedUninit for [MU<T>] {
    type Init = [T];

    unsafe
    fn assume_init_mut<'r>(self: &'r mut [MU<T>]) -> &'r mut [T] {
        let len = self.len();
        unsafe {
            ::core::slice::from_raw_parts_mut(self.as_mut_ptr().cast(), len)
        }
    }
}
