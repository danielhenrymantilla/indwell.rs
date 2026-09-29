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

impl<'r, 'brand, T : ?Sized, Diagnostics> OwnRef<'r, T, Brand<'brand, Diagnostics>> {
    #[must_use = "this is a leaky pattern until you feed this into "]
    pub
    fn into_ghost_init(self: OwnRef<'r, T, Brand<'brand, Diagnostics>>) -> GhostInitProof<'brand, Diagnostics> {
        GhostInitProof(PD)
    }

    pub
    fn unbrand(self: OwnRef<'r, T, Brand<'brand, Diagnostics>>) -> OwnRef<'r, T> {
        unsafe {
            OwnRef::from_raw(self.into_raw())
        }
    }
}

impl<'brand, Diagnostics> Clone for GhostInitProof<'brand, Diagnostics>
where 
    'brand : 'static,
{
    fn clone(&self) -> Self {
        const { unreachable!() }
    }
}
impl<'brand, Diagnostics> Copy for GhostInitProof<'brand, Diagnostics>
where 
    'brand : 'static,
{}

pub 
struct GhostInitProof<'brand, Diagnostics>(
    PD<Brand<'brand, Diagnostics>>,
);

pub trait Is { type Equal : ?Sized; }
impl<T : ?Sized> Is for T { type Equal = T; }

impl<'brand, Diagnostics> GhostInitProof<'brand, Diagnostics> {
    pub
    fn indwell<'r, T, Diagnostics2>(self, r: impl InitProvable<'r, 'brand, T, Diagnostics2>) -> OwnRef<'r, T, Brand<'brand, Diagnostics>>
    where 
        Diagnostics2 : Is<Equal = Diagnostics>,
        // Diagnostics : Is<Equal = Diagnostics2>,
    {
        fn cast<'r, 'brand, T, D>(it: OwnRef<'r, T, Brand<'brand, D>>)
          -> OwnRef<'r, T, Brand<'brand, <D as Is>::Equal>>
        {
            it
        }

        cast(InitProvable::proven_init(GhostInitProof(PD), r))
    }
}

pub
trait InitProvable<'r, 'brand, T, Diagnostics> {
    fn proven_init(_: GhostInitProof<'brand, Diagnostics>, this: Self) -> OwnRef<'r, T, Brand<'brand, Diagnostics>>;
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
