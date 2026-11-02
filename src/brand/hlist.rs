use crate::*;

#[macro_export]
/// <!-- rust-analyzer delimiter nudge
/// ```rust ,ignore
/// HList![];
/// ``` -->
macro_rules! ඞHList {
    [] => { () };

    [ $Hd:ty, ..$Rest:ty $(,)? ] => {
        ($Hd, $Rest)
    };

    [ $Hd:ty $(, $Rest:ty)* $(,)? ] => {
        (
            $Hd,
            $crate::brand::hlist::HList![$($Rest),*]
        )
    };

}
#[doc(inline)]
pub use ඞHList as HList;

impl<Hd : MaybeBrandedPlace, Tl : MaybeBrandedPlace>
    crate::brand::ImplMaybeBrand
for
    HList![Hd, ..Tl]
{}

impl<Hd : MaybeBrandedPlace>
    crate::brand::ImplMaybeBrand
for
    HList![Hd]
{}

impl<Hd : IsBrandedPlace, Tl : IsBrandedPlace>
    IsBrandedPlace
for
    HList![Hd, ..Tl]
{}

impl<Hd : IsBrandedPlace>
    IsBrandedPlace
for
    HList![Hd]
{}

pub
struct Fields<T>(
    T,
);

impl<B : MaybeBrandedPlace>
    crate::brand::ImplMaybeBrand
for
    Fields<B>
{}

impl<B : IsBrandedPlace>
    IsBrandedPlace
for
    Fields<B>
{}

pub
struct SliceElem<T>(
    T,
);

impl<B : MaybeBrandedPlace>
    crate::brand::ImplMaybeBrand
for
    SliceElem<B>
{}

/// An un-implementation.
impl<B : /* IsBrandedPlace */>
    /* !*/IsBrandedPlace
for
    SliceElem<B>
where
    Self : crate::brand::MaybeBrandedPlace,
    for<'n> Self : diagnostics::DeliberatelyNotABrandedPlace,
{}

mod diagnostics {
    #[diagnostic::on_unimplemented(
        message = "`{Self}` is deliberately not deemed an `IsBrandedPlace`",

        note = "\
indeed, as the `.iter_out()` API handles out *concurrent, distinct* `OutRef`s \"branded\" with \
`{Self}`, if `{Self}` were `IsBrandedPlace`, it would then be possible to unsoundly use one \
initialized `OutRef` as proof of initialization/notarization/indwellment for the other one:

    let mut iter = out_slice.iter_out();
    let (a, b) = (iter.next().unwrap(), iter.next().unwrap());
    indwell!(b <- a.write(some_value)); // Uh-oh…

",
    )]
    #[expect(drop_bounds)]
    pub
    trait DeliberatelyNotABrandedPlace : Copy + Drop {}
}
