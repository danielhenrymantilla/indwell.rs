use crate::*;

pub use crate::proc_macros::OutRefProject;

#[repr(transparent)]
pub
struct OutRef<'r, T : ?Sized, P : MaybeBrandedPlace = NoBrandedPlace> {
    raw: ptr::NonNull<T>,
    _phantom: PD<fn(&mut T) -> (&'r (), P)>,
    _owns_t: PD<T>,
}

impl<'r, T : ?Sized, P : MaybeBrandedPlace> OutRef<'r, T, P> {
    pub
    fn into_raw(self) -> ptr::NonNull<T> {
        self.raw
    }
}

pub
trait AsManuallyDropMut {
    /// Cast the current `&mut` reference into a <code>\&mut [ManuallyDrop]\<T\></code> reference
    /// so as to, notably, be able to call [`.as_out()`] on it.
    ///
    /// Note that such a pattern is **prone to memory leaks**. And trying to `unsafe`ly manually
    /// call [`ManuallyDrop::drop()`] or [`ManuallyDrop::take()`] on it before the [`.as_out()`]
    /// call will probably just result in a high risk of double-frees in case of unwinds (panics).
    ///
    /// Consider, instead, operating on the `T` *by value* to begin with; wrapping it, by
    /// value, in a [`ManuallyDrop`], or better, in a struct with a drop flag.
    ///
    /// [ManuallyDrop]: `MD`
    /// [`ManuallyDrop`]: `MD`
    /// [`ManuallyDrop::drop()`]: `MD::drop()`
    /// [`ManuallyDrop::take()`]: `MD::take()`
    /// [`.as_out()`]: `AsOut::as_out()`
    fn as_manually_drop_mut_leaky_and_unchecked<'r>(self: &'r mut Self) -> &'r mut MD<Self>
    where
        &'r mut MD<Self> : AsOut<Self>,
    {
        unsafe {
            // Safety: `repr(transparent)` referee, same layout of the (potentially wide) reference,
            // and unless the caller uses `unsafe`, the worst they can do API-wise is a leak.
            mem::transmute(self)
        }
    }
}
impl<T : ?Sized> AsManuallyDropMut for T {}

pub
trait AsOut<T : ?Sized> {
    fn as_out(&mut self) -> OutRef<'_, T>;
}

// Warning: this leaks memory!
impl<T : Copy> AsOut<T> for T {
    fn as_out<'r>(self: &'r mut T) -> OutRef<'r, T> {
        unsafe {
            OutRef::from_raw(self.into())
        }
    }
}

impl<T : ?Sized> AsOut<T> for MD<T> {
    fn as_out(self: &mut MD<T>) -> OutRef<'_, T> {
        unsafe {
            mem::transmute(self)
        }
    }
}

impl<T : Sized> AsOut<T> for MU<T> {
    fn as_out(self: &mut MU<T>) -> OutRef<'_, T> {
        unsafe {
            OutRef::from_raw(ptr::NonNull::cast(self.into()))
        }
    }
}

impl<T : Sized> AsOut<[T]> for [MU<T>] {
    fn as_out<'r>(self: &'r mut [MU<T>]) -> OutRef<'r, [T]> {
        unsafe {
            mem::transmute(self)
        }
    }
}

impl<'r, T : ?Sized> OutRef<'r, T, NoBrandedPlace> {
    #[doc(alias = "new")]
    pub
    fn from_ref(r: &'r mut impl AsOut<T>) -> OutRef<'r, T, NoBrandedPlace> {
        r.as_out()
    }

    pub
    unsafe
    fn from_raw(raw: ptr::NonNull<T>) -> OutRef<'r, T, NoBrandedPlace> {
        unsafe {
            Self::from_raw_branded_unchecked(raw)
        }
    }
}

