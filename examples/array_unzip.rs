use ::core::ops::ControlFlow;
use ::enroot::prelude::*;

pub
fn unzip<A, B, const N: usize>(array: [(A, B); N]) -> ([A; N], [B; N]) {
    branded_uninit!(let array_a: [A; N], array_b: [B; N]);
    let (array_a, array_b) = &mut (array_a, array_b);

    let out_as: OutRef<'_, [A], BrandedPlace<'_, _>> = unsize!(array_a.as_out());
    let out_bs: OutRef<'_, [B], BrandedPlace<'_, _>> = unsize!(array_b.as_out());

    let (full_a, full_b) = {
        let mut a_and_b_s = array.into_iter();
        let mut mb_a_and_b = a_and_b_s.next();
        let (mut iter_out_a, mut mb_out_a) = out_as.iter_out();
        let (mut iter_out_b, mut mb_out_b) = out_bs.iter_out();
        loop {
            match (mb_a_and_b, mb_out_a, mb_out_b) {
                (
                    Some((a, b)),
                    ControlFlow::Continue(out_a), // `: OutRef<'_, T, SliceElem<array_a>>`
                    ControlFlow::Continue(out_b),
                ) => {
                    mb_a_and_b = a_and_b_s.next();
                    mb_out_a = iter_out_a.next(
                        out_a.write(a), // `: OwnRef<'_, T, SliceElem<array_a>>`
                    ); // subsumes ownership of that `&own T` into `iter_out_a` (who would drop it).
                    mb_out_b = iter_out_b.next(out_b.write(b));
                    continue
                },
                (
                    None,
                    ControlFlow::Break(full_a), // OwnRef<'_, [A], array_a>
                    ControlFlow::Break(full_b),
                ) => {
                    ::core::mem::forget((iter_out_a, iter_out_b, a_and_b_s));
                    break (full_a, full_b);
                },
                _ => unsafe { ::core::hint::unreachable_unchecked() },
            }
        }
    };
    (
        // Requires matching `array_a` brand.
        indwell!(array_a <- full_a) // OwnRef<'_, [A; N], array_a>
            .deref_move() // [A; N]
        ,
        indwell!(array_b <- full_b).deref_move(),
    )
}

const M: usize = 10;

#[unsafe(no_mangle)]
pub fn unzip_(
    arr: [(String, i32); M],
) -> ([String; M], [i32; M])
{
    unzip(arr)
}

#[test]
fn foo() {
    let arr = ::core::array::from_fn(|i| {
        (format!("{i}"), i as _)
    });
    unzip_(arr);
}

#[unsafe(no_mangle)]
fn my_init() -> [i32; 100] {
    branded_uninit!(let array: [i32; 100]);
    let array = &mut array;
    indwell!(array <- unsize!(array.as_out() => [_]).emplace_each_with(|out| out.write(42)))
        .deref_move()
}

#[unsafe(no_mangle)]
fn my_init2() -> [i32; 100] {
    [42; 100]
}

fn unzip2<T, U, const N: usize>(arr: [(T, U); N]) -> ([T; N], [U; N]) {
    let arr = ::core::mem::ManuallyDrop::new(arr);
    (
        std::array::from_fn(|i| unsafe { (&raw const arr[i].0).read() }),
        std::array::from_fn(|i| unsafe { (&raw const arr[i].1).read() }),
    )
}

#[unsafe(no_mangle)]
pub fn unzip2_(
    arr: [(String, i32); M],
) -> ([String; M], [i32; M])
{
    unzip2(arr)
}
