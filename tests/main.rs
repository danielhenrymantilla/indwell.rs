use std::ptr;

use enroot::{out_ref, own_ref::GhostInitProof};
// use ::enroot::out_ref::uninit;
use ::enroot::prelude::*;

#[test]
fn indwell() {
    // brand!(as let root);
    // let (uninit_value, evil, ..) = root.shatter();
    // let mut uninit_value = uninit::<String, _>(uninit_value);
    // let mut evil = uninit::<String, _>(evil);
    branded_places!(let uninit_value: String, _evil: String);
    // let mut boxed;
    let init: OwnRef<'_, String>;
    {
        let out: OutRef<'_, _, Brand<'_, _>> = uninit_value.as_out_ref();
        let own: OwnRef<'_, _, Brand<'_, _>> = out.write("some value".into());
        // init = own.into_ghost_init().indwell(&mut _evil).unbrand();
        init = own.into_ghost_init().indwell(&mut uninit_value).unbrand();
    }
    assert_eq!(dbg!(&*init), "some value");
}

struct Foo {
    s: String,
    x: i32,
}

fn demo<'r, 'brand, Diagnostics>(
    out_foo: OutRef<'r, Foo, Brand<'brand, Diagnostics>>
) -> OwnRef<'r, Foo, Brand<'brand, Diagnostics>>
{
    let mut out_foo = Foo::field_initer(out_foo);

    let out_fields = out_foo.project();
    let own_fields = FooProjectedOwnRefs {
        s: out_fields.s.write("heh".into()),
        x: out_fields.x.write(42),
    };
    
    own_fields.into_ghost_init().indwell(out_foo)
}

#[allow(nonstandard_style)]
const _: () = {
    trait s {} impl s for () {}
    trait x {} impl x for () {}

    impl Foo {
        fn field_initer<'r, 'brand, Diagnostics>(
            out_ref: OutRef<'r, Self, Brand<'brand, Diagnostics>>
        ) -> OutRef<'r, Self, HList![
            Brand<'brand, Diagnostics>,
            Brand<'brand, impl s>,
            Brand<'brand, impl x>,
        ]>
        {
            unsafe {
                OutRef::<_, HList![
                    Brand<'brand, Diagnostics>,
                    Brand<'brand, ()>,
                    Brand<'brand, ()>,
                ]>::from_raw_branded_unchecked(
                    OutRef::into_raw(out_ref)
                )
            }
        }
    }
};

struct FooProjectedOutRefs<'s, 'brand_s, 'brand_x, DS, DX> {
    s: OutRef<'s, String, Brand<'brand_s, DS>>,
    x: OutRef<'s, i32, Brand<'brand_x, DX>>,
}

struct FooProjectedOwnRefs<'s, 'brand_s, 'brand_x, DS, DX> {
    s: OwnRef<'s, String, Brand<'brand_s, DS>>,
    x: OwnRef<'s, i32, Brand<'brand_x, DX>>,
}

impl<'s, 'brand_s, 'brand_x, DS, DX> FooProjectedOwnRefs<'s, 'brand_s, 'brand_x, DS, DX> {
    fn into_ghost_init(self) -> FooGhostInit<'brand_s, 'brand_x, DS, DX> {
        let Self { s, x } = self;
        let s = s.into_ghost_init();
        let x = x.into_ghost_init();
        FooGhostInit { s, x }
    }
}

struct FooGhostInit<'brand_s, 'brand_x, DS, DX> {
    s: GhostInitProof<'brand_s, DS>,
    x: GhostInitProof<'brand_x, DX>,
}

impl<'s, 'brand_s, 'brand_x, DS, DX> FooGhostInit<'brand_s, 'brand_x, DS, DX> {
    fn indwell<'r, 'brand, T, Diagnostics>(
        self: FooGhostInit<'brand_s, 'brand_x, DS, DX>,
        out_ref: OutRef<'r, T, HList![
            Brand<'brand, Diagnostics>,
            Brand<'brand_s, DS>,
            Brand<'brand_x, DX>,
        ]>,
    ) -> OwnRef<'r, T, Brand<'brand, Diagnostics>> {
        unsafe {
            OwnRef::from_raw_branded_unchecked(
                OutRef::into_raw(out_ref)
            )
        }
    }
}

impl<'brand, 'brand_s, 'brand_x, Diagnostics, DS, DX>
    ::enroot::out_ref::Project<HList![
        Brand<'brand, Diagnostics>,
        Brand<'brand_s, DS>,
        Brand<'brand_x, DX>,
    ]>
for
    Foo
{
    type Projection<'s> = FooProjectedOutRefs<'s, 'brand_s, 'brand_x, DS, DX>;

    fn project_impl<'s>(
        out_ref: OutRef<'s, Foo, HList![
            Brand<'brand, Diagnostics>,
            Brand<'brand_s, DS>,
            Brand<'brand_x, DX>,
        ]>
    ) -> Self::Projection<'s> {
        let raw = out_ref.into_raw();
        FooProjectedOutRefs {
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
    
    // type HeadBrand = Brand<'brand, Diagnostics>;
    
    // type GhostInits = 
    
    // fn indwell(
    //     _: Self::GhostInits,
    //     enroot::out_ref: OutRef<'s, Self, HList![
    //         Brand<'brand, Diagnostics>,
    //         Brand<'brand_s, DS>,
    //         Brand<'brand_x, DX>,
    //     ]>,
    // ) -> OwnRef<'s, Self, Self::HeadBrand> {
    //     todo!()
    // }
}