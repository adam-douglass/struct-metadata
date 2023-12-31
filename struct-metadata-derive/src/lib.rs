//! Derive macro for the struct-metadata package.

#![warn(missing_docs, non_ascii_idents, trivial_numeric_casts,
    noop_method_call, single_use_lifetimes, trivial_casts,
    unused_lifetimes, nonstandard_style, variant_size_differences)]
#![deny(keyword_idents)]
#![warn(clippy::missing_docs_in_private_items)]
#![allow(clippy::needless_return, clippy::while_let_on_iterator)]

use proc_macro::{self, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, Token, Ident, LitBool};

/// Derive macro for the Described trait
#[proc_macro_derive(Described, attributes(metadata, metadata_type, metadata_sequence, serde))]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput {ident, attrs, data, ..} = parse_macro_input!(input);

    let metadata_type = parse_metadata_type(&attrs);

    match data {
        syn::Data::Struct(data) => {

            let kind = match data.fields {
                syn::Fields::Named(fields) => {
                    let mut children = vec![];

                    for field in &fields.named {
                        let SerdeParams {rename, .. } = _parse_serde_renames(&field.attrs);
                        let name = field.ident.clone().unwrap();
                        let ty = &field.ty;
                        let ty = quote_spanned!(ty.span() => <#ty as struct_metadata::Described::<#metadata_type>>::metadata());
                        let docs = parse_doc_comment(&field.attrs);
                        let metadata: proc_macro2::TokenStream = parse_metadata_params(&metadata_type, &field.attrs);

                        let name = if let Some(rename) = rename {
                            rename
                        } else {
                            name.to_string()
                        };

                        children.push(quote!{struct_metadata::Entry::<#metadata_type> {
                            label: #name,
                            docs: #docs,
                            metadata: #metadata,
                            type_info: #ty
                        }});
                    }

                    quote!(struct_metadata::Kind::Struct::<#metadata_type> {
                        name: stringify!(#ident),
                        children: vec![#(#children),*]
                    })
                },
                syn::Fields::Unnamed(fields) => {
                    if fields.unnamed.is_empty() {
                        quote!(struct_metadata::Kind::<#metadata_type>::Struct { name: stringify!(#ident), children: vec![] })
                    } else if fields.unnamed.len() == 1 {
                        let ty = &fields.unnamed[0].ty;
                        let ty = quote_spanned!(ty.span() => <#ty as struct_metadata::Described::<#metadata_type>>::metadata());
                        quote!(struct_metadata::Kind::<#metadata_type>::Aliased { name: stringify!(#ident), kind: Box::new(#ty)})
                    } else {
                        panic!("tuple struct not supported")
                    }
                },
                syn::Fields::Unit => {
                    quote!(struct_metadata::Kind::<#metadata_type>::Struct { name: stringify!(#ident), children: vec![] })
                },
            };

            let docs = parse_doc_comment(&attrs);
            let metadata: proc_macro2::TokenStream = parse_metadata_params(&metadata_type, &attrs);
            let output = quote! {
                impl struct_metadata::Described::<#metadata_type> for #ident {
                    fn metadata() -> struct_metadata::Descriptor::<#metadata_type> {
                        struct_metadata::Descriptor::<#metadata_type> {
                            docs: #docs,
                            kind: #kind,
                            metadata: #metadata,
                        }
                    }
                }
            };

            output.into()
        }

        syn::Data::Enum(data) => {

            let mut all_variants = vec![];

            for variant in data.variants {

                if !variant.fields.is_empty() {
                    return syn::Error::new(variant.fields.span(), "Only enums without field values are supported.").into_compile_error().into()
                }

                let name = variant.ident.clone();
                let docs = parse_doc_comment(&variant.attrs);
                let metadata: proc_macro2::TokenStream = parse_metadata_params(&metadata_type, &variant.attrs);

                let name = quote_spanned!(variant.span() => #ident::#name.to_string());

                all_variants.push(quote!{struct_metadata::Variant::<#metadata_type> {
                    label: #name,
                    docs: #docs,
                    metadata: #metadata,
                }});
            }

            let docs = parse_doc_comment(&attrs);
            let metadata: proc_macro2::TokenStream = parse_metadata_params(&metadata_type, &attrs);
            let output = quote! {
                impl struct_metadata::Described::<#metadata_type> for #ident {
                    fn metadata() -> struct_metadata::Descriptor::<#metadata_type> {
                        struct_metadata::Descriptor::<#metadata_type> {
                            docs: #docs,
                            kind: struct_metadata::Kind::<#metadata_type>::Enum {
                                name: stringify!(#ident),
                                variants: vec![#(#all_variants),*]
                            },
                            metadata: #metadata,
                        }
                    }
                }
            };

            output.into()
        }

        _ => {
            panic!("only structs are supported")
        }
    }
}

/// Helper function to pull out docstrings
fn parse_doc_comment(attrs: &[syn::Attribute]) -> proc_macro2::TokenStream {
    let mut lines = vec![];
    for attr in attrs {
        if let Ok(syn::Meta::NameValue(meta)) = attr.parse_meta() {
            if let syn::Lit::Str(doc) = meta.lit {
                // return Some()
                lines.push(doc.value().trim().to_string());
            }
        }
    }

    if lines.is_empty() {
        quote! { None }
    } else {
        quote!{ Some(vec![
            #( #lines, )*
        ])}
    }
}

