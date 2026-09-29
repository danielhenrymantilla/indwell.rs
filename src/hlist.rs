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
pub use ඞHList as HList;

impl<Hd : MaybeBrand, Tl : MaybeBrand>
    crate::brand::MaybeBrandSeal
for
    HList![Hd, ..Tl]
{}

impl<Hd : MaybeBrand>
    crate::brand::MaybeBrandSeal
for
    HList![Hd]
{}