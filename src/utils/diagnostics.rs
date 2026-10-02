pub trait Is { type EqualTo : ?Sized; }

impl<T : ?Sized> Is for T { type EqualTo = T; }
