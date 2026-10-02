#![allow(unused)]
#![warn(unused_must_use)]

use ::core::ptr;
use ::enroot::prelude::*;

fn demo_init<'r, Brand : PlaceBrand>(
    out_foo: OutRef<'r, Foo, Brand>
) -> OwnRef<'r, Foo, Brand>
{
    let mut out_foo = Foo::append_field_init_brands(out_foo);

    let out_fields = out_foo.project();
    let s = out_fields.s.write("heh".into());
    let x = out_fields.x.write(42);
    let owned_fields = FooOwnRefs { s, x };

    owned_fields.into_ghost_init().indwell(out_foo)
}

struct Foo {
    s: String,
    x: i32,
}

#[allow(nonstandard_style)]
const _: () = {
    enum s {}
    enum x {}

    impl Foo {
        fn append_field_init_brands<'r, FooPlace : PlaceBrand>(
            out_ref: OutRef<'r, Foo, FooPlace>
        ) -> OutRef<'r, Foo, HList![
            FooPlace,
            Brand<'static, impl use<FooPlace> + ::enroot::brand::Place<x>>,
            Brand<'static, impl use<FooPlace> + ::enroot::brand::Place<s>>,
        ]>
        {
            unsafe {
                OutRef::<_, HList![
                    FooPlace,
                    Brand<'_, ()>,
                    Brand<'_, ()>,
                ]>::from_raw_branded_unchecked(
                    OutRef::into_raw(out_ref)
                )
            }
        }
    }
};

struct FooOutRefs<'s, PlaceS : PlaceBrand, PlaceX : PlaceBrand> {
    s: OutRef<'s, String, PlaceS>,
    x: OutRef<'s, i32, PlaceX>,
}

struct FooOwnRefs<'s, PlaceS : PlaceBrand, PlaceX : PlaceBrand> {
    s: OwnRef<'s, String, PlaceS>,
    x: OwnRef<'s, i32, PlaceX>,
}

impl<'s, PlaceS : PlaceBrand, PlaceX : PlaceBrand> FooOwnRefs<'s, PlaceS, PlaceX> {
    #[must_use = "this is leaking memory unless you call `.indwell()`"]
    fn into_ghost_init(self) -> FooGhostInit<PlaceS, PlaceX> {
        let Self { s, x } = self;
        FooGhostInit {
            s: s.into_ghost_init(),
            x: x.into_ghost_init(),
        }
    }
}

struct FooGhostInit<PlaceS : PlaceBrand, PlaceX : PlaceBrand> {
    s: ::enroot::own_ref::GhostInitProof<PlaceS>,
    x: ::enroot::own_ref::GhostInitProof<PlaceX>,
}

impl<PlaceS : PlaceBrand, PlaceX : PlaceBrand> FooGhostInit<PlaceS, PlaceX> {
    fn indwell<'s, T, Brand : PlaceBrand>(
        self: FooGhostInit<PlaceS, PlaceX>,
        out_ref: OutRef<'s, T, HList![Brand, PlaceS, PlaceX]>,
    ) -> OwnRef<'s, T, Brand>
    {
        unsafe {
            OwnRef::from_raw_branded_unchecked(
                OutRef::into_raw(out_ref)
            )
        }
    }
}

impl<Brand : PlaceBrand, PlaceS : PlaceBrand, PlaceX : PlaceBrand>
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
