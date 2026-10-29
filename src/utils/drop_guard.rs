use ::safe_manually_drop::{DropManually, SafeManuallyDrop};

pub(crate)
struct UnarmedDropGuard<State, F : FnOnce(State)> {
    pub(crate) state: State,
    pub(crate) drop: F,
}

pub(crate)
struct DropGuard<State, F : FnOnce(State)>(
    SafeManuallyDrop<UnarmedDropGuard<State, F>, Self>,
);

impl<State, F : FnOnce(State)>
    DropManually<UnarmedDropGuard<State, F>>
for
    DropGuard<State, F>
{
    fn drop_manually(UnarmedDropGuard { state, drop }: UnarmedDropGuard<State, F>) {
        drop(state)
    }
}

impl<State, F : FnOnce(State)> UnarmedDropGuard<State, F> {
    pub(crate)
    fn armed(self: UnarmedDropGuard<State, F>) -> DropGuard<State, F> {
        DropGuard(SafeManuallyDrop::new(self))
    }
}

impl<State, F : FnOnce(State)> DropGuard<State, F> {
    pub(crate)
    fn disarm(self: DropGuard<State, F>) -> UnarmedDropGuard<State, F> {
        self.0.into_inner_defusing_impl_Drop()
    }
}

impl<State, F : FnOnce(State)> ::core::ops::Deref for DropGuard<State, F> {
    fn deref(&self) -> &State {
        &self.0.state
    }
    type Target = State;
}

impl<State, F : FnOnce(State)> ::core::ops::DerefMut for DropGuard<State, F> {
    fn deref_mut(&mut self) -> &mut State {
        &mut self.0.state
    }
}
