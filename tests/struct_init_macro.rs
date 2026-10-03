use ::enroot::prelude::*;

pub
fn demo_init<'r, FooPlace : IsBrandedPlace>(
    out_foo: OutRef<'r, Foo<Bar<String>>, FooPlace>
) -> OwnRef<'r, Foo<Bar<String>>, FooPlace>
{
    let mut out_foo = Foo::attach_field_places(out_foo);

    let out_fields = out_foo.project();
    let s = {
        let mut s = Bar::attach_field_places(out_fields.s);
        let out_s = s.project();
        BarOwnRefs(
            out_s.0.write("heh".into()),
            out_s.1.write(27),
        ).into_ghost_init().indwell(s)
    };
    let x = out_fields.x.write(42);
    let owned_fields = FooOwnRefs { s, x };

    owned_fields.into_ghost_init().indwell(out_foo)
}

pub
fn demo_init2() {
    branded_uninit!(let x);
    let mut b = Box::new(x);
    let _b: Box<i32> = b.as_out_ref().write(42_i32).into_ghost_init().indwell(b);
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
