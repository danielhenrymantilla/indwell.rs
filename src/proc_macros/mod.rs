#![allow(non_snake_case, unused_imports)]

use ::core::{
    mem,
    ops::Not as _,
};
use ::proc_macro::{
    TokenStream,
};
use proc_macro2::Span;
use ::proc_macro2::{
    TokenStream as TokenStream2,
};
use ::quote::{
    format_ident,
    // quote,
    quote_spanned,
    ToTokens,
};
use ::syn::{*,
    parse::{Parse, Parser, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::{self as subvisit, VisitMut},
    Result,
};

use self::utils::{
    quote, parse_quote,
    let_quote,
};

mod utils;

#[proc_macro_derive(OutRefProject)] pub
fn __(input: TokenStream) -> TokenStream {
    derive_out_ref(input.into())
        .map(|(output, warning_span)| utils::mb_file_expanded(warning_span, output))
        .map_err(|mut error| {
            let mut errors =
                error
                    .into_iter()
                    .map(|err| Error::new_spanned(
                        &err.to_compile_error(),
                        format!("`#[derive(OutRefProject)]`: {err}"),
                    ))
            ;
            error = errors.next().unwrap();
            error.extend(errors);
            error
        })
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn derive_out_ref(input: TokenStream2) -> Result<(TokenStream2, Span)> {
    let input: &ItemStruct = &parse2(input)?;

    let warning_span = input.ident.span();

    let context = &Context::new(input);

    let brands_appender = brands_appender(input, context);

    let project_impl = project_impl(input, context);

    let out_refs_struct = projection_struct(input, context, WhichRef::OutRef);
    let own_refs_struct = projection_struct(input, context, WhichRef::OwnRef);

    let into_ghost_init_with_def = into_ghost_init_with_def(input, context);

    let indwell = indwell(input, context);

    let ret = quote!(
        #[allow(warnings, clippy::all)]
        const _: () = {
            #brands_appender

            #project_impl
        };

        #[allow(warnings, clippy::all)]
        #out_refs_struct

        #[allow(warnings, clippy::all)]
        #own_refs_struct

        #into_ghost_init_with_def

        #indwell
    );
    Ok((ret, warning_span))
}

struct Context {
    /// Privacy/visibility of the `struct`.
    pub_: Visibility,
    // /// Min/intersection of all the visibilities (max/union of all the privacies) of the `struct`
    // /// and its fields.
    // min_pub: Visibility,
    /// `'__s`
    ref_lt: Lifetime,
    /// `__Place_s, __Place_x, …`
    place_params: Vec<Ident>,
    /// `__Place_s: #IsBrandedPlace, __Place_x : #IsBrandedPlace, …`
    intro_place_params: Vec<GenericParam>,
    // The following are self-explanatory idents derived from the type name.
    StructOutRefs: Ident,
    StructOwnRefs: Ident,
    StructGhostInit: Ident,
}

impl Context {
    fn new(input: &ItemStruct) -> Self {
        let_quote!(
            use ::enroot::{
                brand::IsBrandedPlace,
            };
        );

        // TODO: ensure the lifetime is not already present.
        let ref_lt = Lifetime::new("'__s", Span::mixed_site());
        let place_params =
            input.fields.members().map(|field: Member| -> Ident {
                    let field_name: &dyn ::quote::IdentFragment = match &field {
                        Member::Named(ident) => ident,
                        Member::Unnamed(index) => index,
                    };
                    format_ident!("__Place_{}", field_name)
            })
            .collect::<Vec<_>>()
        ;
        let intro_place_params =
            place_params.iter().map(|PlaceParam @ _| -> GenericParam {
                parse_quote!(
                    #PlaceParam : #IsBrandedPlace
                )
            })
            .collect::<Vec<_>>()
        ;

        Self {
            // pub_: input.vis.clone(),
            pub_:
                // We can't really resolve what the actual min might be in the general case
                // (arbitrary `pub(in some::path)` case), but we don't need fine-grained privacy
                // here (field privacy in and of itself will take care of it).
                //
                // What we want is to determine whether we go can fully `pub` or not. Otherwise any
                // non-`pub` privacy will do.
                Iterator::chain(
                    ::core::iter::once(&input.vis),
                    input.fields.iter().map(|f| &f.vis),
                )
                .find(|v| matches!(v, Visibility::Public { .. }).not())
                .unwrap_or(&input.vis)
                .clone()
            ,
            ref_lt,
            place_params,
            intro_place_params,
            StructOutRefs: format_ident!("{}OutRefs", input.ident),
            StructOwnRefs: format_ident!("{}OwnRefs", input.ident),
            StructGhostInit: format_ident!("{}GhostInit", input.ident),
        }
    }
}

#[derive(Debug)]
enum WhichRef {
    OwnRef,
    OutRef,
}

fn projection_struct(
    input: &ItemStruct,
    Context {
        pub_,
        ref_lt,
        place_params,
        intro_place_params,
        StructOutRefs,
        StructOwnRefs,
        ..
    }: &Context,
    which_ref: WhichRef,
) -> ItemStruct
{
    let_quote!(
        use ::enroot::{
            brand::IsBrandedPlace,
            own_ref::OwnRef,
            out_ref::OutRef,
        };
    );

    let mut ret = input.clone();

    let ORef @ _;
    (ret.ident, ORef) = match which_ref {
        WhichRef::OutRef => (StructOutRefs.clone(), OutRef),
        WhichRef::OwnRef => (StructOwnRefs.clone(), OwnRef),
    };
    let mut ORef_respanned = ORef.to_token_stream().into_iter().collect::<Vec<_>>();

    ret.ident = format_ident!("{}{which_ref:?}s", input.ident);


    ret.generics.params.insert(0, parse_quote!(#ref_lt));
    ret.generics.params.extend(intro_place_params.clone());

    ret.fields.iter_mut().zip(place_params).for_each(|(f, PlaceParam @ _)| {
        let FieldTy @ _ = &mut f.ty;
        let span = FieldTy.span();
        // This way if a diagnostic wants to complain about the privacy of a field of a tuple struct
        // it will highlight the beginning and end of these tokens, _i.e._,
        // the first token of `#(#ORef_spanned)*` as well as `>`. So both need to be spanned at
        // the field type (or at least the beginning thereof).
        ORef_respanned.iter_mut().for_each(|tt| tt.set_span(span));
        let closing_angle_bracket = {
            // This might be excessive… but it does improve the diagnostic…
            let last_span = FieldTy.to_token_stream().into_iter().last().unwrap().span();
            quote_spanned!(last_span=>
                >
            )
        };
        *FieldTy = parse_quote_spanned!(span=>
            #(#ORef_respanned)* <#ref_lt, #FieldTy, #PlaceParam #closing_angle_bracket /* > */
        );
    });

    if matches!(pub_, Visibility::Public { .. }).not() {
        match which_ref {
            WhichRef::OwnRef => ret.vis.clone_from(&pub_),
            WhichRef::OutRef => {
                // We can't really make it non-`pub` as it's used as the assoc type of a `pub trait`
                ret.attrs.push(parse_quote_spanned!(pub_.span()=>
                    #[doc(hidden)]
                ))
            },
        }
    }

    ret
}

fn project_impl(
    input: &ItemStruct,
    Context {
        ref_lt,
        place_params: EachPlaceParam @ _,
        intro_place_params,
        StructOutRefs,
        ..
    }: &Context,
) -> TokenStream2
{
    let_quote!(
        use ::enroot::{
            brand::IsBrandedPlace,
            hlist::HList,
            out_ref::{OutRef, OutRefProject, derive_marker::OutRefProject as derive_marker_OutRefProject},
        };
    );

    let StructName @ _ = &input.ident;

    // TODO: ensure this name is available, etc.
    let BrandParam @ _ = format_ident!(
        "__Brand",
    );

    let mut generics = input.generics.clone();
    generics.params.push(parse_quote!(
        #BrandParam : #IsBrandedPlace
    ));
    generics.params.extend(intro_place_params.clone());

    let (intro_generics, _, mb_where_clause) = generics.split_for_impl();
    let (_, fwd_generics, _) = input.generics.split_for_impl();

    let raw = Ident::new("raw", Span::mixed_site());

    let each_field_init = input.fields.members().map(|member| quote_spanned!(member.span()=>
        #member: unsafe {
            #OutRef::from_raw_branded_unchecked(
                ::core::ptr::NonNull::new_unchecked(
                    &raw mut (*#raw.as_ptr()).#member,
                ),
            )
        }
    ));

    let mut generics_with_lt = input.generics.clone();
    generics_with_lt.params.insert(0, parse_quote!(#ref_lt));
    generics_with_lt.params.extend(intro_place_params.clone());
    let (_, fwd_all_generics, _) = generics_with_lt.split_for_impl();

    let (simple_intro, _, simple_mb_where_clause) = input.generics.split_for_impl();

    quote!(
        impl #simple_intro
            #derive_marker_OutRefProject
        for
            #StructName #fwd_generics
        #simple_mb_where_clause
        {}

        impl #intro_generics
            #OutRefProject<
                #HList![#BrandParam #(, #EachPlaceParam)*],
            >
        for
            #StructName #fwd_generics
        #mb_where_clause
        {
            type Projection<#ref_lt> = #StructOutRefs #fwd_all_generics ;

            fn project_impl<#ref_lt>(
                out_ref: #OutRef<#ref_lt, Self, #HList![#BrandParam #(, #EachPlaceParam)*]>,
            ) -> #StructOutRefs #fwd_all_generics
            {
                let #raw = out_ref.into_raw();
                #StructOutRefs {
                    #(#each_field_init,)*
                }
            }
        }
    )
}

fn brands_appender(
    input: &ItemStruct,
    Context { pub_, ref_lt, .. }: &Context,
) -> TokenStream2
{

    // TODO: check these idents are not already present in `input`.
    let EachFieldNameEnum @ _ =
        &input.fields.members().map(|member| match member {
            Member::Named(ident) => ident,
            Member::Unnamed(index) => format_ident!("_{}", index),
        })
        .collect::<Vec<_>>()
    ;

    let each_field_pub = input.fields.iter().map(|f| &f.vis);

    let (intro_generics, fwd_generics, mb_where_clause) = input.generics.split_for_impl();

    let StructName @ _ = &input.ident;

    let_quote!(
        use ::enroot::{
            brand::{BrandedPlace, IsBrandedPlace, Place},
            hlist::HList,
            out_ref::{OutRef, OutRefProject},
        };
    );

    let EachGenericTy @ _ =
        input
            .generics
            .type_params()
            .map(|TypeParam { ident, .. }| ident)
            .collect::<Vec<_>>()
    ;

    let EachFieldSubBrand @ _ = EachFieldNameEnum.iter().map(|field| quote_spanned!(field.span()=>
        #BrandedPlace<
            'static,
            impl use<__SelfPlace #(, #EachGenericTy)*> + #Place<#field>,
        >
    ));

    let EachFieldSubBrandConcrete @ _ = (0..input.fields.len()).map(|_| quote!(
        #BrandedPlace<
            '_, // 'static
            (),
        >
    ));

    quote!(
        #(
            #each_field_pub
            enum #EachFieldNameEnum {}
        )*

        impl #intro_generics #StructName #fwd_generics
        #mb_where_clause
        {
            /// Magic function provided by <code>#[derive([OutRefProject])]</code>, to enable usage
            /// of [`OutRef::project()`][`::enroot::out_ref::OutRef::project()`].
            ///
            /// [OutRefProject]: `derive@::enroot::out_ref::OutRefProject`
            #pub_
            fn attach_field_places<#ref_lt, __SelfPlace : #IsBrandedPlace>(
                out_ref: #OutRef<#ref_lt, Self, __SelfPlace>
            ) -> #OutRef<#ref_lt, Self, #HList![
                __SelfPlace, #(
                #EachFieldSubBrand ,)*
            ]>
            {
                unsafe {
                    #OutRef::<Self, #HList![
                        __SelfPlace, #(
                        #EachFieldSubBrandConcrete ,)*
                    ]>::from_raw_branded_unchecked(
                        #OutRef::into_raw(out_ref)
                    )
                }
            }
        }
    )
}

