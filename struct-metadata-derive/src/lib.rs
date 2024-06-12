//! Derive macro for the struct-metadata package.

#![warn(missing_docs, non_ascii_idents, trivial_numeric_casts,
    noop_method_call, single_use_lifetimes, trivial_casts,
    unused_lifetimes, nonstandard_style, variant_size_differences)]
#![deny(keyword_idents)]
#![warn(clippy::missing_docs_in_private_items)]
#![allow(clippy::needless_return, clippy::while_let_on_iterator)]


use convert_case::Casing;
use proc_macro::{self, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, Token, Ident, LitBool};

/// Derive macro for the MetadataKind trait
#[proc_macro_derive(MetadataKind)]
pub fn derive_metadata_kind(input: TokenStream) -> TokenStream {
    let DeriveInput {ident, ..} = parse_macro_input!(input);

    let output = quote! {
        impl struct_metadata::MetadataKind for #ident {}
    };

    output.into()

}

/// Derive macro for the Described trait
#[proc_macro_derive(Described, attributes(metadata, metadata_type, metadata_sequence, serde))]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput {ident, attrs, data, ..} = parse_macro_input!(input);

    let metadata_type = parse_metadata_type(&attrs);
    let serde_attrs = _parse_serde_attrs(&attrs);

    // ident will refer to the TYPE NAME, outer_name will refer to the presented name in metadata for the type
    let outer_name = match serde_attrs.rename {
        Some(new_name) => quote!(#new_name),
        None => quote_spanned!(ident.span() => stringify!(#ident)),
    };

    match data {
        syn::Data::Struct(data) => {

            let kind = match data.fields {
                syn::Fields::Named(fields) => {
                    let mut children = vec![];
                    let mut flattened_children = vec![];

                    for field in &fields.named {
                        let SerdeFieldAttrs {rename, flatten } = _parse_serde_field_attrs(&field.attrs);
                        let name = field.ident.clone().unwrap();
                        let ty = &field.ty;
                        if flatten {
                            let fields = quote_spanned!(ty.span() => <#ty as struct_metadata::Described::<#metadata_type>>::metadata());
                            flattened_children.push(fields);
                            continue
                        }
                        let ty = quote_spanned!(ty.span() => <#ty as struct_metadata::Described::<#metadata_type>>::metadata());
                        let docs = parse_doc_comment(&field.attrs);
                        let metadata: proc_macro2::TokenStream = parse_metadata_params(&metadata_type, &field.attrs);

                        let name = if let Some(rename) = rename {
                            quote!(#rename)
                        } else if let Some(case) = serde_attrs.rename_all {
                            let new_name = name.to_string().to_case(case);
                            quote!(#new_name)
                        } else {
                            quote!(stringify!(#name))
                        };

                        children.push(quote!{struct_metadata::Entry::<#metadata_type> {
                            label: #name,
                            docs: #docs,
                            metadata: #metadata,
                            type_info: #ty
                        }});
                    }

                    if flattened_children.is_empty() {
                        quote!(struct_metadata::Kind::Struct::<#metadata_type> {
                            name: #outer_name,
                            children: vec![#(#children),*]
                        })
                    } else {
                        quote!(struct_metadata::Kind::<#metadata_type>::new_struct(#outer_name, vec![#(#children),*], &mut [#(#flattened_children),*]))
                    }
                },
                syn::Fields::Unnamed(fields) => {
                    if fields.unnamed.is_empty() {
                        quote!(struct_metadata::Kind::<#metadata_type>::Struct { name: #outer_name, children: vec![] })
                    } else if fields.unnamed.len() == 1 {
                        let ty = &fields.unnamed[0].ty;
                        let ty = quote_spanned!(ty.span() => <#ty as struct_metadata::Described::<#metadata_type>>::metadata());
                        quote!(struct_metadata::Kind::<#metadata_type>::Aliased { name: #outer_name, kind: Box::new(#ty)})
                    } else {
                        panic!("tuple struct not supported")
                    }
                },
                syn::Fields::Unit => {
                    quote!(struct_metadata::Kind::<#metadata_type>::Struct { name: #outer_name, children: vec![] })
                },
            };

            let docs = parse_doc_comment(&attrs);
            let metadata: proc_macro2::TokenStream = parse_metadata_params(&metadata_type, &attrs);
            let output = quote! {
                impl struct_metadata::Described::<#metadata_type> for #ident {
                    fn metadata() -> struct_metadata::Descriptor::<#metadata_type> {
                        let mut data = struct_metadata::Descriptor::<#metadata_type> {
                            docs: #docs,
                            kind: #kind,
                            metadata: #metadata,
                        };
                        data.propagate(None);
                        data
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
                let SerdeFieldAttrs {rename, ..} = _parse_serde_field_attrs(&variant.attrs);

                let name = if let Some(name) = rename {
                    quote!(#name)
                } else if let Some(case) = serde_attrs.rename_all {
                    let new_name = name.to_string().to_case(case);
                    quote!(#new_name)
                } else {
                    quote_spanned!(variant.span() => stringify!(#name))
                };

                all_variants.push(quote!{struct_metadata::Variant::<#metadata_type> {
                    label: #name.to_owned(),
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
                                name: #outer_name,
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
            panic!("derive is not supported for this type")
        }
    }
}

/// Derive macro for the Described trait for enums where the varient labels provided should come
/// from the to_string method rather than raw varient names
#[proc_macro_derive(DescribedEnumString, attributes(metadata, metadata_type, metadata_sequence, serde))]
pub fn derive_enum_string(input: TokenStream) -> TokenStream {
    let DeriveInput {ident, attrs, data, ..} = parse_macro_input!(input);

    let metadata_type = parse_metadata_type(&attrs);

    match data {
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
            panic!("DescribedEnumString only applies to enum types")
        }
    }
}


/// Helper function to pull out docstrings
/// syn always stores comments as attribute pairs with the path "doc"
fn parse_doc_comment(attrs: &[syn::Attribute]) -> proc_macro2::TokenStream {
    let mut lines = vec![];
    for attr in attrs {
        if attr.style != syn::AttrStyle::Outer {
            continue
        }

        if let syn::Meta::NameValue(pair) = &attr.meta {
            if !pair.path.is_ident("doc") { continue }
            if let Some(doc) = pair.value.span().source_text() {
                let doc = doc.strip_prefix("///").unwrap_or(&doc);
                lines.push(doc.trim().to_string());
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

/// Parse metadata type if its a sequence type in the form of
/// #[metadata_sequence(Vec<(&'static str, &'static str)>)]
/// syn stores them as a metadata path followed by a list of tokens
fn _parse_metadata_sequence(attrs: &[syn::Attribute]) -> Option<proc_macro2::TokenStream> {
    for attr in attrs {
        if let syn::Meta::List(meta) = &attr.meta {
            if meta.path.is_ident("metadata_sequence") {
                let MetadataType(name, _) = syn::parse2(meta.tokens.clone()).expect("Invalid metadata_sequence");
                return Some(quote!{ #name })    
            }
        }
    }
    None
}

/// Parse metadata type if its a struct in the form of
/// #[metadata_type(Properties, defaults: false)]
fn _parse_metadata_type(attrs: &[syn::Attribute]) -> Option<(proc_macro2::TokenStream, bool)> {
    for attr in attrs {
        if let syn::Meta::List(meta) = &attr.meta {
            if meta.path.is_ident("metadata_type") {
                let MetadataType(name, defaults) = syn::parse2(meta.tokens.clone()).expect("Invalid metadata_type");
                return Some((quote!{ #name }, defaults))    
            }
        }
    }
    None
}

/// Parse out the metadata attribute
fn parse_metadata_params(metatype: &MetadataKind, attrs: &[syn::Attribute]) -> proc_macro2::TokenStream {
    match metatype {
        MetadataKind::Sequence(_) => {
            for attr in attrs {
                if let syn::Meta::List(meta) = &attr.meta {
                    if meta.path.is_ident("metadata") {
                        let MetadataParams (names, values) = syn::parse2(meta.tokens.clone())
                            .unwrap_or_else(|_| panic!("Invalid metadata attribute: {}", meta.tokens));
                        return quote!{ [
                            #((stringify!(#names), stringify!(#values).into())),*
                        ].into_iter().collect() }
                    }
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
                if let syn::Meta::List(meta) = &attr.meta {
                    if meta.path.is_ident("metadata") {
                        let MetadataParams (names, values) = syn::parse2(meta.tokens.clone())
                            .unwrap_or_else(|_| panic!("Invalid metadata attribute: {}", meta.tokens));
                        return quote!{
                            #type_name {
                                #(#names: #values.into(),)*
                                #defaults
                            }
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
        // let content;
        // syn::parenthesized!(content in input);
        let key = input.parse()?;

        if input.is_empty() {
            return Ok(MetadataType(key, true));
        }

        let defaults;

        input.parse::<Token![,]>()?;

        let param: Ident = input.parse()?;
        if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
        } else {
            input.parse::<Token![=]>()?;
        }
        let value: LitBool = input.parse()?;

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
        // let content;
        // syn::parenthesized!(content in input);
        // let lifetime = content.parse()?;
        let mut names = vec![];
        let mut values = vec![];

        loop {
            let key = input.parse()?;
            if input.peek(Token![:]) {
                input.parse::<Token![:]>()?;
            } else {
                input.parse::<Token![=]>()?;
            }
            let value = input.parse()?;
            names.push(key);
            values.push(value);

            if input.is_empty() {
                break
            }
            input.parse::<Token![,]>()?;
        }

        Ok(MetadataParams(names, values))
    }
}

/// Parse metadata type if its a struct
fn _parse_serde_field_attrs(attrs: &[syn::Attribute]) -> SerdeFieldAttrs {
    for attr in attrs {
        if let syn::Meta::List(meta) = &attr.meta {
            if meta.path.is_ident("serde") {
                return syn::parse2(meta.tokens.clone()).expect("Invalid serde");
            }
        }
    }
    Default::default()
}

/// Helper to parse out the serde attribute
#[derive(Default)]
struct SerdeFieldAttrs {
    /// Contains new name if this field is renamed
    rename: Option<String>,
    flatten: bool,
}

impl syn::parse::Parse for SerdeFieldAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // let content;
        // syn::parenthesized!(content in input);

        let mut out = SerdeFieldAttrs::default();
        // let mut names: Vec<syn::Ident> = vec![];
        // let mut values: Vec<syn::Expr> = vec![];

        loop {
            let key: syn::Ident = input.parse()?;
            if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;

                let value: syn::LitStr = input.parse()?;

                if key == "rename" {
                    out.rename = Some(value.value());
                }
            }

            if key == "flatten" {
                out.flatten = true;
            }

            if input.is_empty() {
                break
            }
            input.parse::<Token![,]>()?;
        }

//         Ok(MetadataParams(names, values))
        Ok(out)
    }
}


/// Parse metadata type if its a struct
fn _parse_serde_attrs(attrs: &[syn::Attribute]) -> SerdeAttrs {
    for attr in attrs {
        if let syn::Meta::List(meta) = &attr.meta {
            if meta.path.is_ident("serde") {
                return syn::parse2(meta.tokens.clone()).expect("Invalid serde");
            }
        }
    }
    Default::default()
}

/// Helper to parse out the serde attribute
#[derive(Default)]
struct SerdeAttrs {
    /// Contains new name if this field is renamed
    rename: Option<String>,
    rename_all: Option<convert_case::Case>,
}

impl syn::parse::Parse for SerdeAttrs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // let content;
        // syn::parenthesized!(content in input);

        let mut out = SerdeAttrs::default();
        // let mut names: Vec<syn::Ident> = vec![];
        // let mut values: Vec<syn::Expr> = vec![];

        loop {
            let key: syn::Ident = input.parse()?;
            if input.peek(Token![=]) {
                input.parse::<Token![=]>()?;

                let value: syn::LitStr = input.parse()?;

                if key == "rename" {
                    out.rename = Some(value.value());
                }

                if key == "rename_all" {
                    out.rename_all = Some(fetch_case(&value)?);
                }
            }

            if input.is_empty() {
                break
            }
            input.parse::<Token![,]>()?;
        }

        Ok(out)
    }
}

/// Determine the case conversion scheme for a given name
fn fetch_case(name: &syn::LitStr) -> syn::Result<convert_case::Case> {
    Ok(match name.value().to_lowercase().as_str() {
        "lowercase" | "lower" => convert_case::Case::Lower,
        "uppercase" | "upper" => convert_case::Case::Upper,
        "pascalcase" | "pascal" | "uppercamel" => convert_case::Case::Pascal,
        "camelcase" | "camel" => convert_case::Case::Camel,
        "snake_case" => convert_case::Case::Snake,
        "upper_snake_case" | "screaming_snake_case" => convert_case::Case::UpperSnake,
        "kebab_case" => convert_case::Case::Kebab,
        "upper_kebab_case" | "screaming_kebab_case" => convert_case::Case::UpperKebab,
        _ => return Err(syn::Error::new(name.span(), format!("Unsupported case string: {}", name.value())))
    })
}