use crate::own_ref::{GhostInitProof, InitProvable};

use super::*;

#[repr(transparent)]
pub
struct OutRef<'r, T : ?Sized, P : MaybeBrandedPlace = NoBrandedPlace> {
    raw: ptr::NonNull<T>,
    _phantom: PD<fn(fn(&'r (), &mut T)) -> P>,
    _owns_t : PD<T>,
}

impl<'r, T : ?Sized, P : MaybeBrandedPlace> OutRef<'r, T, P> {
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

impl<'r, T : Sized, BrandedPlace : IsBrandedPlace>
    InitProvable<BrandedPlace>
for
    OutRef<'r, T, BrandedPlace>
{
    fn proven_init(
        _: GhostInitProof<BrandedPlace>,
        out_ref: OutRef<'r, T, BrandedPlace>,
    ) -> OwnRef<'r, T, BrandedPlace>
    {
        unsafe {
            OwnRef::from_raw_branded_unchecked(out_ref.into_raw())
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
    witnessed: BrandedPlace,
) -> BrandedUninit<T, BrandedPlace>
{
    BrandedUninit::uninit(witnessed)
}

impl<T, BrandedPlace : IsBrandedPlace> BrandedUninit<T, BrandedPlace> {
    pub
    fn uninit(_witnessed: BrandedPlace) -> BrandedUninit<T, BrandedPlace> {
        Self { value: MU::uninit(), _brand: PD }
    }

    pub
    fn as_out_ref(self: &mut BrandedUninit<T, BrandedPlace>) -> OutRef<'_, T, BrandedPlace> {
        unsafe {
            OutRef::from_raw_branded_unchecked(ptr::NonNull::from(&mut self.value).cast())
        }
    }
}

impl<'r, T : Sized, BrandedPlace : IsBrandedPlace>
    InitProvable<BrandedPlace>
for
    &'r mut BrandedUninit<T, BrandedPlace>
{
    fn proven_init(
        _: GhostInitProof<BrandedPlace>,
        place: &'r mut BrandedUninit<T, BrandedPlace>,
    ) -> OwnRef<'r, T, BrandedPlace>
    {
        unsafe {
            OwnRef::from_raw_branded_unchecked(ptr::NonNull::from(&mut place.value).cast())
        }
    }
    type OwnedInit = OwnRef<'r, T, BrandedPlace>;
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
/// Foo
#[doc(inline)]
pub use ඞbranded_uninit as branded_uninit;

// #[::implied_bounds::implied_bounds]
// pub
// trait BrandsFor<Struct> : IsBrandedPlace
// where
//     Struct : OutRefProject<Self>,
// {}

// impl<Brands, Struct> BrandsFor<Struct> for Brands
// where
//     Self : IsBrandedPlace,
//     Struct : OutRefProject<Brands>,
// {}

impl<'r, 'brand, T : ?Sized, Brands : IsBrandedPlace>
    OutRef<'r, T, Brands>
{
    pub
    fn project<'s>(self: &'s mut OutRef<'r, T, Brands>)
      -> <T as OutRefProject<Brands>>::Projection<'s>
    where
        T : OutRefProject<Brands>,
    {
        OutRefProject::project_impl(self.r())
    }
}

pub use crate::proc_macros::OutRefProject;

pub
trait OutRefProject<Brands : IsBrandedPlace> {
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
