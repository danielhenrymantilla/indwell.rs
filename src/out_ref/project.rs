use ::never_say_never::Never;
use crate::{
    brand::hlist::SliceElem, indwell::GhostInitProof, utils::diagnostics::Is,
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
    ) -> impl ExactSizeIterator<Item = OutRef<'r, T, SliceElem<FullSlicePlace>>>
    where
        T : 'r,
    {
        unsafe { self.as_uninit_mut() }.iter_mut().map(|uninit_mut: &mut MU<T>| unsafe {
            OutRef::from_raw_branded_unchecked(ptr::NonNull::from(uninit_mut).cast())
        })
    }
}

pub
struct SoulCollector<'r, FullSlicePlace : IsBrandedPlace> {
    num_indwelled: usize,
    _phantom: PD<OutRef<'r, (), SliceElem<FullSlicePlace>>>,
}

impl<'r, FullSlicePlace : IsBrandedPlace>
    Default
for
    SoulCollector<'r, FullSlicePlace>
{
    fn default() -> SoulCollector<'r, FullSlicePlace> {
        Self {
            num_indwelled: 0,
            _phantom: PD,
        }
    }
}

impl<'r, T, FullSlicePlace : IsBrandedPlace>
    Extend<OwnRef<'r, T, SliceElem<FullSlicePlace>>>
for
    SoulCollector<'r, FullSlicePlace>
{
    fn extend<Iterable>(
        self: &'_ mut SoulCollector<'r, FullSlicePlace>,
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
    SoulCollector<'r, FullSlicePlace>
{
    fn from_iter<Iterable>(iterable: Iterable) -> SoulCollector<'r, FullSlicePlace>
    where
        Iterable : IntoIterator<Item = OwnRef<'r, T, SliceElem<FullSlicePlace>>>,
    {
        let mut ret = Self::default();
        ret.extend(iterable);
        ret
    }
}

impl<'r, FullSlicePlace : IsBrandedPlace> SoulCollector<'r, FullSlicePlace> {
    pub
    fn into_ghost_init(self) -> SoulCollector<'static, FullSlicePlace> {
        unsafe {
            ::core::mem::transmute(self)
        }
    }
}

#[derive(Debug)]
pub
struct MissingSouls;

impl<FullSlicePlace : IsBrandedPlace> SoulCollector<'static, FullSlicePlace> {
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
        FullSlicePlace : Is<EqualTo = FullSlicePlace_>,
    {
        if self.num_indwelled < husk.len() {
            Err(MissingSouls)
        } else {
            Ok(unsafe {
                GhostInitProof::branded_place_assume_init().indwell(husk)
            })
        }
    }
}

pub
trait UninitSlice<FullSlicePlace : IsBrandedPlace>
:
    indwell::UninitPlace<FullSlicePlace> +
{
    fn len(&self) -> usize;
}

impl<'r, T, FullSlicePlace : IsBrandedPlace>
    UninitSlice<FullSlicePlace>
for
    OutRef<'r, [T], FullSlicePlace>
{
    fn len(&self) -> usize {
        self.raw.len()
    }
}
impl<'r, T, FullSlicePlace : IsBrandedPlace, const N: usize>
    UninitSlice<FullSlicePlace>
for
    OutRef<'r, [T; N], FullSlicePlace>
{
    fn len(&self) -> usize {
        N
    }
}

impl<'r, T, FullSlicePlace : IsBrandedPlace, const N: usize>
    UninitSlice<FullSlicePlace>
for
    &'r mut BrandedUninit<[T; N], FullSlicePlace>
{
    fn len(&self) -> usize {
        N
    }
}

impl<T, FullSlicePlace : IsBrandedPlace, FullSlicePlace_ : IsBrandedPlace, const N: usize>
    UninitSlice<FullSlicePlace>
for
    BrandedUninit<[T; N], FullSlicePlace_>
where
    FullSlicePlace: Is<EqualTo = FullSlicePlace_>,
    FullSlicePlace_: Is<EqualTo = FullSlicePlace>,
{
    fn len(&self) -> usize {
        N
    }
}

impl<T, FullSlicePlace : IsBrandedPlace, const N: usize>
    UninitSlice<FullSlicePlace>
for
    Box<BrandedUninit<[T; N], FullSlicePlace>>
{
    fn len(&self) -> usize {
        N
    }
}
