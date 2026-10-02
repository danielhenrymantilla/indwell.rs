use crate::utils::diagnostics::{self, Is};

use super::*;

pub
struct GhostInitProof<B : IsBrandedPlace>(
    pub(crate) PD<fn() -> B>,
);

impl<BrandedPlace : IsBrandedPlace> GhostInitProof<BrandedPlace> {
    pub
    fn indwell<'r, BrandedPlace_ : IsBrandedPlace, Husk : InitProvable<BrandedPlace_>>(
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
        fn indwell_helper<'r, BrandedPlace_ : IsBrandedPlace, Husk : InitProvable<BrandedPlace_>>(
            _: GhostInitProof<<BrandedPlace_ as Is>::EqualTo>,
            it: Husk,
        ) -> Husk::OwnedInit
        {
            InitProvable::proven_init(GhostInitProof(PD), it)
        }

        indwell_helper(self, r)
    }
}

pub
trait InitProvable<B : IsBrandedPlace> {
    fn proven_init(
        _: GhostInitProof<B>,
        this: Self,
    ) -> Self::OwnedInit
    ;
    type OwnedInit;
}
