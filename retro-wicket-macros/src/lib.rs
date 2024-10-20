#![feature(proc_macro_expand)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Error, Expr, ExprRange, LitStr, Token,
};
use syn::{ExprLit, Lit, LitInt};

#[proc_macro]
pub fn include_textures(input: TokenStream) -> TokenStream {
    struct Input {
        name: String,
        start: usize,
        end_inclusive: usize,
    }

    impl Parse for Input {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let name: LitStr = input.parse()?;
            let _: Token![,] = input.parse()?;
            let range: ExprRange = input.parse()?;
            let Some(ref start) = range.start else {
                return Err(Error::new_spanned(
                    range,
                    "expected range to have a start limit",
                ));
            };
            let Expr::Lit(ExprLit {
                lit: Lit::Int(ref start @ LitInt { .. }),
                ..
            }) = **start
            else {
                return Err(Error::new_spanned(
                    start,
                    "expected range start to be an integer literal",
                ));
            };
            let start = start.base10_parse()?;

            let Some(ref end_inclusive) = range.end else {
                return Err(Error::new_spanned(
                    range,
                    "expected range to have an end limit",
                ));
            };
            let Expr::Lit(ExprLit {
                lit: Lit::Int(ref end_inclusive @ LitInt { .. }),
                ..
            }) = **end_inclusive
            else {
                return Err(Error::new_spanned(
                    end_inclusive,
                    "expected range end to be an integer literal",
                ));
            };
            let end_inclusive = end_inclusive.base10_parse()?;

            Ok(Self {
                name: name.value(),
                start,
                end_inclusive,
            })
        }
    }

    let Input {
        name,
        start,
        end_inclusive,
    } = parse_macro_input!(input as Input);
    let tokens: proc_macro2::TokenStream = (start..=end_inclusive)
        .map(|suffix| {
            let name = format!("{name}{suffix}");
            quote! {
                include_texture!(#name),
            }
        })
        .collect();
    let tokens = quote! { [#tokens] };
    tokens.into()
}

#[proc_macro]
pub fn hex(input: TokenStream) -> TokenStream {
    let to_string = input.to_string();
    u32::from_str_radix(&to_string, 16)
        .ok()
        .filter(|_| to_string.len() == 6)
        .map_or_else(
            || {
                Error::new_spanned(
                    proc_macro2::TokenStream::from(input),
                    "expected a 6 digit hexadecimal number without a #",
                )
                .to_compile_error()
            },
            |colour| {
                let [_, red, green, blue] = colour.to_be_bytes();
                quote! {
                    macroquad::color_u8!(#red, #green, #blue, 255)
                }
            },
        )
        .into()
}
