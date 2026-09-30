use super::*;

pub(crate) use let_quote::let_quote;
pub(crate) mod let_quote;

pub(crate) use mb_file_expanded::mb_file_expanded;
mod mb_file_expanded;

macro_rules! quote {(
    $($input:tt)*
) => (
    ::quote::quote_spanned!(::proc_macro2::Span::mixed_site()=>
        $($input)*
    )
)}
pub(crate) use quote;

macro_rules! parse_quote {(
    $($input:tt)*
) => (
    ::syn::parse_quote_spanned!(::proc_macro2::Span::mixed_site()=>
        $($input)*
    )
)}
pub(crate) use parse_quote;

pub(crate) fn compile_warning(
    span: &dyn ToTokens,
    message: &str,
) -> TokenStream2 {
    let mut spans = span.to_token_stream().into_iter().map(|tt| tt.span());
    let fst = spans.next().unwrap_or_else(Span::call_site);
    let lst = spans.fold(fst, |_, cur| cur);
    let ref message = ["\n", message].concat();
    let warning = Ident::new("warning", fst);
    quote_spanned!(lst=>
        #[allow(nonstandard_style, clippy::all)]

        const _: () = {
            struct enroot_ {
                #[deprecated(note = #message)]
                #warning: ()
            }
            //                  fst    lst
            let _ = enroot_ { #warning: () };
            //                ^^^^^^^^^^^^
        };
    )
}
