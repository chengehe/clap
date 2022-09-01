// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>,
// Kevin Knapp (@kbknapp) <kbknapp@gmail.com>, and
// Ana Hobden (@hoverbear) <operator@hoverbear.org>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use proc_macro2::TokenStream;
use proc_macro_error::{abort, abort_call_site};
use quote::quote;
use quote::quote_spanned;
use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Comma, Data, DataEnum, DeriveInput, Fields,
    Ident, Variant,
};

use crate::dummies;
use crate::item::{Item, Kind, Name};

pub fn derive_value_enum(input: &DeriveInput) -> TokenStream {
    let ident = &input.ident;

    dummies::value_enum(ident);

    match input.data {
        Data::Enum(ref e) => {
            let name = Name::Derived(ident.clone());
            let item = Item::from_value_enum(input, name);
            gen_for_enum(&item, ident, e)
        }
        _ => abort_call_site!("`#[derive(ValueEnum)]` only supports enums"),
    }
}

pub fn gen_for_enum(item: &Item, item_name: &Ident, e: &DataEnum) -> TokenStream {
    let lits = lits(&e.variants, item);
    let value_variants = gen_value_variants(&lits);
    let to_possible_value = gen_to_possible_value(&lits);

    quote! {
        #[allow(dead_code, unreachable_code, unused_variables, unused_braces)]
        #[allow(
            clippy::style,
            clippy::complexity,
            clippy::pedantic,
            clippy::restriction,
            clippy::perf,
            clippy::deprecated,
            clippy::nursery,
            clippy::cargo,
            clippy::suspicious_else_formatting,
        )]
        #[deny(clippy::correctness)]
        impl clap::ValueEnum for #item_name {
            #value_variants
            #to_possible_value
        }
    }
}

fn lits(variants: &Punctuated<Variant, Comma>, parent_item: &Item) -> Vec<(TokenStream, Ident)> {
    variants
        .iter()
        .filter_map(|variant| {
            let item = Item::from_value_enum_variant(
                variant,
                parent_item.casing(),
                parent_item.env_casing(),
            );
            if let Kind::Skip(_) = &*item.kind() {
                None
            } else {
                if !matches!(variant.fields, Fields::Unit) {
                    abort!(variant.span(), "`#[derive(ValueEnum)]` only supports unit variants. Non-unit variants must be skipped");
                }
                let fields = item.field_methods(false);
                let name = item.cased_name();
                Some((
                    quote_spanned! { variant.span()=>
                        clap::builder::PossibleValue::new(#name)
                        #fields
                    },
                    variant.ident.clone(),
                ))
            }
        })
        .collect::<Vec<_>>()
}

fn gen_value_variants(lits: &[(TokenStream, Ident)]) -> TokenStream {
    let lit = lits.iter().map(|l| &l.1).collect::<Vec<_>>();

    quote! {
        fn value_variants<'a>() -> &'a [Self]{
            &[#(Self::#lit),*]
        }
    }
}

fn gen_to_possible_value(lits: &[(TokenStream, Ident)]) -> TokenStream {
    let (lit, variant): (Vec<TokenStream>, Vec<Ident>) = lits.iter().cloned().unzip();

    quote! {
        fn to_possible_value<'a>(&self) -> ::std::option::Option<clap::builder::PossibleValue> {
            match self {
                #(Self::#variant => Some(#lit),)*
                _ => None
            }
        }
    }
}
