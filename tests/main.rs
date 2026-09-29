use std::ptr;

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

struct OutFooFields<'s, 'brand_s, 'brand_x, DS, DX> {
    s: OutRef<'s, String, Brand<'brand_s, DS>>,
    x: OutRef<'s, i32, Brand<'brand_x, DX>>,
}

impl<'s, 'brand, 'brand_s, 'brand_x, Diagnostics, DS, DX>
    ::enroot::out_ref::Project<'s, HList![
        Brand<'brand, Diagnostics>,
        Brand<'brand_s, DS>,
        Brand<'brand_x, DX>,
    ]>
for
    Foo
{
    type Projection = OutFooFields<'s, 'brand_s, 'brand_x, DS, DX>;

    fn project_impl(
        out_ref: OutRef<'s, Foo, HList![
            Brand<'brand, Diagnostics>,
            Brand<'brand_s, DS>,
            Brand<'brand_x, DX>,
        ]>
    ) -> Self::Projection {
        let raw = out_ref.into_raw();
        OutFooFields {
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
    
    type HeadBrand = Brand<'brand, Diagnostics>;
    
    type GhostInits = 
    
    fn indwell(
        _: Self::GhostInits,
        enroot::out_ref: OutRef<'s, Self, HList![
            Brand<'brand, Diagnostics>,
            Brand<'brand_s, DS>,
            Brand<'brand_x, DX>,
        ]>,
    ) -> OwnRef<'s, Self, Self::HeadBrand> {
        todo!()
    }
}