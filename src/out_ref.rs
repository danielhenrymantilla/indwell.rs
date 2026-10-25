use crate::*;

pub use crate::proc_macros::OutRefProject;

#[repr(transparent)]
pub
struct OutRef<'r, T : ?Sized, P : MaybeBrandedPlace = NoBrandedPlace> {
    raw: ptr::NonNull<T>,
    _phantom: PD<fn(fn(&'r (), &mut T)) -> P>,
    _owns_t: PD<T>,
}

impl<'r, T : ?Sized, P : MaybeBrandedPlace> OutRef<'r, T, P> {
    pub
    fn into_raw(self) -> ptr::NonNull<T> {
        self.raw
    }
}

pub
trait AsOutRef<T : ?Sized> {
    fn as_out_ref(&mut self) -> OutRef<'_, T>;
}

pub
trait AsManuallyDropMut {
    /// Cast the current `&mut` reference into a <code>\&mut [ManuallyDrop]\<T\></code> reference
    /// so as to, notably, be able to call [`.as_out_ref()`] on it.
    ///
    /// Note that such a pattern is **prone to memory leaks**. And trying to `unsafe`ly manually
    /// call [`ManuallyDrop::drop()`] or [`ManuallyDrop::take()`] on it before the [`.as_out_ref()`]
    /// call will probably just result in a high risk of double-frees in case of unwinds (panics).
    ///
    /// Consider, instead, operating on the `T` *by value* to begin with; wrapping it, by
    /// value, in a [`ManuallyDrop`], or better, in a struct with a drop flag.
    ///
    /// [ManuallyDrop]: `MD`
    /// [`ManuallyDrop`]: `MD`
    /// [`ManuallyDrop::drop()`]: `MD::drop()`
    /// [`ManuallyDrop::take()`]: `MD::take()`
    /// [`.as_out_ref()`]: `AsOutRef::as_out_ref()`
    fn as_manually_drop_mut_leaky_and_unchecked<'r>(self: &'r mut Self) -> &'r mut MD<Self>
    where
        &'r mut MD<Self> : AsOutRef<Self>,
    {
        unsafe {
            // Safety: `repr(transparent)` referee, same layout of the (potentially wide) reference,
            // and unless the caller uses `unsafe`, the worst they can do API-wise is a leak.
            mem::transmute(self)
        }
    }
}
impl<T : ?Sized> AsManuallyDropMut for T {}

// Warning: this leaks memory!
impl<T : Copy> AsOutRef<T> for T {
    fn as_out_ref<'r>(self: &'r mut T) -> OutRef<'r, T> {
        unsafe {
            OutRef::from_raw(self.into())
        }
    }
}

impl<T : ?Sized> AsOutRef<T> for MD<T> {
    fn as_out_ref(self: &mut MD<T>) -> OutRef<'_, T> {
        unsafe {
            mem::transmute(self)
        }
    }
}

impl<T : Sized> AsOutRef<T> for MU<T> {
    fn as_out_ref(self: &mut MU<T>) -> OutRef<'_, T> {
        unsafe {
            OutRef::from_raw(ptr::NonNull::cast(self.into()))
        }
    }
}

