use super::*;

pub
trait MaybeBrand
:
    Sized + Send + Sync + seal::Sealed +
{}

pub
trait PlaceBrand
:
    MaybeBrand +
{}

pub
struct Brand<'lt, Place = ()>(
    PD<fn(&mut &'lt ()) -> Place>,
);

impl<'lt, Place> PlaceBrand for Brand<'lt, Place> {}

pub
type Unbranded<Place = ()> = Brand<'static, Place>;

pub(crate) use seal::Sealed as ImplMaybeBrand;
mod seal {
    use super::*;


    pub
    trait Sealed {}

    #[diagnostic::do_not_recommend]
    impl<T : Sealed + Send + Sync> MaybeBrand for T {}

    impl<Place> Sealed for Brand<'_, Place> {}
}

impl Brand<'_> {
    pub
    fn with_new<R>(scope: impl FnOnce(Brand<'_>) -> R) -> R {
        scope(Self(PD))
    }

    pub
    async fn with_new_async<R>(scope: impl AsyncFnOnce(Brand<'_>) -> R) -> R {
        scope(Self(PD)).await
    }
}

pub trait Place<T> {}
impl<T> Place<T> for () {}

#[macro_export]
macro_rules! ඞbrands {(
    as let$($binding:ident),+ $(,)?
) => (
    $(
        let $binding = &$crate::brand::ඞ::borrowee_upper_bounding_the_lifetime();
        let $binding = &{
            $crate::brand::ඞ::borrower_with_impl_dropck_thus_lower_bounding_the_lifetime(
                $binding,
                // the place here is merely diagnostical, it does not really have
                // "branding" capabilities.
                //
                // Indeed, no need to do `Brand<'brand, impl use<Brand<'brand>> + Place<$binding>`,
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
pub use ඞbrands as brands;

// Helpers for the macro API.
#[doc(hidden)] /** Not part of the public API */ pub
mod ඞ {
    use super::*;

    pub
    trait GetBrand<'brand, Place> {
        fn get_brand(_: &Self) -> Brand<'brand, Place>;
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
            fn get_brand(_: &Self) -> Brand<'brand, Place> {
                Brand(PD)
            }
        }
        S {}
    }
}
