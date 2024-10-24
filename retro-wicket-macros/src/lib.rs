#![feature(proc_macro_expand)]
use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while},
    combinator::{all_consuming, eof, not, opt, rest},
    error::Error as NomError,
    multi::many1,
    sequence::{preceded, tuple},
    Parser,
};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::{collections::HashMap, hash::Hash, str::FromStr};
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
    let tokens: TokenStream2 = (start..=end_inclusive)
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
                    TokenStream2::from(input),
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

#[proc_macro]
pub fn poly(input: TokenStream) -> TokenStream {
    fn inner(input: &TokenStream) -> Option<TokenStream> {
        let input = input.to_string().replace(' ', "");
        let coefficient = preceded(
            not(eof),
            alt((take_until("x"), rest::<&str, NomError<_>>))
                .map(f32::from_str)
                .map(Result::ok)
                .map(|coefficient| coefficient.unwrap_or(1.)),
        );
        let term = tuple((
            coefficient,
            opt(preceded(
                tag("x"),
                opt(preceded(
                    tag("^"),
                    take_while(|char: char| char.is_ascii_digit())
                        .map(u32::from_str)
                        .map(Result::ok),
                ))
                .map(Option::flatten)
                .map(|exponent| exponent.unwrap_or(1)),
            ))
            .map(|exponent| exponent.unwrap_or(0)),
        ))
        .map(|(coefficient, exponent)| (exponent, coefficient));
        let mut polynomial = all_consuming(preceded(tag("y="), many1(term)));
        let terms = polynomial(&input)
            .ok()?
            .1
            .into_iter()
            .collect::<HashMap<_, _>>();
        let terms = (0..=terms.keys().max().copied()?)
            .map(|exponent| terms.get(&exponent).unwrap_or(&0.))
            .rev()
            .collect_vec();
        Some(
            quote! {
                Polynomial([#( #terms ),*])
            }
            .into(),
        )
    }
    inner(&input).unwrap_or_else(|| {
        Error::new_spanned(TokenStream2::from(input), "expected a valid polynomial")
            .to_compile_error()
            .into()
    })
}
