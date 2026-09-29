use crate::own_ref::{GhostInitProof, InitProvable};

use super::*;

#[repr(transparent)]
pub
struct OutRef<'r, T : ?Sized, B : MaybeBrand = Unbranded> {
    raw: ptr::NonNull<T>,
    _phantom: PD<fn(B, fn(&'r (), &mut T))>,
    _owns_t : PD<T>,
}

impl<'r, T : ?Sized, B : MaybeBrand> OutRef<'r, T, B> {
    pub
    fn into_raw(self) -> ptr::NonNull<T> {
        MD::new(self).raw
    }

    pub
    fn leak(self) -> &'r mut MD<T> {
        unsafe { mem::transmute(self) }
    }
}

pub
trait AsOutRef<T : ?Sized> {
    fn as_out_ref(&mut self) -> OutRef<'_, T>;
}

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
            mem::transmute(self)
        }
    }
}

impl<'r, T : ?Sized> OutRef<'r, T, Unbranded> {
    #[doc(alias = "new")]
    pub
    fn from_ref(r: &'r mut impl AsOutRef<T>) -> OutRef<'r, T, Unbranded> {
        r.as_out_ref()
    }

    pub
    unsafe
    fn from_raw(raw: ptr::NonNull<T>) -> OutRef<'r, T, Unbranded> {
        unsafe {
            Self::from_raw_branded_unchecked(raw)
        }
    }
}

impl<'r, T : ?Sized, B : MaybeBrand> OutRef<'r, T, B> {
    pub
    fn reborrow(&mut self) -> OutRef<'_, T, B> {
        unsafe {
            mem::transmute_copy(self)
        }
    }

    /// Convenience alias for [`Self::reborrow()`].
    pub
    fn r(&mut self) -> OutRef<'_, T, B> {
        self.reborrow()
    }

    pub
    fn write(self, value: T) -> OwnRef<'r, T, B>
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
    fn from_raw_branded_unchecked(raw: ptr::NonNull<T>) -> OutRef<'r, T, B> {
        Self { raw, _phantom: PD, _owns_t : PD }
    }
}

impl<'r, T : ?Sized> OutRef<'r, T> {
    pub
    fn branded<Brand : MaybeBrand>(self: OutRef<'r, T>, _witnessed: Brand) -> OutRef<'r, T, Brand> {
        unsafe {
            OutRef::from_raw_branded_unchecked(self.into_raw())
        }
    }
}

impl<'r, 'brand, T : Sized, Diagnostics>
    InitProvable<'r, 'brand, T, Diagnostics>
for
    OutRef<'r, T, Brand<'brand, Diagnostics>>
{    
    fn proven_init(
        _: GhostInitProof<'brand, Diagnostics>,
        out_ref: OutRef<'r, T, Brand<'brand, Diagnostics>>,
    ) -> OwnRef<'r, T, Brand<'brand, Diagnostics>>
    {
        unsafe {
            OwnRef::from_raw_branded_unchecked(out_ref.into_raw())
        }
    }
}

pub
struct BrandedPlace<'brand, T, Diagnostics = ()> {
    value: MU<T>,
    _brand: PD<(Brand<'brand>, fn() -> Diagnostics)>,
}

pub
fn uninit<'brand, T, Diagnostics>(_witnessed: Brand<'brand, Diagnostics>)
  -> BrandedPlace<'brand, T, Diagnostics>
{
    BrandedPlace { value: MU::uninit(), _brand: PD }
}

impl<'brand, T, Diagnostics> BrandedPlace<'brand, T, Diagnostics> {
    pub
    fn uninit(_witnessed: Brand<'brand>) -> BrandedPlace<'brand, T, Diagnostics> {
        Self { value: MU::uninit(), _brand: PD }
    }

    pub
    fn as_out_ref(self: &mut BrandedPlace<'brand, T, Diagnostics>)
      -> OutRef<'_, T, Brand<'brand, Diagnostics>>
    {
        unsafe {
            OutRef::from_raw_branded_unchecked(ptr::NonNull::from(&mut self.value).cast())
        }
    }
}

impl<'r, 'brand, T : Sized, Diagnostics>
    InitProvable<'r, 'brand, T, Diagnostics>
for
    &'r mut BrandedPlace<'brand, T, Diagnostics>
{    
    fn proven_init(
        _: GhostInitProof<'brand, Diagnostics>,
        place: &'r mut BrandedPlace<'brand, T, Diagnostics>,
    ) -> OwnRef<'r, T, Brand<'brand, Diagnostics>>
    {
        unsafe {
            OwnRef::from_raw_branded_unchecked(ptr::NonNull::from(&mut place.value).cast())
        }
    }
}

#[doc(hidden)] /** Not part of the public API */ #[macro_export]
macro_rules! ඞbranded_places {(
    let $($binding:ident $(: $T:ty)?),+ $(,)?
) => (
    $crate::brand::brands!(as let $($binding),+);
    $(
        #[allow(unused_mut)]
        let mut $binding = $crate::out_ref::uninit $(::<$T, _>)? ($binding);
    )+
)}
#[doc(inline)]
pub use ඞbranded_places as branded_places;

#[::implied_bounds::implied_bounds]
pub
trait BrandsFor<Struct> : MaybeBrand
where 
    Struct : Project<Self>,
{}

impl<Brands, Struct> BrandsFor<Struct> for Brands
where
    Self : MaybeBrand,
    Struct : Project<Brands>,
{}

impl<'r, 'brand, T : ?Sized, Brands : MaybeBrand>
    OutRef<'r, T, Brands>
{
    pub
    fn project<'s>(self: &'s mut OutRef<'r, T, Brands>)
      -> <T as Project<Brands>>::Projection<'s>
    where
        T : Project<Brands>,
    {
        Project::project_impl(self.r())
    }
}

pub
trait Project<Brands : MaybeBrand> {
    type Projection<'s>;
    fn project_impl<'s>(
        out_ref: OutRef<'s, Self, Brands>,
    ) -> Self::Projection<'s>;

    // type HeadBrand;
    // type GhostInits;
    // fn indwell(
    //     _: Self::GhostInits,
    //     out_ref: OutRef<'s, Self, Brands>,
    // ) -> OwnRef<'s, Self, Self::HeadBrand>;
}