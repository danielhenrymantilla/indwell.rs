use crate::brand::seal2::ShatterShard;

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
struct Brand<'lt, Diagnostics = (), H : seal::IsHelper = H1<'lt>>(
    PD<(fn(&'lt ()), fn() -> Diagnostics, H)>,
);

mod seal2 {
    pub trait ShatterShard<const N: usize, Diagnostics> {}
    impl<const N: usize, Diagnostics> ShatterShard<N, Diagnostics> for () {}
}

impl<'lt, Diagnostics> Brand<'lt, Diagnostics> {
    pub fn shatter(self) -> (
        Brand<'lt, impl ShatterShard<0, Diagnostics>>,
        Brand<'lt, impl ShatterShard<1, Diagnostics>>,
        Brand<'lt, impl ShatterShard<2, Diagnostics>>,
        Brand<'lt, impl ShatterShard<3, Diagnostics>>,
        Brand<'lt, impl ShatterShard<4, Diagnostics>>,
        Brand<'lt, impl ShatterShard<5, Diagnostics>>,
        Brand<'lt, impl ShatterShard<6, Diagnostics>>,
        Brand<'lt, impl ShatterShard<7, Diagnostics>>,
    )
    {
        (
            Brand::<()>(PD),
            Brand::<()>(PD),
            Brand::<()>(PD),
            Brand::<()>(PD),
            Brand::<()>(PD),
            Brand::<()>(PD),
            Brand::<()>(PD),
            Brand::<()>(PD),
        )
    }
}

pub
type Unbranded<Diagnostics = ()> = Brand<'static, Diagnostics, H0>;

pub trait MaybeBrand : seal::Sealed {}
impl<H : seal::IsHelper, Diagnostics> MaybeBrand for Brand<'_, Diagnostics, H> {}

mod seal {
    use super::*;

    pub trait Sealed {}
    impl<H : IsHelper, Diagnostics> Sealed for Brand<'_, Diagnostics, H> {}

    pub
    trait IsHelper {}

    impl<'brand> IsHelper for H1<'brand> {}

    #[expect(coherence_leak_check)]
    impl<'brand> IsHelper for H0 {}

    impl<Hd : MaybeBrand, Tl : MaybeBrand>
        MaybeBrand
    for
        HList![Hd, ..Tl]
    {
        type Head = 
    }

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

pub struct Place<T>(pub T);

#[macro_export]
macro_rules! ඞbrands {(
    as let$($binding:ident),+ $(,)?
) => (
    $(
        let $binding = &$crate::brand::ඞ::borrowee_upper_bounding_the_lifetime();
        let $binding = &{
            $crate::brand::ඞ::borrower_with_impl_dropck_thus_lower_bounding_the_lifetime(
                $binding,
                {
                    #[allow(warnings, clippy::all)]
                    trait $binding {} impl $binding for () {}
                    fn $binding() -> $crate::brand::Place<impl $binding> {
                        $crate::brand::Place(())
                    }
                    $binding
                },
            )
        };
        let $binding = $crate::brand::ඞ::GetBrand::get_brand($binding);
    )+
)}
pub use ඞbrands as brands;

// Helpers for the macro API.
#[doc(hidden)] /** Not part of the public API */ pub
mod ඞ {
    use super::*;

    pub
    trait GetBrand<'brand, Diagnostics> {
        fn get_brand(_: &Self) -> Brand<'brand, Diagnostics>;
    }

    mod sealed {
        pub trait Borrowee {}
    }

    #[inline(always)]
    pub
    fn borrowee_upper_bounding_the_lifetime() -> impl sealed::Borrowee {
        struct S {}
        impl sealed::Borrowee for S {}
        S {}
    }


    /// Safety: can only be called by the macro API.
    #[inline(always)]
    pub
    fn borrower_with_impl_dropck_thus_lower_bounding_the_lifetime<'r, Diagnostics>(
        _: &'r impl sealed::Borrowee,
        _: impl FnOnce() -> Diagnostics,
    ) -> impl GetBrand<'r, Diagnostics> {
        struct S {}
        impl<'brand, Diagnostics> GetBrand<'brand, Diagnostics> for S {
            #[inline(always)]
            fn get_brand(_: &Self) -> Brand<'brand, Diagnostics> {
                Brand(PD)
            }
        }
        S {}
    }
}