fn into_ghost_init_with_def(
    input: &ItemStruct,
    Context {
        pub_,
        ref_lt,
        place_params: EachPlaceParam @ _,
        intro_place_params: IntroEachPlaceParam @ _,
        StructOwnRefs,
        StructGhostInit,
        ..
    }: &Context,
) -> TokenStream2
{
    let_quote!(
        use ::enroot::{
            indwell::GhostInitProof,
        };
    );

    let mut generics = input.generics.clone();
    generics.params.insert(0, parse_quote!( #ref_lt ));
    generics.params.extend(IntroEachPlaceParam.clone());
    let (intro_generics, fwd_generics, mb_where_clause) = generics.split_for_impl();

    let each_member = input.fields.members().collect::<Vec<_>>();

    let ghost_init_fields = match &input.fields {
        Fields::Named(_) => quote!(
            {
                #(
                    #each_member: #GhostInitProof<#EachPlaceParam>,
                )*
            }
        ),
        Fields::Unnamed(_) => quote!(
            (
                #(
                    #GhostInitProof<#EachPlaceParam>,
                )*
            );
        ),
        // y tho?
        Fields::Unit => quote!(
            ;
        ),
    };

    quote!(
        #[allow(warnings, clippy::all)]
        impl #intro_generics
            #StructOwnRefs #fwd_generics
        #mb_where_clause
        {
            #[must_use = "this is leaking memory unless you call `.indwell()`"]
            #pub_
            fn into_ghost_init(self) -> #StructGhostInit<#(#EachPlaceParam),*> {
                #StructGhostInit {
                    #(
                        #each_member: self.#each_member.into_ghost_init(),
                    )*
                }
            }
        }
        // where:
        #[allow(warnings, clippy::all)]
        #pub_
        struct #StructGhostInit<#(#IntroEachPlaceParam),*>
            #ghost_init_fields
    )
}

