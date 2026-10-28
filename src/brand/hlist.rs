use crate::*;

pub struct Fields<T>(T);

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

impl<Tl : MaybeBrandedPlace>
    crate::brand::ImplMaybeBrand
for
    Fields<Tl>
{}

impl<Tl : IsBrandedPlace>
    IsBrandedPlace
for
    Fields<Tl>
{}
