use crate::*;

use crate::utils::diagnostics::{self, Is};

pub
struct GhostInitProof<BrandedPlace : IsBrandedPlace>(
    PD<fn() -> BrandedPlace>,
);

impl<BrandedPlace : IsBrandedPlace> GhostInitProof<BrandedPlace> {
    #[doc(alias = "new")]
    pub
    unsafe
    fn assume_branded_place_init() -> Self {
        Self(PD)
    }
}

impl<BrandedPlace : IsBrandedPlace> GhostInitProof<BrandedPlace> {
    pub
    unsafe
    fn indwell<'r, BrandedPlace_ : IsBrandedPlace, Husk : UninitPlace<BrandedPlace_>>(
        self,
        r: Husk,
    ) -> Husk::OwnedInit
    where
        // this may seem weird, but it improves the diagnostics upon `BrandedPlace`
        // mismatch considerably (indeed, it would default to saying that
        // `InitProvable<ActualPlace>` is not implemented for `Thing<BadPlace>`, whereas
        // this way it does see the `InitProvable<_>` impl, and instead "just" complains
        // about `ActualPlace ≠ BadPlace`, which is exactly the desired diagnostics).
        BrandedPlace_ : diagnostics::Is<EqualTo = BrandedPlace>,
    {
        fn indwell_helper<'r, BrandedPlace_ : IsBrandedPlace, Husk : UninitPlace<BrandedPlace_>>(
            _: GhostInitProof<<BrandedPlace_ as Is>::EqualTo>,
            it: Husk,
        ) -> Husk::OwnedInit
        {
            unsafe {
                UninitPlace::proven_init(GhostInitProof(PD), it)
            }
        }

        indwell_helper(self, r)
    }
}

pub
unsafe
trait UninitPlace<BrandedPlace : IsBrandedPlace> : Sized {
    unsafe // for obvious reasons
    fn assume_init(self) -> Self::OwnedInit;

    type OwnedInit;

    unsafe // because of the discovered non-lifetime-protected drop-in-place exploit.
    fn proven_init(
        _: GhostInitProof<BrandedPlace>,
        this: Self,
    ) -> Self::OwnedInit
    {
        // Implementations are free to override this default impl, but it's a sane default
        // in exchange of ma(r)king the enscoping trait `unsafe`.
        unsafe {
            Self::assume_init(this)
        }
    }
}

#[doc(hidden)] /** Not part of the public API */ #[macro_export]
macro_rules! ඞindwell {(
    $uninit_place:ident <- $own_ref:expr $(,)?
) => (
    match $own_ref.into_ghost_init() {
        ghost_init => unsafe { ghost_init.indwell($uninit_place) },
    }
)}
pub use ඞindwell as indwell;
