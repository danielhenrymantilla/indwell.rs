use super::*;

#[repr(transparent)]
pub
struct OwnRef<'r, T : ?Sized, B : MaybeBrandedPlace = NoBrandedPlace> {
    raw: ptr::NonNull<T>,
    _phantom: PD<(B, &'r ())>,
    _owns_t: PD<T>,
}

impl<'r, T : ?Sized, B : MaybeBrandedPlace> Drop for OwnRef<'r, T, B> {
    fn drop(&mut self) {
        unsafe { self.raw.as_ptr().drop_in_place() }
    }
}

impl<'r, T : ?Sized, B : MaybeBrandedPlace> OwnRef<'r, T, B> {
    pub
    fn deref_move(self: OwnRef<'r, T, B>) -> (T, &'r mut MaybeDropped<T>)
    where
        T : Sized,
    {
        let raw = self.into_raw();
        unsafe {
            (
                raw.read(),
                raw.cast().as_mut(),
            )
        }
    }

    pub
    fn into_raw(self) -> ptr::NonNull<T> {
        MD::new(self).raw
    }

    pub
    unsafe
    fn from_raw_branded_unchecked(raw: ptr::NonNull<T>) -> OwnRef<'r, T, B> {
        Self { raw, _phantom: PD, _owns_t: PD }
    }

    pub
    fn leak(self) -> &'r mut MD<T> {
        unsafe { mem::transmute(self) }
    }
}

impl<'r, T : ?Sized, BrandedPlace : IsBrandedPlace> OwnRef<'r, T, BrandedPlace> {
    #[must_use = "this is leaking memory unless you call `.indwell()`"]
    pub
    fn into_ghost_init(self: OwnRef<'r, T, BrandedPlace>) -> GhostInitProof<BrandedPlace> {
        GhostInitProof(PD)
    }

    pub
    fn unbrand(self: OwnRef<'r, T, BrandedPlace>) -> OwnRef<'r, T> {
        unsafe {
            OwnRef::from_raw(self.into_raw())
        }
    }
}

pub
struct GhostInitProof<B : IsBrandedPlace>(
    PD<B>,
);

pub trait Is { type EqualTo : ?Sized; }
impl<T : ?Sized> Is for T { type EqualTo = T; }

impl<BrandedPlace : IsBrandedPlace> GhostInitProof<BrandedPlace> {
    pub
    fn indwell<'r, Brand2 : IsBrandedPlace, Husk : InitProvable<Brand2>>(
        self,
        r: Husk,
    ) -> Husk::OwnedInit
    where
        Brand2 : Is<EqualTo = BrandedPlace>,
    {
        fn indwell_helper<'r, Brand2 : IsBrandedPlace, Husk : InitProvable<Brand2>>(
            _: GhostInitProof<<Brand2 as Is>::EqualTo>,
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

impl<'r, T : ?Sized> OwnRef<'r, T, NoBrandedPlace> {
    #[doc(alias = "new")]
    pub
    unsafe
    fn assume_owns(r: &'r mut MD<T>) -> OwnRef<'r, T, NoBrandedPlace> {
        unsafe { Self::from_raw((&mut **r).into()) }
    }

    pub
    unsafe
    fn from_raw(raw: ptr::NonNull<T>) -> OwnRef<'r, T, NoBrandedPlace> {
        unsafe { Self::from_raw_branded_unchecked(raw) }
    }
}

impl<'r, T : ?Sized, B : MaybeBrandedPlace> ::core::ops::Deref for OwnRef<'r, T, B> {
    fn deref(&self) -> &T {
        unsafe { self.raw.as_ref() }
    }
    type Target = T;
}

impl<'r, T : ?Sized, B : MaybeBrandedPlace> ::core::ops::DerefMut for OwnRef<'r, T, B> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.raw.as_mut() }
    }
}
