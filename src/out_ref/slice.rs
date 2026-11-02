use super::*;

use crate::{
    brand::hlist::SliceElem,
};

impl<'r, T> OutRef<'r, [T], NoBrandedPlace> {
    pub
    fn len(&self) -> usize {
        self.raw.as_ptr().len()
    }

    pub
    fn get_out<'get>(
        self: &'get mut OutRef<'r, [T], NoBrandedPlace>,
        idx: usize,
    ) -> Option<
            OutRef<'get, T, NoBrandedPlace>,
        >
    {
        (idx < self.len()).then(|| {
            let wide_ptr: *mut [T] = self.raw.as_ptr();
            let thin_base: *mut T = wide_ptr as _;
            unsafe {
                OutRef::from_raw_branded_unchecked(ptr::NonNull::new(thin_base.add(idx)).unwrap())
            }
        })
    }

    pub
    fn split_at_out<'get>(
        self: &'get mut OutRef<'r, [T], NoBrandedPlace>,
        mid_idx: usize,
    ) -> Option<(
            OutRef<'get, [T], NoBrandedPlace>,
            OutRef<'get, [T], NoBrandedPlace>,
        )>
    {
        let right_len = self.len().checked_sub(mid_idx)?;
        let wide_ptr: *mut [T] = self.raw.as_ptr();
        let thin_base: *mut T = wide_ptr as _;
        let ret = unsafe {
            let (wide_left, wide_right) = (
                ::core::ptr::slice_from_raw_parts_mut(thin_base.add(0), mid_idx),
                ::core::ptr::slice_from_raw_parts_mut(thin_base.add(mid_idx), right_len),
            );
            (
                OutRef::from_raw(ptr::NonNull::new(wide_left).unwrap()),
                OutRef::from_raw(ptr::NonNull::new(wide_right).unwrap()),
            )
        };
        Some(ret)
    }
}

generate_scoped_slice_emplacers!(
    [/* no async await */],
    emplace_each_with,
    try_emplace_each_with,
    with_new_tainted,
    FnMut,
);
generate_scoped_slice_emplacers!(
    [async await],
    emplace_each_with_async,
    try_emplace_each_with_async,
    with_new_tainted_async,
    AsyncFnMut,
);
// where
macro_rules! generate_scoped_slice_emplacers {(
    [ $($async:ident $await:ident)? ],
    $emplace_each_with_mb_async:ident,
    $try_emplace_each_with_mb_async:ident,
    $with_new_tainted_mb_async:ident,
    $MbAsyncFnMut:ident $(,)?
) => (
    impl<'r, T, P : MaybeBrandedPlace> OutRef<'r, [T], P> {
        pub
        $($async)?
        fn $emplace_each_with_mb_async (
            self: OutRef<'r, [T], P>,
            mut emplace_one: impl for<'s, 'id> $MbAsyncFnMut(
                OutRef<'s, T, BrandedPlace<'id, SliceElem<P>>>,
            ) -> OwnRef<'s, T, BrandedPlace<'id, SliceElem<P>>>,
        ) -> OwnRef<'r, [T], P>
        where
            T : 'r,
        {
            let Ok::<_, Never>(own_ref) =
                self.$try_emplace_each_with_mb_async(
                    $($async)? move |out_ref| Ok( emplace_one(out_ref) $(.$await)? )
                )
                    $(.$await)?
            ;
            own_ref
        }

        pub
        $($async)?
        fn $try_emplace_each_with_mb_async<E>(
            self: OutRef<'r, [T], P>,
            mut try_emplace_one: impl for<'s, 'id> $MbAsyncFnMut(
                OutRef<'s, T, BrandedPlace<'id, SliceElem<P>>>,
            ) -> Result<
                    OwnRef<'s, T, BrandedPlace<'id, SliceElem<P>>>,
                    E,
                >
            ,
        ) -> Result<OwnRef<'r, [T], P>, E>
        where
            T : 'r,
        {
            struct State<'r, T> {
                uninit_slice_mut: &'r mut [MU<T>],
                /// Number of indwelled, and thus initialized, items (`0..num_indwelled`).
                num_indwelled: usize,
            }
            let mut guard = crate::utils::UnarmedDropGuard {
                state: State {
                    uninit_slice_mut: unsafe { self.as_uninit_mut() },
                    num_indwelled: 0,
                },
                drop: |State { uninit_slice_mut, num_indwelled }| unsafe {
                    drop(OwnRef::assume_owns_assumed_init(
                        &mut uninit_slice_mut[..num_indwelled]
                    ));
                },
            }.armed();

            let g = &mut *guard;
            for uninit_mut in &mut *g.uninit_slice_mut {
                BrandedPlace::$with_new_tainted_mb_async($($async)? |bruh| -> Result<_, E> {
                    let out_ref: OutRef<'_, T, BrandedPlace<'_, SliceElem<P>>> =
                        uninit_mut.as_out().branded(bruh)
                    ;
                    let own_ref: OwnRef<'_, T, BrandedPlace<'_, SliceElem<P>>> =
                        try_emplace_one(out_ref)
                            $(.$await)?
                            ?
                    ;
                    // "soul transfusion": transfer/move ownership of that `T` into `guard`
                    g.num_indwelled += {
                        ::core::mem::forget(own_ref);
                        1
                    };
                    Ok(())
                })
                    $(.$await)?
                    ?
                ;
            }
            debug_assert_eq!(g.num_indwelled, g.uninit_slice_mut.len());
            let State { uninit_slice_mut, num_indwelled: _ } = guard.disarm().state;

            Ok(unsafe {
                mem::transmute(uninit_slice_mut)
            })
        }
    }
)}
use generate_scoped_slice_emplacers;

