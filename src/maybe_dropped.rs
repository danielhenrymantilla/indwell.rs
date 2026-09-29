use super::*;

pub
struct MaybeDropped<T: ?Sized>(MD<T>);

impl<T : Sized> MaybeDropped<T> {
    pub
    fn write(&mut self, value: T) -> OwnRef<'_, T> {
        self.0 = MD::new(value);
        unsafe {
            OwnRef::from_raw((&mut *self.0).into())
        }
    }
}
