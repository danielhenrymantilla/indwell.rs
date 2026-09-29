use ::enroot::out_ref::uninit;
use ::enroot::prelude::*;

#[test]
fn indwell() {
    // brand!(as let root);
    // let (uninit_value, evil, ..) = root.shatter();
    // let mut uninit_value = uninit::<String, _>(uninit_value);
    // let mut evil = uninit::<String, _>(evil);
    branded_places!(let uninit_value: String, _evil: String);
    let mut boxed;
    let init: OwnRef<'_, String>;
    {
        let out: OutRef<'_, _, Brand<'_, _>> = uninit_value.as_out_ref();
        let own: OwnRef<'_, _, Brand<'_, _>> = out.write("some value".into());
        // init = own.into_ghost_init().indwell(&mut _evil).unbrand();
        init = own.into_ghost_init().indwell(if ::rand::random() {
            boxed = Box::new(uninit_value);
            &mut boxed
        } else {
            &mut uninit_value
        }).unbrand();
    }
    assert_eq!(dbg!(&*init), "some value");
}

struct Foo {
    s: String,
    x: i32,
}



struct OutFooFields<'s, 'brand_s, 'brand_x, DS, DX> {
    s: OutRef<'s, String, Brand<'brand_s, DS>>,
    x: OutRef<'s, String, Brand<'brand_x, DX>>,
}

impl<'s, 'brand, 'brand_s, 'brand_x, T, Diagnostics, DS, DX>
    ::enroot::out_ref::Project<'s, (
        Brand<'brand, Diagnostics>,
        Brand<'brand_s, DS>,
        Brand<'brand_x, DX>,
    )>
for
    Foo
{
    type Projection = OutFooFields<'s, 'brand_s, 'brand_x, DS, DX>;

    fn project_impl(
        out_ref: OutRef<'s, T, (
            Brand<'brand, Diagnostics>,
            Brand<'brand_s, DS>,
            Brand<'brand_x, DX>,
        )>
    ) -> Self::Projection {
        let raw = out_ref.r().into_raw();
        todo!()
    }
}