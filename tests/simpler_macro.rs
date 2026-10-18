use ::enroot::prelude::*;

pub
fn demo_init<'r, FooPlace : IsBrandedPlace>(
    mut out_foo: OutRef<'r, Foo, FooPlace>
) -> OwnRef<'r, Foo, FooPlace>
{
    // let mut out_foo = Foo::attach_field_places(out_foo);

    let out_fields = out_foo.project();
    let s = out_fields.s.write("heh".into());
    let x = out_fields.x.write(42);
    let y = out_fields.y.write(27);
    let owned_fields = FooOwnRefs { s, x, y };

    indwell!(out_foo <- owned_fields)
}

#[derive(::enroot::prelude::OutRefProject)]
pub
struct Foo {
    s: String,
    x: i32,
    y: i32,
}