impl<'r, T : ?Sized> OutRef<'r, T, NoBrandedPlace> {
    #[doc(alias = "new")]
    pub
    fn from_ref(r: &'r mut impl AsOutRef<T>) -> OutRef<'r, T, NoBrandedPlace> {
        r.as_out_ref()
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
}

impl<'r, T : ?Sized> OutRef<'r, T, NoBrandedPlace> {
    pub
    fn branded<BrandedPlace : IsBrandedPlace>(
        self: OutRef<'r, T, NoBrandedPlace>,
        _witnessed: BrandedPlace,
    ) -> OutRef<'r, T, BrandedPlace>
    {
        unsafe {
            OutRef::from_raw_branded_unchecked(self.into_raw())
        }
    }
}

unsafe
impl<'r, T : Sized, BrandedPlace : IsBrandedPlace>
    indwell::UninitPlace<BrandedPlace>
for
    OutRef<'r, T, BrandedPlace>
{
    unsafe
    fn assume_init(
        self: OutRef<'r, T, BrandedPlace>,
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
    fn as_out_ref(self: &mut BrandedUninit<T, BrandedPlace>) -> OutRef<'_, T, BrandedPlace> {
        unsafe {
            OutRef::from_raw_branded_unchecked(ptr::NonNull::from(&mut self.value).cast())
        }
    }
}

unsafe
impl<'r, T : Sized, BrandedPlace : IsBrandedPlace>
    indwell::UninitPlace<BrandedPlace>
for
    &'r mut BrandedUninit<T, BrandedPlace>
{
    unsafe
    fn assume_init(
        self: &'r mut BrandedUninit<T, BrandedPlace>,
    ) -> OwnRef<'r, T, BrandedPlace>
    {
        unsafe {
            OwnRef::from_raw_branded_unchecked(ptr::NonNull::from(&mut self.value).cast())
        }
    }
    type OwnedInit = OwnRef<'r, T, BrandedPlace>;
}

unsafe
impl<T : Sized, BrandedPlace : IsBrandedPlace>
    indwell::UninitPlace<BrandedPlace>
for
    Box<BrandedUninit<T, BrandedPlace>>
{
    unsafe
    fn assume_init(
        self: Box<BrandedUninit<T, BrandedPlace>>,
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

impl<'r, T : ?Sized, FieldPlaces : IsBrandedPlace>
    OutRef<'r, T, FieldPlaces>
{
    pub
    fn project<'s, Projection>(self: &'s mut OutRef<'r, T, FieldPlaces>)
      -> Projection
    where
        T : derive_marker::OutRefProject,
        T : OutRefProject<FieldPlaces, Projection<'s> = Projection>,
    {
        OutRefProject::project_impl(self.r())
    }
}

#[diagnostic::on_unimplemented(
    message = "Cannot `.project()` without having called `{Self}::attach_field_places()` first",
    label = "call `<{Self}>::attach_field_places()` on its receiver first, shadowing it with a `let`",
    note = "\
        In order for `.project()` + `out.each_field.write(…)` + `indwell!(out <- ...)` to be safe \
        and sound, it is necessary to:\n \
         1. attach extra `PlaceBrand`s for each one of the fields,\n \
         2. do the `.project()` on that, `.write(…)`-init each of the fields,\n \
         3. and finally `indwell!(out <- …)` where `out` is the version with the field placebrands \
            attached.\n\
        \n    \
        let mut out_foo = Foo::attach_field_places(out_foo);\n    \
        let out_fields = out_foo.project();\n    \
        let own_fields = FooOwnRefs {{\n        \
            bar: out_fields.bar.write(…),\n        \
            quux: out_fields.quux.write(…),\n    \
        }};\n    \
        let own_foo = indwell!(out_foo <- own_fields);\n\
    "
)]
pub
trait OutRefProject<BrandedPlaceAndFieldPlaces : IsBrandedPlace>
:
    // derive_marker::OutRefProject;
{
    type Projection<'s>;
    fn project_impl<'s>(
        out_ref: OutRef<'s, Self, BrandedPlaceAndFieldPlaces>,
    ) -> Self::Projection<'s>
    ;
}

#[doc(hidden)] /** Not part of the public API */ pub
mod derive_marker {
    #[diagnostic::on_unimplemented(
        message = "Missing `#[derive(OutRefProject)]` on `{Self}`",
        label = "add `#[derive(OutRefProject)]` on its receiver",
        // note = "From there, you will need "
    )]
    pub trait OutRefProject {}
}

pub(crate) mod more_diagnostics_tricks {
    use super::*;

    pub
    trait OutRefProject<BrandedPlaceAndFieldPlaces : IsBrandedPlace> {
        // This is an assoc function provided through a `prelude`-provided extension trait.
        //
        // This way somebody somehow guessing to use this but having forgotten the `derive` will
        // get better diagnostics than a mere "assoc function not found".
        //
        /// Magic function provided by <code>#[derive([OutRefProject])]</code>, to enable usage
        /// of [`OutRef::project()`].
        ///
        /// [OutRefProject]: `derive@OutRefProject`
        fn attach_field_places<'r>(_: OutRef<'r, Self, impl IsBrandedPlace>)
          -> OutRef<'r, Self, BrandedPlaceAndFieldPlaces>
        where
            Self : derive_marker::OutRefProject,
        {
            Self::NEVER
        }
    }
    impl<'brand, T, Place> OutRefProject<BrandedPlace<'brand, Place>> for T {}

    mod derive_marker {
        #[diagnostic::on_unimplemented(
            message = "Missing `#[derive(OutRefProject)]` on `{Self}`",
            label = "needs a `#[derive(OutRefProject)]`",
        )]
        pub
        trait OutRefProject {
            const NEVER: ::never_say_never::Never;
        }
    }
}

impl<'r, T : ?Sized, TPlace : IsBrandedPlace, FieldPlaces: IsBrandedPlace>
    OutRef<'r, T, HList![TPlace, ..FieldPlaces]>
{
    pub
    fn discard_field_places(
        self: OutRef<'r, T, HList![TPlace, ..FieldPlaces]>,
    ) -> OutRef<'r, T, TPlace>
    {
        unsafe {
            OutRef::from_raw_branded_unchecked(self.into_raw())
        }
    }
}
