use enroot::{brand::BrandedPlace, indwell::GhostInitProof};
use ::enroot::prelude::*;

fn f() -> Result<i32, ()> {
    Ok(42)
}

fn exploit<Place: IsBrandedPlace>(mut out_ref: OutRef<'_, String, Place>) {
    let soul = out_ref.r().write(String::from("hey")).into_ghost_init();
    let it = out_ref.r().write(String::from("oh no"));
    drop(it);
    // BAD BAD BAD
    let it = soul.indwell(out_ref);
}

impl Foo<Bar<String>> {
    pub fn new_simple<SelfPlace : IsBrandedPlace>(
        s: String,
        out: OutRef<'_, Self, SelfPlace>
    ) -> Result<OwnRef<'_, Self, SelfPlace>, ()>
    {
        let mut out = Self::attach_field_places(out);

        let mut out_fields = out.r();
        let s = {
            let mut out_s = Bar::attach_field_places(out_fields.project().s);
            let out_s_fields = out_s.project();
            BarOwnRefs(
                out_s_fields.0.write(s),
                out_s_fields.1.write(f()?),
            ).into_ghost_init().indwell(out_s)
        }.into_ghost_init();
        let mut out_fields = out.project();
        let x = out_fields.x.write(42);
        out_fields.s.r().write(Bar("oh no".into(), f()?));
        let s = s.indwell(out_fields.s);
        let owned_fields = FooOwnRefs { s, x };

        Ok(owned_fields.into_ghost_init().indwell(out))
    }

    pub fn new<SelfPlace : IsBrandedPlace>(
        s: String,
    ) -> impl InPlaceTryCtor<Self, SelfPlace, Error = ()>
    {
        move |out| Self::new_simple(s, out)
    }
}

pub fn demo_3() -> Result<(), ()> {
    let _b: Box<Foo<Bar<String>>> = box_emplace!(Foo::new("heh".into()))?;
    stack_emplace!(let _x: &own Foo<Bar<String>> = |out| Foo::new_simple("heh".into(), out));
    stack_emplace!(let _x: &own Foo<Bar<String>> = Foo::new("heh".into()));
    Ok(())
}

pub
trait InPlaceCtor<T : ?Sized, TPlace : IsBrandedPlace>
:
    FnOnce(OutRef<'_, T, TPlace>) -> OwnRef<'_, T, TPlace>
{}

impl<
    T : ?Sized,
    TPlace : IsBrandedPlace,
    Self_,
>
    InPlaceCtor<T, TPlace>
for
    Self_
where
    Self : FnOnce(OutRef<'_, T, TPlace>) -> OwnRef<'_, T, TPlace>,
{}

pub
trait InPlaceTryCtor<T : ?Sized, TPlace : IsBrandedPlace>
:
    FnOnce(OutRef<'_, T, TPlace>) -> Result<OwnRef<'_, T, TPlace>, Self::Error>
{
    type Error;

    fn try_in_place_init(
        self: Self,
        out_ref: OutRef<'_, T, TPlace>,
    ) -> Result<OwnRef<'_, T, TPlace>, Self::Error>
    where
        Self : Sized,
    {
        self(out_ref)
    }
}

impl<
    T : ?Sized,
    TPlace : IsBrandedPlace,
    Self_,
    Error,
>
    InPlaceTryCtor<T, TPlace>
for
    Self_
where
    Self : FnOnce(OutRef<'_, T, TPlace>) -> Result<OwnRef<'_, T, TPlace>, Error>,
{
    type Error = Error;
}

macro_rules! box_emplace {( $try_ctor:expr $(,)? ) => (|| -> _ {
    branded_uninit!(let uninit);
    let mut b: Box<BrandedUninit<_, BrandedPlace<'_, _>>> = Box::new(uninit);
    let out_uninit: OutRef<'_, _, BrandedPlace<'_, _>> = (&mut *b).as_out_ref();
    let own_uninit: OwnRef<'_, _, BrandedPlace<'_, _>> =
        InPlaceTryCtor::<_, BrandedPlace<'_, _>>::try_in_place_init(
            $try_ctor,
            out_uninit,
        )?
    ;
    let init_token: GhostInitProof<BrandedPlace<'_, _>> = own_uninit.into_ghost_init();
    Ok(init_token.indwell(b))
}())}
pub(crate) use box_emplace;

macro_rules! stack_emplace {(
    let $(mut $(@if $mut:tt)?)? $binding:ident : &own $T:ty = $try_ctor:expr
) => (
    branded_uninit!(let uninit_in_stack);
    let $($(if $mut)? mut)? $binding: OwnRef<'_, $T> = {
        let it =
            InPlaceTryCtor::<_, BrandedPlace<'_, _>>::try_in_place_init(
                $try_ctor,
                uninit_in_stack.as_out_ref(),
            )?
        ;
        it  .into_ghost_init() // .deref()
            .indwell(&mut uninit_in_stack) // .with()
            .unbrand()
        }
    ;
)}
pub(crate) use stack_emplace;

pub
fn demo_init2() -> Box<i32> {
    branded_uninit!(let x);
    let mut b = Box::new(x);
    let b: Box<i32> = b.as_out_ref().write(42_i32).into_ghost_init().indwell(b);
    b
}

use scoped::*;
mod scoped {
    #[derive(::enroot::prelude::OutRefProject)]
    pub
    struct Foo<S : Send>
    where
        S : Sync,
    {
        pub(super) s: S,
        pub(super) x: i32,
    }

    #[derive(::enroot::prelude::OutRefProject)]
    pub
    struct Bar<S>(pub(super) S, pub(super) i32);
}
