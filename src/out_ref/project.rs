use ::never_say_never::Never;
use ::std::ops::ControlFlow;
use crate::{
    brand::hlist::SliceElem,
};

use super::*;

impl<'r, T : ?Sized, FieldPlaces : IsBrandedPlace>
    OutRef<'r, T, FieldPlaces>
{
    pub
    fn project<'s, Projection>(self: &'s mut OutRef<'r, T, FieldPlaces>)
      -> Projection
    where
        T : derive_marker::OutRefProject,
        T : OutRefProject<FieldPlaces, Projection<'s> = Projection>,
    {
        OutRefProject::project_impl(self.r())
    }
}

#[diagnostic::on_unimplemented(
    message = "Cannot `.project()` without having called `{Self}::attach_field_places()` first",
    label = "call `<{Self}>::attach_field_places()` on its receiver first, shadowing it with a `let`",
    note = "\
        In order for `.project()` + `out.each_field.write(…)` + `indwell!(out <- ...)` to be safe \
        and sound, it is necessary to:\n \
         1. attach extra `PlaceBrand`s for each one of the fields,\n \
         2. do the `.project()` on that, `.write(…)`-init each of the fields,\n \
         3. and finally `indwell!(out <- …)` where `out` is the version with the field placebrands \
            attached.\n\
        \n    \
        let mut out_foo = Foo::attach_field_places(out_foo);\n    \
        let out_fields = out_foo.project();\n    \
        let own_fields = FooOwnRefs {{\n        \
            bar: out_fields.bar.write(…),\n        \
            quux: out_fields.quux.write(…),\n    \
        }};\n    \
        let own_foo = indwell!(out_foo <- own_fields);\n\
    "
)]
pub
trait OutRefProject<BrandedPlaceAndFieldPlaces : IsBrandedPlace>
:
    // derive_marker::OutRefProject;
{
    type Projection<'s>;
    fn project_impl<'s>(
        out_ref: OutRef<'s, Self, BrandedPlaceAndFieldPlaces>,
    ) -> Self::Projection<'s>
    ;
}

#[doc(hidden)] /** Not part of the public API */ pub
mod derive_marker {
    #[diagnostic::on_unimplemented(
        message = "Missing `#[derive(OutRefProject)]` on `{Self}`",
        label = "add `#[derive(OutRefProject)]` on its receiver",
        // note = "From there, you will need "
    )]
    pub trait OutRefProject {}
}

pub(crate)
mod more_diagnostics_tricks {
    use super::*;

    pub
    trait OutRefProject<BrandedPlaceAndFieldPlaces : IsBrandedPlace> {
        // This is an assoc function provided through a `prelude`-provided extension trait.
        //
        // This way somebody somehow guessing to use this but having forgotten the `derive` will
        // get better diagnostics than a mere "assoc function not found".
        //
        /// Magic function provided by <code>#[derive([OutRefProject])]</code>, to enable usage
        /// of [`OutRef::project()`].
        ///
        /// [OutRefProject]: `derive@OutRefProject`
        fn attach_field_places<'r>(_: OutRef<'r, Self, impl IsBrandedPlace>)
          -> OutRef<'r, Self, BrandedPlaceAndFieldPlaces>
        where
            Self : derive_marker::OutRefProject,
        {
            Self::NEVER
        }
    }
    impl<'id, T, Place> OutRefProject<BrandedPlace<'id, Place>> for T {}

    mod derive_marker {
        #[diagnostic::on_unimplemented(
            message = "Missing `#[derive(OutRefProject)]` on `{Self}`",
            label = "needs a `#[derive(OutRefProject)]`",
        )]
        pub
        trait OutRefProject {
            const NEVER: ::never_say_never::Never;
        }
    }
}

impl<'r, T : ?Sized, TPlace : IsBrandedPlace, FieldPlaces: IsBrandedPlace>
    OutRef<'r, T, HList![TPlace, ..FieldPlaces]>
{
    pub
    fn discard_field_places(
        self: OutRef<'r, T, HList![TPlace, ..FieldPlaces]>,
    ) -> OutRef<'r, T, TPlace>
    {
        unsafe {
            OutRef::from_raw_branded_unchecked(self.into_raw())
        }
    }
}

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
                    // TODO: use `OwnRef::assume_owns()` once it handles slices.
                    ::core::ptr::drop_in_place::<[T]>(
                        ::core::ptr::slice_from_raw_parts_mut(
                            uninit_slice_mut.as_mut_ptr().cast(),
                            num_indwelled,
                        ),
                    );
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
    ) -> (
            EmplacingIterable<'r, T, SliceElem<FullSlicePlace>>,
            ControlFlow<
                OwnRef<'r, [T], FullSlicePlace>,
                OutRef<'r, T, SliceElem<FullSlicePlace>>,
            >,
        )
    {
        let mut iter = EmplacingIterable {
            raw: self.raw,
            num_indwelled: 0,
            _brand: PD,
        };
        let mb_item = unsafe {
            iter.next_unchecked()
        };
        (iter, mb_item)
    }
}

