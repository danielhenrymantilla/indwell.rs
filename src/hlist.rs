#[rustfmt::skip]
/// ```rust ,ignore
/// HList![];
/// ```
macro_rules! HList {
    [] => { () };

    [ $Hd:ty, ..$Rest:ty ] => {
        ($Hd, $Rest)
    };

    [ $Hd:ty $(, $Rest:ty)* $(,)? ] => {
        (
            $Hd,
            $crate::HList[$($Rest),*]
        )
    };

}

type X<Hd, Tl> = HList![Hd, ..Tl];
