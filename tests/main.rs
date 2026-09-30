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

fn demo<'r, Brand : PlaceBrand>(
    out_foo: OutRef<'r, Foo, Brand>
) -> OwnRef<'r, Foo, Brand>
{
    let mut out_foo = Foo::field_init_base(out_foo);
    // branded_places!(let a: Foo, evil: Foo);
    // [
    //     Foo::field_init_base(a.as_out_ref()),
    //     Foo::field_init_base(evil.as_out_ref()),
    // ];

    let out_fields = out_foo.project();
    let own_fields = FooOwnRefs {
        s: out_fields.s.write("heh".into()),
        x: out_fields.x.write(42),
    };
    // [
    //     own_fields.s.into_ghost_init(),
    //     own_fields.x.into_ghost_init(),
    // ];
    
    own_fields.into_ghost_init().indwell(out_foo)
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
        fn field_init_base<'r, FooPlace : PlaceBrand>(
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

struct FooOutRefs<'s, BS : PlaceBrand, BX : PlaceBrand> {
    s: OutRef<'s, String, BS>,
    x: OutRef<'s, i32, BX>,
}

struct FooOwnRefs<'s, BS : PlaceBrand, BX : PlaceBrand> {
    s: OwnRef<'s, String, BS>,
    x: OwnRef<'s, i32, BX>,
}

impl<'s, BS : PlaceBrand, BX : PlaceBrand> FooOwnRefs<'s, BS, BX> {
    fn into_ghost_init(self) -> FooGhostInit<BS, BX> {
        let Self { s, x } = self;
        let s = s.into_ghost_init();
        let x = x.into_ghost_init();
        FooGhostInit { s, x }
    }
}

struct FooGhostInit<BS : PlaceBrand, BX : PlaceBrand> {
    s: GhostInitProof<BS>,
    x: GhostInitProof<BX>,
}

impl<BS : PlaceBrand, BX : PlaceBrand> FooGhostInit<BS, BX> {
    fn indwell<'s, T, Brand : PlaceBrand>(
        self: FooGhostInit<BS, BX>,
        out_ref: OutRef<'s, T, HList![Brand, BS, BX]>,
    ) -> OwnRef<'s, T, Brand> {
        unsafe {
            OwnRef::from_raw_branded_unchecked(
                OutRef::into_raw(out_ref)
            )
        }
    }
}

impl<Brand : PlaceBrand, BS : PlaceBrand, BX : PlaceBrand>
    ::enroot::out_ref::Project<HList![Brand, BS, BX]>
for
    Foo
{
    type Projection<'s> = FooOutRefs<'s, BS, BX>;

    fn project_impl<'s>(
        out_ref: OutRef<'s, Foo, HList![Brand, BS, BX]>
    ) -> Self::Projection<'s> {
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