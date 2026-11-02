#![feature(maybe_uninit_array_assume_init)]

use ::enroot::prelude::*;

pub
fn unzip<A, B, const N: usize>(array: [(A, B); N]) -> ([A; N], [B; N]) {
    branded_uninit!(let array_a: [A; N], array_b: [B; N]);

    let out_as: OutRef<'_, [A], BrandedPlace<'_, _>> = unsize!(array_a.as_out());
    let out_bs: OutRef<'_, [B], BrandedPlace<'_, _>> = unsize!(array_b.as_out());

    let (souls_a, souls_b) =
        array
            .into_iter()
            .zip(Iterator::zip(
                out_as.iter_out(),
                out_bs.iter_out(),
            ))
            .map(|((a, b), (out_a, out_b))| (
                out_a.write(a),
                out_b.write(b),
            ))
            .collect::<(SoulCollector<_, _>, SoulCollector<_, _>)>()
    ;
    (
        // Requires matching `array_a` brand.
        indwell!(array_a <- souls_a).expect("enough souls"),
        indwell!(array_b <- souls_b).expect("enough souls"),
    )
}

const M: usize = 2;

#[unsafe(no_mangle)]
pub
fn unzip_(
    arr: [(String, i32); M],
) -> ([String; M], [i32; M])
{
    unzip(arr)
}

#[test]
fn sanity_check() {
    let arr = ::core::array::from_fn(|i| {
        (format!("{i}"), i as _)
    });
    unzip_(arr);
}

fn unzip2<T, U, const N: usize>(array: [(T, U); N]) -> ([T; N], [U; N]) {
    use ::core::mem::MaybeUninit as MU;
    let (mut a, mut b) = ([const { MU::uninit() }; N], [const { MU::uninit() }; N]);
    let n =
        array
            .into_iter()
            .zip(
                Iterator::zip(
                    a.iter_mut(),
                    b.iter_mut(),
                )
            )
            .map(|((a, b), (out_a, out_b))| {
                out_a.write(a);
                out_b.write(b);
            })
            .count()
    ;
    assert_eq!(n, N);
    unsafe {
        (
            MU::array_assume_init(a),
            MU::array_assume_init(b),
        )
    }
}

#[unsafe(no_mangle)]
pub
fn unzip2_(
    arr: [(String, i32); M],
) -> ([String; M], [i32; M])
{
    unzip2(arr)
}

fn unzip3<T, U, const N: usize>(arr: [(T, U); N]) -> ([T; N], [U; N]) {
    let arr = ::core::mem::ManuallyDrop::new(arr);
    (
        std::array::from_fn(|i| unsafe { (&raw const arr[i].0).read() }),
        std::array::from_fn(|i| unsafe { (&raw const arr[i].1).read() }),
    )
}

#[unsafe(no_mangle)]
pub
fn unzip3_(
    arr: [(String, i32); M],
) -> ([String; M], [i32; M])
{
    unzip3(arr)
}
