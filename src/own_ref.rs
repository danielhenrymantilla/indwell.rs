use super::*;

#[repr(transparent)]
pub
struct OwnRef<'r, T : ?Sized, B : MaybeBrand = Unbranded> {
    raw: ptr::NonNull<T>,
    _phantom: PD<(B, &'r ())>,
    _owns_t: PD<T>,
}

impl<'r, T : ?Sized, B : MaybeBrand> Drop for OwnRef<'r, T, B> {
    fn drop(&mut self) {
        unsafe { self.raw.as_ptr().drop_in_place() }
    }
}

impl<'r, T : ?Sized, B : MaybeBrand> OwnRef<'r, T, B> {
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

impl<'r, T : ?Sized, Brand : PlaceBrand> OwnRef<'r, T, Brand> {
    #[must_use = "this is leaking memory unless you call `.indwell()`"]
    pub
    fn into_ghost_init(self: OwnRef<'r, T, Brand>) -> GhostInitProof<Brand> {
        GhostInitProof(PD)
    }

    pub
    fn unbrand(self: OwnRef<'r, T, Brand>) -> OwnRef<'r, T> {
        unsafe {
            OwnRef::from_raw(self.into_raw())
        }
    }
}

pub
struct GhostInitProof<B : PlaceBrand>(
    PD<B>,
);

pub trait Is { type EqualTo : ?Sized; }
impl<T : ?Sized> Is for T { type EqualTo = T; }

impl<Brand : PlaceBrand> GhostInitProof<Brand> {
    pub
    fn indwell<'r, Brand2 : PlaceBrand, Husk : InitProvable<Brand2>>(
        self,
        r: Husk,
    ) -> Husk::OwnedInit
    where
        Brand2 : Is<EqualTo = Brand>,
    {
        fn indwell_helper<'r, Brand2 : PlaceBrand, Husk : InitProvable<Brand2>>(
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
trait InitProvable<B : PlaceBrand> {
    fn proven_init(
        _: GhostInitProof<B>,
        this: Self,
    ) -> Self::OwnedInit
    ;
    type OwnedInit;
}

impl<'r, T : ?Sized> OwnRef<'r, T, Unbranded> {
    #[doc(alias = "new")]
    pub
    unsafe
    fn assume_owns(r: &'r mut MD<T>) -> OwnRef<'r, T, Unbranded> {
        unsafe { Self::from_raw((&mut **r).into()) }
    }

    pub
    unsafe
    fn from_raw(raw: ptr::NonNull<T>) -> OwnRef<'r, T, Unbranded> {
        unsafe { Self::from_raw_branded_unchecked(raw) }
    }
}

impl<'r, T : ?Sized, B : MaybeBrand> ::core::ops::Deref for OwnRef<'r, T, B> {
    fn deref(&self) -> &T {
        unsafe { self.raw.as_ref() }
    }
    type Target = T;
}

impl<'r, T : ?Sized, B : MaybeBrand> ::core::ops::DerefMut for OwnRef<'r, T, B> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.raw.as_mut() }
    }
}
