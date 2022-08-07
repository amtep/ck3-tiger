use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::{parse_quote, Data, DeriveInput, Field, Fields, GenericParam, Generics, Ident};

pub fn derive_verify(input: DeriveInput) -> TokenStream {
    let name = &input.ident;

    // Add a bound Verify to every type parameter
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let body = match input.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let fields_verify = fields.named.iter().map(|f| derive_verify_field(f));
                quote! {
                    Self { #(#fields_verify ,)* }
                }
            }
            Fields::Unnamed(_) => {
                abort_call_site!("`#[derive(Verify)]` does not support unnamed fields yet")
            }
            Fields::Unit => {
                quote!()
            }
        },
        Data::Enum(_) => abort_call_site!("`#[derive(Verify)]` does not support enums"),
        Data::Union(_) => abort_call_site!("`#[derive(Verify)]` does not support unions"),
    };

    let verify = match crate_name("ck3-mod-validator") {
        Ok(FoundCrate::Itself) => quote! { crate::verify::Verify },
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote! { #ident::verify::Verify }
        }
        Err(_) => {
            abort_call_site!("`#[derive(Verify)]` could not find ck3_mod_validator crate")
        }
    };

    quote! {
        impl #impl_generics #verify for #name #type_generics #where_clause {
            fn from_scope(scope: Scope, errors: &mut Errors) -> Self {
                #body
            }
        }
    }
}

fn derive_verify_field(field: &Field) -> TokenStream {
    let name = &field.ident;
    // Placeholder
    quote! {
        #name: ::core::default::Default::default()
    }
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(Verify));
        }
    }
    generics
}