/// Description of metadata type being used
enum MetadataKind {
    /// Metadata is being described by a struct
    Type(proc_macro2::TokenStream, bool),
    /// Metadata is being described by something that implements FromIterator<(&'static str, &'static str)>
    Sequence(proc_macro2::TokenStream),
}

impl ToTokens for MetadataKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            MetadataKind::Type(has, _) => tokens.extend(has.clone()),
            MetadataKind::Sequence(has) => tokens.extend(has.clone()),
        }
    }
}

/// Helper function to find the type being used for metadata
fn parse_metadata_type(attrs: &[syn::Attribute]) -> MetadataKind {
    let metadata_type = _parse_metadata_type(attrs);
    let metadata_sequence = _parse_metadata_sequence(attrs);
    match metadata_type {
        Some((tokens, defaults)) => match metadata_sequence {
            Some(_) => panic!("Only one of metadata_type and metadata_sequence may be set."),
            None => MetadataKind::Type(tokens, defaults),
        },
        None => match metadata_sequence {
            Some(tokens) => MetadataKind::Sequence(tokens),
            None => MetadataKind::Sequence(quote!(std::collections::HashMap<&'static str, &'static str>)),
        },
    }
}

/// Parse metadata type if its a sequence type
fn _parse_metadata_sequence(attrs: &[syn::Attribute]) -> Option<proc_macro2::TokenStream> {
    for attr in attrs {
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "metadata_sequence" {
            let MetadataType(name, _) = syn::parse2(attr.tokens.clone()).expect("Invalid metadata_sequence");
            return Some(quote!{ #name })
        }
    }
    None
}

/// Parse metadata type if its a struct
fn _parse_metadata_type(attrs: &[syn::Attribute]) -> Option<(proc_macro2::TokenStream, bool)> {
    for attr in attrs {
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "metadata_type" {
            let MetadataType(name, defaults) = syn::parse2(attr.tokens.clone()).expect("Invalid metadata_type");
            return Some((quote!{ #name }, defaults))
        }
    }
    None
}

/// Parse out the metadata attribute
fn parse_metadata_params(metatype: &MetadataKind, attrs: &[syn::Attribute]) -> proc_macro2::TokenStream {
    match metatype {
        MetadataKind::Sequence(_) => {
            for attr in attrs {
                if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "metadata" {
                    let MetadataParams (names, values) = syn::parse2(attr.tokens.clone()).expect("Invalid metadata attribute");
                    return quote!{ [
                        #((stringify!(#names), stringify!(#values).into())),*
                    ].into_iter().collect() }
                }
            }
            quote!{ Default::default() }
        },
        MetadataKind::Type(type_name, defaults) => {
            let defaults = if *defaults {
                quote!(..Default::default())
            } else {
                quote!()
            };

            for attr in attrs {
                if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "metadata" {
                    let MetadataParams (names, values) = syn::parse2(attr.tokens.clone()).expect("Invalid metadata attribute");
                    return quote!{
                        #type_name {
                            #(#names: #values.into(),)*
                            #defaults
                        }
                    }
                }
            }
            quote!{ Default::default() }
        }
    }
}

/// Helper to parse out the metadata_type attribute
struct MetadataType(syn::Type, bool);
impl syn::parse::Parse for MetadataType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        let key = content.parse()?;

        if content.is_empty() {
            return Ok(MetadataType(key, true));
        }

        let defaults;

        content.parse::<Token![,]>()?;

        let param: Ident = content.parse()?;
        if content.peek(Token![:]) {
            content.parse::<Token![:]>()?;
        } else {
            content.parse::<Token![=]>()?;
        }
        let value: LitBool = content.parse()?;

        if param == "defaults" {
            defaults = value.value;
        } else {
            panic!("Unknown type parameter: {param}")
        }

        Ok(MetadataType(key, defaults))
    }
}

/// Helper to parse out the metadata attribute
struct MetadataParams(Vec<syn::Ident>, Vec<syn::Expr>);
impl syn::parse::Parse for MetadataParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        // let lifetime = content.parse()?;
        let mut names = vec![];
        let mut values = vec![];

        loop {
            let key = content.parse()?;
            if content.peek(Token![:]) {
                content.parse::<Token![:]>()?;
            } else {
                content.parse::<Token![=]>()?;
            }
            let value = content.parse()?;
            names.push(key);
            values.push(value);

            if content.is_empty() {
                break
            }
            content.parse::<Token![,]>()?;
        }

        Ok(MetadataParams(names, values))
    }
}

/// Parse metadata type if its a struct
fn _parse_serde_renames(attrs: &[syn::Attribute]) -> SerdeParams {
    for attr in attrs {
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "serde" {
            return syn::parse2(attr.tokens.clone()).expect("Invalid serde");
        }
    }
    Default::default()
}

/// Helper to parse out the serde attribute
#[derive(Default)]
struct SerdeParams {
    /// Contains new name if this field is renamed
    rename: Option<String>,
    // rename_all: ,
    // rename
}
impl syn::parse::Parse for SerdeParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);

        let mut out = SerdeParams::default();
        // let mut names: Vec<syn::Ident> = vec![];
        // let mut values: Vec<syn::Expr> = vec![];

        loop {
            let key: syn::Ident = content.parse()?;
            if content.peek(Token![=]) {
                content.parse::<Token![=]>()?;

                let value: syn::LitStr = content.parse()?;

                if key == "rename" {
                    out.rename = Some(value.value());
                }
            }

            if content.is_empty() {
                break
            }
            content.parse::<Token![,]>()?;
        }

//         Ok(MetadataParams(names, values))
        Ok(out)
    }
}

