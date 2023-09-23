#![warn(missing_docs, non_ascii_idents, trivial_numeric_casts,
    unused_crate_dependencies, noop_method_call, single_use_lifetimes, trivial_casts,
    unused_lifetimes, nonstandard_style, variant_size_differences)]
#![deny(keyword_idents)]
#![warn(clippy::missing_docs_in_private_items)]
#![allow(clippy::needless_return, clippy::while_let_on_iterator)]

use proc_macro::{self, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Token, Ident, LitBool};


#[proc_macro_derive(Described, attributes(metadata, metadata_type, metadata_sequence))]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput {ident, attrs, data, ..} = parse_macro_input!(input);

    let metadata_type = parse_metadata_type(&attrs);

    match data {
        syn::Data::Struct(data) => {

            let kind = match data.fields {
                syn::Fields::Named(fields) => {
                    let mut children = vec![];

                    for field in fields.named {
                        let name = field.ident.unwrap();
                        let ty = &field.ty;
                        let ty = quote!(<#ty as struct_metadata::Described>::metadata());
                        let docs = parse_doc_comment(&field.attrs);
                        let metadata: proc_macro2::TokenStream = parse_metadata_params(&metadata_type, &field.attrs);

                        children.push(quote!{struct_metadata::Entry {
                            label: stringify!(#name).to_owned(),
                            docs: #docs,
                            metadata: #metadata,
                            type_info: #ty
                        }});
                    }

                    quote!(struct_metadata::Kind::Struct {
                        name: stringify!(#ident).to_owned(),
                        children: vec![#(#children),*]
                    })
                },
                syn::Fields::Unnamed(fields) => {
                    if fields.unnamed.is_empty() {
                        quote!(struct_metadata::Kind::Struct { name: stringify!(#ident).to_owned(), children: vec![] })
                    } else if fields.unnamed.len() == 1 {
                        let ty = &fields.unnamed[0].ty;
                        quote!(struct_metadata::Kind::Aliased { name: stringify!(#ident).to_owned(), kind: Box::new(<#ty as struct_metadata::Described>::metadata())})
                    } else {
                        panic!("tuple struct not supported")
                    }
                },
                syn::Fields::Unit => {
                    quote!(struct_metadata::Kind::Struct { name: stringify!(#ident).to_owned(), children: vec![] })
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

        _ => {
            panic!("only structs are supported")
        }
    }
}

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

enum MetadataKind {
    Type(proc_macro2::TokenStream, bool),
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

fn _parse_metadata_sequence(attrs: &[syn::Attribute]) -> Option<proc_macro2::TokenStream> {
    for attr in attrs {
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "metadata_sequence" {
            let MetadataType(name, _) = syn::parse2(attr.tokens.clone()).expect("Invalid metadata_sequence");
            return Some(quote!{ #name })
        }
    }
    None
}

fn _parse_metadata_type(attrs: &[syn::Attribute]) -> Option<(proc_macro2::TokenStream, bool)> {
    for attr in attrs {
        if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "metadata_type" {
            let MetadataType(name, defaults) = syn::parse2(attr.tokens.clone()).expect("Invalid metadata_type");
            return Some((quote!{ #name }, defaults))
        }
    }
    None
}


fn parse_metadata_params(metatype: &MetadataKind, attrs: &[syn::Attribute]) -> proc_macro2::TokenStream {
    match metatype {
        MetadataKind::Sequence(_) => {
            for attr in attrs {
                if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "metadata" {
                    let NotesParams (names, values) = syn::parse2(attr.tokens.clone()).expect("Invalid metadata attribute");
                    return quote!{ [
                        #((stringify!(#names), stringify!(#values))),*
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
                    let NotesParams (names, values) = syn::parse2(attr.tokens.clone()).expect("Invalid metadata attribute");
                    return quote!{
                        #type_name {
                            #(#names: #values,)*
                            #defaults
                        }
                    }
                }
            }
            quote!{ Default::default() }
        }
    }
}

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
        content.parse::<Token![:]>()?;
        let value: LitBool = content.parse()?;

        if param == "defaults" {
            defaults = value.value;
        } else {
            panic!("Unknown type parameter: {param}")
        }

        Ok(MetadataType(key, defaults))
    }
}



struct NotesParams(Vec<syn::Ident>, Vec<syn::Expr>);
impl syn::parse::Parse for NotesParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        // let lifetime = content.parse()?;
        let mut names = vec![];
        let mut values = vec![];

        loop {
            let key = content.parse()?;
            content.parse::<Token![:]>()?;
            let value = content.parse()?;
            names.push(key);
            values.push(value);

            if content.is_empty() {
                break
            }
            content.parse::<Token![,]>()?;
        }

        Ok(NotesParams(names, values))
    }
}

