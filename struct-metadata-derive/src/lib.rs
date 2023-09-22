use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};


#[proc_macro_derive(Described)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput {ident, attrs, ..} = parse_macro_input!(input);

    let docs = match parse_doc_comment(&attrs) {
        Some(docs) => quote!{ Some(vec![
            #( #docs, )*
        ])},
        None => quote! { None }
    };

    let output = quote! {
        impl struct_metadata::Described for #ident {
            fn metadata() -> struct_metadata::Descriptor {
                struct_metadata::Descriptor {
                    docs: #docs,
                    name: stringify!(#ident).to_owned(),
                    kind: "struct".to_owned(),
                }
            }
        }
    };

    output.into()
}

fn parse_doc_comment(attrs: &[syn::Attribute]) -> Option<Vec<String>> {
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
        None
    } else {
        Some(lines)
    }
}