#[expect(unused)]
fn indwell(
    input: &ItemStruct,
    Context {
        pub_,
        ref_lt,
        intro_place_params: IntroEachPlaceParam @ _,
        place_params: EachPlaceParam @ _,
        StructOutRefs,
        StructOwnRefs,
        StructGhostInit,
    }: &Context,
) -> TokenStream2
{
    let_quote!(
        use ::enroot::{
            brand::IsBrandedPlace,
            hlist::HList,
            own_ref::OwnRef,
            out_ref::OutRef,
        };
    );

    quote!(
        #[allow(warnings, clippy::all)]
        impl<#(#IntroEachPlaceParam),*>
            #StructGhostInit<#(#EachPlaceParam),*>
        {
            #pub_
            unsafe
            fn indwell<#ref_lt, __T, __SelfPlace : #IsBrandedPlace>(
                self: #StructGhostInit<#(#EachPlaceParam),*>,
                out_ref: #OutRef<#ref_lt, __T, #HList![
                    __SelfPlace #(,
                    #EachPlaceParam )*
                ]>,
            ) -> #OwnRef<#ref_lt, __T, __SelfPlace>
            {
                unsafe {
                    #OwnRef::from_raw_branded_unchecked(
                        #OutRef::into_raw(out_ref)
                    )
                }
            }
        }
    )
}
