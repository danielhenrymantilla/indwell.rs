use super::*;

mod phantom_inference {
    pub
    enum PhantomInference<T> {
        PhantomInferenceUnitStruct,
        _Never(::never_say_never::Never, ::core::marker::PhantomData<T>),
    }
    pub use PhantomInference::*;
    #[expect(unused, hidden_glob_reexports)]
    enum PhantomInferenceUnitStruct {}
}
pub use phantom_inference::PhantomInferenceUnitStruct as PhantomInference;
pub type PhantomInference<T> = phantom_inference::PhantomInference<T>;

impl<T> PhantomInference<T> {
    pub
    fn conjure(self, never: ::never_say_never::Never) -> T {
        never
    }
}

// Non-sealed!
pub
trait Pointer : Sized {
    type Pointee : ?Sized;
}

// Non-sealed!
pub
trait Unsize<Dst : Pointer> : Pointer {
    /// Safety: only to be used by the [`unsize!`] macro.
    unsafe
    fn unsize_with(
        _: Self,
        _: PhantomInference<Dst>,
        pointer_cast: impl FnOnce(*mut Self::Pointee) -> *mut Dst::Pointee,
    ) -> Dst
    ;
}

#[doc(hidden)] /* Not part of the public API */ #[macro_export]
macro_rules! ඞunsize {( $wrapped_ptr:expr $(=> $T:ty)? $(,)? ) => ({
    let infer_ty = $crate::unsize::PhantomInference;
    if let ::core::option::Option::Some(never) = ::core::option::Option::None {
        infer_ty.conjure(never)
    } else {
        match $wrapped_ptr { ptr => unsafe {
            $crate::unsize::Unsize::unsize_with(
                ptr, infer_ty,
                |p| $(-> *mut $T)? { p },
            )
        }}
    }
})}

#[doc(inline)]
pub use ඞunsize as unsize;

use crate::{brand::MaybeBrandedPlace, prelude::OutRef};

impl<'r, T : ?Sized, P : MaybeBrandedPlace> Pointer for OutRef<'r, T, P> {
    type Pointee = T;
}

impl<'r, Src : ?Sized, Dst : ?Sized, P : MaybeBrandedPlace>
    Unsize<OutRef<'r, Dst, P>>
for
    OutRef<'r, Src, P>
{
    #[inline]
    unsafe
    fn unsize_with(
        this: OutRef<'r, Src, P>,
        _: PhantomInference<OutRef<'r, Dst, P>>,
        pointer_cast: impl FnOnce(*mut Src) -> *mut Dst,
    ) -> OutRef<'r, Dst, P>
    {
        unsafe {
            OutRef::from_raw_branded_unchecked(
                ptr::NonNull::new_unchecked(
                    pointer_cast(this.into_raw().as_ptr()),
                ),
            )
        }
    }
}

impl<'r, T : ?Sized, P : MaybeBrandedPlace> Pointer for OwnRef<'r, T, P> {
    type Pointee = T;
}

impl<'r, Src : ?Sized, Dst : ?Sized, P : MaybeBrandedPlace>
    Unsize<OwnRef<'r, Dst, P>>
for
    OwnRef<'r, Src, P>
{
    #[inline]
    unsafe
    fn unsize_with(
        this: OwnRef<'r, Src, P>,
        _: PhantomInference<OwnRef<'r, Dst, P>>,
        pointer_cast: impl FnOnce(*mut Src) -> *mut Dst,
    ) -> OwnRef<'r, Dst, P>
    {
        unsafe {
            OwnRef::from_raw_branded_unchecked(
                ptr::NonNull::new_unchecked(
                    pointer_cast(this.into_raw().as_ptr()),
                ),
            )
        }
    }
}
