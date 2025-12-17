#![doc = "Provides a procedural macro that exposes a compile-time static list of all variants of an enum."]

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Attribute, Data, DeriveInput, Fields, parse_macro_input};

/// Derive macro that exposes compile-time constants for the full set of enum variants.
///
/// ---
/// # Examples
/// ```ignore
/// use enumly::Enumly;
///
/// #[derive(Enumly, Debug, PartialEq)]
/// enum Color {
///     Red,
///     Green,
///     Blue,
/// }
///
/// assert_eq!(Color::COUNT, 3);
/// assert_eq!(Color::VARIANTS, &[Color::Red, Color::Green, Color::Blue]);
/// ```
///
/// ---
/// Fails to compile when any variant is not unit:
/// ```compile_fail
/// use enumly::Enumly;
///
/// #[derive(Enumly)]
/// enum Bad {
///     Tuple(u8),
///     Struct { value: u8 },
/// }
/// ```
///
#[proc_macro_derive(Enumly)]
pub fn derive_enumly(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let Some(err) = non_exhaustive_error(&input.attrs) {
        return err.to_compile_error().into();
    }

    let data_enum = match input.data {
        Data::Enum(data_enum) => data_enum,
        _ => {
            return syn::Error::new(input.ident.span(), "Enumly can only be derived for enums")
                .to_compile_error()
                .into();
        }
    };

    let mut variant_idents = Vec::with_capacity(data_enum.variants.len());

    for variant in data_enum.variants {
        if let Some(err) = non_exhaustive_error(&variant.attrs) {
            return err.to_compile_error().into();
        }

        match variant.fields {
            Fields::Unit => variant_idents.push(variant.ident),
            _ => {
                return syn::Error::new(
                    variant.ident.span(),
                    "Enumly only supports unit variants; tuple and struct variants are not allowed",
                )
                .to_compile_error()
                .into();
            }
        }
    }

    let name = &input.ident;
    let count = variant_idents.len();
    let variant_exprs = variant_idents
        .iter()
        .map(|variant| quote! { Self::#variant });
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub const COUNT: usize = #count;
            pub const VARIANTS: &'static [Self] = &[#(#variant_exprs),*];
        }
    };

    TokenStream::from(expanded)
}

fn non_exhaustive_error(attrs: &[Attribute]) -> Option<syn::Error> {
    attrs
        .iter()
        .find(|attr| attr.path().is_ident("non_exhaustive"))
        .map(|attr| {
            syn::Error::new(
                attr.span(),
                "Enumly does not support #[non_exhaustive] enums or variants",
            )
        })
}
