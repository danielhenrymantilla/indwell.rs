use crate::*;

pub
trait MaybeBrandedPlace
:
    Sized + Send + Sync + seal::Sealed +
{}

#[diagnostic::on_unimplemented(
    message = "unbranded (default) `NoBrandedPlace` parameter cannot be soundly used with this API",
    label = "this instance is missing an `impl BrandedPlace` parameter",
    note = "consider introducing a new `<PlaceFoo : IsBrandedPlace>` generic parameter and using it \
            as second type parameter of the `OutRef/OwnRef`,\n\
            OR\n\
            creating a new branding place through `new_branded_places!()`, and attaching it.\
    ",
)]
pub
trait IsBrandedPlace
:
    MaybeBrandedPlace +
{}

pub
struct BrandedPlace<'brand, Place = ()>(
    PD<fn(&mut &'brand ()) -> Place>,
);

impl<'brand, Place> IsBrandedPlace for BrandedPlace<'brand, Place> {}

pub
enum NoBrandedPlace {}

pub(crate) use seal::Sealed as ImplMaybeBrand;
mod seal {
    use super::*;

    pub trait Sealed {}

    #[diagnostic::do_not_recommend]
    impl<T : Sealed + Send + Sync> MaybeBrandedPlace for T {}

    impl<Place> Sealed for BrandedPlace<'_, Place> {}
    impl Sealed for NoBrandedPlace {}
}

impl BrandedPlace<'_> {
    pub
    fn with_new<R>(scope: impl FnOnce(BrandedPlace<'_>) -> R) -> R {
        scope(Self(PD))
    }

    pub
    async fn with_new_async<R>(scope: impl AsyncFnOnce(BrandedPlace<'_>) -> R) -> R {
        scope(Self(PD)).await
    }
}

pub trait Place<T> {}
impl<T> Place<T> for () {}

#[macro_export]
macro_rules! ඞnew_branded_place {(
    as let $($binding:ident),+ $(,)?
) => (
    $(
        let $binding = $crate::brand::ඞ::borrowee_upper_bounding_the_lifetime();
        let $binding = &{
            $crate::brand::ඞ::borrower_with_impl_dropck_thus_lower_bounding_the_lifetime(
                &$binding,
                // the place here is merely diagnostical, it does not really have
                // "branding" capabilities.
                //
                // And no need to do `Brand<'brand, impl use<Brand<'brand>> + Place<$binding>`,
                // when `Brand<'brand, impl use<> + Place<$binding>>` works just as well.
                {
                    #[allow(nonstandard_style)]
                    enum $binding {}
                    fn $binding() -> impl use<> + $crate::brand::Place<$binding> {}
                    $binding
                },
            )
        };
        let $binding = $crate::brand::ඞ::GetBrand::get_brand($binding);
    )+
)}
#[doc(inline)]
pub use ඞnew_branded_place as new_branded_place;

// Helpers for the macro API.
#[doc(hidden)] /** Not part of the public API */ pub
mod ඞ {
    use super::*;

    pub
    trait GetBrand<'brand, Place> {
        fn get_brand(_: &Self) -> BrandedPlace<'brand, Place>;
    }

    pub trait Borrowee {}

    #[inline(always)]
    pub
    fn borrowee_upper_bounding_the_lifetime() -> impl Borrowee {
        struct S {}
        impl Borrowee for S {}
        S {}
    }

    /// Safety: can only be called by the macro API.
    #[inline(always)]
    pub
    fn borrower_with_impl_dropck_thus_lower_bounding_the_lifetime<'r, Place>(
        _: &'r impl Borrowee,
        _: impl FnOnce() -> Place,
    ) -> impl GetBrand<'r, Place> {
        struct S {}
        impl<'brand, Place> GetBrand<'brand, Place> for S {
            #[inline(always)]
            fn get_brand(_: &Self) -> BrandedPlace<'brand, Place> {
                BrandedPlace(PD)
            }
        }
        S {}
    }
}
