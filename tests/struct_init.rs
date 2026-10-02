use ::core::ptr;
use ::enroot::{
    prelude::*,
    brand::BrandedPlace,
};

pub
fn demo_init<'r, FooPlace : IsBrandedPlace>(
    out_foo: OutRef<'r, Foo, FooPlace>
) -> OwnRef<'r, Foo, FooPlace>
{
    let mut out_foo = Foo::attach_field_places(out_foo);

    let out_fields = out_foo.project();
    let s = out_fields.s.write("heh".into());
    let x = out_fields.x.write(42);
    let owned_fields = FooOwnRefs { s, x };

    owned_fields.into_ghost_init().indwell(out_foo)
}

pub
struct Foo {
    s: String,
    x: i32,
}

#[allow(nonstandard_style)]
const _: () = {
    enum s {}
    enum x {}

    impl Foo {
        fn attach_field_places<'r, FooPlace : IsBrandedPlace>(
            out_ref: OutRef<'r, Foo, FooPlace>
        ) -> OutRef<'r, Foo, HList![
            FooPlace,
            BrandedPlace<'static, impl use<FooPlace> + ::enroot::brand::Place<x>>,
            BrandedPlace<'static, impl use<FooPlace> + ::enroot::brand::Place<s>>,
        ]>
        {
            unsafe {
                OutRef::<_, HList![
                    FooPlace,
                    BrandedPlace<'_, ()>,
                    BrandedPlace<'_, ()>,
                ]>::from_raw_branded_unchecked(
                    OutRef::into_raw(out_ref)
                )
            }
        }
    }
};

pub
struct FooOutRefs<'s, PlaceS : IsBrandedPlace, PlaceX : IsBrandedPlace> {
    s: OutRef<'s, String, PlaceS>,
    x: OutRef<'s, i32, PlaceX>,
}

pub
struct FooOwnRefs<'s, PlaceS : IsBrandedPlace, PlaceX : IsBrandedPlace> {
    s: OwnRef<'s, String, PlaceS>,
    x: OwnRef<'s, i32, PlaceX>,
}

impl<'s, PlaceS : IsBrandedPlace, PlaceX : IsBrandedPlace> FooOwnRefs<'s, PlaceS, PlaceX> {
    #[must_use = "this is leaking memory unless you call `.indwell()`"]
    pub
    fn into_ghost_init(self) -> FooGhostInit<PlaceS, PlaceX> {
        let Self { s, x } = self;
        FooGhostInit {
            s: s.into_ghost_init(),
            x: x.into_ghost_init(),
        }
    }
}

#[allow(unused)]
pub
struct FooGhostInit<PlaceS : IsBrandedPlace, PlaceX : IsBrandedPlace> {
    s: ::enroot::indwell::GhostInitProof<PlaceS>,
    x: ::enroot::indwell::GhostInitProof<PlaceX>,
}

impl<PlaceS : IsBrandedPlace, PlaceX : IsBrandedPlace> FooGhostInit<PlaceS, PlaceX> {
    fn indwell<'s, T, BrandedPlace : IsBrandedPlace>(
        self: FooGhostInit<PlaceS, PlaceX>,
        out_ref: OutRef<'s, T, HList![BrandedPlace, PlaceS, PlaceX]>,
    ) -> OwnRef<'s, T, BrandedPlace>
    {
        unsafe {
            OwnRef::from_raw_branded_unchecked(
                OutRef::into_raw(out_ref)
            )
        }
    }
}

impl<Brand : IsBrandedPlace, PlaceS : IsBrandedPlace, PlaceX : IsBrandedPlace>
    ::enroot::out_ref::OutRefProject<HList![Brand, PlaceS, PlaceX]>
for
    Foo
{
    type Projection<'s> = FooOutRefs<'s, PlaceS, PlaceX>;

    fn project_impl<'s>(
        out_ref: OutRef<'s, Foo, HList![Brand, PlaceS, PlaceX]>
    ) -> Self::Projection<'s>
    {
        let raw = out_ref.into_raw();
        FooOutRefs {
            s: unsafe {
                OutRef::from_raw_branded_unchecked(
                    ptr::NonNull::new(&raw mut (*raw.as_ptr()).s).unwrap(),
                )
            },
            x: unsafe {
                OutRef::from_raw_branded_unchecked(
                    ptr::NonNull::new(&raw mut (*raw.as_ptr()).x).unwrap(),
                )
            },
        }
    }
}