impl<'r, T : ?Sized, P : MaybeBrandedPlace> OutRef<'r, T, P> {
    pub
    fn reborrow(&mut self) -> OutRef<'_, T, P> {
        unsafe {
            mem::transmute_copy(self)
        }
    }

    /// Convenience alias for [`Self::reborrow()`].
    pub
    fn r(&mut self) -> OutRef<'_, T, P> {
        self.reborrow()
    }

    pub
    fn write(self, value: T) -> OwnRef<'r, T, P>
    where
        T : Sized,
    {
        unsafe {
            self.raw.write(value);
            OwnRef::from_raw_branded_unchecked(self.raw)
        }
    }

    pub
    unsafe
    fn from_raw_branded_unchecked(raw: ptr::NonNull<T>) -> OutRef<'r, T, P> {
        Self { raw, _phantom: PD, _owns_t : PD }
    }

    pub
    unsafe
    fn as_uninit_mut<MaybeUninitT : ?Sized>(self: OutRef<'r, T, P>) -> &'r mut MaybeUninitT
    where
        Self : seal::AsUninitMut<'r, Uninit = MaybeUninitT>,
    {
        unsafe { seal::AsUninitMut::as_uninit_mut_impl(self) }
    }
}

mod seal {
    use super::*;

    pub
    trait AsUninitMut<'r> {
        type Uninit : ?Sized;

        unsafe
        fn as_uninit_mut_impl(_: Self) -> &'r mut Self::Uninit;
    }

    impl<'r, T : Sized, P : MaybeBrandedPlace> AsUninitMut<'r> for OutRef<'r, T, P> {
        type Uninit = MU<T>;

        unsafe
        fn as_uninit_mut_impl(this: Self) -> &'r mut MU<T> {
            unsafe { this.raw.cast().as_mut() }
        }
    }

    impl<'r, T : Sized, P : MaybeBrandedPlace> AsUninitMut<'r> for OutRef<'r, [T], P> {
        type Uninit = [MU<T>];

        unsafe
        fn as_uninit_mut_impl(this: Self) -> &'r mut [MU<T>] {
            let len = this.raw.len();
            unsafe {
                ::core::slice::from_raw_parts_mut(
                    this.raw.as_ptr() as *mut T as *mut MU<T>,
                    len,
                )
            }
        }
    }
}

impl<'r, T : ?Sized> OutRef<'r, T, NoBrandedPlace> {
    pub
    unsafe
    fn branded_unchecked<BrandedPlace : IsBrandedPlace>(
        self: OutRef<'r, T, NoBrandedPlace>,
    ) -> OutRef<'r, T, BrandedPlace>
    {
        unsafe {
            OutRef::from_raw_branded_unchecked(self.into_raw())
        }
    }

    pub
    fn branded<BrandedPlace : IsBrandedPlace>(
        self: OutRef<'r, T, NoBrandedPlace>,
        _witnessed: BrandedPlace,
    ) -> OutRef<'r, T, BrandedPlace>
    {
        unsafe {
            Self::branded_unchecked(self)
        }
    }
}

unsafe
impl<'r, T : ?Sized, BrandedPlace : IsBrandedPlace, BrandedPlace_ : IsBrandedPlace>
    indwell::UninitPlace<BrandedPlace>
for
    OutRef<'r, T, BrandedPlace_>
{
    unsafe
    fn assume_init(
        self: OutRef<'r, T, BrandedPlace_>,
    ) -> OwnRef<'r, T, BrandedPlace>
    {
        unsafe {
            OwnRef::from_raw_branded_unchecked(self.into_raw())
        }
    }
    type OwnedInit = OwnRef<'r, T, BrandedPlace>;
}

pub
struct BrandedUninit<T, BrandedPlace : IsBrandedPlace> {
    value: MU<T>,
    _brand: PD<fn() -> BrandedPlace>,
}

impl<T, BrandedPlace : IsBrandedPlace> BrandedUninit<T, BrandedPlace> {
    pub
    fn new_uninit(_branded_place: BrandedPlace) -> BrandedUninit<T, BrandedPlace> {
        Self { value: MU::uninit(), _brand: PD }
    }

    pub
    fn as_out(self: &mut BrandedUninit<T, BrandedPlace>) -> OutRef<'_, T, BrandedPlace> {
        unsafe {
            OutRef::from_raw_branded_unchecked(ptr::NonNull::from(&mut self.value).cast())
        }
    }
}

