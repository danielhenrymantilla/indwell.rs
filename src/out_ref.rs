use crate::*;

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

// Warning: this leaks memory!
impl<T : ?Sized> AsOutRef<T> for T {
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

pub
fn branded_uninit<T, BrandedPlace : IsBrandedPlace>(
    branded_place: BrandedPlace,
) -> BrandedUninit<T, BrandedPlace>
{
    BrandedUninit::new(branded_place)
}

impl<T, BrandedPlace : IsBrandedPlace> BrandedUninit<T, BrandedPlace> {
    pub
    fn new(_branded_place: BrandedPlace) -> BrandedUninit<T, BrandedPlace> {
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

#[doc(hidden)] /** Not part of the public API */ #[macro_export]
macro_rules! ඞbranded_uninit {(
    let $($binding:ident $(: $T:ty)?),+ $(,)?
) => (
    $crate::brand::new_branded_place!(as let $($binding),+);
    $(
        #[allow(unused_mut)]
        let mut $binding = $crate::out_ref::branded_uninit $(::<$T, _>)? ($binding);
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

pub use crate::proc_macros::OutRefProject;

pub mod derive_marker {
    #[diagnostic::on_unimplemented(
        message = "Missing `#[derive(OutRefProject)]` on `{Self}`",
        label = "add `#[derive(OutRefProject)]` on its receiver",
    )]
    pub trait OutRefProject {}
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
trait OutRefProject<BrandedPlaces : IsBrandedPlace>
:
    // derive_marker::OutRefProject;
{
    type Projection<'s>;
    fn project_impl<'s>(
        out_ref: OutRef<'s, Self, BrandedPlaces>,
    ) -> Self::Projection<'s>
    ;
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