pub
struct EmplacingIterable<'r, T, SliceElemPlace : MaybeBrandedPlace> {
    /// Morally, an `Aliased<&'r mut [MU<T>]>`.
    raw: ptr::NonNull<[T]>,
    /// Invariants:
    ///
    ///   - When `num_indwelled < raw.len()`,
    ///
    ///       - We `&own [T]` the `[0..num_indwelled]` range of `*raw`.
    ///
    ///       - The elem at `[num_indwelled]` is being `Out`-written to.
    ///
    ///   - Otherwise, upon witnessing this, [`Self::next_unchecked()`]
    ///     has given the (full, and branded) `&own [T]` away.
    ///
    ///     We reset `num_indwelled` back to `0` to express this and restore the previous state.
    ///
    /// (Note: when `raw.len() == 0`, both states are squashed together.)
    num_indwelled: usize,
    _brand: PD<(&'r mut [MU<T>], SliceElemPlace)>,
}

impl<'r, T, SliceElemPlace : MaybeBrandedPlace>
    Drop
for
    EmplacingIterable<'r, T, SliceElemPlace>
{
    fn drop(&mut self) {
        unsafe {
            // TODO: use `OwnRef::assume_owns()` once it supports `[T]`:
            ptr::drop_in_place::<[T]>(
                ptr::slice_from_raw_parts_mut(
                    self.raw.as_ptr() as *mut T,
                    self.num_indwelled,
                ),
            )
        }
    }
}

impl<'r, T, FullSlicePlace : IsBrandedPlace>
    EmplacingIterable<'r, T, SliceElem<FullSlicePlace>>
{
    /// # Safety
    ///
    /// Must only be called when the previously yielded item was just initialized
    /// (and not dropped). IOW, `0..num_indwelled` must be made of initialized, `&own`-able, `T`s.
    ///
    /// Two special mentions:
    ///
    ///   - it is also fine to call the very first time (`num_indwelled = 0`, and `0..0` is empty).
    unsafe
    fn next_unchecked(&mut self)
      -> ControlFlow<
            OwnRef<'r, [T], FullSlicePlace>,
            OutRef<'r, T, SliceElem<FullSlicePlace>>,
        >
    {
        let _in_range @ true = self.num_indwelled < self.raw.len() else {
            // As per the safety invariant of this function, inductively, every single
            // item in `*raw` has been initialized, one elem at a time.
            //
            //  1. "Take" the `&own` out us:
            self.num_indwelled = 0; // Moved this over the drop logic of `*self`.
            return ControlFlow::Break(unsafe {
                //
                // TODO: make `OwnRef::assume_owns()` handle `[T]`.
                ::core::mem::transmute(self.raw)
            });
        };
        ControlFlow::Continue(unsafe {
            // Safety:
            //
            //   - The lifetime `'r` remains valid for writes as all this stems from an
            //     `OutRef<'r, [T]>`.
            //
            //       - Non-trivially, `*self` does not alias this specific `raw[num_indwelled]`,
            //         as our *possibly-concurrent* drop impl does not range that far.
            //
            //   - The brand is safe since the whole "convoluted" design of this API prevents
            //     multiple such `OutRef`s *with equal brands* from co-existing.
            //
            //     This is a crucial thing lest unsoundness ensue:
            //
            //     ```rs
            //     let [out_0, out_1] = [out_iter.next().unwrap(), out_iter.next().unwrap()];
            //     let own_0 = out_0.write(…);
            //     let own_1 = indwell!(out_1 <- own_0); // Uh-oh…
            //     let _read_uninit = own_1.deref_move().0;
            //     ```
            //
            //     Hence, at least, the requirement for a lending iterator. At which point a
            //     "pipe-in-and-out" `.next()` API was similar, whilst also featuring *indwelling*.
            ::core::mem::transmute(
                (self.raw.as_ptr() as *mut T)
                    .add(self.num_indwelled)
            )
        })
    }

    pub
    fn next(
        &mut self,
        // We could have used `'r` here, but we're so confident in our brands we don't even need to.
        //                       v
        latest_was_init: OwnRef<'_, T, SliceElem<FullSlicePlace>>,
    ) -> ControlFlow<
            OwnRef<'r, [T], FullSlicePlace>,
            OutRef<'r, T, SliceElem<FullSlicePlace>>,
        >
    {
        self.num_indwelled += {
            ::core::mem::forget(latest_was_init);
            1
        };
        unsafe {
            // SAFETY: if it returns `Err` there is no more "stuff" whence the caller
            // could hope to produce a `latest_was_init`, let alone another `.next()` call.
            self.next_unchecked()
        }
    }
}
