use super::*;

pub
struct MaybeDropped<T, P : MaybeBrandedPlace = NoBrandedPlace>(
    BrandedUninit<T, P>,
);

impl<T : Sized, P : MaybeBrandedPlace> MaybeDropped<T, P> {
    pub
    fn write(&mut self, value: T) -> OwnRef<'_, T, P> {
        self.0.as_out().write(value)
    }
}