impl<'r, T, FullSlicePlace : IsBrandedPlace> OutRef<'r, [T], FullSlicePlace> {
    pub
    fn iter_out(
        self: OutRef<'r, [T], FullSlicePlace>,
    ) -> impl Iterator<Item = OutRef<'r, T, SliceElem<FullSlicePlace>>>
            + ExactSizeIterator
            + DoubleEndedIterator
    where
        T : 'r,
    {
        unsafe { self.as_uninit_mut() }.iter_mut().map(|uninit_mut: &mut MU<T>| unsafe {
            OutRef::from_raw_branded_unchecked(ptr::NonNull::from(uninit_mut).cast())
        })
    }
}

pub
struct SoulCollector<'r, T, FullSlicePlace : IsBrandedPlace> {
    num_indwelled: usize,
    _phantom: PD<OwnRef<'r, T, SliceElem<FullSlicePlace>>>,
}

impl<'r, T, FullSlicePlace : IsBrandedPlace>
    Default
for
    SoulCollector<'r, T, FullSlicePlace>
{
    fn default() -> SoulCollector<'r, T, FullSlicePlace> {
        Self {
            num_indwelled: 0,
            _phantom: PD,
        }
    }
}

impl<'r, T, FullSlicePlace : IsBrandedPlace>
    Extend<OwnRef<'r, T, SliceElem<FullSlicePlace>>>
for
    SoulCollector<'r, T, FullSlicePlace>
{
    fn extend<Iterable>(
        self: &'_ mut SoulCollector<'r, T, FullSlicePlace>,
        iterable: Iterable,
    )
    where
        Iterable : IntoIterator<Item = OwnRef<'r, T, SliceElem<FullSlicePlace>>>,
    {
        // We use normal `+` rather than, _e.g._, `saturating_add` or whatnot, to mimic the
        // semantics of `Iterator::count()` in and of itself (wrapping or panicking depending on
        // `overflow-checks`).
        self.num_indwelled += iterable.into_iter().map(::core::mem::forget).count();
    }
}

impl<'r, T, FullSlicePlace : IsBrandedPlace>
    FromIterator<OwnRef<'r, T, SliceElem<FullSlicePlace>>>
for
    SoulCollector<'r, T, FullSlicePlace>
{
    fn from_iter<Iterable>(iterable: Iterable) -> SoulCollector<'r, T, FullSlicePlace>
    where
        Iterable : IntoIterator<Item = OwnRef<'r, T, SliceElem<FullSlicePlace>>>,
    {
        let mut ret = Self::default();
        ret.extend(iterable);
        ret
    }
}

impl<'r, T, FullSlicePlace : IsBrandedPlace> SoulCollector<'r, T, FullSlicePlace> {
    pub
    fn into_ghost_init(self) -> SoulCollector<'static, T, FullSlicePlace> {
        unsafe {
            ::core::mem::transmute(self)
        }
    }
}

impl<T, FullSlicePlace : IsBrandedPlace>
    Drop
