use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_str, punctuated::Punctuated, token::Comma, ExprLit,
    ItemEnum, Variant,
};

#[allow(clippy::redundant_pub_crate)]
pub(crate) fn enum_from(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);
    let name = &input.ident;
    let variants_str = generate_variants_str(&input.variants);
    let variants_byte = generate_variants_byte(&input.variants);

    let expanded = quote! {
        impl TryFrom<&str> for #name {
            type Error = ();

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                match value {
                    #(#variants_str)*
                    _ => Err(()),
                }
            }
        }

        impl TryFrom<&[u8]> for #name {
            type Error = ();

            fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
                match value {
                    #(#variants_byte)*
                    _ => Err(()),
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn generate_variants_str(
    variants: &Punctuated<Variant, Comma>,
) -> Vec<proc_macro2::TokenStream> {
    let mut result = Vec::new();
    for variant in variants {
        let ident = &variant.ident;
        let ident_str = format!("{}", ident);
        result.push(quote! {
            #ident_str => Ok(Self::#ident),
        });
    }
    result
}

fn generate_variants_byte(
    variants: &Punctuated<Variant, Comma>,
) -> Vec<proc_macro2::TokenStream> {
    let mut result = Vec::new();
    for variant in variants {
        let ident = &variant.ident;
        let ident_str = format!("{}", ident);
        let expr_lit: ExprLit = parse_str(&format!(r#"b"{}""#, ident_str))
            .expect("Unable to transform variant");
        result.push(quote! {
            #expr_lit => Ok(Self::#ident),
        });
    }
    result
}