unsafe
impl<'r, T : Sized, BrandedPlace : IsBrandedPlace, BrandedPlace_ : IsBrandedPlace>
    indwell::UninitPlace<BrandedPlace>
for
    &'r mut BrandedUninit<T, BrandedPlace_>
where
    BrandedPlace : utils::diagnostics::Is<EqualTo = BrandedPlace_>,
    BrandedPlace_ : utils::diagnostics::Is<EqualTo = BrandedPlace>,
{
    unsafe
    fn assume_init(
        self: &'r mut BrandedUninit<T, BrandedPlace_>,
    ) -> OwnRef<'r, T, BrandedPlace>
    {
        unsafe {
            indwell::UninitPlace::<BrandedPlace>::assume_init(
                self.as_out(),
            )
        }
    }
    type OwnedInit = OwnRef<'r, T, BrandedPlace>;
}

unsafe
impl<T : Sized, BrandedPlace : IsBrandedPlace, BrandedPlace_ : IsBrandedPlace>
    indwell::UninitPlace<BrandedPlace>
for
    BrandedUninit<T, BrandedPlace_>
where
    BrandedPlace : utils::diagnostics::Is<EqualTo = BrandedPlace_>,
    BrandedPlace_ : utils::diagnostics::Is<EqualTo = BrandedPlace>,
{
    unsafe
    fn assume_init(self: BrandedUninit<T, BrandedPlace_>) -> T
    {
        unsafe {
            self.value.assume_init()
        }
    }
    type OwnedInit = T;
}

unsafe
impl<T : Sized, BrandedPlace : IsBrandedPlace, BrandedPlace_ : IsBrandedPlace>
    indwell::UninitPlace<BrandedPlace>
for
    Box<BrandedUninit<T, BrandedPlace_>>
where
    BrandedPlace : utils::diagnostics::Is<EqualTo = BrandedPlace_>,
    BrandedPlace_ : utils::diagnostics::Is<EqualTo = BrandedPlace>,
{
    unsafe
    fn assume_init(
        self: Box<BrandedUninit<T, BrandedPlace_>>,
    ) -> Box<T>
    {
        unsafe {
            Box::from_raw(Box::into_raw(self).cast())
        }
    }
    type OwnedInit = Box<T>;
}

/// Main [`BrandedUninit<T>`] constructor.
///
/// <code>[branded_uninit!](as let foo: SomeType);</code> is the moral equivalent of
/// <code>[BrandedUninit]::new(new_brand!())</code>.
///
/// In fact, it is literally just sugar for calling [`new_branded_place!()`] followed by
/// [`BrandedUninit::new_uninit()`] on it:
///
/// ```rust
/// # use ::enroot::prelude::*;
/// // 1. Create a new `BrandedPlace` to be inferred/resolved to refer to that of `foo`.
/// new_branded_place!(as let foo_place);
/// // 2. Create a morally `MaybeUninit::uninit()` `foo` but with that `BrandedPlace` attached.
/// let mut foo = BrandedUninit::<SomeType, _>::new_uninit(foo_place);
/// ```
///
/// The `: Type` part in <code>[branded_uninit!](as let varname: Type);</code> can be elided so it
/// be inferred.
///
/// # Example
///
/// ```rust
/// use ::enroot::prelude::*;
///
/// branded_uninit!(as let foo: String);
/// // results in:
/// let _: BrandedUninit<String, BrandedPlace<'_, _>> = foo;
///
/// // And:
/// branded_uninit!(as let foo);
/// // is sugar for:
/// branded_uninit!(as let foo: _);
/// ```
#[doc(hidden)] /* Not part of the public API */ #[macro_export]
macro_rules! ඞbranded_uninit {(
    let $($binding:ident $(: $T:ty)?),+ $(,)?
) => (
    $crate::brand::new_branded_place!(as let $($binding),+);
    $(
        #[allow(unused_mut)]
        let mut $binding = $crate::out_ref::BrandedUninit $(::<$T, _>)? ::new_uninit($binding);
    )+
)}
#[doc(inline)]
pub use ඞbranded_uninit as branded_uninit;

pub use project::{derive_marker, OutRefProject};
pub(crate)
mod project;