for
    SoulCollector<'_, T, FullSlicePlace>
{
    fn drop(&mut self) {
        // static/compile-time-elidable branch.
        let _drop_thus_leak_does_matter @ true = ::core::mem::needs_drop::<T>() else {
            return
        };
        // Cheap runtime branch.
        let Some(leak_count) = ::core::num::NonZero::new(self.num_indwelled) else {
            // OK, nothing being leaked.
            return;
        };
        // Potentially more expensive-at-runtime branch.
        let _not_already_panicking @ true = ::std::thread::panicking().not() else {
            // "leak amplification" in the unwind path (otherwise we'd outright abort,
            // which seems unnecessary and thus excessive).
            return
        };
        let type_name = ::core::any::type_name::<T>();
        panic! {"\
            Missing `indwell!()` on a `SoulCollector`: this would leak memory! \
            ({leak_count} instance(s) of type `{type_name}`.) \
            If you are truly fine with this, work with a `ManuallyDrop<SoulCollector>` \
            to convey this (eg., you can `.extend()` a `default()`-constructed one).\
        "}
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub
struct MissingSouls {}

impl<T, FullSlicePlace : IsBrandedPlace> SoulCollector<'static, T, FullSlicePlace> {
    pub
    unsafe
    fn indwell<'r, FullSlicePlace_ : IsBrandedPlace, Husk : UninitSlice<FullSlicePlace_>>(
        self,
        husk: Husk,
    ) -> Result<
            Husk::OwnedInit,
            MissingSouls,
        >
    where
        FullSlicePlace : utils::diagnostics::Is<EqualTo = FullSlicePlace_>,
    {
        let this = MD::new(self);
        if this.num_indwelled < husk.indwell_count() {
            Err(MissingSouls {})
        } else {
            Ok(unsafe {
                indwell::GhostInitProof::branded_place_assume_init().indwell(husk)
            })
        }
    }
}

pub
unsafe
trait UninitSlice<FullSlicePlace : IsBrandedPlace>
:
    indwell::UninitPlace<FullSlicePlace> +
{
    fn indwell_count(&self) -> usize;
}

unsafe
impl<'r, T, FullSlicePlace : IsBrandedPlace>
    UninitSlice<FullSlicePlace>
for
    OutRef<'r, [T], FullSlicePlace>
{
    fn indwell_count(&self) -> usize {
        self.raw.len()
    }
}

unsafe
impl<'r, T, FullSlicePlace : IsBrandedPlace, const N: usize>
    UninitSlice<FullSlicePlace>
for
    OutRef<'r, [T; N], FullSlicePlace>
{
    fn indwell_count(&self) -> usize {
        N
    }
}

unsafe
impl<'r, T, FullSlicePlace : IsBrandedPlace, FullSlicePlace_ : IsBrandedPlace, const N: usize>
    UninitSlice<FullSlicePlace>
for
    &'r mut BrandedUninit<[T; N], FullSlicePlace_>
where
    FullSlicePlace : utils::diagnostics::Is<EqualTo = FullSlicePlace_>,
    FullSlicePlace_ : utils::diagnostics::Is<EqualTo = FullSlicePlace>,
{
    fn indwell_count(&self) -> usize {
        N
    }
}

unsafe
impl<T, FullSlicePlace : IsBrandedPlace, FullSlicePlace_ : IsBrandedPlace, const N: usize>
    UninitSlice<FullSlicePlace>
for
    BrandedUninit<[T; N], FullSlicePlace_>
where
    FullSlicePlace : utils::diagnostics::Is<EqualTo = FullSlicePlace_>,
    FullSlicePlace_ : utils::diagnostics::Is<EqualTo = FullSlicePlace>,
{
    fn indwell_count(&self) -> usize {
        N
    }
}

unsafe
impl<T, FullSlicePlace : IsBrandedPlace, FullSlicePlace_ : IsBrandedPlace, const N: usize>
    UninitSlice<FullSlicePlace>
for
    Box<BrandedUninit<[T; N], FullSlicePlace_>>
where
    FullSlicePlace : utils::diagnostics::Is<EqualTo = FullSlicePlace_>,
    FullSlicePlace_ : utils::diagnostics::Is<EqualTo = FullSlicePlace>,
{
    fn indwell_count(&self) -> usize {
        N
    }
}
