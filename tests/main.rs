use ::enroot::prelude::*;

#[test]
fn indwell() {
    // brand!(as let root);
    // let (uninit_value, evil, ..) = root.shatter();
    // let mut uninit_value = uninit::<String, _>(uninit_value);
    // let mut evil = uninit::<String, _>(evil);
    branded_uninit!(let uninit_value: String, _evil: String);
    // let mut boxed;
    let init: OwnRef<'_, String>;
    {
        let out: OutRef<'_, _, BrandedPlace<'_, _>> = uninit_value.as_out_ref();
        let own: OwnRef<'_, _, BrandedPlace<'_, _>> = out.write("some value".into());
        // init = own.into_ghost_init().indwell(&mut _evil).unbrand();
        init = own.into_ghost_init().indwell(&mut uninit_value).unbrand();
    }
    assert_eq!(dbg!(&*init), "some value");
}
