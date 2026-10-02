#![allow(unused)]
#![warn(unused_must_use)]

use ::enroot::prelude::*;

fn demo_init<'r, FooPlace : IsBrandedPlace>(
    out_foo: OutRef<'r, Foo<String>, FooPlace>
) -> OwnRef<'r, Foo<String>, FooPlace>
{
    let mut out_foo = Foo::attach_field_places(out_foo);

    let out_fields = out_foo.project();
    let s = out_fields.s.write("heh".into());
    let x = out_fields.x.write(42);
    let owned_fields = FooOwnRefs { s, x };

    owned_fields.into_ghost_init().indwell(out_foo)
}

fn demo_init2() {
    branded_uninit!(let x);
    let mut b = Box::new(x);
    let b = b.as_out_ref().write(42_i32).into_ghost_init().indwell(b);
}

#[derive(OutRefProject)]
struct Foo<S : Send>
where
    S : Sync,
{
    s: S,
    x: i32,
}
