//! Derive macro for the ValidatedDeserialize trait
#![warn(missing_docs, non_ascii_idents, trivial_numeric_casts,
    noop_method_call, single_use_lifetimes, trivial_casts,
    unused_lifetimes, nonstandard_style, variant_size_differences)]
#![deny(keyword_idents)]
#![warn(clippy::missing_docs_in_private_items)]
#![allow(clippy::needless_return, clippy::while_let_on_iterator)]

use proc_macro::TokenStream;
use syn::spanned::Spanned;
use syn::{parenthesized, parse2, parse_macro_input, Attribute, DeriveInput, Ident, Token};
use quote::{quote, quote_spanned, ToTokens};

/// Derive macro for the ValidatedDeserialize trait
#[proc_macro_derive(ValidatedDeserialize, attributes(validated_deserialize, validate, serde))]
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
    let mut other_generics = vec![];
    for generic in generics.params {
        match generic {
            syn::GenericParam::Lifetime(lifetime) => {
                lifetimes.push(lifetime.to_token_stream());
                continue
            },
            syn::GenericParam::Type(_) => {},
            syn::GenericParam::Const(_) => {},
        }
        other_generics.push(generic);
    }

    // parse the attributes
    let ValidatorParameter { validator_name: validator, temporary_name, derives } = read_validated_deserialize_attr(&attrs)?;
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

    // figure out what lifetime to use to limit fields
    // let (proxy_lifetimes, field_lifetime) = match lifetimes.len() {
    //     0 => (vec![], quote!('static)),
    //     1 => (lifetimes.clone(), lifetimes[0].clone()),
    //     _ => {
    //         quote!{'derive_validated_deserialize}
    //     }
    // };

    for (index, field) in data.fields.iter().enumerate() {

        // try to pull out a lifetime to use
        let validate_field = read_validate_field_attr(&field.attrs)?;
        let field_type = &field.ty;
        let field_lifetimes = extract_lifetimes(field_type);
        let field_lifetime = field_lifetimes.first().map(|life| life.to_token_stream()).unwrap_or(quote!('static));
        let visibility = &field.vis;

        // create the field for the proxy type that inherits all the serde parameters
        let field_serde_attributes = select_serde_attributes(&field.attrs);
        match &field.ident {
            Some(field_name) => {
                named_fields = true;
                // struct_brace = proc_macro2::Delimiter::Brace;

                if validate_field {
                    proxy_fields.push(quote_spanned!{ field_type.span() =>
                        #(#field_serde_attributes)*
                        #visibility #field_name: <#field_type as ValidatedDeserialize<#field_lifetime, #validator>>::ProxyType,
                    });
            
                    field_conversion.push(quote_spanned!{ field_type.span() =>
                        #field_name: <#field_type as ValidatedDeserialize<'de, #validator>>::validate(input.#field_name, validator)?,
                    })
                } else {
                    proxy_fields.push(quote_spanned!{ field_type.span() =>
                        #(#field_serde_attributes)*
                        #visibility #field_name: #field_type,
                    });
            
                    field_conversion.push(quote_spanned!{ field_type.span() =>
                        #field_name: input.#field_name,
                    })
                }
            },
            None => {
                let field_type = &field.ty;
                if !field_serde_attributes.is_empty() {
                    return Err(syn::Error::new(field_serde_attributes[0].span(), "Unexpected attribute"))
                }
                let index = syn::Index::from(index);

                if validate_field {
                    proxy_fields.push(quote_spanned!{ field_type.span() =>
                        #(#field_serde_attributes)*
                        #visibility <#field_type as ValidatedDeserialize<#field_lifetime, #validator>>::ProxyType,
                    });
            
                    field_conversion.push(quote_spanned!{ field_type.span() =>
                        #(#field_serde_attributes)*
                        #index: <#field_type as ValidatedDeserialize<'de, #validator>>::validate(input.#index, validator)?,
                    })
                } else {
                    proxy_fields.push(quote_spanned!{ field_type.span() =>
                        #(#field_serde_attributes)*
                        #visibility #field_type,
                    });
            
                    field_conversion.push(quote_spanned!{ field_type.span() =>
                        #(#field_serde_attributes)*
                        #index: input.#index,
                    })
                }
            }
        }
    }

    // select the proxy type name
    let proxy_type_name = match temporary_name {
        Some(name) => name,
        None => Ident::new(&(ident.to_string() + "Unvalidated"), ident.span())
    };
   
    let mut limit_lifetimes = vec![];
    for lifetime in &lifetimes {
        limit_lifetimes.push(quote!{#lifetime: 'derive_validated_deserialize})
    }

    // set the bracket type
    let proxy_fields = if named_fields { 
        quote! { { #(#proxy_fields)* } }
    } else {
        quote! { ( #(#proxy_fields)* ); }
    };

    // build the proxy type 
    let proxy_declare = quote! {
        #[derive(serde::Deserialize, #(#derives),*)]
        #(#container_serde_attributes)*
        pub struct #proxy_type_name <#(#lifetimes),* #(#other_generics),*> #proxy_fields
    };

    // // build the proxy type 
    // let proxy_declare = quote! {
    //     #[derive(serde::Deserialize, #(#derives),*)]
    //     #(#container_serde_attributes)*
    //     pub struct #proxy_type_name <'derive_validated_deserialize: #(#lifetimes)+*, #(#limit_lifetimes),* #(#other_generics),*> #proxy_fields
    // };

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
            type ProxyType = #proxy_type_name<#(#lifetimes),*>;

            fn validate(input: Self::ProxyType, validator: &#validator) -> ::std::result::Result<Self, ::std::string::String> {
                Ok(Self {
                    #(#field_conversion)*
                })
            }
        }
    })
}

/// Check whether a field has been marked for validation
fn read_validate_field_attr(attrs: &[Attribute]) -> syn::Result<bool> {
    for attr in attrs {
        if attr.path().is_ident("validate") {
            return Ok(true)
        }
    }
    Ok(false)
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
    validator_name: Ident,

    /// Name given to the temporary type
    temporary_name: Option<Ident>,

    /// Derives to be added to the temporary type
    derives: Vec<Ident>,
}

impl syn::parse::Parse for ValidatorParameter {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let validator_name: Ident = input.parse()?;
        let mut derives = vec![];
        let mut temporary_name = None;

        while input.parse::<Token![,]>().is_ok() {
            let arg_name: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            if arg_name == "derive" {
                let content;
                parenthesized!(content in input);
                derives = content.parse_terminated(Ident::parse, Token![,])?.into_iter().collect();
            } else if arg_name == "name" {
                temporary_name = Some(input.parse()?);
            } else {
                return Err(syn::Error::new_spanned(arg_name, "Unexpected parameter"))
            }
        }

        Ok(Self {validator_name, derives, temporary_name})
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

/// Try to guess the lifetimes used in a type
fn extract_lifetimes(ty: &syn::Type) -> Vec<syn::Lifetime> {
    match ty {
        syn::Type::Array(ty) => extract_lifetimes(&ty.elem),
        syn::Type::BareFn(_) => todo!("extract_lifetimes BareFn"),
        syn::Type::Group(ty) => extract_lifetimes(&ty.elem),
        syn::Type::ImplTrait(_) => todo!("extract_lifetimes ImplTrait"),
        syn::Type::Infer(_) => todo!("extract_lifetimes Infer"),
        syn::Type::Macro(_) => todo!("extract_lifetimes Macro"),
        syn::Type::Never(_) => vec![],
        syn::Type::Paren(ty) => extract_lifetimes(&ty.elem),
        syn::Type::Path(path) => {
            let mut lives = vec![];
            for seg in &path.path.segments {
                if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                    for arg in &args.args {
                        match arg {
                            syn::GenericArgument::Lifetime(life) => lives.push(life.clone()),
                            syn::GenericArgument::Type(ty) => lives.append(&mut extract_lifetimes(ty)),
                            syn::GenericArgument::AssocType(ty) => lives.append(&mut extract_lifetimes(&ty.ty)),
                            _ => {},
                        }
                    }
                }
            }
            lives
        },
        syn::Type::Ptr(_) => todo!("extract_lifetimes Ptr"),
        syn::Type::Reference(ty) => {
            let mut lifes = extract_lifetimes(&ty.elem);
            if let Some(life) = &ty.lifetime {
                lifes.push(life.clone());
            }
            lifes
        },
        syn::Type::Slice(slice) => {
            extract_lifetimes(&slice.elem)
        },
        syn::Type::TraitObject(_) => todo!("extract_lifetimes TraitObject"),
        syn::Type::Tuple(types) => {
            let mut files = vec![];
            for ty in &types.elems {
                files.append(&mut extract_lifetimes(ty))
            }
            files
        },
        syn::Type::Verbatim(_) => todo!("extract_lifetimes Verbatim"),
        _ => todo!("extract_lifetimes Other"),
    }
}