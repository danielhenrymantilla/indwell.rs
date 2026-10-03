use super::*;

pub(crate)
fn mb_file_expanded(warning_span: Span, output: TokenStream2) -> TokenStream2 {
    let mut debug_macros_dir = match ::std::env::var_os("DEBUG_MACROS_LOCATION") {
        Some(it) => ::std::path::PathBuf::from(it),
        None => return output,
    };
    let hopefully_unique = {
        // ::rand::random() at home:
        use ::std::hash::*;
        let ref mut hasher = ::std::collections::hash_map::RandomState::new().build_hasher();
        hasher.finish()
    };

    debug_macros_dir.push("enroot-debugged-proc-macros");
    ::std::fs::create_dir_all(&debug_macros_dir).unwrap_or_else(|err| {
        panic!(
            "`DEBUG_MACROS_LOCATION`-error: failed to create {}: {}",
            debug_macros_dir.display(),
            err,
        )
    });
    let ref file_name = {
        debug_macros_dir.push(format!("{:016x}.rs", hopefully_unique));
        debug_macros_dir.into_os_string().into_string().expect(
            "`DEBUG_MACROS_LOCATION`-error: \
                non-UTF-8 paths are not supported\
            ",
        )
    };

    ::std::fs::write(
        file_name,
        ::std::panic::catch_unwind(|| ::prettyplease::unparse(&parse_quote!(#output)))
            .unwrap_or_else(|_| quote!(#output).to_string()),
    )
    .unwrap_or_else(|err| {
        panic!(
            "`DEBUG_MACROS_LOCATION`-error: failed to write to `{}`: {}",
            file_name, err
        )
    });

    let (start_span, end_span) = (
        quote_spanned!(warning_span.located_at(Span::mixed_site())=> . ),
        quote_spanned!(warning_span=> . ),
    );
    let warning = compile_warning(
        &quote!(#start_span #end_span),
        &format!("[DEBUG_MACROS_LOCATION]: Output emitted to {file_name}"),
    );
    quote!(
        #warning

        ::core::include! {
            #file_name
        }
    )
}
