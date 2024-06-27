//! Derive macro for the ValidatedDeserialize trait
#![warn(missing_docs, non_ascii_idents, trivial_numeric_casts,
    noop_method_call, single_use_lifetimes, trivial_casts,
    unused_lifetimes, nonstandard_style, variant_size_differences)]
#![deny(keyword_idents)]
#![warn(clippy::missing_docs_in_private_items)]
#![allow(clippy::needless_return, clippy::while_let_on_iterator)]

use proc_macro::TokenStream;
use syn::spanned::Spanned;
use syn::{parse2, parse_macro_input, Attribute, DeriveInput, Ident};
use quote::{quote, ToTokens};

/// Derive macro for the ValidatedDeserialize trait
#[proc_macro_derive(ValidatedDeserialize, attributes(validated_deserialize, serde))]
pub fn derive_validated_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    match _derive_validated_deserialize(input) {
        Ok(stream) => stream.into(),
        Err(err) => err.into_compile_error().into()
    }
}

/// Result returning inner implementation for derive_validated_deserialize
fn _derive_validated_deserialize(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let DeriveInput {ident, attrs, data, generics, ..} = input;

    // parse out the lifetime parameters
    let mut lifetimes = vec![];
    for generic in generics.params {
        match generic {
            syn::GenericParam::Lifetime(lifetime) => lifetimes.push(lifetime.to_token_stream()),
            syn::GenericParam::Type(_) => todo!(),
            syn::GenericParam::Const(_) => todo!(),
        }
    }

    // parse the attributes
    let ValidatorParameter { type_name: validator } = read_validated_deserialize_attr(&attrs)?;
    let container_serde_attributes = select_serde_attributes(&attrs);

    // extract struct data
    let data = match data {
        syn::Data::Struct(data) => data,
        syn::Data::Enum(data) => return Err(syn::Error::new(data.enum_token.span, "Only structs are supported")),
        syn::Data::Union(data) => return Err(syn::Error::new(data.union_token.span, "Only structs are supported")),
    };

    // extract the field list
    let mut proxy_fields: Vec<proc_macro2::TokenStream> = vec![];
    let mut field_conversion: Vec<proc_macro2::TokenStream> = vec![];
    let mut named_fields = false;
    for (index, field) in data.fields.iter().enumerate() {

        // try to pull out a lifetime to use
        let field_type = &field.ty;
        // let lifetime = if let Type::Reference(typeref) = field_type {
        //     match &typeref.lifetime {
        //         Some(life) => life.to_token_stream(),
        //         None => quote!{'derive_validated_deserialize},
        //     }
        // } else {
        //     quote!{'derive_validated_deserialize}
        // };
        let lifetime = quote!{'derive_validated_deserialize};

        // create the field for the proxy type that inherits all the serde parameters
        let field_serde_attributes = select_serde_attributes(&field.attrs);
        match &field.ident {
            Some(field_name) => {
                named_fields = true;

                proxy_fields.push(quote!{
                    #(#field_serde_attributes)*
                    #[serde(borrow)]
                    #field_name: <#field_type as ValidatedDeserialize<#lifetime, #validator>>::ProxyType,
                });
        
                field_conversion.push(quote!{
                    #field_name: <#field_type as ValidatedDeserialize<'de, #validator>>::validate(input.#field_name, &validator)?,
                })
            },
            None => {
                let field_type = &field.ty;
                if !field_serde_attributes.is_empty() {
                    return Err(syn::Error::new(field_serde_attributes[0].span(), "Unexpected attribute"))
                }
                let index = syn::Index::from(index);

                proxy_fields.push(quote!{
                    #(#field_serde_attributes)*
                    #[serde(borrow)]
                    <#field_type as ValidatedDeserialize<#lifetime, #validator>>::ProxyType,
                });
        
                field_conversion.push(quote!{
                    #index: <#field_type as ValidatedDeserialize<'de, #validator>>::validate(input.#index, &validator)?,
                })
            }
        }
    }

    // select the proxy type name (todo make configurable)
    let proxy_type = Ident::new(&(ident.to_string() + "Unvalidated"), ident.span());
   
    let mut limit_lifetimes = vec![];
    for lifetime in &lifetimes {
        limit_lifetimes.push(quote!{#lifetime: 'derive_validated_deserialize})
    }

    // build the proxy type 
    let proxy_declare = if named_fields { 
        quote! {
            #[derive(serde::Deserialize)]
            #(#container_serde_attributes)*
            struct #proxy_type <'derive_validated_deserialize: #(#lifetimes)+*, #(#limit_lifetimes),*> {
                #(#proxy_fields)*
            }
        }
    } else {
        quote! {
            #[derive(serde::Deserialize)]
            #(#container_serde_attributes)*
            struct #proxy_type <'derive_validated_deserialize: #(#lifetimes)+*, #(#limit_lifetimes),*> (
                #(#proxy_fields)*
            );
        }
    };

    let mut limit_lifetimes = vec![];
    for lifetime in &lifetimes {
        // limit_lifetimes.push(quote!{#lifetime})
        limit_lifetimes.push(quote!{#lifetime: 'de})
    }

    // emit derivation
    Ok(quote!{
        #proxy_declare

        #[automatically_derived]
        impl<'de: #(#lifetimes)+*, #(#limit_lifetimes),*> ValidatedDeserialize<'de, #validator> for #ident <#(#lifetimes),*> {
            type ProxyType = #proxy_type<'de, #(#lifetimes),*>;

            fn validate(input: Self::ProxyType, validator: &#validator) -> Result<Self, String> {
                Ok(Self {
                    #(#field_conversion)*
                })
            }
        }
    })
}

/// Given a set of struct attributes get the parameters defined for validated_deserialize
fn read_validated_deserialize_attr(attrs: &[Attribute]) -> syn::Result<ValidatorParameter> {
    for attr in attrs {
        if let syn::Meta::List(items) = &attr.meta {
            if items.path.is_ident("validated_deserialize") {
                return parse2(items.tokens.clone());
            }
        }
    }
    panic!("validated_deserialize must be provided the validator type for ValidatedDeserialize")
}

/// Parameterization of the ValidatedDeserialize macro
struct ValidatorParameter {
    /// What type will be passed as a validator
    type_name: Ident
}

impl syn::parse::Parse for ValidatorParameter {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let type_name = input.parse()?;
        Ok(Self {type_name})
    }
}

/// capture serde params just as a token stream for forwarding to the unvalidated type
fn select_serde_attributes(attrs: &[Attribute]) -> Vec<proc_macro2::TokenStream> {
    let mut output = vec![];
    for attr in attrs {
        if attr.path().is_ident("serde") {
            output.push(attr.to_token_stream())
        }
    }
    output
}