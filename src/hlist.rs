use crate::prelude::*;

#[macro_export]
/// ```rust ,ignore
/// HList![];
/// ```
macro_rules! ඞHList {
    [] => { () };

    [ $Hd:ty, ..$Rest:ty ] => {
        ($Hd, $Rest)
    };

    [ $Hd:ty $(, $Rest:ty)* $(,)? ] => {
        (
            $Hd,
            $crate::hlist::HList![$($Rest),*]
        )
    };

}
#[doc(inline)]
pub use ඞHList as HList;

impl<Hd : MaybeBrand, Tl : MaybeBrand>
    crate::brand::ImplMaybeBrand
for
    HList![Hd, ..Tl]
{}

impl<Hd : MaybeBrand>
    crate::brand::ImplMaybeBrand
for
    HList![Hd]
{}

impl<Hd : PlaceBrand, Tl : PlaceBrand>
    PlaceBrand
for
    HList![Hd, ..Tl]
{}

impl<Hd : PlaceBrand>
    PlaceBrand
for
    HList![Hd]
{}