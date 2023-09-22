use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};


#[proc_macro_derive(Described)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput {ident, attrs, data, ..} = parse_macro_input!(input);

    match data {
        syn::Data::Struct(data) => {

            let kind = match data.fields {
                syn::Fields::Named(fields) => {
                    let mut names = vec![];
                    let mut types = vec![];

                    for field in fields.named {
                        names.push(field.ident.unwrap());
                        let ty = &field.ty;
                        types.push(quote!(<#ty as struct_metadata::Described>::metadata()))
                    }

                    quote!(Kind::Struct { children: vec![
                        #(#names, #types),*
                    ] })
                },
                syn::Fields::Unnamed(fields) => {
                    if fields.unnamed.is_empty() {
                        quote!(Kind::Struct { children: vec![] })
                    } else if fields.unnamed.len() == 1 {
                        let ty = &fields.unnamed[0].ty;
                        quote!(<#ty as struct_metadata::Described>::metadata().kind)
                    } else {
                        panic!("tuple struct not supported")
                    }
                },
                syn::Fields::Unit => {
                    quote!(Kind::Struct { children: vec![] })
                },
            };

            let docs = parse_doc_comment(&attrs);
            let output = quote! {
                impl struct_metadata::Described for #ident {
                    fn metadata() -> struct_metadata::Descriptor {
                        struct_metadata::Descriptor {
                            docs: #docs,
                            name: stringify!(#ident).to_owned(),
                            kind: #kind,
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
        let meta = attr.parse_meta().unwrap();
        if let syn::Meta::NameValue(meta) = meta {
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
