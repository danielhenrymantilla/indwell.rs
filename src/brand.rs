use super::*;

// We want two pair of type "ctors" so that:
//  1. `H1<'lt> <: H0`, but
//  1. `H1<'lt>` be invariant over `'lt`.
with_h1_h0! {
    // We got non-covariance because of `fn(&'lt (), _)`.
    // And non-contravariance because of `fn(_, fn(&'lt ())`.
    type H1<'lt> = fn(&'lt (), fn(&'lt ()));
    // - Since `&'static () <: &'lt ()`, we have
    //   `fn(&'lt (), _) <: fn(&'static (), _)`.
    // - `fn(&()) = for<'a> fn(&'a ()) <: fn(&'lt ())` (the "pick `for<'a = 'lt>` rule"). So
    //   `fn(_, fn(&'lt ()) <: fn(_, fn(&()))`.
    // Thus, `H1<'lt> <: H0`.
    type H0 = fn(&'static (), fn(&()));
}
// where:
macro_rules! with_h1_h0 {( $($type_defs:tt)* ) => (
    // 1. Emit it.
    $($type_defs)*

    /// 2. Check Check `H1<'_> <: H0`:
    /// 
    /// ```rust
    #[doc = stringify!($($type_defs)*)]
    ///
    /// fn _assert_h1_subtypes_h0<'lt>(h: *const *const H1<'lt>) -> *const *const H0 {
    ///     h
    /// }
    /// ```
    /// 
    /// 3. Check `H1<'_>` is indeed invariant over `<'_>`:
    /// 
    /// ```rust ,compile_fail
    #[doc = stringify!($($type_defs)*)]
    /// 
    /// fn assert_covariant<'x>(x: *const *const H1<'static>) -> *const *const H1<'x> {
    ///     x // Error, non-covariant.
    /// }
    /// ```
    /// 
    /// ```rust ,compile_fail
    #[doc = stringify!($($type_defs)*)]
    ///
    /// fn assert_contravariant<'x>(x: *const *const H1<'x>) -> *const *const H1<'static> {
    ///     x // Error, non-contravariant.
    /// }
    /// ```
    /// 
    const _ASSERT_INVARIANT: () = ();
)}
use with_h1_h0;

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
struct Brand<'lt, Place = (), H : seal::IsHelper = H1<'lt>>(
    PD<(fn(&'lt ()), fn() -> Place, H)>,
);

impl<'lt, Place> PlaceBrand for Brand<'lt, Place> {}

pub
type Unbranded<Place = ()> = Brand<'static, Place, H0>;

pub(crate) use seal::Sealed as ImplMaybeBrand;
mod seal {
    use super::*;


    pub
    trait Sealed {}

    #[diagnostic::do_not_recommend]
    impl<T : Sealed + Send + Sync> MaybeBrand for T {}

    impl<H : IsHelper, Place> Sealed for Brand<'_, Place, H> {}

    pub
    trait IsHelper {}

    impl<'brand> IsHelper for H1<'brand> {}

    #[expect(coherence_leak_check)]
    impl<'brand> IsHelper for H0 {}
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
