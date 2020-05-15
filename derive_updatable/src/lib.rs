#![feature(specialization)]
extern crate syn;
#[macro_use] extern crate quote;

use proc_macro::TokenStream;
use syn::{
    parse_macro_input,
    DeriveInput,
    Ident,
    Fields,
    Data,
    Type,
    export::{
        TokenStream2,
    },
};

#[proc_macro_derive(Updatable)]
pub fn derive_updatable(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let updatable_impl = updatable_impl(input.clone());
    let update_struct = update_struct(input.clone());
    let update_impl = update_impl(input);
    TokenStream::from(quote! {
        #updatable_impl
        #update_struct
        #update_impl
    })
}
fn update_struct(input: DeriveInput) -> syn::export::TokenStream2 {
    let DeriveInput {
        data,
        vis,
        ident,
        attrs: _,
        generics: _,
    } = input.clone();
    let update_ident = ident_append(ident.clone(), "Update");
    let update_builder = update_builder(input.clone());
    let update_fields =
        match data {
            Data::Struct(data) => update_fields(data.fields),
            Data::Enum(_) | Data::Union(_) => unimplemented!(),
        };
    quote! {
        #[derive(Clone, Default, Debug, PartialEq)]
        #vis struct #update_ident #update_fields
        #update_builder
    }
}
fn update_builder(input: DeriveInput) -> syn::export::TokenStream2 {
    let ident = input.ident.clone();
    let update_ident = ident_append(ident.clone(), "Update");
    let update_setters =
        match input.data {
            Data::Struct(data) => update_setters(data.fields),
            Data::Enum(_) | Data::Union(_) => unimplemented!(),
        };
    quote! {
        impl #ident {
            pub fn update() -> #update_ident {
               <#update_ident as Default>::default()
            }
        }
        impl #update_ident {
            pub fn builder() -> #update_ident {
                Self::default()
            }
            #update_setters
        }
    }
}
fn update_setters(fields: Fields) -> TokenStream2 {
    let field_ident = field_idents(fields.clone());
    let field_ty = field_types(fields.clone());
    quote! {
        #(pub fn #field_ident(self, update: <#field_ty as Updatable>::Update) -> Self {
                let mut new = self;
                new.#field_ident = Some(update);
                new
            })*
    }
}
fn updatable_impl(input: DeriveInput) -> TokenStream2 {
    let ident = input.ident.clone();
    let update_ident = ident_append(input.ident, "Update");
    quote! {
        impl Updatable for #ident {
            type Update = #update_ident;
        }
    }
}
fn update_impl(input: DeriveInput) -> TokenStream2 {
    let ident = input.ident.clone();
    let update_ident = ident_append(input.ident, "Update");
    let update_calls =
        match input.data {
            Data::Struct(data_struct) => update_calls(data_struct.fields),
            Data::Enum(_) | Data::Union(_) => unimplemented!(),
        };
    quote! {
        impl Update<#ident> for #update_ident {
            fn update(&self, data: &mut #ident) {
                #update_calls
            }
        }
    }
}
fn ident_append<S: AsRef<str>>(ident: Ident, append: S) -> Ident {
    Ident::new(&(ident.to_string() + append.as_ref()), ident.span())
}
fn update_fields(fields: Fields) -> Fields {
    match fields.clone() {
        Fields::Named(mut inner) => {
            inner.named
                .iter_mut()
                .for_each(|field| {
                    let ty = field.ty.clone();
                    field.ty = syn::parse2(quote!{
                        Option<<#ty as Updatable>::Update>
                    }).unwrap();
                });
            Fields::Named(inner)
        },
        Fields::Unnamed(_) => {
            unimplemented!()
        },
        Fields::Unit => {
            unimplemented!()
        },
    }
}
fn update_calls(fields: Fields) -> TokenStream2 {
    let field_ident = field_idents(fields);
    quote! {
        #(self.#field_ident.clone().map(|update| update.update(&mut data.#field_ident));)*
    }
}
fn field_idents(fields: Fields) -> Vec<Ident> {
    match fields.clone() {
        Fields::Named(inner) => {
            inner.named
                .iter()
                .map(|field| {
                    field.ident.clone().unwrap()
                })
                .collect()
        },
        Fields::Unnamed(_) => {
            unimplemented!()
        },
        Fields::Unit => {
            unimplemented!()
        },
    }
}
fn field_types(fields: Fields) -> Vec<Type> {
    match fields.clone() {
        Fields::Named(inner) => {
            inner.named
                .iter()
                .map(|field| {
                    field.ty.clone()
                })
                .collect()
        },
        Fields::Unnamed(_) => {
            unimplemented!()
        },
        Fields::Unit => {
            unimplemented!()
        },
    }
}

