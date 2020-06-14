use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Fields, ItemStruct};

pub(crate) fn from_form(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let name = &input.ident;
    let variants_str = generate_variants_str(&input.fields);

    let expanded = quote! {
        impl<'a, I: Iterator<Item = (Cow<'a, str>, Cow<'a, str>)>> From<I> for #name<'a> {

            #[inline]
            fn from(iter: I) -> Self {
                let mut data = Self::default();
                for (key, val) in iter {
                    if val.is_empty() {
                        continue;
                    }
                    match key.as_ref() {
                        #(#variants_str)*
                        _ => {}
                    }
                }
                data
            }
        }
    };

    TokenStream::from(expanded)
}

fn generate_variants_str(fields: &Fields) -> Vec<proc_macro2::TokenStream> {
    let mut result = Vec::new();
    for (i, field) in fields.iter().enumerate() {
        let ident = &field.ident;
        let ident_str = ident
            .as_ref()
            .map_or_else(|| i.to_string(), |ident| format!("{}", ident));
        result.push(quote! {
            #ident_str => data.#ident = Some(val),
        });
    }
    result
}
