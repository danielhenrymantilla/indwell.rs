use super::*;

pub(crate) struct LazyQuote(
    pub(crate) fn() -> TokenStream2,
    pub(crate) ::core::cell::RefCell<Option<TokenStream2>>,
);

impl ::quote::ToTokens for LazyQuote {
    fn to_tokens(
        self: &'_ LazyQuote,
        tokens: &'_ mut TokenStream2,
    ) {
        self.1
            .borrow_mut()
            .get_or_insert_with(self.0)
            .to_tokens(tokens)
    }
}

#[cfg_attr(rustfmt, rustfmt::skip)]
macro_rules! let_quote {(
    $( #[rustfmt::skip] )?
    use $($contents:tt)*
) => (
    $crate::utils::let_quote::__let_quote! {
        [
            []
            []
        ]
        $($contents)*
    }
)}
pub(crate) use let_quote;

#[cfg_attr(rustfmt, rustfmt::skip)]
macro_rules! __let_quote {
    (
        [
            $fst:tt
            $snd:tt
            $($deeper:tt)*
        ]
        {
            $($inner:tt)*
        } $(,
            $($rest:tt)*
        )? $(;)?
    ) => (
        $crate::utils::let_quote::__let_quote! {
            [
                $fst // duplicate fst
                $fst
                $snd
                $($deeper)*
            ]
            $($inner)*
        }
        $crate::utils::let_quote::__let_quote! {
            [
                $snd // replace fst with duplicate of snd
                $snd
                $($deeper)*
            ]
            $($($rest)*)?
        }
    );

    (
        [
            [$($path:tt)*] // fst
            $snd:tt
            $($deeper:tt)*
        ]
        $last_segment:ident $(as $rename:ident)? $(,
        $($rest:tt)* )? $(;)?
    ) => (
        let quoted = crate::utils::let_quote::LazyQuote(
            || ::quote::quote_spanned!(::proc_macro2::Span::mixed_site()=>
                $($path)* $last_segment
            ),
            None.into(),
        );
        #[allow(nonstandard_style, unused_variables)]
        #[cfg(all(
            $($rename = "__if_provided",
                any(),
            )?
        ))]
        let ref $last_segment @ _ = quoted;
    $(
        #[allow(nonstandard_style)]
        let ref $rename @ _ = quoted;
    )?
        $crate::utils::let_quote::__let_quote! {
            [
                $snd // replace fst with duplicate of snd
                $snd
                $($deeper)*
            ]
            $($($rest)*)?
        }
    );

    (
        [
            [$($path:tt)*]
            $($deeper:tt)*
        ]
        $mid_segment:tt
        $($rest:tt)*
    ) => (
        $crate::utils::let_quote::__let_quote! {
            [
                [$($path)* $mid_segment]
                $($deeper)*
            ]
            $($rest)*
        }
    );

    (
        $path:tt
        /* nothing left */
    ) => ();
}
pub(crate) use __let_quote;
